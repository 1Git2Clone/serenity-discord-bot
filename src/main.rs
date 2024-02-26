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

mod data;
use data::bot_data::BOT_TOKEN;
mod message_commands;
use crate::message_commands::message_create::Handler;
mod bot_utils;

/*
 * Serenity handling part
 * Credit:
 * https://github.com/serenity-rs/serenity/blob/current/examples/e01_basic_ping_bot/src/main.rs
 */

pub use serenity::model::gateway::Ready;
pub use serenity::prelude::*;

// #endregion

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&*BOT_TOKEN, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client.");

    if let Err(why) = client.start().await {
        print!("Client error: {why:?}");
    }
}
