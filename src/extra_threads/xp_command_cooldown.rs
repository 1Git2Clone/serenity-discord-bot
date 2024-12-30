//! This file is running on a seperate thread for optimization purposes.
//!
//! We don't want a GC-line part of the code interrupt the main flow of the program so we just take
//! it out and move it to another thread. This is the nice part about global state management.

use chrono::Utc;
use std::{
    sync::{MutexGuard, PoisonError},
    time::Duration,
};
use tokio::spawn;

use crate::{data::user_data::UserData, prelude::*};

fn remove_expired_cooldowns<'src>() -> Result<(), PoisonError<MutexGuard<'src, UserData>>> {
    process_mutex(&USER_COOLDOWNS, |mut cooldowns| {
        let current_timestamp: i64 = Utc::now().timestamp();

        cooldowns.retain(|_, timestamp| *timestamp + *XP_COOLDOWN_NUMBER_SECS > current_timestamp);
    })?;

    Ok(())
}

pub fn periodically_clean_users_on_diff_thread() {
    spawn(async move {
        loop {
            std::thread::sleep(Duration::from_secs(100));
            let res = remove_expired_cooldowns();
            if let Err(why) = res {
                eprintln!("{why}");
                break;
            };
        }
    });
}
