use crate::{data::cache, enums::schemas::AiReviewGuildsTable, prelude::*};

const AI_REVIEW_GUILDS_KEY: &str = "ai:review_guilds";

/// Load the enabled review guilds from the DB into Redis, then seed Redis
/// if empty.
pub async fn init_review_guilds(pool: &PgPool) -> Result<(), Error> {
    let Some(mut conn) = cache::conn().await else {
        return Ok(());
    };

    if cache::set_members(&mut conn, AI_REVIEW_GUILDS_KEY).await?.is_empty() {
        for guild_id in AiReviewGuildsTable::fetch_all(pool).await? {
            cache::set_add(&mut conn, AI_REVIEW_GUILDS_KEY, guild_id as u64).await?;
        }
    }
    Ok(())
}

/// Check whether `/ai-review run` is enabled for a guild. Falls back to
/// `false` when Redis is unavailable.
pub async fn is_review_guild(guild_id: u64) -> bool {
    let Some(mut conn) = cache::conn().await else {
        return false;
    };
    cache::set_contains(&mut conn, AI_REVIEW_GUILDS_KEY, guild_id)
        .await
        .unwrap_or(false)
}

/// Enable or disable `/ai-review run` for a guild in both the DB and Redis.
/// Returns `true` if the state changed.
#[tracing::instrument(
    skip(pool),
    fields(category = "sql", guild_id = %guild_id, enabled = %enabled)
)]
pub async fn set_review_guild(pool: &PgPool, guild_id: u64, enabled: bool) -> Result<bool, Error> {
    let mut conn = match cache::conn().await {
        Some(c) => Some(c),
        None => {
            return set_review_guild_db_only(pool, guild_id, enabled).await;
        }
    };

    // Try Redis-aware path; fall back to DB-only on transient errors.
    match set_review_guild_redis(pool, guild_id, enabled, conn.as_mut().unwrap()).await {
        Ok(changed) => Ok(changed),
        Err(e) => {
            tracing::warn!(error = %e, guild_id, enabled, "Redis error in set_review_guild; falling back to DB-only");
            set_review_guild_db_only(pool, guild_id, enabled).await
        }
    }
}

async fn set_review_guild_db_only(pool: &PgPool, guild_id: u64, enabled: bool) -> Result<bool, Error> {
    let currently = AiReviewGuildsTable::fetch_all(pool)
        .await?
        .contains(&(guild_id as i64));
    if enabled == currently {
        return Ok(false);
    }
    if enabled {
        AiReviewGuildsTable::register(pool, guild_id as i64).await?;
    } else {
        AiReviewGuildsTable::unregister(pool, guild_id as i64).await?;
    }
    Ok(true)
}

async fn set_review_guild_redis(
    pool: &PgPool,
    guild_id: u64,
    enabled: bool,
    conn: &mut redis::aio::ConnectionManager,
) -> Result<bool, Error> {
    if enabled {
        if cache::set_contains(conn, AI_REVIEW_GUILDS_KEY, guild_id).await? {
            return Ok(false);
        }
        AiReviewGuildsTable::register(pool, guild_id as i64).await?;
        cache::set_add(conn, AI_REVIEW_GUILDS_KEY, guild_id).await?;
    } else {
        if !cache::set_contains(conn, AI_REVIEW_GUILDS_KEY, guild_id).await? {
            return Ok(false);
        }
        AiReviewGuildsTable::unregister(pool, guild_id as i64).await?;
        cache::set_remove(conn, AI_REVIEW_GUILDS_KEY, guild_id).await?;
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data::command_data::Error, enums::schemas::AiReviewGuildsTable, tests::test_pool};

    type TestResult = Result<(), Error>;

    // Use large sentinel IDs to avoid colliding with production data or other tests.
    // Each test owns exactly one ID; no two tests share an ID to avoid races under
    // parallel test execution.
    const ID_INSERT: u64 = 0x5AFE_0001_0000_0007;
    const ID_ENABLE: u64 = 0x5AFE_0001_0000_0003;
    const ID_DISABLE: u64 = 0x5AFE_0001_0000_0004;

    // These tests require both a DB and Redis to fully validate. When
    // REDIS_URL is not set, is_review_guild always returns false (safe
    // fallback), so the assertions skip the Redis check.

    #[tokio::test]
    async fn set_review_guild_enables_in_redis_and_db() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        let has_redis = cache::conn().await.is_some();

        // Ensure clean slate.
        AiReviewGuildsTable::unregister(&pool, ID_ENABLE as i64).await?;
        if let Some(mut conn) = cache::conn().await {
            let _: Result<(), _> = cache::set_remove(&mut conn, AI_REVIEW_GUILDS_KEY, ID_ENABLE).await;
        }

        let changed = set_review_guild(&pool, ID_ENABLE, true).await?;
        assert!(changed);
        if has_redis {
            assert!(is_review_guild(ID_ENABLE).await);
        }

        // cleanup
        set_review_guild(&pool, ID_ENABLE, false).await?;
        Ok(())
    }

    #[tokio::test]
    async fn set_review_guild_disables_in_redis_and_db() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        let has_redis = cache::conn().await.is_some();

        AiReviewGuildsTable::unregister(&pool, ID_DISABLE as i64).await?;
        if let Some(mut conn) = cache::conn().await {
            let _: Result<(), _> = cache::set_remove(&mut conn, AI_REVIEW_GUILDS_KEY, ID_DISABLE).await;
        }

        set_review_guild(&pool, ID_DISABLE, true).await?;
        let changed = set_review_guild(&pool, ID_DISABLE, false).await?;
        assert!(changed);
        if has_redis {
            assert!(!is_review_guild(ID_DISABLE).await);
        }
        Ok(())
    }

    #[tokio::test]
    async fn set_review_guild_noop_when_already_enabled() -> TestResult {
        let pool = PgPoolOptions::new().connect_lazy("postgres://localhost/unused")?;
        // Can't reach Redis from a lazy pool test, but the no-op path should
        // return false regardless.
        let changed = set_review_guild(&pool, ID_INSERT, false).await?;
        assert!(!changed);
        Ok(())
    }
}