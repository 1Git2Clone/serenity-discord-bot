use crate::prelude::*;
use std::sync::Mutex;

pub type UserData = HashMap<(UserId, GuildId), i64>;
pub type UserCooldowns = Mutex<UserData>;

lazy_static! {
    pub static ref USER_COOLDOWNS: UserCooldowns = Mutex::new(HashMap::new());
}
