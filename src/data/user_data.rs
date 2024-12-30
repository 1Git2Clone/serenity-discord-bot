use crate::prelude::*;
use std::sync::Mutex;

pub type UserData = HashMap<(UserId, GuildId), i64>;
pub type UserCooldowns = Arc<Mutex<UserData>>;

lazy_static! {
    pub static ref USER_COOLDOWNS: UserCooldowns = Arc::new(Mutex::new(HashMap::new()));
}
