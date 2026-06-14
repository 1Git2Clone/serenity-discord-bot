//! `/custom reaction` command: register an image+regex reaction, or remove one.

use crate::{data::custom_reactions, prelude::*};

/// Strip Discord CDN attachment signing query params (`?ex=&is=&hm=`).
fn strip_cdn_signing(url: &str) -> String {
    if url.starts_with("https://cdn.discordapp.com/attachments/")
        && let Some(base) = url.split('?').next()
    {
        return base.to_string();
    }
    url.to_string()
}

// ── Autocomplete ──────────────────────────────────────────────────────────────

async fn autocomplete_reaction(ctx: Context<'_>, partial: &str) -> Vec<String> {
    let Some(guild_id) = ctx.guild_id() else {
        return vec![];
    };
    custom_reactions::autocomplete_reactions(&ctx.data().pool, guild_id.get() as i64, partial).await
}

// ── Subcommands ───────────────────────────────────────────────────────────────

/// Register a new custom reaction (url or attachment + regex pattern).
#[poise::command(
    slash_command,
    rename = "add",
    required_permissions = "MANAGE_CHANNELS",
    guild_only
)]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().qualified_name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
    )
)]
async fn custom_reaction_add(
    ctx: Context<'_>,
    #[description = "Image URL to show when the pattern matches"] url: Option<String>,
    #[description = "Image attachment to show when the pattern matches"] attachment: Option<
        serenity::Attachment,
    >,
    #[description = "Rust regex (no lookahead/backref). Use (?i) for case-insensitive."]
    pattern: String,
    #[description = "If true, matches anywhere in the message; if false, must match the whole trimmed message."]
    anywhere: Option<bool>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command can only be used in a server.")
            .await?;
        return Ok(());
    };

    let image_url = match (url, attachment) {
        (Some(_), Some(_)) => {
            ctx.say("Provide either `url` or `attachment`, not both.")
                .await?;
            return Ok(());
        }
        (None, None) => {
            ctx.say("Provide either `url` or `attachment`.").await?;
            return Ok(());
        }
        (Some(u), None) => strip_cdn_signing(&u),
        (None, Some(a)) => strip_cdn_signing(&a.url),
    };

    let anywhere = anywhere.unwrap_or(false);

    // Reject page/share links that render blank in an embed before storing.
    if let Err(msg) = custom_reactions::validate_image_url(&image_url) {
        ctx.say(msg).await?;
        return Ok(());
    }

    // Validate pattern before writing to the DB.
    if let Err(msg) = custom_reactions::compile_pattern(&pattern, anywhere) {
        ctx.say(msg).await?;
        return Ok(());
    }

    let (_id, seq) = custom_reactions::register(
        &ctx.data().pool,
        guild_id.get() as i64,
        &pattern,
        &image_url,
        anywhere,
    )
    .await?;

    ctx.say(format!(
        "Registered reaction #{seq} — pattern `{pattern}` (anywhere: {anywhere})."
    ))
    .await?;

    Ok(())
}

/// Remove a custom reaction by choosing from the autocomplete list.
#[poise::command(
    slash_command,
    rename = "remove",
    required_permissions = "MANAGE_CHANNELS",
    guild_only
)]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().qualified_name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
    )
)]
async fn custom_reaction_remove(
    ctx: Context<'_>,
    #[description = "Which reaction to remove (pick from the list)"]
    #[autocomplete = "autocomplete_reaction"]
    name: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command can only be used in a server.")
            .await?;
        return Ok(());
    };

    // Autocomplete value is "{seq} — {preview}". Parse the leading per-guild
    // number; keep the preview (if present) so `remove` can confirm the row at
    // that number still matches the one that was picked — a plain typed number
    // has no preview and skips that check.
    let (seq_str, expected_preview) = match name.split_once(" — ") {
        Some((seq, preview)) => (seq.trim(), Some(preview)),
        None => (name.trim(), None),
    };
    let Some(seq) = seq_str.parse::<i64>().ok() else {
        ctx.say("Couldn't parse that selection — please pick one from the autocomplete list.")
            .await?;
        return Ok(());
    };

    use custom_reactions::RemoveOutcome;
    match custom_reactions::remove(
        &ctx.data().pool,
        guild_id.get() as i64,
        seq,
        expected_preview,
    )
    .await?
    {
        RemoveOutcome::Removed(pattern) => {
            ctx.say(format!("Removed reaction #{seq} — pattern `{pattern}`."))
                .await?;
        }
        RemoveOutcome::NotFound => {
            ctx.say(format!(
                "No reaction #{seq} in this server — run `/custom reaction list` to see current numbers."
            ))
            .await?;
        }
        RemoveOutcome::Changed => {
            ctx.say(
                "The reaction list changed since you opened it — run `/custom reaction list` and try again.",
            )
            .await?;
        }
    }

    Ok(())
}

/// Short, recognizable hint for a stored image URL: host plus the final path
/// segment, query string dropped — so a listing stays readable instead of
/// dumping long signed CDN URLs.
fn url_hint(url: &str) -> String {
    let after_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    let no_query = after_scheme
        .split(['?', '#'])
        .next()
        .unwrap_or(after_scheme);
    let (host, path) = no_query.split_once('/').unwrap_or((no_query, ""));
    let path = path.trim_end_matches('/');
    let hint = match path.rsplit('/').find(|s| !s.is_empty()) {
        Some(last) if path.contains('/') => format!("{host}/.../{last}"),
        Some(last) => format!("{host}/{last}"),
        None => host.to_string(),
    };
    // An empty/scheme-only URL would yield a blank hint and a dangling " — ".
    if hint.is_empty() {
        "(no url)".to_string()
    } else {
        hint
    }
}

/// List every custom reaction in the server with its per-guild number.
#[poise::command(
    slash_command,
    rename = "list",
    required_permissions = "MANAGE_CHANNELS",
    guild_only
)]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().qualified_name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
    )
)]
async fn custom_reaction_list(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command can only be used in a server.")
            .await?;
        return Ok(());
    };

    let entries = custom_reactions::list_live(&ctx.data().pool, guild_id.get() as i64).await;

    let body = if entries.is_empty() {
        "No custom reactions in this server. Add one with `/custom reaction add`.".to_string()
    } else {
        // Stay under Discord's 2000-char message limit; with the 25-per-guild
        // cap this only bites on pathologically long patterns/URLs.
        const MAX_LEN: usize = 1900;
        let mut out = format!("Custom reactions ({}):", entries.len());
        // `i` doubles as the count already shown: at index `i` the prior `i`
        // entries have been appended.
        for (i, e) in entries.iter().enumerate() {
            let line = format!(
                "\n{}. `{}` (anywhere: {}) — {}",
                i + 1,
                custom_reactions::pattern_preview(&e.pattern),
                e.anywhere,
                url_hint(&e.image_url)
            );
            if out.len() + line.len() > MAX_LEN {
                out.push_str(&format!(
                    "\n…and {} more — too long to show.",
                    entries.len() - i
                ));
                break;
            }
            out.push_str(&line);
        }
        out
    };

    // Ephemeral: the listing exposes the stored image URLs, so keep it to the
    // invoking moderator.
    ctx.send(poise::CreateReply::default().content(body).ephemeral(true))
        .await?;
    Ok(())
}

/// Manage custom image reactions triggered by regex patterns.
#[poise::command(
    slash_command,
    rename = "reaction",
    subcommands(
        "custom_reaction_add",
        "custom_reaction_remove",
        "custom_reaction_list"
    ),
    required_permissions = "MANAGE_CHANNELS",
    guild_only
)]
pub async fn custom_reaction(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Use `/custom reaction add`, `/custom reaction remove`, or `/custom reaction list`.")
        .await?;
    Ok(())
}

/// Top-level `/custom` group.
#[poise::command(
    slash_command,
    subcommands("custom_reaction"),
    required_permissions = "MANAGE_CHANNELS",
    guild_only
)]
pub async fn custom(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Use `/custom reaction add`, `/custom reaction remove`, or `/custom reaction list`.")
        .await?;
    Ok(())
}
