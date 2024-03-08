use sqlx::Row;

use super::*;
use crate::data::database_interactions::{connect_to_db, fetch_user_level};
use crate::data::{
    bot_data::DATABASE_COLUMNS,
    bot_data::DATABASE_FILENAME,
    bot_data::START_TIME,
    command_data::{Context, Error},
};
use crate::enums::schemas::DatabaseSchema::*;

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
    let selected_user = cmd_utils::get_user(ctx, user).await;
    let response = format!(
        "**{}**'s account was created at {}",
        selected_user.name,
        selected_user.created_at()
    );
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

/// Displays the user's level
#[poise::command(slash_command, prefix_command, rename = "level")]
pub async fn level(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let message_guild_id = match ctx.guild_id() {
        Some(msg) => msg,
        None => {
            ctx.reply("Please use the fucking guild chats you sick fuck!")
                .await?;
            return Ok(());
        }
    };
    let selected_user = cmd_utils::get_user(ctx, user).await;
    let db = connect_to_db(DATABASE_FILENAME.to_string()).await;
    let level_and_xp_row_option = match db.await {
        Ok(pool) => {
            println!("Connected to the database: {pool:?}");
            fetch_user_level(&pool, &selected_user, message_guild_id).await?
        }
        Err(_) => {
            ctx.reply(format!(
                "Please wait for {} to chat more then try again later...",
                selected_user.name
            ))
            .await?;
            return Ok(());
        }
    };
    let level_and_xp_row = if let Some(lvl_and_xp_row) = level_and_xp_row_option {
        lvl_and_xp_row
    } else {
        ctx.reply(format!(
            "Please wait for {} to chat more then try again later...",
            selected_user.name
        ))
        .await?;
        return Ok(());
    };
    let level = level_and_xp_row.get::<i32, &str>(DATABASE_COLUMNS[&Level]);
    let xp = level_and_xp_row.get::<i32, &str>(DATABASE_COLUMNS[&ExperiencePoints]);
    ctx.say(format!(
        "```User:  {}\nLevel: {}\nXp:    {}```",
        selected_user.name, level, xp
    ))
    .await?;
    Ok(())
}
