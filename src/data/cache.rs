//! Shared Redis connection, usable by any feature that enables `redis`.

use std::env;

use redis::aio::ConnectionManager;
use tokio::sync::OnceCell;

static REDIS: OnceCell<Option<ConnectionManager>> = OnceCell::const_new();

/// A handle to the shared connection (cloning multiplexes over one connection),
/// or `None` when `REDIS_URL` is unset or the connection failed.
pub async fn conn() -> Option<ConnectionManager> {
    REDIS
        .get_or_init(|| async {
            let url = env::var("REDIS_URL").ok()?;
            match redis::Client::open(url) {
                Ok(client) => match client.get_connection_manager().await {
                    Ok(conn) => {
                        tracing::info!("Connected to Redis.");
                        Some(conn)
                    }
                    Err(why) => {
                        tracing::warn!("Redis connection failed: {why}");
                        None
                    }
                },
                Err(why) => {
                    tracing::warn!("Invalid REDIS_URL: {why}");
                    None
                }
            }
        })
        .await
        .clone()
}

/// Connect at startup so the status is logged before first use.
pub async fn init() {
    let _ = conn().await;
}
