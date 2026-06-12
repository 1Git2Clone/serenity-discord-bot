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

/// Connect to Redis through the shared cache handle. Loads `.env` first, then
/// returns `None` when `REDIS_URL` is absent or unreachable, so Redis tests
/// can skip gracefully.
///
/// The shared manager's background task dies with the tokio runtime of
/// whichever test created it, so PING (with retries) to let it reconnect on
/// this test's runtime before handing it out.
#[cfg(feature = "redis")]
pub(crate) async fn test_redis() -> Option<redis::aio::ConnectionManager> {
    dotenv::dotenv().ok();
    let mut conn = crate::data::cache::conn().await?;
    for _ in 0..10 {
        let pong: Result<String, _> = redis::cmd("PING").query_async(&mut conn).await;
        if pong.is_ok() {
            return Some(conn);
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    None
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
