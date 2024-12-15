//! The event handler is a unique part of Discord bots as a whole.
//!
//! The main purpose of it is to listen to specific events and do an action based on the event
//! information. More information on all the ways you can manage the
//! poise::serenity_prelude::FullEvent enum can be found on the poise documentation page:
//! https://docs.rs/poise/latest/poise/serenity_prelude/enum.FullEvent.html

#[cfg(feature = "debug")]
use crate::data::bot_data::START_TIME;
use crate::{
    data::{
        command_data::{Data, Error},
        database::DATABASE_FILENAME,
    },
    database::connect_to_db,
    database::level_system::*,
    utils::{replies::handle_replies, string_manipulation::remove_emojis_and_embeds_from_str},
};
use poise::serenity_prelude as serenity;
use rand::Rng;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        #[cfg(feature = "debug")]
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
                #[cfg(feature = "debug")]
                println!(
                    "! NEW MESSAGE !\nGuildID:  {:?}\nUserID:   {}\nUsername: {}\nMsg:      {}\n",
                    &new_message.guild_id,
                    &new_message.author.id,
                    &new_message.author.name,
                    &new_message.content,
                );
                let text_patterns = ["hutao", "hu tao"];
                let trimmed_emojis = remove_emojis_and_embeds_from_str(msg);

                let db = connect_to_db(DATABASE_FILENAME.to_string()).await;
                match db.await {
                    Ok(ref pool) => {
                        let obtained_xp: i32 = rand::thread_rng().gen_range(5..=15);

                        #[cfg(feature = "debug")]
                        println!("Connected to the database: {pool:?}");

                        match text_patterns
                            .iter()
                            .any(|text| trimmed_emojis.contains(text))
                        {
                            true => handle_replies(pool, ctx, new_message, &trimmed_emojis).await?,
                            false => {
                                #[cfg(feature = "debug")]
                                println!(
                                    "Msg: {} has an emoji or doesn't contain: [{}]",
                                    msg,
                                    text_patterns.join(" / ")
                                );
                            }
                        };

                        let status =
                            add_or_update_db_user(pool, new_message, ctx, obtained_xp).await;

                        if let Err(err) = status {
                            eprintln!("{}", err);
                        }
                    }
                    Err(why) => eprintln!("Failed to connect to the database: {why:?}"),
                }
            }
        },
        _ => {}
    }
    Ok(())
}
