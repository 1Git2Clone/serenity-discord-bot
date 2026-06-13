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

    // Validate pattern before writing to the DB.
    if let Err(msg) = custom_reactions::compile_pattern(&pattern, anywhere) {
        ctx.say(msg).await?;
        return Ok(());
    }

    let id = custom_reactions::register(
        &ctx.data().pool,
        guild_id.get() as i64,
        &pattern,
        &image_url,
        anywhere,
    )
    .await?;

    ctx.say(format!(
        "Registered reaction #{id} — pattern `{pattern}` (anywhere: {anywhere})."
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

    // Autocomplete value is "{id} — {pattern}"; parse the leading id.
    let reaction_id: i64 = match name.split_whitespace().next().and_then(|s| s.parse().ok()) {
        Some(id) => id,
        None => {
            ctx.say("Couldn't parse that selection — please pick one from the autocomplete list.")
                .await?;
            return Ok(());
        }
    };

    let deleted =
        custom_reactions::remove(&ctx.data().pool, reaction_id, guild_id.get() as i64).await?;

    if deleted {
        ctx.say(format!("Removed reaction #{reaction_id}.")).await?;
    } else {
        ctx.say("No matching live reaction found — it may have already been removed.")
            .await?;
    }

    Ok(())
}

/// Manage custom image reactions triggered by regex patterns.
#[poise::command(
    slash_command,
    rename = "reaction",
    subcommands("custom_reaction_add", "custom_reaction_remove"),
    required_permissions = "MANAGE_CHANNELS",
    guild_only
)]
pub async fn custom_reaction(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Use `/custom reaction add` or `/custom reaction remove`.")
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
    ctx.say("Use `/custom reaction add` or `/custom reaction remove`.")
        .await?;
    Ok(())
}
