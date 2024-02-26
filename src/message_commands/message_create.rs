// #region (./main.rs) imports

pub use serenity::async_trait;
pub use serenity::model::channel::Message;
pub use serenity::model::gateway::Ready;
pub use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use crate::bot_utils::prefix_command::prefix_command;

// #endregion

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        use crate::data::commands::*;

        let _channel = match msg.channel_id.to_channel(&ctx).await {
            Ok(channel) => channel,
            Err(why) => {
                println!("Error getting channel: {why:?}");
                return;
            }
        };

        match msg.content.to_lowercase() {
            // Help.
            cmd if cmd == prefix_command(HELP) => {
                let mut response = MessageBuilder::new();
                response.push_line_safe("## Available commands:\n");
                for element in COMMAND_LIST.iter() {
                    response.push_line_safe(element.to_string());
                }
                response.build();
                if let Err(why) = msg.channel_id.say(&ctx.http, &response.to_string()).await {
                    println!("Error sending message: {why:?}");
                }
            }
            // Ping.
            cmd if cmd == prefix_command(PING) => {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                    println!("Error sending message: {why:?}");
                }
            }
            _ => {}
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
