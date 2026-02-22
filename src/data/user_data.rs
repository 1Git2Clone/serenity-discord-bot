use crate::prelude::*;
use moka::future::Cache;
use std::time::Duration;

lazy_static! {
    pub static ref USER_COOLDOWNS: Cache<(UserId, GuildId), ()> = Cache::builder()
        .time_to_live(Duration::from_secs(*XP_COOLDOWN_NUMBER_SECS as u64))
        .build();
}
