use std::sync::LazyLock;

use crate::{enums::schemas::AiReviewGuildsTable, prelude::*};
use dashmap::DashSet;

/// Guilds where `/ai-review run` is allowed. Backed by the `ai_review_guilds`
/// table but kept in memory so the command avoids a DB hit per invocation.
/// Populated by [`init_review_guilds`] at startup.
pub static AI_REVIEW_GUILDS: LazyLock<DashSet<u64>> = LazyLock::new(DashSet::new);

/// Load the enabled guilds from the DB into the in-memory set.
pub async fn init_review_guilds(pool: &PgPool) -> Result<(), Error> {
    for guild_id in AiReviewGuildsTable::fetch_all(pool).await? {
        AI_REVIEW_GUILDS.insert(guild_id as u64);
    }
    Ok(())
}

pub fn is_review_guild(guild_id: u64) -> bool {
    AI_REVIEW_GUILDS.contains(&guild_id)
}

/// Enable or disable `/ai-review run` for a guild in both the DB and the
/// in-memory set. Returns `true` if the state changed.
#[tracing::instrument(
    skip(pool),
    fields(category = "sql", guild_id = %guild_id, enabled = %enabled)
)]
pub async fn set_review_guild(pool: &PgPool, guild_id: u64, enabled: bool) -> Result<bool, Error> {
    if enabled {
        if AI_REVIEW_GUILDS.contains(&guild_id) {
            return Ok(false);
        }
        AiReviewGuildsTable::register(pool, guild_id as i64).await?;
        AI_REVIEW_GUILDS.insert(guild_id);
    } else {
        if !AI_REVIEW_GUILDS.contains(&guild_id) {
            return Ok(false);
        }
        AiReviewGuildsTable::unregister(pool, guild_id as i64).await?;
        AI_REVIEW_GUILDS.remove(&guild_id);
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data::command_data::Error, enums::schemas::AiReviewGuildsTable};

    type TestResult = Result<(), Error>;

    // Use large sentinel IDs to avoid colliding with production data or other tests.
    // Each test owns exactly one ID; no two tests share an ID to avoid races under
    // parallel test execution.
    const ID_IS_REVIEW: u64 = 0x5AFE_0001_0000_0001; // never inserted — absence check only
    const ID_INSERT: u64 = 0x5AFE_0001_0000_0007; // insert/remove round-trip only
    const ID_INIT: u64 = 0x5AFE_0001_0000_0002;
    const ID_ENABLE: u64 = 0x5AFE_0001_0000_0003;
    const ID_DISABLE: u64 = 0x5AFE_0001_0000_0004;
    const ID_NOOP_ON: u64 = 0x5AFE_0001_0000_0005;
    const ID_NOOP_OFF: u64 = 0x5AFE_0001_0000_0006;

    #[test]
    fn is_review_guild_unknown_returns_false() {
        assert!(!is_review_guild(ID_IS_REVIEW));
    }

    #[test]
    fn is_review_guild_after_manual_insert_returns_true() {
        AI_REVIEW_GUILDS.insert(ID_INSERT);
        assert!(is_review_guild(ID_INSERT));
        AI_REVIEW_GUILDS.remove(&ID_INSERT);
    }

    // These two tests return before touching the DB, so a lazy (never-connects) pool suffices.
    #[tokio::test]
    async fn set_review_guild_noop_when_already_enabled() -> TestResult {
        let pool = PgPoolOptions::new().connect_lazy("postgres://localhost/unused")?;
        AI_REVIEW_GUILDS.insert(ID_NOOP_ON);
        let changed = set_review_guild(&pool, ID_NOOP_ON, true).await?;
        assert!(!changed);
        AI_REVIEW_GUILDS.remove(&ID_NOOP_ON);
        Ok(())
    }

    #[tokio::test]
    async fn set_review_guild_noop_when_already_disabled() -> TestResult {
        let pool = PgPoolOptions::new().connect_lazy("postgres://localhost/unused")?;
        let changed = set_review_guild(&pool, ID_NOOP_OFF, false).await?;
        assert!(!changed);
        Ok(())
    }

    /// Connect to the database. Loads `.env` first (for local dev where the shell
    /// hasn't exported DATABASE_URL) then skips the test if the var is absent or
    /// the connection fails.
    async fn test_pool() -> Option<PgPool> {
        dotenv::dotenv().ok();
        let url = std::env::var("DATABASE_URL").ok()?;
        PgPoolOptions::new().connect(&url).await.ok()
    }

    #[tokio::test]
    async fn set_review_guild_enables_in_memory_and_db() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        // Ensure clean slate in case a previous run left debris.
        AI_REVIEW_GUILDS.remove(&ID_ENABLE);
        AiReviewGuildsTable::unregister(&pool, ID_ENABLE as i64).await?;

        let changed = set_review_guild(&pool, ID_ENABLE, true).await?;
        assert!(changed);
        assert!(is_review_guild(ID_ENABLE));

        // cleanup
        set_review_guild(&pool, ID_ENABLE, false).await?;
        Ok(())
    }

    #[tokio::test]
    async fn set_review_guild_disables_in_memory_and_db() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        AI_REVIEW_GUILDS.remove(&ID_DISABLE);
        AiReviewGuildsTable::unregister(&pool, ID_DISABLE as i64).await?;

        set_review_guild(&pool, ID_DISABLE, true).await?;
        let changed = set_review_guild(&pool, ID_DISABLE, false).await?;
        assert!(changed);
        assert!(!is_review_guild(ID_DISABLE));
        Ok(())
    }

    #[tokio::test]
    async fn init_review_guilds_populates_from_db() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        AI_REVIEW_GUILDS.remove(&ID_INIT);
        AiReviewGuildsTable::unregister(&pool, ID_INIT as i64).await?;

        AiReviewGuildsTable::register(&pool, ID_INIT as i64).await?;
        init_review_guilds(&pool).await?;
        assert!(is_review_guild(ID_INIT));

        // cleanup
        set_review_guild(&pool, ID_INIT, false).await?;
        Ok(())
    }
}
