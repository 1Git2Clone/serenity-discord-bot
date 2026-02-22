pub mod bot_mentions;
pub mod level_system;

use crate::prelude::*;

/// Used to establish the database connection with its predetermined parameters.
#[tracing::instrument(fields(category = "sql",))]
pub async fn connect_to_db() -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        // .max_connections(5)
        .connect(&DATABASE_FILENAME)
        .await
}
