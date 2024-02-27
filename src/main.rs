// #region Notes about the external imports
/*
 * The tokio async runtime:
 * - https://tokio.rs/
 * - https://github.com/tokio-rs/tokio/
 * NOTE: It's kind of necessary to use it for better responsiveness, especially in bigger servers.
 *
 * The serenity-rs API for the discord bot functionality:
 * - https://github.com/serenity-rs/serenity/
 * NOTE: It has very nice and comprehensive examples under this folder:
 * - https://github.com/serenity-rs/serenity/tree/current/examples
 *
 * lazy_static:
 * - https://github.com/rust-lang-nursery/lazy-static.rs
 * Why? Well its for the ease of modularity.
 * Having the data seperated in a different folder
 * makes the project more organized (at least in my opinion).
 *
 */

// #endregion

// #region All imports (./lib.rs)

/*
 * The bot_data like the BOT_TOKEN is handled using the dotenv-rs dependency!
 * - https://github.com/dotenv-rs/dotenv/
 *
 */

mod commands;
use commands::general_commands::age;

mod data;
use data::bot_data::{BOT_PREFIX, BOT_TOKEN};
use data::command_data::Data;

mod event_handler;
use event_handler::handler::event_handler;

// #endregion

// Used the quickstart guide for poise (serenity-rs command framework)
// https://github.com/serenity-rs/poise/blob/current/examples/quickstart/main.rs

use poise::serenity_prelude as serenity;
use std::sync::atomic::AtomicU32;

// User data, which is stored and accessible in all command invocations

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let token = BOT_TOKEN.to_string();
    // ```rust
    // non_priveliged()
    // ```
    // Should be enough for most cases. I set it to all because I wanted to log the message
    // content.
    let intents = serenity::GatewayIntents::all();

    let framework = poise::Framework::builder()
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    poise_mentions: AtomicU32::new(0),
                })
            })
        })
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

            commands: vec![age()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(BOT_PREFIX.into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
