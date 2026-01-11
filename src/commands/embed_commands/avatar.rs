use crate::prelude::*;

/// Get the avatar for someone.
#[poise::command(slash_command, prefix_command)]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        target_user = %user.as_ref().map(|u| u.id.get()).unwrap_or(0),
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
        extra_msg = %msg.as_deref().unwrap_or("")
    )
)]
pub async fn avatar(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
    #[rest] msg: Option<String>,
) -> Result<(), Error> {
    let target_replied_user = user.as_ref().unwrap_or(get_replied_user(ctx).await);
    let response: String = format!("**{}**'s avatar:", target_replied_user.name);
    let user_avatar_as_embed: String = target_replied_user.face().replace(".webp", ".png");

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(user_avatar_as_embed);
    let full_respone = poise::CreateReply::default().embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}
