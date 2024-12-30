use std::sync::MutexGuard;

use crate::{data::user_data::UserData, prelude::*};

pub fn process_user_cooldowns<'src, F, Res>(f: F) -> Result<Res, Error>
where
    F: FnOnce(MutexGuard<'src, UserData>) -> Res,
    Res: std::any::Any,
{
    let mutex = USER_COOLDOWNS.lock().map_err(|why| format!("{why}"))?;
    Ok(f(mutex))
}
