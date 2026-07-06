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
    let key = prompt_key(guild_id);
    let key_for_read = key.clone();
    let key_for_write = key.clone();
    cache::write_through::get_or_load::<String, _>(
        move |conn| Box::pin(async move { cache::get_string(conn, &key_for_read).await }),
        move || async move {
            match GuildAiSettingsTable::fetch(pool, guild_id).await {
                Ok(prompt) => Ok::<String, Error>(prompt.unwrap_or_default()),
                Err(e) => {
                    tracing::warn!(error = %e, guild_id, "Failed to fetch guild AI prompt");
                    Err(Error::from(e))
                }
            }
        },
        move |conn, value| {
            Box::pin(async move {
                cache::set_string_ex(conn, &key_for_write, value, PROMPT_TTL_SECS).await
            })
        },
    )
    .await
    // Empty string is the negative-cache sentinel for "no prompt set".
    .ok()
    .and_then(|s| if s.is_empty() { None } else { Some(s) })
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
        // A second read with no intervening mutation is served from the Redis
        // cache (the prior read populated it), not the DB.
        assert_eq!(
            get_guild_prompt(&pool, GUILD).await.as_deref(),
            Some("be extra cheerful")
        );

        assert!(delete_guild_prompt(&pool, GUILD).await?);
        assert!(get_guild_prompt(&pool, GUILD).await.is_none());
        // The cached negative sentinel also serves "no prompt" from the cache.
        assert!(get_guild_prompt(&pool, GUILD).await.is_none());
        Ok(())
    }
}
