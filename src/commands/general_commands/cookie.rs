use crate::prelude::*;

/// Give someone a cookie!
#[poise::command(slash_command, prefix_command, rename = "cookie")]
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
pub async fn cookie(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
    #[rest] msg: Option<String>,
) -> Result<(), Error> {
    let author = ctx.author();
    let target_replied_user = user.as_ref().unwrap_or(get_replied_user(ctx).await);
    let response = if target_replied_user != author {
        format!(
            "{} gave a cookie to **{}**! :cookie:",
            ctx.author().name,
            target_replied_user.name,
        )
    } else {
        String::from("Here's a cookie for you! :cookie: :heart:")
    };
    ctx.say(response).await?;
    Ok(())
}
