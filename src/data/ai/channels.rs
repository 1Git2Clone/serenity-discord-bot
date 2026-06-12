use crate::{data::cache, enums::schemas::AiChannelsTable, prelude::*};

const AI_CHANNELS_KEY: &str = "ai:channels";

/// Load the registered AI channels from the DB into Redis, then seed Redis
/// if empty (handles cold start without touching the DB on every check).
pub async fn init_registered_channels(pool: &PgPool) -> Result<(), Error> {
    let Some(mut conn) = cache::conn().await else {
        return Ok(());
    };

    // Seed Redis from the DB if the set is empty.
    if !cache::key_exists(&mut conn, AI_CHANNELS_KEY).await? {
        for channel_id in AiChannelsTable::fetch_all(pool).await? {
            cache::set_add(&mut conn, AI_CHANNELS_KEY, channel_id as u64).await?;
        }
    }
    Ok(())
}

/// Check whether a channel has AI auto-replies enabled. This runs per
/// message, so a Redis answer (hit or miss) is trusted; the DB is only
/// queried when Redis is unavailable or the call errors.
pub async fn is_ai_channel(pool: &PgPool, channel_id: u64) -> bool {
    if let Some(mut conn) = cache::conn().await
        && let Ok(contains) = cache::set_contains(&mut conn, AI_CHANNELS_KEY, channel_id).await
    {
        return contains;
    }
    AiChannelsTable::fetch_all(pool)
        .await
        .map(|v| v.contains(&(channel_id as i64)))
        .unwrap_or(false)
}

/// Toggle a channel's AI registration. The DB decides the new state; the
/// Redis set is a best-effort cache update on top.
/// Returns `true` if it's now registered, `false` if it was removed.
pub async fn toggle_ai_channel(pool: &PgPool, channel_id: u64, guild_id: u64) -> Result<bool, Error> {
    // A no-op register means the channel was already there: toggle off.
    let registered = if AiChannelsTable::register(pool, channel_id as i64, guild_id as i64).await? {
        true
    } else {
        AiChannelsTable::unregister(pool, channel_id as i64).await?;
        false
    };

    if let Some(mut conn) = cache::conn().await {
        let res = if registered {
            cache::set_add(&mut conn, AI_CHANNELS_KEY, channel_id).await
        } else {
            cache::set_remove(&mut conn, AI_CHANNELS_KEY, channel_id).await
        };
        if let Err(e) = res {
            tracing::warn!(error = %e, channel_id, "Failed to update Redis AI-channel cache");
        }
    }

    Ok(registered)
}
