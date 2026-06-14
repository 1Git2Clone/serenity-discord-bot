use crate::{data::cache, enums::schemas::GuildAiSettingsTable, prelude::*};

/// Max length of a guild's extra system prompt.
pub const MAX_PROMPT_LEN: usize = 1000;

/// How long a guild's extra prompt (or its absence) is cached in Redis.
const PROMPT_TTL_SECS: u64 = 1800;

fn prompt_key(guild_id: i64) -> String {
    format!("ai:guild_prompt:{guild_id}")
}

/// The guild's extra system prompt, preferring Redis. A cache miss reads the DB
/// and re-populates the cache — including the "no prompt" case, stored as an
/// empty-string sentinel — so the per-message hot path stays off the DB for the
/// many guilds that never set one.
pub async fn get_guild_prompt(pool: &PgPool, guild_id: i64) -> Option<String> {
    let Some(mut conn) = cache::conn().await else {
        return match GuildAiSettingsTable::fetch(pool, guild_id).await {
            Ok(prompt) => prompt,
            Err(e) => {
                tracing::warn!(error = %e, guild_id, "Failed to fetch guild AI prompt");
                None
            }
        };
    };

    let key = prompt_key(guild_id);
    if let Ok(Some(cached)) = cache::get_string(&mut conn, &key).await {
        // An empty string is the negative-cache sentinel for "no prompt set".
        return (!cached.is_empty()).then_some(cached);
    }

    // Cache miss (or read error): consult the DB and write the result back. Only
    // a value the DB actually returned is cached, so a transient DB error isn't.
    // A reader that races a concurrent mutation can repopulate a stale value for
    // up to PROMPT_TTL_SECS; mod prompt changes are rare and the staleness is
    // bounded, so that window is accepted rather than guarded with versioning.
    let prompt = match GuildAiSettingsTable::fetch(pool, guild_id).await {
        Ok(prompt) => prompt,
        Err(e) => {
            tracing::warn!(error = %e, guild_id, "Failed to fetch guild AI prompt");
            return None;
        }
    };
    if let Err(e) = cache::set_string_ex(
        &mut conn,
        &key,
        prompt.as_deref().unwrap_or(""),
        PROMPT_TTL_SECS,
    )
    .await
    {
        tracing::warn!(error = %e, guild_id, "Failed to cache guild AI prompt");
    }
    prompt
}

/// Set or replace the guild's extra prompt, then drop the cache entry.
pub async fn set_guild_prompt(pool: &PgPool, guild_id: i64, text: &str) -> Result<(), Error> {
    GuildAiSettingsTable::upsert(pool, guild_id, text).await?;
    invalidate(guild_id).await;
    Ok(())
}

/// Delete the guild's extra prompt. Returns `true` if one existed. Drops the
/// cache entry either way.
pub async fn delete_guild_prompt(pool: &PgPool, guild_id: i64) -> Result<bool, Error> {
    let existed = GuildAiSettingsTable::delete(pool, guild_id).await?;
    invalidate(guild_id).await;
    Ok(existed)
}

/// Drop the cached prompt so the next read repopulates from the DB.
async fn invalidate(guild_id: i64) {
    if let Some(mut conn) = cache::conn().await
        && let Err(e) = cache::del(&mut conn, &prompt_key(guild_id)).await
    {
        tracing::warn!(error = %e, guild_id, "Failed to invalidate guild AI prompt cache");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_pool;

    type TestResult = Result<(), Error>;

    // Sentinel guild id in its own namespace.
    const GUILD: i64 = 0x5AFE_0005_0000_0001;

    #[tokio::test]
    async fn set_get_delete_roundtrip() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        delete_guild_prompt(&pool, GUILD).await?;
        assert!(get_guild_prompt(&pool, GUILD).await.is_none());

        set_guild_prompt(&pool, GUILD, "be extra spooky").await?;
        assert_eq!(
            get_guild_prompt(&pool, GUILD).await.as_deref(),
            Some("be extra spooky")
        );

        // Replacing keeps a single value.
        set_guild_prompt(&pool, GUILD, "be extra cheerful").await?;
        assert_eq!(
            get_guild_prompt(&pool, GUILD).await.as_deref(),
            Some("be extra cheerful")
        );

        assert!(delete_guild_prompt(&pool, GUILD).await?);
        assert!(get_guild_prompt(&pool, GUILD).await.is_none());
        Ok(())
    }
}
