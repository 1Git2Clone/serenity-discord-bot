use crate::prelude::*;

#[tracing::instrument(
    fields(
        category = "sql",
        db_pool = ?db
    )
)]
pub async fn fetch_mentions(db: &SqlitePool) -> Result<usize, sqlx::Error> {
    let query = format!(
        "SELECT `{}` FROM `{}`",
        MentionsSchema::Mentions.as_str(),
        MENTIONS_TABLE_NAME,
    );
    let queried_mentions: Option<i64> = sqlx::query_scalar(&query).fetch_one(db).await?;

    Ok(queried_mentions.unwrap_or(0) as usize)
}

#[tracing::instrument(
    fields(
        category = "sql",
        db_pool = ?db,
        updated_mentions = %updated_mentions
    )
)]
pub async fn update_mentions(
    db: &SqlitePool,
    updated_mentions: usize,
) -> Result<SqliteQueryResult, sqlx::Error> {
    let query = format!(
        "UPDATE `{}` SET `{}` = ?",
        MENTIONS_TABLE_NAME,
        MentionsSchema::Mentions.as_str()
    );

    sqlx::query(&query)
        .bind(updated_mentions as i64)
        .execute(db)
        .await
}

#[tracing::instrument(
    fields(
        category = "sql",
        db_pool = ?db,
        mentions_to_add = %n
    )
)]
pub async fn add_mentions(db: &SqlitePool, n: usize) -> Result<usize, sqlx::Error> {
    let fetched_mentions = fetch_mentions(db).await?;

    update_mentions(db, fetched_mentions + n).await?;

    Ok(fetched_mentions + n)
}
