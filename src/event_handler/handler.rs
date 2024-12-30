//! The event handler is a unique part of Discord bots as a whole.
//!
//! The main purpose of it is to listen to specific events and do an action based on the event
//! information. More information on all the ways you can manage the
//! poise::serenity_prelude::FullEvent enum can be found on the poise documentation page:
//! https://docs.rs/poise/latest/poise/serenity_prelude/enum.FullEvent.html

use super::helper_functions::handle_message;
use crate::prelude::*;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        #[cfg(feature = "debug")]
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!(
                "\nStart time: {}ms\n=> Logged in as: {}",
                START_TIME.elapsed().as_millis(),
                data_about_bot.user.tag()
            );
        }

        serenity::FullEvent::Ratelimit { data } => {
            eprintln!(
                "- (!) - There's a rate limit for the bot right now! [{:?} seconds left!]",
                data.timeout.as_secs()
            );
        }
        serenity::FullEvent::Message { new_message } => {
            handle_message(ctx, data, new_message).await?;
        }

        _ => (),
    }

    Ok(())
}
