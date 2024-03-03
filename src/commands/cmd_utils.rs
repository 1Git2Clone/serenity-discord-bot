use super::*;
use crate::data::command_data::Context;
pub async fn get_user(
    ctx: Context<'_>,
    user: Option<serenity::User>,
) -> poise::serenity_prelude::User {
    let is_msg: Option<&poise::serenity_prelude::model::channel::Message> = match ctx {
        poise::Context::Prefix(prefix_cmd) => Some(prefix_cmd.msg),
        poise::Context::Application(_) => None,
    };
    let msg = match is_msg {
        Some(msg) => msg.to_owned(),
        None => poise::serenity_prelude::model::channel::Message::default(),
    };

    let ref_msg = match msg.referenced_message {
        Some(referenced_message) => referenced_message,
        None => Box::default(),
    };
    user.unwrap_or_else(|| {
        if ref_msg.author.id != 1 {
            ref_msg.author
        } else {
            ctx.author().clone()
        }
    })
}
