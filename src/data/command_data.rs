use poise::serenity_prelude as serenity;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Data {
    pub hutao_mentions: AtomicU32,
    pub bot_user: Arc<serenity::CurrentUser>,
    pub bot_avatar: Arc<str>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
#[allow(unused)]
pub type Context<'a> = poise::Context<'a, Data, Error>;
