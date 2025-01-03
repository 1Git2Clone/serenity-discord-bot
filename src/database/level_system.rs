use crate::{
    data::database::{
        ADD_USER_LEVEL_QUERY, FETCH_TOP_NINE_USERS_IN_GUILD_QUERY, FETCH_USER_LEVEL_AND_RANK_QUERY,
        FETCH_USER_LEVEL_QUERY, UPDATE_USER_LEVEL_QUERY,
    },
    prelude::*,
};

/// Adds a new database user with the schema from `crate::data:bot_data.rs`.
/// That's the reason why the function isn't public.
async fn add_user_if_not_exists(
    db: &SqlitePool,
    user: &User,
    guild_id: GuildId,
) -> Result<(), Error> {
    sqlx::query(&ADD_USER_LEVEL_QUERY)
        .bind(user.id.to_string())
        .bind(guild_id.to_string())
        .bind(DEFAULT_XP)
        .bind(DEFAULT_LEVEL)
        .execute(db)
        .await?;

    Ok(())
}

pub async fn fetch_user_level(
    db: &SqlitePool,
    user: &User,
    guild_id: GuildId,
) -> Result<Option<SqliteRow>, Error> {
    let res = sqlx::query(&FETCH_USER_LEVEL_QUERY)
        .bind(user.id.to_string())
        .bind(guild_id.to_string())
        .fetch_optional(db)
        .await?;

    Ok(res)
}

pub async fn fetch_user_level_and_rank(
    db: &SqlitePool,
    user: &User,
    guild_id: serenity::GuildId,
) -> Result<Option<(i64, SqliteRow)>, Error> {
    let sql = sqlx::query(&FETCH_USER_LEVEL_AND_RANK_QUERY)
        .bind(user.id.to_string())
        .bind(guild_id.to_string())
        .fetch_optional(db)
        .await?;

    match sql {
        Some(row) => Ok(Some((
            row.get::<i64, &str>(LevelsSchema::Rank.as_str()),
            row,
        ))),
        None => Ok(None),
    }
}
pub async fn fetch_top_nine_levels_in_guild(
    db: &SqlitePool,
    guild_id: serenity::GuildId,
) -> Result<Vec<SqliteRow>, Error> {
    Ok(sqlx::query(&FETCH_TOP_NINE_USERS_IN_GUILD_QUERY)
        .bind(guild_id.to_string())
        .fetch_all(db)
        .await?)
}

/// Adds a db user id + guild id if there's none or updates the pair with the new values.
///
/// The function is the one which is referrred to in the event handler because it's more likely
/// that the user already exists. If it doesn't then we add it by calling add_user_if_not_exists()
/// with its parameters.
///
/// Additionally, we directly use the guild_id instead of the event as the
/// parameter for add_user_if_not_exists() in order to save computing resources.
pub async fn add_or_update_db_user(
    db: &SqlitePool,
    message: &serenity::Message,
    ctx: &serenity::Context,
    obtained_xp: u32,
) -> Result<(), Error> {
    let xp_addition_cooldown: i64 = *XP_COOLDOWN_NUMBER_SECS;
    let current_timestamp = chrono::offset::Utc::now().timestamp();

    let user = &message.author;
    let Some(guild_id) = message.guild_id else {
        return Ok(());
    };

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

    // First we need to check if there's some user_id+guild_id pair that matches
    let level_query = fetch_user_level(db, user, guild_id).await?;

    let Some(query_row) = level_query else {
        add_user_if_not_exists(db, user, guild_id).await?;
        return Ok(());
    };

    let queried_level = query_row.get::<u32, &str>(LevelsSchema::Level.as_str());
    let added_experience_points =
        query_row.get::<u32, &str>(LevelsSchema::ExperiencePoints.as_str()) + obtained_xp;

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

    sqlx::query(&UPDATE_USER_LEVEL_QUERY)
        .bind(update.updated_experience)
        .bind(update.updated_level)
        .bind(user.id.to_string())
        .bind(guild_id.to_string())
        .execute(db)
        .await?;

    Ok(())
}
