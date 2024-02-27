use crate::data::command_data::{Data, Error};
use poise::serenity_prelude as serenity;
use std::sync::atomic::Ordering;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    // TODO Add normal prefix message command handling
    let prefix = "!";
    // // To surpress the single match arm clippy warning.
    // if let serenity::FullEvent::Ready { data_about_bot, .. } = event {
    //     println!("{} logged in!", data_about_bot.user.name);
    // }
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::Message { new_message } => {
            if new_message.content.to_lowercase().starts_with(prefix)
                && new_message.content.to_lowercase().contains("hutao")
            {
                let mentions = data.poise_mentions.load(Ordering::SeqCst) + 1;
                data.poise_mentions.store(mentions, Ordering::SeqCst);
                new_message
                    .reply(ctx, format!("Hu Tao has been mentioned {} times", mentions))
                    .await?;
            }
        }
        _ => {}
    }
    Ok(())
}
