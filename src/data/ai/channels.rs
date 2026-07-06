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
/// queried when Redis is unavailable, the call errors, or the channel isn't
/// in the set. A confirmed DB hit is written back to Redis so subsequent
/// reads stay off the DB.
pub async fn is_ai_channel(pool: &PgPool, channel_id: u64) -> bool {
    cache::write_through::get_or_load::<bool, _>(
        move |conn| {
            let key = AI_CHANNELS_KEY;
            let id = channel_id;
            Box::pin(async move { cache::set_contains(conn, key, id).await.map(Some) })
        },
        move || async move {
            Ok::<bool, Error>(
                AiChannelsTable::fetch_all(pool)
                    .await
                    .map(|v| v.contains(&(channel_id as i64)))
                    .unwrap_or(false),
            )
        },
        move |conn, &registered| {
            let key = AI_CHANNELS_KEY;
            let id = channel_id;
            Box::pin(async move {
                if registered {
                    cache::set_add(conn, key, id).await
                } else {
                    Ok(())
                }
            })
        },
    )
    .await
    .unwrap_or(false)
}

/// Toggle a channel's AI registration. The DB decides the new state; the
/// Redis set is a best-effort cache update on top.
/// Returns `true` if it's now registered, `false` if it was removed.
pub async fn toggle_ai_channel(
    pool: &PgPool,
    channel_id: u64,
    guild_id: u64,
) -> Result<bool, Error> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_pool;

    type TestResult = Result<(), Error>;

    // Sentinel IDs in their own namespace, distinct from the schemas.rs tests.
    const CHANNEL: u64 = 0x5AFE_0003_0000_0001;
    const GUILD: u64 = 0x5AFE_0003_0000_0002;

    /// One test for the whole toggle/check/init flow so parallel tests can't
    /// interleave on the shared sentinel channel.
    #[tokio::test]
    async fn toggle_check_init_roundtrip() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };

        // Idempotent cleanup in case a previous run died mid-test.
        if is_ai_channel(&pool, CHANNEL).await {
            toggle_ai_channel(&pool, CHANNEL, GUILD).await?;
        }
        assert!(!is_ai_channel(&pool, CHANNEL).await);

        assert!(toggle_ai_channel(&pool, CHANNEL, GUILD).await?);
        assert!(is_ai_channel(&pool, CHANNEL).await);

        // Seeding is a no-op when already initialized, an actual seed
        // otherwise; either way it must succeed and keep the channel visible.
        init_registered_channels(&pool).await?;
        assert!(is_ai_channel(&pool, CHANNEL).await);

        assert!(!toggle_ai_channel(&pool, CHANNEL, GUILD).await?);
        assert!(!is_ai_channel(&pool, CHANNEL).await);
        Ok(())
    }
}
