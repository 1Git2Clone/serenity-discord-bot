use crate::data::command_data::Error;
use crate::data::database::{MENTIONS_TABLE, MENTIONS_TABLE_NAME};
use crate::enums::schemas::MentionsSchema;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::Row;
use sqlx::SqlitePool;

pub async fn fetch_mentions(db: &SqlitePool) -> Result<i64, Error> {
    let query = format!(
        "SELECT `{}` FROM `{}`",
        MENTIONS_TABLE[&MentionsSchema::Mentions],
        MENTIONS_TABLE_NAME,
    );
    let sql = sqlx::query(&query).fetch_optional(db).await?;

    let row = match sql {
        Some(row) => row,
        None => return Err(format!("Couldn't find a row to select (SQL: {})", query).into()),
    };

    let queried_mentions = row.get::<i64, &str>(MENTIONS_TABLE[&MentionsSchema::Mentions]);

    Ok(queried_mentions)
}

pub async fn update_mentions(
    db: &SqlitePool,
    updated_mentions: i64,
) -> Result<SqliteQueryResult, sqlx::Error> {
    let query = format!(
        "UPDATE `{}` SET `{}` = ?",
        MENTIONS_TABLE_NAME,
        MENTIONS_TABLE[&MentionsSchema::Mentions]
    );

    sqlx::query(&query).bind(updated_mentions).execute(db).await
}

pub async fn add_mentions(db: &SqlitePool, n: i64) -> Result<i64, Error> {
    let fetched_mentions = fetch_mentions(db).await?;

    update_mentions(db, fetched_mentions + n).await?;

    fetch_mentions(db).await
}
