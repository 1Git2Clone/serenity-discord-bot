//! The event handler is a unique part of Discord bots as a whole.
//! The main purpose of it is to listen to specific events and do an action based on the event
//! information. More information on all the ways you can manage the
//! poise::serenity_prelude::FullEvent enum can be found on the poise documentation page:
//! https://docs.rs/poise/latest/poise/serenity_prelude/enum.FullEvent.html

use crate::data::{
    // bot_data::BOT_PREFIX,
    bot_data::{DATABASE_FILENAME, START_TIME},
    command_data::{Data, Error},
    database_interactions::*,
};
use poise::serenity_prelude as serenity;
use rand::Rng;
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
        serenity::FullEvent::Ratelimit { data } => {
            eprintln!(
                "- (!) - There's a rate limit for the bot right now! [{:?} seconds left!]",
                data.timeout.as_secs()
            )
        }
        serenity::FullEvent::Message { new_message } => match &new_message.content.to_lowercase() {
            // This is a hassle to deal with so I'm implicitly
            // moving the match up to here.
            _ if new_message.author.bot => {}

            msg => {
                println!(
                    "! NEW MESSAGE !\nGuildID:  {}\nUserID:   {}\nUsername: {}\nMsg:      {}\n",
                    &new_message.guild_id.unwrap_or_default(),
                    &new_message.author.id,
                    &new_message.author.name,
                    &new_message.content,
                );
                if (msg.contains("hutao") || msg.contains("hu tao"))
                    && msg.contains("damn")
                    && msg.contains("mains")
                {
                    data.poise_mentions.fetch_add(1, Ordering::SeqCst);
                    new_message.reply(ctx, "Any last words?").await?;
                } else if msg.contains("hutao") || msg.contains("hu tao") {
                    let mentions = data.poise_mentions.fetch_add(1, Ordering::SeqCst);
                    new_message
                        .reply(ctx, format!("Hu Tao has been mentioned {} times", mentions))
                        .await?;
                }

                let db = connect_to_db(DATABASE_FILENAME.to_string()).await;
                match db.await {
                    Ok(pool) => {
                        let obtained_xp: i32 = rand::thread_rng().gen_range(5..=15);
                        println!("Connected to the database: {pool:?}");
                        let status = add_or_update_db_user(
                            pool,
                            new_message.to_owned(),
                            ctx.to_owned(),
                            obtained_xp,
                        )
                        .await;
                        println!("Status: {:#?}", status);
                    }
                    Err(why) => eprintln!("Failed to connect to the database: {why:?}"),
                }
            }
        },
        _ => {}
    }
    Ok(())
}
