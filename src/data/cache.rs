//! Generic Redis connection layer. Not tied to any one feature — callers decide
//! what keys/structures to store. Enabled by the `redis` feature.

use std::env;

use redis::aio::ConnectionManager;
use tokio::sync::OnceCell;

/// Shared Redis connection, or `None` when `REDIS_URL` is unset or unreachable.
/// Callers fall back to whatever source of truth they have.
static REDIS: OnceCell<Option<ConnectionManager>> = OnceCell::const_new();

/// A cloned handle to the shared connection (cheap; clones multiplex over one
/// connection), or `None` if Redis isn't configured/available.
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
                        tracing::warn!("Redis unavailable ({why}); callers fall back.");
                        None
                    }
                },
                Err(why) => {
                    tracing::warn!("Invalid REDIS_URL ({why}); callers fall back.");
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
