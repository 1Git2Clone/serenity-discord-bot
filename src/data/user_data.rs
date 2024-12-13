use lazy_static::lazy_static;
use poise::serenity_prelude as serenity;
use serenity::model::id::GuildId;
use serenity::model::id::UserId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type UserData = HashMap<(UserId, GuildId), i64>;

lazy_static! {
    pub(crate) static ref USER_COOLDOWNS: Arc<Mutex<UserData>> =
        Arc::new(Mutex::new(HashMap::new()));
}
