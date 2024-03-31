use super::*;
use crate::data::bot_data::START_TIME;
use crate::data::command_data::{Context, Error};

/// Displays the bot's current uptime
#[poise::command(slash_command, prefix_command, rename = "uptime")]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    let bot_user = ctx.http().get_current_user().await?;
    let bot_avatar = bot_user.face().replace(".webp", ".png");

    let time = START_TIME.elapsed().as_secs();

    let units = [("days", 86400), ("hours", 3600), ("minutes", 60)];
    let (unit, value) = units
        .iter()
        .find(|(_, divisor)| time >= *divisor)
        .unwrap_or(&("seconds", 1));

    let parsed_time = match value {
        1 => format!("{} seconds", time as f64 / value.to_owned() as f64),
        _ => format!("{:.2} {} ", time as f64 / value.to_owned() as f64, unit),
    };
    ctx.send(
        poise::CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title("Bot Uptime")
                .field(parsed_time, "", false)
                .color((255, 0, 0))
                .footer(serenity::CreateEmbedFooter::new(bot_user.tag()).icon_url(bot_avatar)),
        ),
    )
    .await?;

    Ok(())
}

/// Just try it.
#[poise::command(slash_command, prefix_command, rename = "uptime")]
pub async fn boom(ctx: Context<'_>) -> Result<(), Error> {
    let bot_user = ctx.http().get_current_user().await?;
    let bot_avatar = bot_user.face().replace(".webp", ".png");

    ctx.send(
        poise::CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .image("https://cdn.discordapp.com/attachments/1129364410566193192/1223359511356641321/ea022b6f5e25129f8c865b6b2d8e2f33.jpg?ex=66199154&is=66071c54&hm=3fc8357942f1ea01c76b2c249c6db654ef6572d00a5e7d65af4de3266d39ae6b&")
                .color((255, 0, 0))
                .footer(serenity::CreateEmbedFooter::new(bot_user.tag()).icon_url(bot_avatar)),
        ),
    )
    .await?;

    Ok(())
}
