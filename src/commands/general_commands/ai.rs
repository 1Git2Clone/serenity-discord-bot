use std::time::Duration;

use ::serenity::all::{GetMessages, Message};
use tokio::time::sleep;

use crate::{
    data::ai::{
        self, AI_CHANNEL_CACHE, AI_MAX_MSG_CONTEXT, AI_RATE_LIMIT, AI_RATE_LIMIT_SECS, AiMessage,
    },
    prelude::*,
};

fn make_prompt(
    ctx: &Context<'_>,
    previous_messages: &[Message],
    current_message: String,
) -> Vec<AiMessage> {
    // The persona is baked into the provider (`ai::SYSTEM_PROMPT`); only the
    // conversation belongs here.
    let mut res = Vec::with_capacity(previous_messages.len() + 1);

    for m in previous_messages {
        res.push(AiMessage::new(
            if m.author.id.get() == ctx.data().bot_user.id.get() {
                "assistant"
            } else {
                "user"
            },
            &m.content,
        ));
    }

    res.push(AiMessage::new("user", &current_message));

    res
}

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

    let prompt = make_prompt(&ctx, &messages, message);
    let response = ai::chat(&prompt).await?;

    ctx.say(response).await?;

    Ok(())
}
