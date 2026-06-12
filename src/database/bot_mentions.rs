use crate::prelude::*;

#[tracing::instrument(
    fields(
        category = "sql",
        db_pool = ?db,
        mentions_to_add = %n
    )
)]
pub async fn add_mentions(db: &PgPool, n: i64) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        "UPDATE bot_mentions SET mentions = mentions + $1 RETURNING mentions",
        n
    )
    .fetch_one(db)
    .await?;

    Ok(row.mentions)
}
