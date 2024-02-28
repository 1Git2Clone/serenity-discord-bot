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
            // This is a hassle to deal with so I'm implicitly
            // moving the match up to here.
            _ if new_message.author.bot => {}

            msg if msg.contains("hutao") || msg.contains("hu tao") => {
                let mentions = data.poise_mentions.load(Ordering::SeqCst) + 1;
                data.poise_mentions.store(mentions, Ordering::SeqCst);
                new_message
                    .reply(ctx, format!("Hu Tao has been mentioned {} times", mentions))
                    .await?;
            }
            _ => {
                if !new_message.author.bot {
                    println!(
                        "MESSAGE:\nUserID: {}\nUsername: {}\nMsg: {:#?}\n",
                        &new_message.author.id, &new_message.author.name, &new_message.content
                    );
                }
            }
        },
        serenity::FullEvent::Ratelimit { data } => {
            eprintln!(
                "- (!) - There's a rate limit for the bot right now! [{:?} seconds left!]",
                data.timeout.as_secs()
            )
        }
        _ => {}
    }
    Ok(())
}
