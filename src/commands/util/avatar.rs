use crate::prelude::*;

/// Get the avatar for someone.
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
    let target = user
        .as_ref()
        .unwrap_or(cmd_utils::get_replied_user(ctx).await);

    let embed = serenity::CreateEmbed::new()
        .title(format!("**{}**'s avatar:", target.name))
        .color((255, 0, 0))
        .image(target.face().replace(".webp", ".png"));
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}
