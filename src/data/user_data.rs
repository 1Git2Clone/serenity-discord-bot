use crate::prelude::*;
use std::sync::Mutex;

type UserData = HashMap<(UserId, GuildId), i64>;

lazy_static! {
    pub static ref USER_COOLDOWNS: Arc<Mutex<UserData>> = Arc::new(Mutex::new(HashMap::new()));
}
