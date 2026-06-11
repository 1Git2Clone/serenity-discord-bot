use crate::{
    data::ai::review::{self, config::AI_REVIEW_GUARD},
    prelude::*,
};

/// Request an AI code review of a GitHub pull request.
#[poise::command(slash_command, rename = "ai-review", guild_only, check = "has_review_role")]
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
pub async fn ai_review(ctx: Context<'_>, url: String, pr: u64) -> Result<(), Error> {
    // Parse the URL: accept only https://github.com/<owner>/<repo> with
    // optional trailing slash and optional `.git` suffix.
    let (owner, repo) = match parse_github_url(&url) {
        Ok(parsed) => parsed,
        Err(e) => {
            let _ = ctx.say(format!("Invalid URL: {e}")).await;
            return Ok(());
        }
    };

    // Acquire the global review guard (one review at a time).
    let Some(guard) = AI_REVIEW_GUARD.try_acquire(0) else {
        ctx.say("A review is already running — please wait for it to finish.")
            .await?;
        return Ok(());
    };

    ctx.say(format!(
        "Review of `{owner}/{repo}#{pr}` started — I'll post in this channel when it's done."
    ))
    .await?;

    let http = std::sync::Arc::clone(&ctx.serenity_context().http);
    let channel_id = ctx.channel_id();

    // Spawn the pipeline so the interaction can return before the 15-minute
    // token expiry.
    tokio::spawn(async move {
        // Move the guard into the task so it's held for the full duration.
        let _guard = guard;

        let result = review::run_review(owner.clone(), repo.clone(), pr).await;

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
                    .say(&http, format!("Review of `{owner}/{repo}#{pr}` posted: {comment_url}"))
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
                        format!(
                            "Review of `{owner}/{repo}#{pr}` failed:\n```\n{err_text}\n```"
                        ),
                    )
                    .await;
            }
        }
    });

    Ok(())
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

// ── Role check ──────────────────────────────────────────────────────────────

async fn has_review_role(ctx: Context<'_>) -> Result<bool, Error> {
    // The review env vars are optional (the rest of the `ai` feature works
    // without them), so a missing var must not hit the panicking statics —
    // that would poison the `LazyLock` and kill the command for the whole
    // process lifetime.
    if std::env::var("GITHUB_APP_TOKEN").is_err() || std::env::var("AI_REVIEW_ROLE").is_err() {
        let _ = ctx
            .say("AI review is not configured (`GITHUB_APP_TOKEN` / `AI_REVIEW_ROLE` are unset).")
            .await;
        return Ok(false);
    }
    let role_id = serenity::RoleId::new(*review::config::AI_REVIEW_ROLE);
    let Some(member) = ctx.author_member().await else {
        let _ = ctx
            .say("This command is only available to server members.")
            .await;
        return Ok(false);
    };
    if !member.roles.contains(&role_id) {
        let _ = ctx
            .say("You don't have permission to use `/ai-review`.")
            .await;
        return Ok(false);
    }
    Ok(true)
}
