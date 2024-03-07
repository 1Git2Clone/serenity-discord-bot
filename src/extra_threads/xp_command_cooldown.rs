use chrono::Utc;
use std::time::Duration;

use crate::data::{bot_data::XP_COOLDOWN_NUMBER_SECS, user_data::USER_COOLDOWNS};

fn remove_expired_cooldowns() {
    let mut cooldowns = USER_COOLDOWNS.lock().unwrap();
    let current_timestamp: i64 = Utc::now().timestamp();

    cooldowns.retain(|_, timestamp| *timestamp + *XP_COOLDOWN_NUMBER_SECS > current_timestamp);
}

pub fn periodically_clean_users_on_diff_thread() {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(100));
        remove_expired_cooldowns();
    });
}
