use crate::prelude::*;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command, rename = "age")]
pub async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
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
