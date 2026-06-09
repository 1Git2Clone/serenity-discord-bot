//! The no-interaction embed commands, built on
//! [`send_embed`](super::spec::send_embed).

use super::spec::send_embed;
use crate::prelude::*;

/// Bury yourself (perhaps to help Hu Tao's busines idk...)
#[poise::command(discard_spare_arguments, prefix_command, slash_command)]
pub async fn selfbury(ctx: Context<'_>) -> Result<(), Error> {
    let image = cmd_utils::get_rand_embed_from_type(&EmbedType::SelfBury)?.to_string();
    let title = format!("**{}** *buries themselves*", ctx.author().name);
    send_embed(ctx, Some(title), image).await
}

/// Send a peek GIF in the chat (you lurker)
#[poise::command(discard_spare_arguments, prefix_command, slash_command)]
pub async fn peek(ctx: Context<'_>) -> Result<(), Error> {
    let image = cmd_utils::get_rand_embed_from_type(&EmbedType::Peek)?.to_string();
    let title = format!("{} is lurking . . .", ctx.author().name);
    send_embed(ctx, Some(title), image).await
}

/// Get a Ryan Gosling drive GIF.
#[poise::command(discard_spare_arguments, slash_command, prefix_command)]
pub async fn drive(ctx: Context<'_>) -> Result<(), Error> {
    let image = cmd_utils::get_rand_embed_from_type(&EmbedType::RyanGoslingDrive)?.to_string();
    send_embed(ctx, None, image).await
}

/// Get a motivation chair GIF
#[poise::command(discard_spare_arguments, slash_command, prefix_command)]
pub async fn chair(ctx: Context<'_>) -> Result<(), Error> {
    let image = cmd_utils::get_rand_embed_from_type(&EmbedType::Chair)?.to_string();
    send_embed(ctx, Some("You need some motivation!".to_string()), image).await
}

/// Just try it.
#[poise::command(discard_spare_arguments, slash_command, prefix_command)]
pub async fn boom(ctx: Context<'_>) -> Result<(), Error> {
    send_embed(ctx, None, Assets::HuBoom.to_string()).await
}

/// Get the avatar for someone.
// Not on `send_embed`: it shows the target's avatar with no bot footer.
#[poise::command(discard_spare_arguments, slash_command, prefix_command)]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        target_user = %user.as_ref().map(|u| u.id.get()).unwrap_or(0),
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
    )
)]
pub async fn avatar(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target = user.as_ref().unwrap_or(get_replied_user(ctx).await);

    let embed = serenity::CreateEmbed::new()
        .title(format!("**{}**'s avatar:", target.name))
        .color((255, 0, 0))
        .image(target.face().replace(".webp", ".png"));
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}
