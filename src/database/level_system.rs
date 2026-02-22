use crate::prelude::*;

/// Adds a new database user with the schema from `crate::data:bot_data.rs`.
/// That's the reason why the function isn't public.

#[tracing::instrument(
    fields(
        category = "sql",
        db_pool = ?db,
        user_id = %user.id.get(),
        guild_id = %guild_id.get()
    )
)]
async fn add_user_if_not_exists(db: &PgPool, user: &User, guild_id: GuildId) -> Result<(), Error> {
    LevelsTable::add_user_level(
        db,
        user.id.into(),
        guild_id.into(),
        DEFAULT_XP,
        DEFAULT_LEVEL,
    )
    .await?;

    Ok(())
}

/// Adds a db user id + guild id if there's none or updates the pair with the new values.
///
/// The function is the one which is referrred to in the event handler because it's more likely
/// that the user already exists. If it doesn't then we add it by calling add_user_if_not_exists()
/// with its parameters.
///
/// Additionally, we directly use the guild_id instead of the event as the
/// parameter for add_user_if_not_exists() in order to save computing resources.
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "sql",
        db_pool = ?db,
        author = %message.author.id,
        guild_id = ?message.guild_id,
        message = ?message,
        obtained_xp = %obtained_xp
    )
)]
pub async fn add_or_update_db_user(
    db: &PgPool,
    message: &serenity::Message,
    ctx: &serenity::Context,
    obtained_xp: i32,
) -> Result<(), Error> {
    let xp_addition_cooldown: i64 = *XP_COOLDOWN_NUMBER_SECS;
    let current_timestamp = chrono::offset::Utc::now().timestamp();

    let user = &message.author;
    let Some(guild_id) = message.guild_id else {
        return Ok(());
    };

    if USER_COOLDOWNS
        .lock()
        .map_err(|why| format!("{why}"))?
        .get(&(user.id, guild_id))
        .is_some()
    {
        return Ok(());
    }

    USER_COOLDOWNS
        .lock()
        .map(|mut cooldown_timestamps| {
            let key = &(user.id, guild_id);

            let last_rewarded_user_message_timestamp =
                &*cooldown_timestamps.entry(*key).or_insert(current_timestamp);

            if (last_rewarded_user_message_timestamp + xp_addition_cooldown) <= current_timestamp {
                cooldown_timestamps.remove(key);
            }
        })
        .map_err(|why| format!("{why}"))?;

    let xp_lvl_res = LevelsTable::fetch_user_level(db, user.id.into(), guild_id.into()).await;

    let Ok(xp_lvl) = xp_lvl_res else {
        add_user_if_not_exists(db, user, guild_id).await?;
        return Ok(());
    };

    let queried_level = xp_lvl.1;
    let added_experience_points = xp_lvl.0 + obtained_xp;

    let update = update_level(added_experience_points, queried_level).await;

    if update.updated_level > queried_level {
        message
            .reply(
                ctx,
                format!(
                    "{} leveled up to level: {}",
                    user.name, update.updated_level
                ),
            )
            .await?;
    }

    LevelsTable::update_user_level(
        db,
        update.updated_experience,
        update.updated_level,
        user.id.into(),
        guild_id.into(),
    )
    .await?;

    Ok(())
}
