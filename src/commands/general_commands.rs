use super::*;
use crate::data::{
    bot_data::START_TIME,
    command_data::{Context, Error},
};

/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom:
                // OLD.
                // "* Note: You can use\n/help command_name\nfor more details on a specific command.",
                "* Note: You can use \'/help <command_name>\' for more details on a specific command.",
            ..Default::default()
        },
    )
    .await?;

    if command.is_some() {
        return Ok(());
    }

    ctx.defer_ephemeral().await?;
    // OLD.
    // let reply_text = "This bot has been made using Rust with the [serenity-rs](<https://github.com/serenity-rs/serenity>) and [poise](<https://github.com/serenity-rs/poise>) frameworks.\nIt's open source and hosted on my [github profile](<https://github.com/1Kill2Steal/serenity-discord-bot>).\nUnfortunately you can't select users by replying to messages yet. I'm just not sure at how to implement it. *(Skill Issue...)*";
    let reply_text = "This bot has been made using Rust with the [serenity-rs](<https://github.com/serenity-rs/serenity>) and [poise](<https://github.com/serenity-rs/poise>) frameworks.\nIt's open source and hosted on my [github profile](<https://github.com/1Kill2Steal/serenity-discord-bot>).\n";
    let reply = poise::CreateReply::default()
        .content(reply_text)
        .ephemeral(true);
    ctx.send(reply).await?;

    Ok(())
}

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command, rename = "age")]
pub async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = cmd_utils::get_user(ctx, user).await;
    let response = format!("**{}**'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

/// Displays the bot's current uptime
#[poise::command(slash_command, prefix_command, rename = "uptime")]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    let response = format!(
        "The bot has been running for: {} seconds",
        START_TIME.elapsed().as_secs()
    );
    ctx.say(response).await?;
    Ok(())
}
