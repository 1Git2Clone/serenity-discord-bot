pub mod bot_mentions;
pub mod level_system;

use crate::prelude::*;

/// Used to establish the database connection with its predetermined parameters as well as run the
/// migration script to ensure that the database has its schema prepared for use.
#[tracing::instrument(fields(category = "sql",))]
pub async fn connect_to_db() -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        // .max_connections(5)
        .connect(&DATABASE_FILENAME)
        .await?;

    sqlx::migrate!().run(&pool).await.map_err(|e| {
        tracing::error!("Failed to run migrations: {}", e);
        e
    })?;

    tracing::info!("Database migrations completed successfully");

    Ok(pool)
}
