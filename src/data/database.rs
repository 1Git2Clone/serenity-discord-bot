use poise::serenity_prelude::{FullEvent, Message, User};
/// https://stackoverflow.com/questions/72763578/how-to-create-a-sqlite-database-with-rust-sqlx
use sqlx::{sqlite::SqliteConnectOptions, sqlite::SqlitePoolOptions, Error, SqlitePool};
use std::{future::Future, path::Path};

use crate::data::bot_data::DATABASE_COLUMNS;

use super::bot_data::DATABASE_USERS;

pub async fn connect_to_db(
    filename: impl AsRef<Path>,
) -> impl Future<Output = Result<SqlitePool, Error>> {
    SqlitePoolOptions::new().max_connections(5).connect_with(
        SqliteConnectOptions::new()
            .filename(filename)
            .create_if_missing(true),
    )
}

pub async fn add_user_if_not_exists(
    db: SqlitePool,
    user: &User,
    event: FullEvent,
) -> Result<(), Error> {
    let message = match event {
        FullEvent::Message { new_message } => new_message.guild_id,
        _ => Message::default().guild_id, // basically 1
    };
    let guild_id = match message {
        Some(guild_id) => guild_id,
        None => {
            return Ok(());
        }
    };

    let query_cooldown_secs: i64 = 60;
    let current_timestamp = chrono::offset::Utc::now().timestamp();
    let last_query_timestamp_as_secs: Option<i64> = sqlx::query_scalar(
        format!(
            "SELECT `{}` FROM `{}` WHERE `{}` = ? AND `{}` = ?",
            DATABASE_COLUMNS["last_query_timestamp"],
            DATABASE_USERS.to_owned(),
            DATABASE_COLUMNS["user_id"],
            DATABASE_COLUMNS["guild_id"]
        )
        .as_str(),
    )
    .bind(user.id.to_string())
    .bind(guild_id.to_string())
    .fetch_optional(&db)
    .await?;

    println!("{:#?}", last_query_timestamp_as_secs);

    if let Some(timestamp) = last_query_timestamp_as_secs {
        if current_timestamp - timestamp < query_cooldown_secs {
            println!("In cooldown!");
            return Ok(());
        }
    }

    println!("Message Guild Id: {:?}", guild_id);

    // ignoring already saved user_id + guild_id tuples
    let query = format!(
        "INSERT OR IGNORE INTO `{}` (`{}`, `{}`, `{}`, `{}`)
         VALUES (?, ?, ?, ?)",
        DATABASE_USERS.to_owned(),
        DATABASE_COLUMNS["user_id"],
        DATABASE_COLUMNS["guild_id"],
        DATABASE_COLUMNS["experience_points"],
        DATABASE_COLUMNS["level"],
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
