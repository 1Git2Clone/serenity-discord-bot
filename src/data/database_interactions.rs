use crate::commands::level_logic::update_level;
use crate::enums::schemas::DatabaseSchema::*;
use poise::serenity_prelude as serenity;
use serenity::User;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
/// https://stackoverflow.com/questions/72763578/how-to-create-a-sqlite-database-with-rust-sqlx
use sqlx::{sqlite::SqliteConnectOptions, sqlite::SqlitePoolOptions, Error, SqlitePool};
use std::{future::Future, path::Path};

use crate::data::bot_data::{DATABASE_COLUMNS, XP_COOLDOWN_NUMBER_SECS};
use crate::data::user_data::USER_COOLDOWNS;

use super::bot_data::DATABASE_USERS;

/// Used to establish the database connection with its predetermined parameters.
pub async fn connect_to_db(
    filename: impl AsRef<Path>,
) -> impl Future<Output = Result<SqlitePool, Error>> {
    SqlitePoolOptions::new().connect_with(
        SqliteConnectOptions::new()
            .filename(filename)
            .create_if_missing(true),
    )
}

/// Adds a new database user with the schema from `crate::data:bot_data.rs`.
/// That's the reason why the function isn't public.
async fn add_user_if_not_exists(
    db: SqlitePool,
    user: &User,
    guild_id: serenity::GuildId,
) -> Result<(), Error> {
    println!("Adding user to database...");
    println!("Message Guild Id: {:?}", guild_id);

    // ignoring already saved user_id + guild_id tuples
    let query = format!(
        "INSERT INTO `{}` (`{}`, `{}`, `{}`, `{}`)
         VALUES (?, ?, ?, ?)",
        DATABASE_USERS.to_owned(),
        DATABASE_COLUMNS[&UserId],
        DATABASE_COLUMNS[&GuildId],
        DATABASE_COLUMNS[&ExperiencePoints],
        DATABASE_COLUMNS[&Level],
    );

    sqlx::query(&query)
        .bind(user.id.to_string())
        .bind(guild_id.to_string())
        .bind(0)
        .bind(1)
        .execute(&db)
        .await?;

    Ok(())
}

pub async fn fetch_user_level(
    db: &SqlitePool,
    user: &User,
    guild_id: serenity::GuildId,
) -> Result<Option<SqliteRow>, Error> {
    sqlx::query(
        format!(
            "SELECT `{}`, `{}`, `{}`
             FROM `{}`
             WHERE `{}` = ? AND `{}` = ?",
            DATABASE_COLUMNS[&UserId],
            DATABASE_COLUMNS[&ExperiencePoints],
            DATABASE_COLUMNS[&Level],
            //
            DATABASE_USERS.to_owned(),
            //
            DATABASE_COLUMNS[&UserId],
            DATABASE_COLUMNS[&GuildId]
        )
        .as_str(),
    )
    .bind(user.id.to_string())
    .bind(guild_id.to_string())
    .fetch_optional(db)
    .await
}

pub async fn fetch_user_level_and_rank(
    db: &SqlitePool,
    user: &User,
    guild_id: serenity::GuildId,
) -> Result<Option<(i64, SqliteRow)>, Error> {
    let sql = sqlx::query(
        "
        SELECT us.*,
               (SELECT COUNT(*)
                FROM user_stats AS inner_u
                WHERE inner_u.guild_id = us.guild_id
                      AND (inner_u.level > us.level OR 
                          (inner_u.level = us.level AND inner_u.experience_points >= us.experience_points))
               ) AS rank
        FROM user_stats AS us
        WHERE us.user_id = ? AND us.guild_id = ?
        ORDER BY level DESC, experience_points DESC
        ",
    )
    .bind(user.id.to_string())
    .bind(guild_id.to_string())
    .fetch_optional(db)
    .await?;

    match sql {
        Some(row) => Ok(Some((row.get::<i64, &str>("rank"), row))),
        None => Ok(None),
    }
}
pub async fn fetch_top_nine_levels_in_guild(
    db: &SqlitePool,
    guild_id: serenity::GuildId,
) -> Result<Vec<SqliteRow>, Error> {
    sqlx::query(
        format!(
            "SELECT
            COALESCE(`{}`, 'Unknown user') AS `{}`,
            COALESCE(`{}`, 0) AS `{}`,
            COALESCE(`{}`, 0) AS `{}`
            FROM `{}`
            WHERE `{}` = ?
            ORDER BY {} DESC, {} DESC
            LIMIT 9",
            DATABASE_COLUMNS[&UserId],
            DATABASE_COLUMNS[&UserId],
            //
            DATABASE_COLUMNS[&ExperiencePoints],
            DATABASE_COLUMNS[&ExperiencePoints],
            //
            DATABASE_COLUMNS[&Level],
            DATABASE_COLUMNS[&Level],
            //
            DATABASE_USERS.to_owned(),
            DATABASE_COLUMNS[&GuildId],
            DATABASE_COLUMNS[&Level],
            DATABASE_COLUMNS[&ExperiencePoints],
        )
        .as_str(),
    )
    .bind(guild_id.to_string())
    .fetch_all(db)
    .await
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
    db: SqlitePool,
    message: serenity::Message,
    ctx: serenity::Context,
    obtained_xp: i32,
) -> Result<(), Error> {
    let query_cooldown_secs: i64 = *XP_COOLDOWN_NUMBER_SECS;
    let current_timestamp = chrono::offset::Utc::now().timestamp();

    let user = &message.author;
    let Some(guild_id) = message.guild_id else {
        return Ok(());
    };

    {
        // Mutex guard dropping scope in order to send the data safely accross threads
        // https://doc.rust-lang.org/std/sync/struct.Mutex.html

        let mut cooldown_timestamps = USER_COOLDOWNS.lock().unwrap();
        println!("{:#?}", cooldown_timestamps);

        if let Some(user_in_guild_timestamp) = cooldown_timestamps.get(&(user.id, guild_id)) {
            if user_in_guild_timestamp + query_cooldown_secs > current_timestamp {
                println!(
                    "> The user {} is on cooldown!\n Time remaining: {:#?}",
                    user.name,
                    current_timestamp - (user_in_guild_timestamp + query_cooldown_secs)
                );
                return Ok(());
            }
        };

        cooldown_timestamps.insert((user.id, guild_id), current_timestamp);
        println!("{:#?}", cooldown_timestamps);
    }

    // First we need to check if there's some user_id+guild_id pair that matches
    let level_query: Option<SqliteRow> = fetch_user_level(&db, user, guild_id).await?;

    let query_row = match level_query {
        Some(row) => row,
        None => {
            println!("Adding user to the database...");
            add_user_if_not_exists(db, user, guild_id).await?;
            return Ok(());
        }
    };

    let queried_level = query_row.get::<i32, &str>(DATABASE_COLUMNS[&Level]);
    let queried_experience_points = query_row.get::<i32, &str>(DATABASE_COLUMNS[&ExperiencePoints]);
    let added_experience_points = queried_experience_points + obtained_xp;

    let update = update_level(added_experience_points, queried_level).await;
    let updated_experience_points_option = update.get(&ExperiencePoints);
    let updated_level_option = update.get(&Level);

    let updated_experience_points = match updated_experience_points_option {
        Some(update) => update,
        None => {
            eprintln!("Failed to update ExperiencePoints!");
            return Ok(());
        }
    };
    let updated_level = match updated_level_option {
        Some(update) => update,
        None => {
            eprintln!("Failed to update Level!");
            return Ok(());
        }
    };

    if *updated_level > queried_level {
        let _ = message
            .reply(
                ctx,
                format!("{} leveled up to level: {}", user.name, updated_level),
            )
            .await;
    }

    println!(
        "> Level: {}\n> Experience Points: {}",
        updated_level, updated_experience_points
    );
    println!("Message in Guild Id: {:?}", guild_id);

    // ignoring already saved user_id + guild_id tuples
    let query = format!(
        "UPDATE `{}`
         SET `{}` = ?, `{}` = ?
         WHERE `{}` = ? AND `{}` = ?",
        DATABASE_USERS.to_owned(),
        //
        DATABASE_COLUMNS[&ExperiencePoints],
        DATABASE_COLUMNS[&Level],
        //
        DATABASE_COLUMNS[&UserId],
        DATABASE_COLUMNS[&GuildId],
    );

    sqlx::query(&query)
        .bind(updated_experience_points)
        .bind(updated_level)
        .bind(user.id.to_string())
        .bind(guild_id.to_string())
        .execute(&db)
        .await?;

    Ok(())
}
