use crate::prelude::*;

/// Bury someone
#[poise::command(prefix_command, slash_command)]
pub async fn bury(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let Some(target_replied_user) = user else {
        ctx.send(poise::CreateReply::default().content(format!(
            "{} Just use the `!selfbury` or `/selfbury` command bruh...",
            ctx.author().mention()
        )))
        .await?;
        return Ok(());
    };

    let embed_item: &str = cmd_utils::get_rand_embed_from_type(&EmbedType::Bury)?;

    let response: String = user_interaction(
        &ctx,
        ctx.guild_id(),
        ctx.author(),
        &target_replied_user,
        |u1, u2| format!("**{}** *buries* **{}**", u1, u2),
    )
    .await;

    let bot_user = Arc::clone(&ctx.data().bot_user);

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(embed_item.to_string())
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(Arc::clone(&ctx.data().bot_avatar).to_string()),
        );

    let full_respone = make_full_response(&ctx, &target_replied_user, Some(embed)).await;
    ctx.send(full_respone).await?;

    Ok(())
}
