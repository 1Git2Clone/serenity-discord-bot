use super::*;
use crate::data::{command_data::Context, embed_media::COMMANDS};
use crate::enums::command_enums::EmbedType;
use rand::prelude::SliceRandom;

pub async fn get_user(
    ctx: Context<'_>,
    user: Option<serenity::User>,
) -> poise::serenity_prelude::User {
    let initial_user = user.as_ref().unwrap_or_else(|| ctx.author());

    if initial_user != ctx.author() {
        return initial_user.to_owned();
    }

    let is_msg: Option<&poise::serenity_prelude::model::channel::Message> = match ctx {
        poise::Context::Prefix(prefix_cmd) => Some(prefix_cmd.msg),
        poise::Context::Application(_) => None,
    };
    if is_msg.is_none() {
        return initial_user.to_owned();
    }

    let msg = match is_msg {
        Some(msg) => msg.to_owned(),
        None => poise::serenity_prelude::model::channel::Message::default(),
    };
    if msg.id == 1 {
        return initial_user.to_owned();
    }

    let ref_msg = match msg.referenced_message {
        Some(referenced_message) => referenced_message,
        None => Box::default(),
    };
    if ref_msg.author.id == 1 {
        initial_user.to_owned()
    } else {
        ref_msg.author
    }
}

pub async fn get_embed_from_type(embed_type: &EmbedType) -> &'static str {
    let embed_option = COMMANDS[embed_type].choose(&mut rand::thread_rng());
    match embed_option {
        Some(embed) => embed,
        // Set a default one just in case!
        None => "",
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
