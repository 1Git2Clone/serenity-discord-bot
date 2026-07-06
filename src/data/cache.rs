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

// ── Write-through get-or-load scaffold ──────────────────────────────────────

/// Shared get-or-load flow for write-through Redis caches.
///
/// Three call sites repeat the same scaffolding (cache → DB on miss → write
/// back, Redis errors logged not propagated). The cache *shape* — set vs
/// string vs hash, TTL, negative-sentinel convention, key naming — varies
/// per feature and stays at the call site; this module owns only the flow.
///
/// A `None` from `read_cache` triggers the DB load. Callers that need a
/// negative sentinel (e.g. an empty string for "no prompt set") encode it as
/// `Some("")` and translate back to `None` themselves — that keeps the
/// helper shape-agnostic.
pub mod write_through {
    use std::future::Future;
    use std::pin::Pin;

    use redis::aio::ConnectionManager;

    pub async fn get_or_load<T, LoadFut>(
        read_cache: impl for<'a> FnOnce(
            &'a mut ConnectionManager,
        ) -> Pin<
            Box<dyn Future<Output = Result<Option<T>, redis::RedisError>> + Send + 'a>,
        >,
        load_from_db: impl FnOnce() -> LoadFut + Send,
        write_cache: impl for<'a> FnOnce(
            &'a mut ConnectionManager,
            &'a T,
        ) -> Pin<
            Box<dyn Future<Output = Result<(), redis::RedisError>> + Send + 'a>,
        >,
    ) -> Result<T, crate::data::command_data::Error>
    where
        T: Send,
        LoadFut: Future<Output = Result<T, crate::data::command_data::Error>> + Send,
    {
        let Some(mut conn) = super::conn().await else {
            return load_from_db().await;
        };

        match read_cache(&mut conn).await {
            Ok(Some(value)) => return Ok(value),
            Ok(None) => {}
            Err(e) => {
                tracing::warn!(error = %e, "write_through: cache read failed, falling back to DB");
            }
        }

        let value = load_from_db().await?;

        if let Err(e) = write_cache(&mut conn, &value).await {
            tracing::warn!(error = %e, "write_through: cache write-back failed");
        }

        Ok(value)
    }

    #[cfg(test)]
    mod tests {
        use super::super::{get_string, set_string_ex};
        use super::*;
        use crate::tests::test_redis;

        type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

        /// Namespaced key so parallel tests (and stale runs) can't collide.
        fn test_key(name: &str) -> String {
            format!("test:write_through:{name}:{}", rand::random::<u64>())
        }

        #[tokio::test]
        async fn cache_hit_skips_db() -> TestResult {
            let Some(mut conn) = test_redis().await else {
                return Ok(());
            };
            let key = test_key("hit");
            set_string_ex(&mut conn, &key, "from-cache", 30).await?;
            let key_for_write = key.clone();
            let key_for_del = key.clone();

            let db_calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let dc = std::sync::Arc::clone(&db_calls);
            let value: String = get_or_load(
                move |c| Box::pin(async move { get_string(c, &key).await }),
                move || async move {
                    dc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok::<_, Box<dyn std::error::Error + Send + Sync>>("from-db".to_string())
                },
                move |c, v| Box::pin(async move { set_string_ex(c, &key_for_write, v, 30).await }),
            )
            .await?;
            assert_eq!(value, "from-cache");
            assert_eq!(
                db_calls.load(std::sync::atomic::Ordering::SeqCst),
                0,
                "DB must not be touched on cache hit"
            );

            let _: () = redis::cmd("DEL")
                .arg(&key_for_del)
                .query_async(&mut conn)
                .await?;
            Ok(())
        }

        #[tokio::test]
        async fn cache_miss_loads_from_db_and_writes_back() -> TestResult {
            let Some(mut conn) = test_redis().await else {
                return Ok(());
            };
            let key = test_key("miss");
            let key_for_write = key.clone();
            let key_for_read2 = key.clone();
            let key_for_del = key.clone();

            let value: String = get_or_load(
                move |c| Box::pin(async move { get_string(c, &key).await }),
                || async {
                    Ok::<_, Box<dyn std::error::Error + Send + Sync>>("from-db".to_string())
                },
                move |c, v| Box::pin(async move { set_string_ex(c, &key_for_write, v, 30).await }),
            )
            .await?;
            assert_eq!(value, "from-db");

            // Second call now hits the cache — DB closure must not be called.
            let value2: String = get_or_load(
                move |c| Box::pin(async move { get_string(c, &key_for_read2).await }),
                || async {
                    Err::<String, _>(Box::<dyn std::error::Error + Send + Sync>::from(
                        "DB should not be called on second read",
                    ))
                },
                |_c, _v| Box::pin(async { Ok::<(), redis::RedisError>(()) }),
            )
            .await?;
            assert_eq!(value2, "from-db");

            let _: () = redis::cmd("DEL")
                .arg(&key_for_del)
                .query_async(&mut conn)
                .await?;
            Ok(())
        }

        #[tokio::test]
        async fn db_load_failure_propagates_and_cache_is_not_written() -> TestResult {
            let Some(mut conn) = test_redis().await else {
                return Ok(());
            };
            let key = test_key("dbfail");
            let key_for_exists = key.clone();

            let write_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let wc = std::sync::Arc::clone(&write_count);
            let result: Result<String, _> = get_or_load(
                move |c| Box::pin(async move { get_string(c, &key).await }),
                || async { Err::<String, _>("db down".into()) },
                move |_c, _v| {
                    Box::pin(async move {
                        wc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        Ok::<(), redis::RedisError>(())
                    })
                },
            )
            .await;
            assert!(result.is_err(), "DB failure must propagate");

            assert_eq!(
                write_count.load(std::sync::atomic::Ordering::SeqCst),
                0,
                "no write-back when DB failed"
            );
            let exists: bool = redis::cmd("EXISTS")
                .arg(&key_for_exists)
                .query_async(&mut conn)
                .await?;
            assert!(!exists);
            Ok(())
        }
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
