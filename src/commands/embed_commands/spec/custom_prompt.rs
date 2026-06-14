//! `/custom prompt` command: manage this server's extra AI system prompt.

use crate::{data::ai, prelude::*};

/// Set or replace this server's extra AI system prompt.
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
async fn custom_prompt_add(
    ctx: Context<'_>,
    #[description = "Instructions appended to the AI's system prompt for this server."]
    text: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command can only be used in a server.")
            .await?;
        return Ok(());
    };

    let text = text.trim();
    if text.is_empty() {
        ctx.say("The prompt can't be empty.").await?;
        return Ok(());
    }
    if text.len() > ai::MAX_PROMPT_LEN {
        ctx.say(format!(
            "Prompt is {} chars; the limit is {}.",
            text.len(),
            ai::MAX_PROMPT_LEN
        ))
        .await?;
        return Ok(());
    }

    ai::set_guild_prompt(&ctx.data().pool, guild_id.get() as i64, text).await?;

    ctx.say(format!(
        "Set this server's custom AI prompt ({} chars).",
        text.len()
    ))
    .await?;
    Ok(())
}

/// Show this server's current extra AI system prompt.
#[poise::command(
    slash_command,
    rename = "show",
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
async fn custom_prompt_show(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command can only be used in a server.")
            .await?;
        return Ok(());
    };

    let body = match ai::get_guild_prompt(&ctx.data().pool, guild_id.get() as i64).await {
        Some(prompt) => format!("Custom AI prompt for this server:\n\n{prompt}"),
        None => "No custom AI prompt set. Add one with `/custom prompt add`.".to_string(),
    };

    // Ephemeral: server config the invoking moderator asked for, no need to
    // clutter the channel.
    ctx.send(poise::CreateReply::default().content(body).ephemeral(true))
        .await?;
    Ok(())
}

/// Remove this server's extra AI system prompt.
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
async fn custom_prompt_remove(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command can only be used in a server.")
            .await?;
        return Ok(());
    };

    let reply = if ai::delete_guild_prompt(&ctx.data().pool, guild_id.get() as i64).await? {
        "Removed this server's custom AI prompt."
    } else {
        "No custom AI prompt set — nothing to remove."
    };
    ctx.say(reply).await?;
    Ok(())
}

/// Manage this server's extra AI system prompt.
#[poise::command(
    slash_command,
    rename = "prompt",
    subcommands("custom_prompt_add", "custom_prompt_show", "custom_prompt_remove"),
    required_permissions = "MANAGE_CHANNELS",
    guild_only
)]
pub async fn custom_prompt(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Use `/custom prompt add`, `/custom prompt show`, or `/custom prompt remove`.")
        .await?;
    Ok(())
}
