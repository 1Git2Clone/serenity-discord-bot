use crate::prelude::*;

/// Kick someone
#[poise::command(prefix_command, slash_command)]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_replied_user = user.as_ref().unwrap_or(get_replied_user(ctx).await);
    let embed_item: &str = cmd_utils::get_rand_embed_from_type(&EmbedType::Kick)?;
    if same_user(target_replied_user, ctx.author()) {
        ctx.send(poise::CreateReply::default().content(format!(
            "{}, why would you kick yourself...? Weirdo...",
            target_replied_user.mention()
        )))
        .await?;
        return Ok(());
    }

    let response: String = user_interaction(
        &ctx,
        ctx.guild_id(),
        ctx.author(),
        target_replied_user,
        |u1, u2| format!("**{}** *kicks* **{}**", u1, u2),
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

    let full_respone = make_full_response(&ctx, target_replied_user, Some(embed)).await;
    ctx.send(full_respone).await?;

    Ok(())
}
