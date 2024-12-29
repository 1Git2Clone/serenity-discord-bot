use crate::prelude::*;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Data {
    pub bot_user: Arc<serenity::CurrentUser>,
    pub bot_avatar: Arc<str>,
    pub available_commands: Vec<String>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
