use crate::data::{
    command_data::{Context, Error},
    embed_media::COMMANDS,
};
use crate::enums::command_enums::EmbedType;
use ::serenity::all::Mentionable;
use poise::serenity_prelude as serenity;
use rand::prelude::SliceRandom;
use sqlx::error::BoxDynError;

pub async fn get_replied_user(ctx: Context<'_>) -> &serenity::User {
    let poise::Context::Prefix(msg_ctx) = ctx else {
        return ctx.author();
    };
    let ref_msg = msg_ctx.msg.referenced_message.as_deref();
    ref_msg.map_or(ctx.author(), |x| &x.author)
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

pub async fn make_full_response(
    ctx: &Context<'_>,
    target_replied_user: &serenity::User,
    embed: Option<serenity::CreateEmbed>,
) -> poise::CreateReply {
    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_replied_user.mention()),
    };

    let mut reply = poise::CreateReply::default().content(match ping_on_shash_command {
        Some(ping) => format!("{}", ping),
        None => "".into(),
    });

    if let Some(e) = embed {
        reply = reply.embed(e);
    };

    reply
}
