use crate::prelude::*;

/// Just try it.
#[poise::command(slash_command, prefix_command)]
pub async fn boom(ctx: Context<'_>, #[rest] _msg: Option<String>) -> Result<(), Error> {
    let bot_user = Arc::clone(&ctx.data().bot_user);
    let bot_avatar = bot_user.face().replace(".webp", ".png");

    ctx.send(
        poise::CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .image(Assets::HuBoom.to_string())
                .color((255, 0, 0))
                .footer(serenity::CreateEmbedFooter::new(bot_user.tag()).icon_url(bot_avatar)),
        ),
    )
    .await?;

    Ok(())
}
