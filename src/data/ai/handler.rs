use tracing::Instrument;

use super::{
    channels::is_ai_channel,
    config::{check_ai_rate_limit, try_acquire_channel_lock},
    context::channel_context,
    provider::chat,
};
use crate::prelude::*;

/// Reply to a message, honoring the per-user rate limit and the per-channel
/// processing lock. Triggers in DMs, /aichannel-registered channels, and any
/// channel where the bot is mentioned.
#[tracing::instrument(
    skip(ctx, data, new_message),
    fields(
        category = "ai_auto_reply",
        author = %new_message.author.id,
        channel_id = %new_message.channel_id,
    )
)]
pub async fn handle_ai_channel_message(
    ctx: &serenity::Context,
    data: &Data,
    new_message: &serenity::Message,
) -> Result<(), Error> {
    let is_dm = new_message.guild_id.is_none();
    let is_mentioned = new_message
        .mentions
        .iter()
        .any(|u| u.id == data.bot_user.id);
    if !is_dm && !is_ai_channel(&data.pool, new_message.channel_id.get()).await && !is_mentioned {
        return Ok(());
    }

    if check_ai_rate_limit(new_message.author.id.get()).await {
        return Ok(());
    }

    let channel_id = new_message.channel_id.get();
    let Some(_lock) = try_acquire_channel_lock(channel_id).await else {
        return Ok(());
    };

    new_message
        .channel_id
        .broadcast_typing(ctx)
        .instrument(tracing::info_span!("broadcast_typing", category = "discord"))
        .await
        .ok();

    // The triggering message is already in the window (recorded in `handle_message`
    // before this runs), so no trailing turn is appended.
    let prompt = channel_context(ctx, new_message.channel_id, data.bot_user.id.get(), None).await;
    let response = chat(&prompt).await?;

    new_message
        .reply(ctx, response)
        .instrument(tracing::info_span!("discord_reply", category = "discord"))
        .await?;

    Ok(())
}
