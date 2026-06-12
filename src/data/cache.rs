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

// ── Lock helpers ────────────────────────────────────────────────────────────

/// Try to acquire a Redis-backed lock. Returns `true` if acquired.
/// The lock auto-expires after `ttl_secs` as a safety net against crashes.
pub async fn try_acquire_lock(
    conn: &mut ConnectionManager,
    key: &str,
    ttl_secs: u64,
) -> bool {
    let result: Option<String> = redis::cmd("SET")
        .arg(key)
        .arg("1")
        .arg("NX")
        .arg("EX")
        .arg(ttl_secs)
        .query_async(conn)
        .await
        .ok()
        .flatten();
    result.is_some()
}

/// Release a Redis-backed lock. Best-effort; the TTL is the real safety net.
pub async fn release_lock(conn: &mut ConnectionManager, key: &str) {
    let _: Result<(), _> = redis::cmd("DEL").arg(key).query_async(conn).await;
}

// ── Rate limit helpers ──────────────────────────────────────────────────────

/// Check and increment a Redis-backed rate limiter. Returns `true` if the
/// caller is rate-limited (count > 1), `false` if this is the first hit in
/// the window. On first hit, sets the key's TTL.
pub async fn check_rate_limit(
    conn: &mut ConnectionManager,
    key: &str,
    ttl_secs: u64,
) -> Result<bool, redis::RedisError> {
    let count: i64 = redis::cmd("INCR").arg(key).query_async(conn).await?;
    if count == 1 {
        let _: () = redis::cmd("EXPIRE").arg(key).arg(ttl_secs).query_async(conn).await?;
    }
    Ok(count > 1)
}

// ── Set helpers (for channels / guilds) ─────────────────────────────────────

/// Add a member to a Redis set.
pub async fn set_add(
    conn: &mut ConnectionManager,
    key: &str,
    member: u64,
) -> Result<(), redis::RedisError> {
    let _: () = redis::cmd("SADD").arg(key).arg(member).query_async(conn).await?;
    Ok(())
}

/// Remove a member from a Redis set.
pub async fn set_remove(
    conn: &mut ConnectionManager,
    key: &str,
    member: u64,
) -> Result<(), redis::RedisError> {
    let _: () = redis::cmd("SREM").arg(key).arg(member).query_async(conn).await?;
    Ok(())
}

/// Check whether a member exists in a Redis set.
pub async fn set_contains(
    conn: &mut ConnectionManager,
    key: &str,
    member: u64,
) -> Result<bool, redis::RedisError> {
    redis::cmd("SISMEMBER").arg(key).arg(member).query_async(conn).await
}

/// Return all members of a Redis set.
pub async fn set_members(
    conn: &mut ConnectionManager,
    key: &str,
) -> Result<Vec<u64>, redis::RedisError> {
    redis::cmd("SMEMBERS").arg(key).query_async(conn).await
}
