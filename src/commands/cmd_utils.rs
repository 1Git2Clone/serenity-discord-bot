use crate::data::{
    command_data::{Context, Error},
    embed_media::COMMAND_EMBEDS,
};
use crate::enums::command_enums::EmbedType;
use ::serenity::all::Mentionable;
use poise::serenity_prelude as serenity;
use rand::prelude::SliceRandom;
use sqlx::error::BoxDynError;

/// Works by prepending the `ASSETS_URL` to the `asset_file_name`.
///
/// ```rust
/// use serenity_discord_bot::asset_url;
/// use serenity_discord_bot::commands::cmd_utils::HU_BOOM_URL;
///
/// assert_eq!(
///     "https://raw.githubusercontent.com/1Git2Clone/serenity-discord-bot/main/src/assets/hu_boom.jpg",
///     asset_url!("hu_boom.jpg")
/// );
/// assert_eq!(
///     "https://raw.githubusercontent.com/1Git2Clone/serenity-discord-bot/main/src/assets/hu_boom.jpg",
///     HU_BOOM_URL
/// );
/// ```
#[macro_export]
macro_rules! asset_url {
    ($expr:expr) => {
        concat!(
            "https://raw.githubusercontent.com/1Git2Clone/serenity-discord-bot/main/src/assets/",
            $expr
        )
    };
}

pub const HU_BOOM_URL: &str = asset_url!("hu_boom.jpg");

pub async fn get_replied_user(ctx: Context<'_>) -> &serenity::User {
    let poise::Context::Prefix(msg_ctx) = ctx else {
        return ctx.author();
    };
    let ref_msg = msg_ctx.msg.referenced_message.as_deref();
    ref_msg.map_or(ctx.author(), |x| &x.author)
}

pub fn get_rand_embed_from_type(embed_type: &EmbedType) -> Result<&'static str, Error> {
    let embed_option = COMMAND_EMBEDS[embed_type].choose(&mut rand::thread_rng());
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
