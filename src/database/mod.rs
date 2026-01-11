pub mod bot_mentions;
pub mod level_system;

use crate::prelude::*;

/// Used to establish the database connection with its predetermined parameters.
#[tracing::instrument(
    fields(
        category = "sql",
        filename = %filename.as_ref().display()
    )
)]
pub async fn connect_to_db(filename: impl AsRef<Path>) -> Result<SqlitePool, sqlx::Error> {
    SqlitePoolOptions::new()
        .connect_with(
            SqliteConnectOptions::new()
                .filename(filename)
                .create_if_missing(true),
        )
        .await
}
