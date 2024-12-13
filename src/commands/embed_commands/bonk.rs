use super::*;

/// Bonk someone who's horknee
#[poise::command(prefix_command, slash_command, rename = "bonk")]
pub async fn bonk(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_replied_user = user.as_ref().unwrap_or(get_replied_user(ctx).await);
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Bonk).await?;
    let bot_user = Arc::clone(&ctx.data().bot_user);
    if target_replied_user == ctx.author() {
        ctx.send(
            poise::CreateReply::default().content("バカ！").embed(
                serenity::CreateEmbed::new()
                    .color((255, 0, 0))
                    .image(embed_item.to_string())
                    .footer(
                        serenity::CreateEmbedFooter::new(bot_user.tag())
                            .icon_url(Arc::clone(&ctx.data().bot_avatar).to_string()),
                    ),
            ),
        )
        .await?;
        return Ok(());
    }

    let response: String = format!(
        "**{}** *bonks* **{}**",
        ctx.author().name,
        target_replied_user.name
    );

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(embed_item.to_string())
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(Arc::clone(&ctx.data().bot_avatar).to_string()),
        );

    let full_respone = make_full_response(&ctx, target_replied_user, Some(embed)).await;
    ctx.send(full_respone).await?;

    Ok(())
}
