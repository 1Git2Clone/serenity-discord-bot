use crate::prelude::*;

/// Bury yourself (perhaps to help Hu Tao's busines idk...)
#[poise::command(discard_spare_arguments, prefix_command, slash_command)]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
    )
)]
pub async fn selfbury(ctx: Context<'_>) -> Result<(), Error> {
    let embed_item: &str = cmd_utils::get_rand_embed_from_type(&EmbedType::SelfBury)?;
    let response: String = format!("**{}** *buries themselves*", ctx.author().name,);
    let bot_user = Arc::clone(&ctx.data().bot_user);

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(embed_item.to_string())
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(Arc::clone(&ctx.data().bot_avatar).to_string()),
        );

    let full_respone = poise::CreateReply::default().embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}
