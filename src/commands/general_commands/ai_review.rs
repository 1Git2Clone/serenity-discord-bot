use tracing::Instrument;

use crate::{
    data::ai::review::{
        self,
        config::AI_REVIEW_GUARD,
        github::{
            fetch_login, get_installation_token, has_push_permission,
            poll_device_flow, start_device_flow, GITHUB_TOKEN_CACHE,
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
        ctx.say("This command can only be used in a server.").await?;
        return Ok(());
    };

    let changed = review::set_review_guild(&ctx.data().pool, guild_id.get(), true).await?;
    let reply = if changed {
        "`/ai-review run` is now enabled in this server."
    } else {
        "`/ai-review run` is already enabled in this server."
    };
    ctx.say(reply).await?;

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
        ctx.say("This command can only be used in a server.").await?;
        return Ok(());
    };

    let changed = review::set_review_guild(&ctx.data().pool, guild_id.get(), false).await?;
    let reply = if changed {
        "`/ai-review run` is now disabled in this server."
    } else {
        "`/ai-review run` wasn't enabled in this server."
    };
    ctx.say(reply).await?;

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
            let _ = ctx.say(format!("Invalid URL: {e}")).await;
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
                    .await;
                return Ok(());
            }
            Err(e) => {
                let _ = ctx
                    .say(format!("Could not verify your GitHub permissions: {e}"))
                    .await;
                return Ok(());
            }
        }

        // Acquire the global review guard.
        let Some(guard) = AI_REVIEW_GUARD.try_acquire(0) else {
            ctx.say("A review is already running — please wait for it to finish.")
                .await?;
            return Ok(());
        };

        ctx.say(format!(
            "Review of `{owner}/{repo}#{pr}` started — I'll post in this channel when it's done."
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
        .await?;

        tokio::spawn(
            async move {
                // Poll until the user approves or the code expires.
                let token = match poll_device_flow(&dc).await {
                    Ok(t) => t,
                    Err(e) => {
                        let _ = channel_id
                            .say(
                                &http,
                                format!("GitHub authorization wasn't completed: {e}"),
                            )
                            .await;
                        return;
                    }
                };

                // Store the token in the cache.
                GITHUB_TOKEN_CACHE.insert(user_id, token.clone()).await;

                // Tell the requester which account got linked, so a wrong-account
                // approval is visible immediately.
                if let Ok(login) = fetch_login(&token).await {
                    let _ = channel_id
                        .say(
                            &http,
                            format!("<@{user_id}> linked GitHub account `{login}`."),
                        )
                        .await;
                }

                // Verify push permission.
                match has_push_permission(&token, &owner, &repo).await {
                    Ok(true) => {}
                    Ok(false) => {
                        let _ = channel_id
                            .say(
                                &http,
                                format!(
                                    "You need push access to `{owner}/{repo}` to request a review."
                                ),
                            )
                            .await;
                        return;
                    }
                    Err(e) => {
                        let _ = channel_id
                            .say(&http, format!("Could not verify your GitHub permissions: {e}"))
                            .await;
                        return;
                    }
                }

                // Acquire the global review guard.
                let Some(guard) = AI_REVIEW_GUARD.try_acquire(0) else {
                    let _ = channel_id
                        .say(&http, "A review is already running — please wait for it to finish.")
                        .await;
                    return;
                };

                let _ = channel_id
                    .say(
                        &http,
                        format!(
                            "Review of `{owner}/{repo}#{pr}` started — I'll post in this channel when it's done."
                        ),
                    )
                    .await;

                let _guard = guard;
                run_and_report(http, channel_id, owner, repo, pr).await;
            }
            .instrument(tracing::Span::current()),
        );
    }

    Ok(())
}

// ── Shared review runner ─────────────────────────────────────────────────────

async fn run_and_report(
    http: std::sync::Arc<serenity::http::Http>,
    channel_id: serenity::model::id::ChannelId,
    owner: String,
    repo: String,
    pr: u64,
) {
    let owner_span = owner.clone();
    let repo_span = repo.clone();
    async move {
    let bot_token = match get_installation_token(&owner)
        .instrument(tracing::info_span!(
            "get_installation_token",
            category = "ai_review",
            owner = %owner,
        ))
        .await
    {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(
                category = "ai_review",
                owner = %owner,
                repo = %repo,
                pr = %pr,
                error = %e,
                "Failed to generate installation token"
            );
            let _ = channel_id
                .say(
                    &http,
                    format!("Review of `{owner}/{repo}#{pr}` failed: {e}"),
                )
                .await;
            return;
        }
    };
    let result = review::run_review(owner.clone(), repo.clone(), pr, bot_token)
        .instrument(tracing::info_span!(
            "run_review",
            category = "ai_review",
            owner = %owner,
            repo = %repo,
            pr = %pr,
        ))
        .await;

    match result {
        Ok(comment_url) => {
            tracing::info!(
                category = "ai_review",
                owner = %owner,
                repo = %repo,
                pr = %pr,
                "Review posted: {comment_url}"
            );
            let _ = channel_id
                .say(
                    &http,
                    format!("Review of `{owner}/{repo}#{pr}` posted: {comment_url}"),
                )
                .await;
        }
        Err(e) => {
            tracing::error!(
                category = "ai_review",
                owner = %owner,
                repo = %repo,
                pr = %pr,
                error = %e,
                "Review failed"
            );
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
                    format!("Review of `{owner}/{repo}#{pr}` failed:\n```\n{err_text}\n```"),
                )
                .await;
        }
    }
    }.instrument(tracing::info_span!(
        "run_and_report",
        category = "ai_review",
        owner = %owner_span,
        repo = %repo_span,
        pr = %pr,
    ))
    .await;
}

// ── URL parser ──────────────────────────────────────────────────────────────

fn parse_github_url(url: &str) -> Result<(String, String), String> {
    let url = url.trim_end_matches('/');
    let url = url.strip_suffix(".git").unwrap_or(url);
    let prefix = "https://github.com/";
    let rest = url
        .strip_prefix(prefix)
        .ok_or_else(|| format!("URL must start with {prefix}"))?;
    let parts: Vec<&str> = rest.split('/').filter(|p| !p.is_empty()).collect();
    if parts.len() != 2 {
        return Err(format!(
            "Expected URL format: https://github.com/<owner>/<repo>, got: {url}"
        ));
    }
    // Owner/repo end up as subprocess arguments — restrict to GitHub's name
    // charset so they can never be parsed as flags (e.g. a leading `-`).
    for part in &parts {
        if part.starts_with('-')
            || !part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        {
            return Err(format!("Invalid owner/repo segment: {part}"));
        }
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

// ── Availability check ──────────────────────────────────────────────────────

async fn review_available(ctx: Context<'_>) -> Result<bool, Error> {
    // Check all required env vars before touching the panicking LazyLock
    // statics — a missing var would poison the Lock and kill the command
    // for the whole process lifetime.
    for var in &[
        "GITHUB_OAUTH_CLIENT_ID",
        "GITHUB_APP_ID",
        "GITHUB_APP_PRIVATE_KEY_PATH",
    ] {
        if std::env::var(var).is_err() {
            let _ = ctx
                .say(format!(
                    "AI review is not configured (`{var}` is unset)."
                ))
                .await;
            return Ok(false);
        }
    }
    let Some(guild_id) = ctx.guild_id() else {
        let _ = ctx.say("This command can only be used in a server.").await;
        return Ok(false);
    };
    if !review::is_review_guild(guild_id.get()) {
        let _ = ctx
            .say("AI review isn't enabled in this server — an administrator can turn it on with `/ai-review enable`.")
            .await;
        return Ok(false);
    }
    Ok(true)
}
