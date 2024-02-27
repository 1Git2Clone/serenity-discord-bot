use crate::data::{
    // bot_data::BOT_PREFIX,
    command_data::{Data, Error},
};
use poise::serenity_prelude as serenity;
use std::sync::atomic::Ordering;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    // TODO Add normal prefix message command handling
    // // To surpress the single match arm clippy warning.
    // if let serenity::FullEvent::Ready { data_about_bot, .. } = event {
    //     println!("{} logged in!", data_about_bot.user.name);
    // }
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!(
                "\n!!! DISCORD BOT STARTED SUCCESSFULLY !!!\nFrameworks   : [serenity, poise]\nAsync runtime: [tokio]\n\n=> Logged in as: {}",
                data_about_bot.user.tag()
            );
        }
        serenity::FullEvent::Message { new_message } => match &new_message.content.to_lowercase() {
            msg if msg.contains("hutao") => {
                let mentions = data.poise_mentions.load(Ordering::SeqCst) + 1;
                data.poise_mentions.store(mentions, Ordering::SeqCst);
                new_message
                    .reply(ctx, format!("Hu Tao has been mentioned {} times", mentions))
                    .await?;
            }
            _ => {
                println!(
                    "MESSAGE:\nUserID: {}\nUsername: {}\nMsg: {:#?}\n",
                    &new_message.author.id, &new_message.author.name, &new_message.content
                );
            }
        },
        _ => {}
    }
    Ok(())
}
