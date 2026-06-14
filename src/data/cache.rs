//! Shared Redis connection, usable by any feature that enables `redis`.

use std::env;

use redis::aio::ConnectionManager;
#[cfg(not(test))]
use tokio::sync::OnceCell;

#[cfg(not(test))]
static REDIS: OnceCell<Option<ConnectionManager>> = OnceCell::const_new();

/// A handle to the shared connection (cloning multiplexes over one connection),
/// or `None` when `REDIS_URL` is unset or the connection failed.
pub async fn conn() -> Option<ConnectionManager> {
    // Each #[tokio::test] runs on its own runtime, while the shared manager's
    // driver task lives on the runtime that first created it and dies with
    // it. Hand tests a fresh manager per call so none of them ever observes
    // another test's dead connection mid-flight.
    #[cfg(test)]
    {
        // Self-sufficient env loading: tests must see the same REDIS_URL no
        // matter which of them runs (and loads `.env`) first.
        dotenv::dotenv().ok();
        let url = env::var("REDIS_URL").ok()?;
        return redis::Client::open(url)
            .ok()?
            .get_connection_manager()
            .await
            .ok();
    }
    #[cfg(not(test))]
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
    let _: () = redis::cmd("SADD")
        .arg(key)
        .arg(member)
        .query_async(conn)
        .await?;
    Ok(())
}

/// Remove a member from a Redis set.
pub async fn set_remove(
    conn: &mut ConnectionManager,
    key: &str,
    member: u64,
) -> Result<(), redis::RedisError> {
    let _: () = redis::cmd("SREM")
        .arg(key)
        .arg(member)
        .query_async(conn)
        .await?;
    Ok(())
}

/// Check whether a member exists in a Redis set.
pub async fn set_contains(
    conn: &mut ConnectionManager,
    key: &str,
    member: u64,
) -> Result<bool, redis::RedisError> {
    redis::cmd("SISMEMBER")
        .arg(key)
        .arg(member)
        .query_async(conn)
        .await
}

/// Check whether a key exists.
pub async fn key_exists(
    conn: &mut ConnectionManager,
    key: &str,
) -> Result<bool, redis::RedisError> {
    redis::cmd("EXISTS").arg(key).query_async(conn).await
}

// ── String helpers ──────────────────────────────────────────────────────────

/// Get a string value, or `None` if the key is unset.
pub async fn get_string(
    conn: &mut ConnectionManager,
    key: &str,
) -> Result<Option<String>, redis::RedisError> {
    redis::cmd("GET").arg(key).query_async(conn).await
}

/// Set a string value with an expiry (seconds).
pub async fn set_string_ex(
    conn: &mut ConnectionManager,
    key: &str,
    value: &str,
    ttl_secs: u64,
) -> Result<(), redis::RedisError> {
    let _: () = redis::cmd("SET")
        .arg(key)
        .arg(value)
        .arg("EX")
        .arg(ttl_secs)
        .query_async(conn)
        .await?;
    Ok(())
}

/// Delete a key. A no-op if it doesn't exist.
pub async fn del(conn: &mut ConnectionManager, key: &str) -> Result<(), redis::RedisError> {
    let _: () = redis::cmd("DEL").arg(key).query_async(conn).await?;
    Ok(())
}

// ── Hash helpers (for custom reactions) ─────────────────────────────────────

/// Set a field in a Redis hash.
pub async fn hash_set(
    conn: &mut ConnectionManager,
    key: &str,
    field: &str,
    value: &str,
) -> Result<(), redis::RedisError> {
    let _: () = redis::cmd("HSET")
        .arg(key)
        .arg(field)
        .arg(value)
        .query_async(conn)
        .await?;
    Ok(())
}

/// Get all fields and values from a Redis hash.
pub async fn hash_getall(
    conn: &mut ConnectionManager,
    key: &str,
) -> Result<Vec<(String, String)>, redis::RedisError> {
    redis::cmd("HGETALL").arg(key).query_async(conn).await
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_redis;

    type TestResult = Result<(), redis::RedisError>;

    /// Namespaced key so parallel tests (and stale runs) can't collide.
    fn test_key(name: &str) -> String {
        format!("test:cache:{name}:{}", rand::random::<u64>())
    }

    #[tokio::test]
    async fn lock_acquire_release_roundtrip() {
        let Some(mut conn) = test_redis().await else {
            return;
        };
        let key = test_key("lock");

        assert!(try_acquire_lock(&mut conn, &key, "a", 30).await);
        // Held: a second acquire fails, even with another token.
        assert!(!try_acquire_lock(&mut conn, &key, "b", 30).await);

        // Wrong token doesn't release.
        release_lock(&mut conn, &key, "b").await;
        assert!(!try_acquire_lock(&mut conn, &key, "b", 30).await);

        // Matching token does.
        release_lock(&mut conn, &key, "a").await;
        assert!(try_acquire_lock(&mut conn, &key, "b", 30).await);

        release_lock(&mut conn, &key, "b").await;
    }

    #[tokio::test]
    async fn rate_limit_first_hit_allowed_second_blocked() -> TestResult {
        let Some(mut conn) = test_redis().await else {
            return Ok(());
        };
        let key = test_key("rl");

        assert!(!check_rate_limit(&mut conn, &key, 30).await?);
        assert!(check_rate_limit(&mut conn, &key, 30).await?);

        let _: () = redis::cmd("DEL").arg(&key).query_async(&mut conn).await?;
        Ok(())
    }

    #[tokio::test]
    async fn set_add_contains_remove_roundtrip() -> TestResult {
        let Some(mut conn) = test_redis().await else {
            return Ok(());
        };
        let key = test_key("set");

        assert!(!key_exists(&mut conn, &key).await?);
        assert!(!set_contains(&mut conn, &key, 42).await?);

        set_add(&mut conn, &key, 42).await?;
        assert!(key_exists(&mut conn, &key).await?);
        assert!(set_contains(&mut conn, &key, 42).await?);
        assert!(!set_contains(&mut conn, &key, 43).await?);

        set_remove(&mut conn, &key, 42).await?;
        assert!(!set_contains(&mut conn, &key, 42).await?);
        Ok(())
    }

    #[tokio::test]
    async fn lock_guard_releases_on_drop() {
        let Some(mut conn) = test_redis().await else {
            return;
        };
        let key = test_key("guard");

        assert!(try_acquire_lock(&mut conn, &key, "tok", 30).await);
        drop(RedisLockGuard::new(key.clone(), "tok".into()));

        // The drop releases via a spawned task; poll until it lands. Errors
        // count as "not yet": the shared manager may be mid-reconnect after
        // another test's runtime shut down.
        for _ in 0..50 {
            if matches!(key_exists(&mut conn, &key).await, Ok(false)) {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        panic!("lock {key} not released by drop guard");
    }

    #[tokio::test]
    async fn empty_guard_is_noop() {
        // No Redis required: an empty key skips the spawned release.
        drop(RedisLockGuard::new(String::new(), String::new()));
    }

    #[tokio::test]
    async fn hash_set_getall_del_roundtrip() -> TestResult {
        let Some(mut conn) = test_redis().await else {
            return Ok(());
        };
        let key = test_key("hash");

        hash_set(&mut conn, &key, "f1", "v1").await?;
        hash_set(&mut conn, &key, "f2", "v2").await?;

        let pairs = hash_getall(&mut conn, &key).await?;
        // HGETALL returns field/value interleaved; collect into a map to check.
        let map: std::collections::HashMap<&str, &str> = pairs
            .iter()
            .map(|(f, v)| (f.as_str(), v.as_str()))
            .collect();
        assert_eq!(map.get("f1"), Some(&"v1"));
        assert_eq!(map.get("f2"), Some(&"v2"));

        let _: () = redis::cmd("DEL").arg(&key).query_async(&mut conn).await?;
        Ok(())
    }

    #[tokio::test]
    async fn string_get_set_del_roundtrip() -> TestResult {
        let Some(mut conn) = test_redis().await else {
            return Ok(());
        };
        let key = test_key("string");

        assert!(get_string(&mut conn, &key).await?.is_none());

        set_string_ex(&mut conn, &key, "hello", 30).await?;
        assert_eq!(get_string(&mut conn, &key).await?.as_deref(), Some("hello"));

        del(&mut conn, &key).await?;
        assert!(get_string(&mut conn, &key).await?.is_none());
        Ok(())
    }
}
