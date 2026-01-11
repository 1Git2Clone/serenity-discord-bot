use crate::prelude::*;

/// Get a Ryan Gosling drive GIF.
#[poise::command(slash_command, prefix_command)]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
        extra_msg = %msg.as_deref().unwrap_or("")
    )
)]
pub async fn drive(ctx: Context<'_>, #[rest] msg: Option<String>) -> Result<(), Error> {
    let embed_item: &str = cmd_utils::get_rand_embed_from_type(&EmbedType::RyanGoslingDrive)?;
    let bot_user = Arc::clone(&ctx.data().bot_user);

    let embed = serenity::CreateEmbed::new()
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
