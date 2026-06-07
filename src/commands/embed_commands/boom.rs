use crate::prelude::*;

/// Just try it.
#[poise::command(discard_spare_arguments, slash_command, prefix_command)]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
    )
)]
pub async fn boom(ctx: Context<'_>) -> Result<(), Error> {
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
