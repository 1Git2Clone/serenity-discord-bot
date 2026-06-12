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
    token: &str,
    ttl_secs: u64,
) -> bool {
    let result: Option<String> = redis::cmd("SET")
        .arg(key)
        .arg(token)
        .arg("NX")
        .arg("EX")
        .arg(ttl_secs)
        .query_async(conn)
        .await
        .ok()
        .flatten();
    result.is_some()
}

/// Release a Redis-backed lock only if the token matches. Best-effort; the
/// TTL is the real safety net.
pub async fn release_lock(conn: &mut ConnectionManager, key: &str, token: &str) {
    let script = redis::Script::new(
        r#"if redis.call('GET', KEYS[1]) == ARGV[1] then return redis.call('DEL', KEYS[1]) else return 0 end"#,
    );
    let _: Result<i64, _> = script.key(key).arg(token).invoke_async(conn).await;
}

// ── Rate limit helpers ──────────────────────────────────────────────────────

/// Check and increment a Redis-backed rate limiter. Returns `true` if the
/// caller is rate-limited (key already exists), `false` if this is the first
/// hit in the window. Atomically sets the key with TTL on first call.
pub async fn check_rate_limit(
    conn: &mut ConnectionManager,
    key: &str,
    ttl_secs: u64,
) -> Result<bool, redis::RedisError> {
    let result: Option<String> = redis::cmd("SET")
        .arg(key)
        .arg("1")
        .arg("NX")
        .arg("EX")
        .arg(ttl_secs)
        .query_async(conn)
        .await?;
    Ok(result.is_none())
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

/// Check whether a key exists.
pub async fn key_exists(
    conn: &mut ConnectionManager,
    key: &str,
) -> Result<bool, redis::RedisError> {
    redis::cmd("EXISTS").arg(key).query_async(conn).await
}

// ── RAII drop-guard for Redis locks ─────────────────────────────────────────

/// Releases a Redis lock on drop. The DEL is spawned because Drop can't be
/// async; the TTL remains the safety net if the spawn or DEL fails.
pub struct RedisLockGuard {
    key: String,
    token: String,
}

impl RedisLockGuard {
    pub fn new(key: String, token: String) -> Self {
        Self { key, token }
    }
}

impl Drop for RedisLockGuard {
    fn drop(&mut self) {
        let key = std::mem::take(&mut self.key);
        let token = std::mem::take(&mut self.token);
        // Skip no-op (empty) keys.
        if key.is_empty() {
            return;
        }
        tokio::spawn(async move {
            if let Some(mut conn) = conn().await {
                release_lock(&mut conn, &key, &token).await;
            }
        });
    }
}
