use tracing::Instrument;

use crate::{
    data::ai::review::{
        self,
        config::try_acquire_review_guard,
        github::{
            GITHUB_TOKEN_CACHE, fetch_login, get_installation_token, has_push_permission,
            poll_device_flow, start_device_flow,
        },
    },
    prelude::*,
};

/// AI code review of GitHub pull requests.
#[poise::command(
    slash_command,
    rename = "ai-review",
    guild_only,
    subcommands("run", "enable", "disable"),
    subcommand_required
)]
#[allow(
    clippy::unused_async,
    reason = "Poise requires subcommand parents to be async."
)]
pub async fn ai_review(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Allow /ai-review in this server (Administrator only).
#[poise::command(slash_command, guild_only, required_permissions = "ADMINISTRATOR")]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
    )
)]
pub async fn enable(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command can only be used in a server.")
            .instrument(tracing::info_span!("send_error", category = "discord"))
            .await?;
        return Ok(());
    };

    let changed = review::set_review_guild(&ctx.data().pool, guild_id.get(), true).await?;
    let reply = if changed {
        "`/ai-review run` is now enabled in this server."
    } else {
        "`/ai-review run` is already enabled in this server."
    };
    ctx.say(reply)
        .instrument(tracing::info_span!("send_reply", category = "discord"))
        .await?;

    Ok(())
}

/// Disallow /ai-review in this server (Administrator only).
#[poise::command(slash_command, guild_only, required_permissions = "ADMINISTRATOR")]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
    )
)]
pub async fn disable(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command can only be used in a server.")
            .instrument(tracing::info_span!("send_error", category = "discord"))
            .await?;
        return Ok(());
    };

    let changed = review::set_review_guild(&ctx.data().pool, guild_id.get(), false).await?;
    let reply = if changed {
        "`/ai-review run` is now disabled in this server."
    } else {
        "`/ai-review run` wasn't enabled in this server."
    };
    ctx.say(reply)
        .instrument(tracing::info_span!("send_reply", category = "discord"))
        .await?;

    Ok(())
}

/// Request an AI code review of a GitHub pull request.
#[poise::command(slash_command, guild_only, check = "review_available")]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        channel_id = %ctx.channel_id(),
        url = %url,
        pr = %pr,
    )
)]
pub async fn run(ctx: Context<'_>, url: String, pr: u64) -> Result<(), Error> {
    // Parse the URL: accept only https://github.com/<owner>/<repo> with
    // optional trailing slash and optional `.git` suffix.
    let (owner, repo) = match parse_github_url(&url) {
        Ok(parsed) => parsed,
        Err(e) => {
            let _ = ctx
                .say(format!("Invalid URL: {e}"))
                .instrument(tracing::info_span!("send_error", category = "discord"))
                .await;
            return Ok(());
        }
    };

    let user_id = ctx.author().id.get();
    let http = std::sync::Arc::clone(&ctx.serenity_context().http);
    let channel_id = ctx.channel_id();

    // Check if the user already has a cached token.
    if let Some(token) = GITHUB_TOKEN_CACHE.get(&user_id).await {
        // Verify push permission before acquiring the guard.
        match has_push_permission(&token, &owner, &repo).await {
            Ok(true) => {}
            Ok(false) => {
                let _ = ctx
                    .say(format!(
                        "You need push access to `{owner}/{repo}` to request a review."
                    ))
                    .instrument(tracing::info_span!("send_error", category = "discord"))
                    .await;
                return Ok(());
            }
            Err(e) => {
                let _ = ctx
                    .say(format!("Could not verify your GitHub permissions: {e}"))
                    .instrument(tracing::info_span!("send_error", category = "discord"))
                    .await;
                return Ok(());
            }
        }

        // Acquire the global review guard.
        let Some(guard) = try_acquire_review_guard().await else {
            ctx.say("A review is already running — please wait for it to finish.")
                .instrument(tracing::info_span!("send_error", category = "discord"))
                .await?;
            return Ok(());
        };

        ctx.say(format!(
            "Review of `{owner}/{repo}#{pr}` started — I'll post in this channel when it's done."
        ))
        .instrument(tracing::info_span!(
            "send_review_started",
            category = "discord"
        ))
        .await?;

        tokio::spawn(
            async move {
                let _guard = guard;
                run_and_report(http, channel_id, owner, repo, pr).await;
            }
            .instrument(tracing::Span::current()),
        );
    } else {
        // No cached token — start device flow.
        let dc = match start_device_flow().await {
            Ok(dc) => dc,
            Err(e) => {
                let _ = ctx
                    .say(format!("Failed to start GitHub authorization: {e}"))
                    .instrument(tracing::info_span!("send_error", category = "discord"))
                    .await;
                return Ok(());
            }
        };

        ctx.send(
            poise::CreateReply::default()
                .ephemeral(true)
                .content(format!(
                    "Authorize the bot at <{}> and enter code `{}`.\n\
                     The review will start automatically once you approve.",
                    dc.verification_uri, dc.user_code
                )),
        )
        .instrument(tracing::info_span!("send_ephemeral", category = "discord"))
        .await?;

        tokio::spawn(
            async move {
                // Poll until the user approves or the code expires.
                let token = match poll_device_flow(&dc).await {
                    Ok(t) => t,
                    Err(e) => {
                        let _ = channel_id
                            .say(&http, format!("GitHub authorization failed: {e}"))
                            .instrument(tracing::info_span!("send_error", category = "discord"))
                            .await;
                        return;
                    }
                };

                // Cache the token so the user can re-run without auth.
                GITHUB_TOKEN_CACHE.insert(user_id, token.clone()).await;

                // Tell the requester which account got linked, so a wrong-account
                // approval is visible immediately.
                if let Ok(login) = fetch_login(&token).await {
                    let _ = channel_id
                        .say(&http, format!("<@{user_id}> linked GitHub account `{login}`."))
                        .await;
                }

                // Verify push permission.
                match has_push_permission(&token, &owner, &repo).await {
                    Ok(true) => {}
                    Ok(false) => {
                        let _ = channel_id
                            .say(
                                &http,
                                format!("You need push access to `{owner}/{repo}` to request a review."),
                            )
                            .instrument(tracing::info_span!("send_error", category = "discord"))
                            .await;
                        return;
                    }
                    Err(e) => {
                        let _ = channel_id
                            .say(
                                &http,
                                format!("Could not verify your GitHub permissions: {e}"),
                            )
                            .instrument(tracing::info_span!("send_error", category = "discord"))
                            .await;
                        return;
                    }
                }

                // Acquire the global review guard.
                let Some(guard) = try_acquire_review_guard().await else {
                    let _ = channel_id
                        .say(&http, "A review is already running — please wait for it to finish.")
                        .instrument(tracing::info_span!("send_error", category = "discord"))
                        .await;
                    return;
                };

                let _ = channel_id
                    .say(
                        &http,
                        format!("Review of `{owner}/{repo}#{pr}` started — I'll post in this channel when it's done."),
                    )
                    .instrument(tracing::info_span!("send_review_started", category = "discord"))
                    .await;

                let _guard = guard;
                run_and_report(http, channel_id, owner, repo, pr).await;
            }
            .instrument(tracing::Span::current()),
        );
    }

    Ok(())
}

/// Shared: run the review and report the result in the channel.
async fn run_and_report(
    http: Arc<serenity::Http>,
    channel_id: serenity::ChannelId,
    owner: String,
    repo: String,
    pr: u64,
) {
    let bot_token = match get_installation_token(&owner).await {
        Ok(t) => t,
        Err(e) => {
            let _ = channel_id
                .say(
                    &http,
                    format!("Review of `{owner}/{repo}#{pr}` failed: {e}"),
                )
                .await;
            return;
        }
    };

    let owner_msg = owner.clone();
    let repo_msg = repo.clone();
    let result = tokio::spawn(
        async move { review::run_review(owner, repo, pr, bot_token).await }
            .instrument(tracing::info_span!("spawn_review", category = "ai_review",)),
    )
    .await;

    match result {
        Ok(Ok(comment_url)) => {
            let _ = channel_id
                .say(&http, format!("Review posted: {comment_url}"))
                .instrument(tracing::info_span!(
                    "send_review_done",
                    category = "discord"
                ))
                .await;
        }
        Ok(Err(e)) => {
            // Keep the message under Discord's 2000-char cap or the send
            // itself fails and the user gets no feedback at all.
            let mut err_text = e.to_string();
            if err_text.len() > 1600 {
                let mut end = 1600;
                while end > 0 && !err_text.is_char_boundary(end) {
                    end -= 1;
                }
                err_text.truncate(end);
                err_text.push('…');
            }
            let _ = channel_id
                .say(
                    &http,
                    format!(
                        "Review of `{owner_msg}/{repo_msg}#{pr}` failed:\n```\n{err_text}\n```"
                    ),
                )
                .instrument(tracing::info_span!("send_error", category = "discord"))
                .await;
        }
        Err(_) => {
            let _ = channel_id
                .say(&http, "Review panicked — check the logs.")
                .instrument(tracing::info_span!("send_error", category = "discord"))
                .await;
        }
    }
}

// ── URL parsing ─────────────────────────────────────────────────────────────

fn parse_github_url(url: &str) -> Result<(String, String), String> {
    let url = url.trim();
    // Strip protocol prefix.
    let path = url
        .strip_prefix("https://github.com/")
        .ok_or_else(|| "URL must start with `https://github.com/`.".to_string())?;
    // Strip optional trailing slash and `.git` suffix.
    let path = path.trim_end_matches('/').trim_end_matches(".git");
    let mut parts = path.split('/');
    let owner = parts
        .next()
        .ok_or_else(|| "URL must include an owner (e.g. `1Git2Clone`).".to_string())?
        .to_string();
    let repo = parts
        .next()
        .ok_or_else(|| "URL must include a repo name (e.g. `serenity-discord-bot`).".to_string())?
        .to_string();
    if parts.next().is_some() {
        return Err("URL must be of the form `https://github.com/<owner>/<repo>` with no extra path segments.".to_string());
    }
    if owner.is_empty() || repo.is_empty() {
        return Err("Owner and repo must not be empty.".to_string());
    }
    // Owner/repo end up as subprocess arguments — restrict to GitHub's name
    // charset so they can never be parsed as flags (e.g. a leading `-`).
    for part in [&owner, &repo] {
        if part.starts_with('-')
            || !part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        {
            return Err(format!("Invalid owner/repo segment: {part}"));
        }
    }
    Ok((owner, repo))
}

// ── Availability check ──────────────────────────────────────────────────────

async fn review_available(ctx: Context<'_>) -> Result<bool, Error> {
    // Check all required env vars before touching the panicking LazyLock
    // statics — a missing var would poison the Lock and kill the command
    // for the rest of the process lifetime.
    if std::env::var("GITHUB_OAUTH_CLIENT_ID").is_err()
        || std::env::var("GITHUB_APP_ID").is_err()
        || std::env::var("GITHUB_APP_PRIVATE_KEY_PATH").is_err()
    {
        let _ = ctx
            .say("AI review is not configured — contact the bot owner.")
            .instrument(tracing::info_span!("send_error", category = "discord"))
            .await;
        return Ok(false);
    }

    let Some(guild_id) = ctx.guild_id() else {
        let _ = ctx
            .say("This command can only be used in a server.")
            .instrument(tracing::info_span!("send_error", category = "discord"))
            .await;
        return Ok(false);
    };

    if !review::is_review_guild(&ctx.data().pool, guild_id.get()).await {
        let _ = ctx
            .say("AI review isn't enabled in this server — an administrator can turn it on with `/ai-review enable`.")
            .instrument(tracing::info_span!("send_error", category = "discord"))
            .await;
        return Ok(false);
    };

    Ok(true)
}
