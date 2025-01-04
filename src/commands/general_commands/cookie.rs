use crate::prelude::*;

/// Give someone a cookie!
#[poise::command(slash_command, prefix_command, rename = "cookie")]
pub async fn cookie(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
    #[rest] _msg: String,
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
