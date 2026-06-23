use crate::prelude::*;

mod age;
mod avatar;
#[cfg(feature = "util-download")]
pub mod download;
mod uptime;

use age::age;
use avatar::avatar;
#[cfg(feature = "util-download")]
use download::download;
use uptime::uptime;

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
