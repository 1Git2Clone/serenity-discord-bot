use super::*;
use crate::data::{
    command_data::{Context, Error},
    embed_media::COMMANDS,
};
use crate::enums::command_enums::EmbedType;
use rand::prelude::SliceRandom;
use sqlx::error::BoxDynError;

pub async fn get_replied_user(ctx: Context<'_>) -> Option<poise::serenity_prelude::User> {
    let poise::Context::Prefix(msg_ctx) = ctx else {
        return None;
    };
    let ref_msg = msg_ctx.msg.referenced_message.as_deref();
    ref_msg.map(|x| x.author.clone())
}

pub async fn get_embed_from_type(embed_type: &EmbedType) -> Result<&'static str, Error> {
    let embed_option = COMMANDS[embed_type].choose(&mut rand::thread_rng());
    match embed_option {
        Some(embed) => Ok(embed),
        None => Err(BoxDynError::from(
            "Failed to get item from the matching vector of strings from the Hash Map.",
        )),
    }
}

pub async fn get_bot_user(ctx: Context<'_>) -> serenity::User {
    ctx.http()
        .get_user(ctx.framework().bot_id)
        .await
        .expect("Retrieving the bot user shouldn't fail.")
}

pub async fn get_bot_avatar(ctx: Context<'_>, bot_user: Option<serenity::User>) -> String {
    let match_bot_user = match bot_user {
        Some(user) => user,
        None => get_bot_user(ctx).await,
    };
    match_bot_user.face().replace(".webp", ".png")
}
