use super::*;

const HELP_COMMAND_TEXT: &str = concat!(
    "This bot has been made using Rust with the ",
    "[serenity-rs](<https://github.com/serenity-rs/serenity>) and [poise]",
    "(<https://github.com/serenity-rs/poise>) frameworks.\nIt's open source and the ",
    "source code is on my [github profile]",
    "(<https://github.com/1Git2Clone/serenity-discord-bot>).\n"
);

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
    let reply = poise::CreateReply::default()
        .content(HELP_COMMAND_TEXT)
        .ephemeral(true);
    ctx.send(reply).await?;

    Ok(())
}
