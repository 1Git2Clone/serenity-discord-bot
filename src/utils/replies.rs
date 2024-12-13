use crate::data::command_data::{Data, Error};
use poise::serenity_prelude as serenity;
use std::sync::atomic::Ordering;

pub async fn handle_replies(
    ctx: &serenity::Context,
    new_message: &serenity::Message,
    data: &Data,
    msg: &str,
) -> Result<(), Error> {
    if msg
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("")
        == "damnhutaomains"
    {
        data.hutao_mentions.fetch_add(1, Ordering::SeqCst);
        new_message.reply(ctx, "Any last words?").await?;
    } else if msg.contains("hutao") || msg.contains("hu tao") {
        let mentions = data.hutao_mentions.fetch_add(1, Ordering::SeqCst);
        new_message
            .reply(ctx, format!("Hu Tao has been mentioned {} times", mentions))
            .await?;
    }

    Ok(())
}
