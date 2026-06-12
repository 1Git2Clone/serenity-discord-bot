use std::time::Duration;

use tokio::time::sleep;

use crate::{
    data::ai::{self, AI_RATE_LIMIT_SECS, check_ai_rate_limit, try_acquire_channel_lock},
    prelude::*,
};

/// Yap with an AI!
#[poise::command(slash_command, prefix_command, rename = "ai")]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        channel_id = %ctx.channel_id(),
        message = %message
    )
)]
pub async fn ai(ctx: Context<'_>, message: String) -> Result<(), Error> {
    let channel_id = ctx.channel_id();
    let cid = channel_id.get();

    let Some(_lock) = try_acquire_channel_lock(cid).await else {
        tracing::info!(
            "User tried to call the AI in {channel_id} while it's still processing content from within it."
        );
        let already_processing_msg = ctx
            .say(format!(
                "Already processing a prompt in <#{}>...",
                channel_id.get()
            ))
            .await?;

        sleep(Duration::from_secs(3)).await;
        already_processing_msg.delete(ctx).await?;

        return Ok(());
    };

    if check_ai_rate_limit(ctx.author().id.get()).await {
        let rate_limit_msg = ctx
            .say(format!(
                "Rate limited <@{}>. Please wait {} seconds between each prompt.",
                ctx.author().id.get(),
                AI_RATE_LIMIT_SECS
            ))
            .await?;

        sleep(Duration::from_secs(5)).await;
        rate_limit_msg.delete(ctx).await?;

        return Ok(());
    }

    ctx.defer().await?;

    let who = ai::author_name(ctx.author());
    let prompt = ai::channel_context(
        ctx.serenity_context(),
        channel_id,
        ctx.data().bot_user.id.get(),
        Some(&format!("{who}: {message}")),
    )
    .await;
    let response = ai::chat(&prompt).await?;

    ctx.say(response).await?;

    Ok(())
}

/// Toggle AI auto-replies for this channel (Manage Channels only).
#[poise::command(
    slash_command,
    prefix_command,
    rename = "aichannel",
    required_permissions = "MANAGE_CHANNELS",
    guild_only
)]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        channel_id = %ctx.channel_id(),
    )
)]
pub async fn aichannel(ctx: Context<'_>) -> Result<(), Error> {
    let channel_id = ctx.channel_id();
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command can only be used in a server.")
            .await?;
        return Ok(());
    };

    let enabled = ai::toggle_ai_channel(&ctx.data().pool, channel_id.get(), guild_id.get()).await?;

    let reply = if enabled {
        format!(
            "Aiya! I'll reply to every message in <#{}> now~",
            channel_id.get()
        )
    } else {
        format!("I'll stop haunting <#{}>. Farewell~", channel_id.get())
    };
    ctx.say(reply).await?;

    Ok(())
}
