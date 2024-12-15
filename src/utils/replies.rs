use crate::{data::command_data::Error, database::bot_mentions::add_mentions};
use poise::serenity_prelude as serenity;
use sqlx::SqlitePool;

pub async fn handle_replies(
    db: &SqlitePool,
    ctx: &serenity::Context,
    new_message: &serenity::Message,
    msg: &str,
) -> Result<(), Error> {
    let no_whitespace = msg
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("");
    let time_or_times = |count| if count > 1 { "times" } else { "time" };
    let hutao_count = no_whitespace.matches("hutao").count();
    if no_whitespace.matches("damnhutaomains").count() > 0 {
        add_mentions(db, hutao_count).await?;
        new_message.reply(ctx, "Any last words?").await?;
    } else if hutao_count > 0 {
        let mentions = add_mentions(db, hutao_count).await?;
        let mut reply = format!(
            "Hu Tao has been mentioned {} {} | +{} {}.",
            mentions,
            time_or_times(mentions),
            hutao_count,
            time_or_times(hutao_count)
        );
        match hutao_count {
            10..=50 => {
                reply.push_str("\n\nEh!? What's with all these mentions!?");
            }
            51..100 => {
                reply.push_str("\n\nPlease stoppp~!");
            }
            100.. => {
                reply.push_str("\n\nやめて！！");
            }
            _ => (),
        };
        new_message.reply(ctx, reply).await?;
    }

    Ok(())
}
