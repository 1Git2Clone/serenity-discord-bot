use super::*;
use crate::data::command_data::{Context, Error};

// This is where the poise framework shines since with it you can make
// a slash and a prefix command work in one function.
//
// Docs for reference:
// https://docs.rs/poise/latest/poise/reply/fn.send_reply.html

/// Pat someone
#[poise::command(slash_command, prefix_command, rename = "pat")]
pub async fn pat(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());

    if u == ctx.author() {
        ctx.reply("Aww... I'll pat you~ *pat pat*").await?;
        return Ok(());
    }

    let response = format!("**{}** *pats* **{}**", ctx.author().name, u.name);

    let embed = serenity::CreateEmbed::new()
        .color((255, 0, 0))
        .image("https://cdn.discordapp.com/attachments/1187355380087537668/1212438556409077831/gQIhfkz.gif?ex=65f1d665&is=65df6165&hm=cb48d221d2ef26bcc1def5122b28b95e31b73ce224dfecc44bfb95fbc927b02e&");

    let make_message = poise::CreateReply::default().content(response).embed(embed);

    ctx.send(make_message).await?;

    Ok(())
}

/// Get the avatar for someone.
#[poise::command(slash_command, prefix_command, rename = "avatar")]
pub async fn avatar(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());

    let response = format!("**{}**'s avatar:", u.name);
    let user_avatar = u.face().replace(".webp", ".png");
    println!("{user_avatar}");

    let embed = serenity::CreateEmbed::new()
        .color((255, 0, 0))
        .image(user_avatar);
    let make_message = poise::CreateReply::default().content(response).embed(embed);

    ctx.send(make_message).await?;

    Ok(())
}
