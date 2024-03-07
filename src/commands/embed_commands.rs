use ::serenity::all::Mentionable;

use super::*;
use crate::data::command_data::{Context, Error};
use crate::enums::command_enums::EmbedType;

// This is where the poise framework shines since with it you can make
// a slash and a prefix command work in one function.
//
// Docs for reference:
// https://docs.rs/poise/latest/poise/reply/fn.send_reply.html

/// Pat someone
#[poise::command(prefix_command, slash_command, rename = "pat")]
pub async fn pat(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_user = cmd_utils::get_user(ctx, user).await;
    if &target_user == ctx.author() {
        ctx.reply("Aww I'll pat you~ *pat pat*").await?;
        return Ok(());
    }
    let embed_item: &str = cmd_utils::get_embed_from_type(
            &EmbedType::Pat,
            "https://cdn.discordapp.com/attachments/614790390020833281/1183493730339139694/hu-tao-hug.gif?ex=65f7476d&is=65e4d26d&hm=acc5a8f998ee80ae8198019d96c407119686c0168a12d74adf057789eb5a8c75&"
        ).await;

    let response: String = format!("**{}** *pats* **{}**", ctx.author().name, target_user.name);

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_user.mention()),
    };

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(embed_item.to_string());

    let full_respone = poise::CreateReply::default()
        .content(match ping_on_shash_command {
            Some(ping) => format!("{}", ping),
            None => "".to_owned(),
        })
        .embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}

/// Get the avatar for someone.
#[poise::command(slash_command, prefix_command, rename = "avatar")]
pub async fn avatar(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_user: poise::serenity_prelude::User = cmd_utils::get_user(ctx, user).await;
    let response: String = format!("**{}**'s avatar:", target_user.name);
    let user_avatar_as_embed: String = target_user.face().replace(".webp", ".png");

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(user_avatar_as_embed);
    let full_respone = poise::CreateReply::default().embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}
