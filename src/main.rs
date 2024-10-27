// #region Notes about the external imports
//
//
//  Why isn't this in the README.md? Well, I don't think anyone who's just checking out the
//  README.md file will be as interested in these details as someone like you who's checking out
//  the actual code. <3
//
//  Serde:
//  - Serde is life... Data serialization/deserialization is just inevitable.
//
//  Dotenv:
//  - While this is possible to be man
//
//  lazy_static:
//  - https://github.com/rust-lang-nursery/lazy-static.rs
//  Why? Well its for global state management. As much as its not that nice it's also needed for
//  the functionality I'm looking for. Take for example static data for your embed generation
//  commands for example or a more complex example would be something like
//  Arc<Mutex<HashMap<(UserId, GuildId), u32>>>
//  Where you can store a command invocation counter from a specific user in a specific guild on
//  runtime - if you need the runtime handling.
//
//  The tokio async runtime:
//  - https://tokio.rs/
//  - https://github.com/tokio-rs/tokio/
//  NOTE: It's kind of necessary to use it for better responsiveness, especially in bigger servers.
//
//  The serenity-rs API for the discord bot functionality:
//  - https://github.com/serenity-rs/serenity/
//  NOTE: It has very nice and comprehensive examples under this folder:
//  - https://github.com/serenity-rs/serenity/tree/current/examples
//
//  The poise command framework which works with serenity
//  - https://github.com/serenity-rs/poise
//  It's a revolutionized way of handling commands, allowing you to handle message and slash
//  commands in 1 function. Additionally provides useful stuff like building context menus, better
//  embed building and attachment building and also context menu building for both message and
//  slash commands again.
//
//
// #endregion

// #region All imports (./lib.rs)

//
// The bot_data like the BOT_TOKEN is handled using the dotenv-rs dependency!
// - https://github.com/dotenv-rs/dotenv/
//
mod commands;
// use commands::embed_commands::*;
// use commands::general_commands::*;

mod data;
use data::bot_data::{BOT_PREFIXES, BOT_TOKEN, START_TIME};
use data::command_data::Data;

mod enums;

mod extra_threads;
use extra_threads::xp_command_cooldown::periodically_clean_users_on_diff_thread;

mod event_handler;
use event_handler::handler::event_handler;

mod utils;

mod tests;

// #endregion

// Used the quickstart guide for poise (serenity-rs command framework)
// https://github.com/serenity-rs/poise/blob/current/examples/quickstart/main.rs

use poise::serenity_prelude as serenity;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let _ = START_TIME.elapsed().as_secs(); // Dummy data to get the time elapsing started

    dotenv::dotenv().ok();
    periodically_clean_users_on_diff_thread();
    let token = BOT_TOKEN.to_string();
    // ```rust
    // non_privileged()
    // ```
    // Should be enough for most cases. I set it to all because I wanted to log the message
    // content.
    // Either all or non_privileged intents only.
    // https://docs.rs/poise/latest/poise/#gateway-intents
    let intents = serenity::GatewayIntents::all() | serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            // These invocations are not really necessary
            // Feel free to refer to the source code of the poise library
            // https://github.com/serenity-rs/poise/blob/current/examples/invocation_data/main.rs
            // I found the on_error one to be the most useful (that's if it ever fails on runtime)
            //
            // ```rust
            // pre_command: |ctx| {
            //     Box::pin(async move {
            //         println!(
            //             "In pre_command: {:?}",
            //             ctx.invocation_data::<&str>().await.as_deref()
            //         );
            //     })
            // },
            // command_check: Some(|ctx| {
            //     Box::pin(async move {
            //         // Global command check is the first callback that's invoked, so let's set the
            //         // data here
            //         println!("Writing invocation data!");
            //         ctx.set_invocation_data("hello").await;
            //
            //         println!(
            //             "In global check: {:?}",
            //             ctx.invocation_data::<&str>().await.as_deref()
            //         );
            //
            //         Ok(true)
            //     })
            // }),
            // post_command: |ctx| {
            //     Box::pin(async move {
            //         println!(
            //             "In post_command: {:?}",
            //             ctx.invocation_data::<&str>().await.as_deref()
            //         );
            //     })
            // },
            // ```
            //
            on_error: |err| {
                Box::pin(async move {
                    match err {
                        poise::FrameworkError::Command { ctx, .. } => {
                            println!(
                                "In on_error: {:?}",
                                ctx.invocation_data::<&str>().await.as_deref()
                            );
                        }
                        err => poise::builtins::on_error(err).await.unwrap(),
                    }
                })
            },

            prefix_options: poise::PrefixFrameworkOptions {
                prefix: None,
                additional_prefixes: BOT_PREFIXES
                    .iter()
                    .map(|x| poise::Prefix::Literal(x.as_str()))
                    .collect::<Vec<poise::Prefix>>(),
                mention_as_prefix: true,
                case_insensitive_commands: true,
                ignore_bots: true,
                ..Default::default()
            },
            commands: vec![
                commands::general_commands::help(),
                commands::general_commands::age(),
                commands::general_commands::cookie(),
                commands::level_cmds::level(),
                commands::level_cmds::toplevels(),
                commands::embed_commands::tieup(),
                commands::embed_commands::pat(),
                commands::embed_commands::hug(),
                commands::embed_commands::kiss(),
                commands::embed_commands::slap(),
                commands::embed_commands::punch(),
                commands::embed_commands::bonk(),
                commands::embed_commands::nom(),
                commands::embed_commands::kill(),
                commands::embed_commands::kick(),
                commands::embed_commands::bury(),
                commands::embed_commands::selfbury(),
                commands::embed_commands::peek(),
                commands::embed_commands::avatar(),
                commands::embed_commands::drive(),
                commands::embed_commands::chair(),
                commands::embed_commands::boom(),
                commands::embed_commands::uptime(),
            ],
            manual_cooldowns: true,
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    poise_mentions: AtomicU32::new(1),
                    // It's better to clone the bot user once when it starts rather than do http
                    // requests for the serenity::CurrentUser on every comman invocation.
                    bot_user: Arc::from(ready.user.clone()),
                    bot_avatar: Arc::<str>::from(ready.user.face().replace(".webp", ".png")),
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        // .activity(
        //     serenity::ActivityData::streaming("Genshin Impact", "https://twitch.tv/1kill2steal")
        //         .expect("Something went wrong in setting the custom activity data."),
        // )
        .activity(serenity::ActivityData::custom(format!(
            "Usable prefixes: [ {} ]",
            BOT_PREFIXES
                .iter()
                .filter(|&x| x.chars().all(|c| c.is_lowercase()))
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )))
        .status(serenity::OnlineStatus::Idle)
        .await;

    // Heheh, we do a little bit of trolling...
    // webbrowser::open("https://www.youtube.com/watch?v=dQw4w9WgXcQ").expect("");
    client.unwrap().start().await.unwrap();
}
