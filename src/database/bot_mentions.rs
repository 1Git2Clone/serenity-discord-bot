use crate::prelude::*;

#[tracing::instrument(
    fields(
        category = "sql",
        db_pool = ?db,
        mentions_to_add = %n
    )
)]
pub async fn add_mentions(db: &PgPool, n: i64) -> Result<i64, sqlx::Error> {
    let fetched_mentions = MentionsTable::fetch_mentions(db).await?;

    MentionsTable::update_mentions(db, fetched_mentions + n).await?;

    Ok(fetched_mentions + n)
}
