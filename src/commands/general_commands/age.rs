use crate::prelude::*;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command, rename = "age")]
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
pub async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
    #[rest] msg: Option<String>,
) -> Result<(), Error> {
    let target_replied_user = user.as_ref().unwrap_or(get_replied_user(ctx).await);
    let response = format!(
        "**{}**'s account was created at {}",
        target_replied_user.name,
        target_replied_user.created_at()
    );
    ctx.say(response).await?;
    Ok(())
}
