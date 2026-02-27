use crate::prelude::*;
use moka::future::Cache;
use std::time::Duration;

pub static USER_COOLDOWNS: LazyLock<Cache<(UserId, GuildId), ()>> = LazyLock::new(|| {
    Cache::builder()
        .time_to_live(Duration::from_secs(XP_COOLDOWN_NUMBER_SECS as u64))
        .build()
});
