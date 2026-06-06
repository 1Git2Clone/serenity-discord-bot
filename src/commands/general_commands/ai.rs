use std::time::Duration;

use ::serenity::all::GetMessages;
use tokio::time::sleep;

use crate::{
    data::ai::{self, AI_CHANNEL_CACHE, AI_MAX_MSG_CONTEXT, AI_RATE_LIMIT, AI_RATE_LIMIT_SECS},
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
    let Some(__guard) = AI_CHANNEL_CACHE.try_acquire(channel_id.get()) else {
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

    if AI_RATE_LIMIT.get(&ctx.author().id).await.is_some() {
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

    AI_RATE_LIMIT.insert(ctx.author().id, ()).await;

    ctx.defer().await?;

    let messages = match channel_id
        // Discord caps a single fetch at 100; pagination would be needed beyond that.
        .messages(
            &ctx.http(),
            GetMessages::new().limit((*AI_MAX_MSG_CONTEXT).min(100) as u8),
        )
        .await
    {
        Ok(msgs) => msgs
            .into_iter()
            .filter(|m| !m.author.bot || m.author.id.get() == ctx.data().bot_user.id.get())
            .rev()
            .collect(),
        Err(e) => {
            tracing::info!("Failed to get messages! (Error: {})", e);
            vec![]
        }
    };

    let prompt = ai::messages_to_prompt(&messages, ctx.data().bot_user.id.get(), &message);
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
        ctx.say("This command can only be used in a server.").await?;
        return Ok(());
    };

    let enabled =
        ai::toggle_ai_channel(&ctx.data().pool, channel_id.get(), guild_id.get()).await?;

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
