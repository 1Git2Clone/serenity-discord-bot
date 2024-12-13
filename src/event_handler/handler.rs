//! The event handler is a unique part of Discord bots as a whole.
//!
//! The main purpose of it is to listen to specific events and do an action based on the event
//! information. More information on all the ways you can manage the
//! poise::serenity_prelude::FullEvent enum can be found on the poise documentation page:
//! https://docs.rs/poise/latest/poise/serenity_prelude/enum.FullEvent.html

use crate::{
    data::{
        bot_data::{DATABASE_FILENAME, START_TIME},
        command_data::{Data, Error},
        database_interactions::*,
    },
    utils::{replies::handle_replies, string_manipulation::is_in_emoji},
};
use poise::serenity_prelude as serenity;
use rand::Rng;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            #[cfg(feature = "debug")]
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
                #[cfg(feature = "debug")]
                println!(
                    "! NEW MESSAGE !\nGuildID:  {}\nUserID:   {}\nUsername: {}\nMsg:      {}\n",
                    &new_message.guild_id.unwrap_or_default(),
                    &new_message.author.id,
                    &new_message.author.name,
                    &new_message.content,
                );
                let emoji_pattern = "hutao";
                match is_in_emoji(&msg.to_lowercase(), emoji_pattern) {
                    Some(is_emoji) if !is_emoji => {
                        handle_replies(ctx, new_message, data, msg).await?
                    }
                    Some(_emoji) => {
                        #[cfg(feature = "debug")]
                        println!("Msg: {} has an emoji!", msg);
                    }
                    None => {
                        #[cfg(feature = "debug")]
                        println!(
                            "Emoji pattern {} not found in message: {}",
                            emoji_pattern, msg
                        );
                    }
                };

                let db = connect_to_db(DATABASE_FILENAME.to_string()).await;
                match db.await {
                    Ok(pool) => {
                        let obtained_xp: i32 = rand::thread_rng().gen_range(5..=15);

                        #[cfg(feature = "debug")]
                        println!("Connected to the database: {pool:?}");

                        let status =
                            add_or_update_db_user(pool, new_message, ctx, obtained_xp).await;

                        #[cfg(feature = "debug")]
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
