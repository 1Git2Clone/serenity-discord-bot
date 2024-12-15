use super::*;

/// Get a motivation chair GIF
#[poise::command(slash_command, prefix_command)]
pub async fn chair(ctx: Context<'_>) -> Result<(), Error> {
    let embed_item: &str = cmd_utils::get_rand_embed_from_type(&EmbedType::Chair)?;
    let bot_user = Arc::clone(&ctx.data().bot_user);

    let embed = serenity::CreateEmbed::new()
        .title("You need some motivation!")
        .color((255, 0, 0))
        .image(embed_item)
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(Arc::clone(&ctx.data().bot_avatar).to_string()),
        );
    let full_respone = poise::CreateReply::default().embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}
