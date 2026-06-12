use crate::{enums::schemas::AiReviewGuildsTable, prelude::*};

/// Check whether `/ai-review run` is enabled for a guild. This is an
/// authorization gate, so the DB is queried directly — a stale cached "yes"
/// would be unsafe, and the command is invoked rarely enough that one SELECT
/// per run doesn't matter.
pub async fn is_review_guild(pool: &PgPool, guild_id: u64) -> bool {
    AiReviewGuildsTable::fetch_all(pool)
        .await
        .map(|v| v.contains(&(guild_id as i64)))
        .unwrap_or(false)
}

/// Enable or disable `/ai-review run` for a guild. The DB write itself
/// decides whether the state changed. Returns `true` if it did.
#[tracing::instrument(
    skip(pool),
    fields(category = "sql", guild_id = %guild_id, enabled = %enabled)
)]
pub async fn set_review_guild(pool: &PgPool, guild_id: u64, enabled: bool) -> Result<bool, Error> {
    Ok(if enabled {
        AiReviewGuildsTable::register(pool, guild_id as i64).await?
    } else {
        AiReviewGuildsTable::unregister(pool, guild_id as i64).await?
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data::command_data::Error, enums::schemas::AiReviewGuildsTable, tests::test_pool};

    type TestResult = Result<(), Error>;

    // Use large sentinel IDs to avoid colliding with production data or other tests.
    // Each test owns exactly one ID; no two tests share an ID to avoid races under
    // parallel test execution.
    const ID_NOOP: u64 = 0x5AFE_0001_0000_0007;
    const ID_ENABLE: u64 = 0x5AFE_0001_0000_0003;
    const ID_DISABLE: u64 = 0x5AFE_0001_0000_0004;

    #[tokio::test]
    async fn set_review_guild_enables_in_db() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        AiReviewGuildsTable::unregister(&pool, ID_ENABLE as i64).await?;

        let changed = set_review_guild(&pool, ID_ENABLE, true).await?;
        assert!(changed);
        assert!(is_review_guild(&pool, ID_ENABLE).await);

        // cleanup
        AiReviewGuildsTable::unregister(&pool, ID_ENABLE as i64).await?;
        Ok(())
    }

    #[tokio::test]
    async fn set_review_guild_disables_in_db() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        AiReviewGuildsTable::unregister(&pool, ID_DISABLE as i64).await?;

        set_review_guild(&pool, ID_DISABLE, true).await?;
        let changed = set_review_guild(&pool, ID_DISABLE, false).await?;
        assert!(changed);
        assert!(!is_review_guild(&pool, ID_DISABLE).await);
        Ok(())
    }

    #[tokio::test]
    async fn set_review_guild_noop_when_already_disabled() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        AiReviewGuildsTable::unregister(&pool, ID_NOOP as i64).await?;

        let changed = set_review_guild(&pool, ID_NOOP, false).await?;
        assert!(!changed);
        Ok(())
    }
}
