use crate::{
    prelude::*,
    utils::{
        replies::{handle_replies, levenshtein_cmd},
        string_manipulation::{count_links, remove_emojis_and_embeds_from_str},
    },
};

/// The guild's name for the message span, falling back to something
/// identifying rather than to nothing.
///
/// Discord only ever sends ids on the wire, so a dashboard grouping on
/// `guild_id` is a list of snowflakes; this is what makes that list readable.
/// A DM has no guild at all, and the cache can be cold for one we are in but
/// have not seen an event from yet — hence the snowflake as the last resort,
/// which is still a value the guild dashboard can be pointed at.
fn guild_name(ctx: &serenity::Context, new_message: &serenity::Message) -> String {
    let Some(id) = new_message.guild_id else {
        return "DM".to_owned();
    };
    ctx.cache
        .guild(id)
        .map(|g| g.name.clone())
        .unwrap_or_else(|| id.get().to_string())
}

#[tracing::instrument(
    skip_all,
    fields(
        category = "sql",
        db_pool = ?pool,
        author = %new_message.author.id,
        guild_id = %new_message.guild_id.map(GuildId::get).unwrap_or(0),
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
    skip_all,
    fields(
        category = "message_helper",
        author = %new_message.author.id,
        // The unique username, not the per-guild display name: it is what a
        // human types to find someone, and it does not differ between guilds.
        author_name = %new_message.author.name,
        guild_id = %new_message.guild_id.map(GuildId::get).unwrap_or(0),
        guild_name = %guild_name(ctx, new_message),
        channel_id = %new_message.channel_id,
        // No `%`: a sigil makes these strings, and a string cannot be summed
        // by the backend. Recorded as integers so `sum_over_time()` works.
        attachments = new_message.attachments.len() as u64,
        links = count_links(&new_message.content) as u64,
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
        crate::data::ai::record_message(new_message, data.bot_user.id.get()).await;
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

    // Send custom-reaction embeds when the message content matches.
    #[cfg(feature = "redis")]
    send_custom_reactions(ctx, data, new_message).await?;

    Ok(())
}

/// Reply with the red bot-tag embed for every custom reaction whose pattern
/// matches the message, one per match, ordered by id. The matching logic and
/// cache live in [`crate::data::custom_reactions`]; this only does the Discord
/// I/O.
#[cfg(feature = "redis")]
async fn send_custom_reactions(
    ctx: &serenity::Context,
    data: &Data,
    new_message: &serenity::Message,
) -> Result<(), Error> {
    let Some(guild_id) = new_message.guild_id else {
        return Ok(());
    };
    let content = new_message.content.trim();
    let matched =
        crate::data::custom_reactions::matching(&data.pool, guild_id.get() as i64, content).await?;
    for reaction in matched {
        let embed = serenity::CreateEmbed::new()
            .color((255, 0, 0))
            .image(&reaction.image_url)
            .footer(
                serenity::CreateEmbedFooter::new(data.bot_user.tag())
                    .icon_url(data.bot_avatar.to_string()),
            );
        let reply = serenity::CreateMessage::new().embed(embed);
        if let Err(e) = new_message.channel_id.send_message(ctx, reply).await {
            tracing::warn!(error = %e, reaction_id = reaction.id, "Failed to send reaction embed");
        }
    }
    Ok(())
}
