mod create;
mod delete;
mod list;
mod tz;

use create::create;
use delete::delete;
use list::{list, search};
use tz::timezone;

use crate::prelude::*;

/// Discord caps autocomplete responses at 25 entries.
const MAX_AUTOCOMPLETE: usize = 25;

/// Reminders that DM you at a set time.
#[poise::command(
    slash_command,
    rename = "reminder",
    subcommands("create", "list", "search", "delete", "timezone"),
    subcommand_required
)]
pub async fn reminder(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}
