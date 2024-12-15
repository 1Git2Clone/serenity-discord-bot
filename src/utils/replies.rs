use crate::{data::command_data::Error, database::bot_mentions::add_mentions};
use poise::serenity_prelude as serenity;
use sqlx::SqlitePool;

pub async fn handle_replies(
    db: &SqlitePool,
    ctx: &serenity::Context,
    new_message: &serenity::Message,
    msg: &str,
) -> Result<(), Error> {
    if msg
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("")
        == "damnhutaomains"
    {
        add_mentions(db, 1).await?;
        new_message.reply(ctx, "Any last words?").await?;
    } else if msg.contains("hutao") || msg.contains("hu tao") {
        let mentions = add_mentions(db, 1).await?;
        #[cfg(feature = "debug")]
        println!("Mentions: {}", mentions);
        new_message
            .reply(ctx, format!("Hu Tao has been mentioned {} times", mentions))
            .await?;
    }

    Ok(())
}
