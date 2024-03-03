use std::sync::atomic::AtomicU32;

pub struct Data {
    pub poise_mentions: AtomicU32,
    pub user_message: serenity::model::channel::Message,
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
#[allow(unused)]

pub type Context<'a> = poise::Context<'a, Data, Error>;
