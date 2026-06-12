use crate::{
    prelude::*,
    utils::{
        replies::{handle_replies, levenshtein_cmd},
        string_manipulation::remove_emojis_and_embeds_from_str,
    },
};

#[tracing::instrument(
    skip(ctx),
    fields(
        category = "sql",
        db_pool = ?pool,
        author = %new_message.author.id,
        guild_id = ?new_message.guild_id,
        message = ?new_message,
    )
)]
pub async fn handle_database_message_processing(
    ctx: &serenity::Context,
    new_message: &serenity::Message,
    msg: &str,
    pool: &PgPool,
) -> Result<(), Error> {
    let trimmed_emojis = remove_emojis_and_embeds_from_str(msg);

    let obtained_xp = rand::rng().random_range(XP_RANGE);

    if VALID_MENTION_COUNT_PATTERNS
        .iter()
        .any(|text| trimmed_emojis.contains(text))
    {
        handle_replies(pool, ctx, new_message, &trimmed_emojis).await?;
    }

    add_or_update_db_user(pool, new_message, ctx, obtained_xp).await?;

    Ok(())
}

#[tracing::instrument(
    skip(ctx),
    fields(
        category = "message_helper",
        author = %new_message.author.id,
        guild_id = ?new_message.guild_id,
        message = ?new_message,
    )
)]
pub async fn handle_message(
    ctx: &serenity::Context,
    data: &Data,
    new_message: &serenity::Message,
) -> Result<(), Error> {
    // Keep warm channels' context windows fresh — including our own replies, which
    // are filtered out below — but never other bots.
    #[cfg(feature = "ai")]
    if !new_message.author.bot || new_message.author.id == data.bot_user.id {
        crate::data::ai::record_message(new_message).await;
    }

    if new_message.author.bot {
        return Ok(());
    }
    let msg = new_message.content.to_lowercase();

    levenshtein_cmd(ctx, new_message, &data.available_commands).await?;
    handle_database_message_processing(ctx, new_message, &msg, &data.pool).await?;

    // In registered AI channels, reply to the message (rate-limited per user, one
    // at a time per channel). The channel lock dedupes against the `/ai` command.
    #[cfg(feature = "ai")]
    crate::data::ai::handle_ai_channel_message(ctx, data, new_message).await?;

    Ok(())
}
