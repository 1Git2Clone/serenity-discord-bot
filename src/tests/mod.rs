#![cfg(test)]

#[cfg(feature = "network_test")]
mod emojis;
#[cfg(feature = "network_test")]
mod urls;

use sqlx::{PgPool, postgres::PgPoolOptions};

/// Connect to the database. Loads `.env` first (for local dev where the shell
/// hasn't exported DATABASE_URL) then returns `None` if the var is absent or
/// the connection fails, so DB tests can skip gracefully.
pub(crate) async fn test_pool() -> Option<PgPool> {
    dotenv::dotenv().ok();
    let url = std::env::var("DATABASE_URL").ok()?;
    PgPoolOptions::new().connect(&url).await.ok()
}

/// Connect to Redis through the cache handle (a fresh per-call manager in
/// test builds). Loads `.env` first, then returns `None` when `REDIS_URL` is
/// absent or unreachable, so Redis tests can skip gracefully.
#[cfg(feature = "redis")]
pub(crate) async fn test_redis() -> Option<redis::aio::ConnectionManager> {
    dotenv::dotenv().ok();
    crate::data::cache::conn().await
}

fn sized_send_sunc_unpin<T: Sized + Send + Sync + Unpin>() {}

#[test]
fn normal_types() {
    use crate::data::command_data::Data;
    use crate::enums::command_enums::EmbedType;
    use crate::enums::schemas::LevelsTable;

    sized_send_sunc_unpin::<Data>();
    sized_send_sunc_unpin::<EmbedType>();
    sized_send_sunc_unpin::<LevelsTable>();
}
