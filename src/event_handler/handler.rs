use crate::data::{
    // bot_data::BOT_PREFIX,
    self,
    bot_data::{DATABASE_FILENAME, START_TIME},
    command_data::{Data, Error},
    database::add_user_if_not_exists,
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
                "\n!!! DISCORD BOT STARTED SUCCESSFULLY IN {} MILISECONDS !!!\nFrameworks   : [serenity, poise]\nAsync runtime: [tokio]\n\n=> Logged in as: {}",
                START_TIME.elapsed().as_millis(),
                data_about_bot.user.tag()
            );
        }
        serenity::FullEvent::Message { new_message } => match &new_message.content.to_lowercase() {
            // This is a hassle to deal with so I'm implicitly
            // moving the match up to here.
            _ if new_message.author.bot => {}

            msg => {
                if msg.contains("hutao") || msg.contains("hu tao") {
                    let mentions = data.poise_mentions.load(Ordering::SeqCst) + 1;
                    data.poise_mentions.store(mentions, Ordering::SeqCst);
                    new_message
                        .reply(ctx, format!("Hu Tao has been mentioned {} times", mentions))
                        .await?;
                }
                println!(
                    "! NEW MESSAGE !\nGuildID:  {}\nUserID:   {}\nUsername: {}\nMsg:      {}\n",
                    &new_message.guild_id.unwrap_or_default(),
                    &new_message.author.id,
                    &new_message.author.name,
                    &new_message.content,
                );
                let db = data::database::connect_to_db(DATABASE_FILENAME.to_string()).await;
                match db.await {
                    Ok(pool) => {
                        println!("Connected to the database: {pool:?}");
                        let status =
                            add_user_if_not_exists(pool, &new_message.author, event.to_owned())
                                .await;
                        println!("Status: {:#?}", status);
                    }
                    Err(why) => eprintln!("Failed to connect to the database: {why:?}"),
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
