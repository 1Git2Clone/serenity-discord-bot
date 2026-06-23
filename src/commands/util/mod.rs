use crate::prelude::*;

#[cfg(feature = "util-download")]
pub mod download;

// Reuse the existing standalone commands as `/util` subcommands rather than
// copying them — the same `#[poise::command]` fn can be both a top-level
// command and a subcommand.
use crate::commands::embed_commands::{avatar, uptime::uptime};
use crate::commands::general_commands::age::age;
#[cfg(feature = "util-download")]
use download::download;

/// Utility commands — avatar, uptime, age.
#[cfg(not(feature = "util-download"))]
#[poise::command(
    prefix_command,
    slash_command,
    subcommands("avatar", "uptime", "age"),
    subcommand_required
)]
#[allow(clippy::unused_async)]
pub async fn util(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Utility commands — avatar, uptime, age, download.
#[cfg(feature = "util-download")]
#[poise::command(
    prefix_command,
    slash_command,
    subcommands("avatar", "uptime", "age", "download"),
    subcommand_required
)]
#[allow(clippy::unused_async)]
pub async fn util(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}
