//! This file is running on a seperate thread for optimization purposes.
//!
//! We don't want a GC-line part of the code interrupt the main flow of the program so we just take
//! it out and move it to another thread. This is the nice part about global state management.

use chrono::Utc;
use std::time::Duration;
use tokio::spawn;

use crate::data::{bot_data::XP_COOLDOWN_NUMBER_SECS, user_data::USER_COOLDOWNS};

fn remove_expired_cooldowns() {
    let mut cooldowns = USER_COOLDOWNS.lock().unwrap();
    let current_timestamp: i64 = Utc::now().timestamp();

    cooldowns.retain(|_, timestamp| *timestamp + *XP_COOLDOWN_NUMBER_SECS > current_timestamp);
}

pub fn periodically_clean_users_on_diff_thread() {
    spawn(async move {
        loop {
            std::thread::sleep(Duration::from_secs(100));
            remove_expired_cooldowns();
        }
    });
}
