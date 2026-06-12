use crate::{data::cache, enums::schemas::AiChannelsTable, prelude::*};

const AI_CHANNELS_KEY: &str = "ai:channels";

/// Load the registered AI channels from the DB into Redis, then seed Redis
/// if empty (handles cold start without touching the DB on every check).
pub async fn init_registered_channels(pool: &PgPool) -> Result<(), Error> {
    let Some(mut conn) = cache::conn().await else {
        return Ok(());
    };

    // Seed Redis from the DB if the set is empty.
    if cache::set_members(&mut conn, AI_CHANNELS_KEY).await?.is_empty() {
        for channel_id in AiChannelsTable::fetch_all(pool).await? {
            cache::set_add(&mut conn, AI_CHANNELS_KEY, channel_id as u64).await?;
        }
    }
    Ok(())
}

/// Check whether a channel has AI auto-replies enabled. Falls back to `false`
/// when Redis is unavailable (graceful degradation).
pub async fn is_ai_channel(channel_id: u64) -> bool {
    let Some(mut conn) = cache::conn().await else {
        return false;
    };
    cache::set_contains(&mut conn, AI_CHANNELS_KEY, channel_id)
        .await
        .unwrap_or(false)
}

/// Toggle a channel's AI registration in both the DB and Redis.
/// Returns `true` if it's now registered, `false` if it was removed.
pub async fn toggle_ai_channel(pool: &PgPool, channel_id: u64, guild_id: u64) -> Result<bool, Error> {
    let mut conn = match cache::conn().await {
        Some(c) => c,
        None => {
            return toggle_ai_channel_db_only(pool, channel_id, guild_id).await;
        }
    };

    // Try Redis-aware path; fall back to DB-only on transient errors.
    match toggle_ai_channel_redis(pool, channel_id, guild_id, &mut conn).await {
        Ok(changed) => Ok(changed),
        Err(e) => {
            tracing::warn!(error = %e, channel_id, guild_id, "Redis error in toggle_ai_channel; falling back to DB-only");
            toggle_ai_channel_db_only(pool, channel_id, guild_id).await
        }
    }
}

async fn toggle_ai_channel_db_only(pool: &PgPool, channel_id: u64, guild_id: u64) -> Result<bool, Error> {
    let is_registered = AiChannelsTable::fetch_all(pool)
        .await?
        .contains(&(channel_id as i64));
    if is_registered {
        AiChannelsTable::unregister(pool, channel_id as i64).await?;
        Ok(false)
    } else {
        AiChannelsTable::register(pool, channel_id as i64, guild_id as i64).await?;
        Ok(true)
    }
}

async fn toggle_ai_channel_redis(
    pool: &PgPool,
    channel_id: u64,
    guild_id: u64,
    conn: &mut redis::aio::ConnectionManager,
) -> Result<bool, Error> {
    if cache::set_contains(conn, AI_CHANNELS_KEY, channel_id).await? {
        AiChannelsTable::unregister(pool, channel_id as i64).await?;
        cache::set_remove(conn, AI_CHANNELS_KEY, channel_id).await?;
        Ok(false)
    } else {
        AiChannelsTable::register(pool, channel_id as i64, guild_id as i64).await?;
        cache::set_add(conn, AI_CHANNELS_KEY, channel_id).await?;
        Ok(true)
    }
}