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
