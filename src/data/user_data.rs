use lazy_static::lazy_static;
use poise::serenity_prelude as serenity;
use serenity::model::id::GuildId;
use serenity::model::id::UserId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

lazy_static! {
    pub(crate) static ref USER_COOLDOWNS: Arc<Mutex<HashMap<(UserId, GuildId), i64>>> =
        Arc::new(Mutex::new(HashMap::new()));
}
