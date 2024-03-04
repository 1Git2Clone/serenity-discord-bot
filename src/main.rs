// #region Notes about the external imports
///
/// The tokio async runtime:
/// - https://tokio.rs/
/// - https://github.com/tokio-rs/tokio/
/// NOTE: It's kind of necessary to use it for better responsiveness, especially in bigger servers.
///
/// The serenity-rs API for the discord bot functionality:
/// - https://github.com/serenity-rs/serenity/
/// NOTE: It has very nice and comprehensive examples under this folder:
/// - https://github.com/serenity-rs/serenity/tree/current/examples
///
/// lazy_static:
/// - https://github.com/rust-lang-nursery/lazy-static.rs
/// Why? Well its for the ease of modularity.
/// Having the data seperated in a different folder
/// makes the project more organized (at least in my opinion).
///
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
use data::bot_data::{BOT_PREFIX, BOT_TOKEN, START_TIME};
use data::command_data::Data;

mod event_handler;
use event_handler::handler::event_handler;

// #endregion

// Used the quickstart guide for poise (serenity-rs command framework)
// https://github.com/serenity-rs/poise/blob/current/examples/quickstart/main.rs

use poise::serenity_prelude as serenity;
use std::sync::atomic::AtomicU32;

#[tokio::main]
async fn main() {
    let _ = START_TIME.elapsed().as_secs(); // Dummy data to get the time elapsing started

    let _database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("bot_database.sqlite")
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database");

    dotenv::dotenv().ok();
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
                prefix: Some(BOT_PREFIX.into()),
                ..Default::default()
            },
            commands: vec![
                commands::general_commands::help(),
                commands::general_commands::age(),
                commands::general_commands::uptime(),
                commands::embed_commands::pat(),
                commands::embed_commands::avatar(),
            ],
            manual_cooldowns: true,
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    poise_mentions: AtomicU32::new(0),
                    // database: database,
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
            "{BOT_PREFIX} <- The prefix for the bot!"
        )))
        .status(serenity::OnlineStatus::Idle)
        .await;

    // Heheh, we do a little bit of trolling...
    // webbrowser::open("https://www.youtube.com/watch?v=dQw4w9WgXcQ").expect("");
    client.unwrap().start().await.unwrap();
}
