use std::sync::LazyLock;

use crate::{enums::schemas::AiChannelsTable, prelude::*};
use dashmap::DashSet;

/// Channels where the bot auto-replies to every message. Backed by the
/// `ai_channels` table but kept in memory so the message handler avoids a DB hit
/// per message. Populated by [`init_registered_channels`] at startup.
pub static AI_REGISTERED_CHANNELS: LazyLock<DashSet<u64>> = LazyLock::new(DashSet::new);

/// Load the registered AI channels from the DB into the in-memory set.
pub async fn init_registered_channels(pool: &PgPool) -> Result<(), Error> {
    for channel_id in AiChannelsTable::fetch_all(pool).await? {
        AI_REGISTERED_CHANNELS.insert(channel_id as u64);
    }
    Ok(())
}

pub fn is_ai_channel(channel_id: u64) -> bool {
    AI_REGISTERED_CHANNELS.contains(&channel_id)
}

/// Toggle a channel's AI registration in both the DB and the in-memory set.
/// Returns `true` if it's now registered, `false` if it was removed.
pub async fn toggle_ai_channel(pool: &PgPool, channel_id: u64, guild_id: u64) -> Result<bool, Error> {
    if AI_REGISTERED_CHANNELS.contains(&channel_id) {
        AiChannelsTable::unregister(pool, channel_id as i64).await?;
        AI_REGISTERED_CHANNELS.remove(&channel_id);
        Ok(false)
    } else {
        AiChannelsTable::register(pool, channel_id as i64, guild_id as i64).await?;
        AI_REGISTERED_CHANNELS.insert(channel_id);
        Ok(true)
    }
}
