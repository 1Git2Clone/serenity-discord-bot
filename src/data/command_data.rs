use crate::prelude::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Data {
    pub bot_user: Arc<serenity::CurrentUser>,
    pub bot_avatar: Arc<str>,
    pub available_commands: Vec<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub pool: Arc<SqlitePool>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
