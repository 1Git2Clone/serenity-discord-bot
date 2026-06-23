use crate::prelude::*;

/// Displays the bot's current uptime
#[poise::command(discard_spare_arguments, slash_command, prefix_command)]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
    )
)]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    let bot_user = Arc::clone(&ctx.data().bot_user);
    let bot_avatar = bot_user.face().replace(".webp", ".png");

    let time = START_TIME.elapsed().as_secs();

    let units = [("days", 86400), ("hours", 3600), ("minutes", 60)];
    let (unit, value) = units
        .iter()
        .find(|(_, divisor)| time >= *divisor)
        .unwrap_or(&("seconds", 1));

    let parsed_time = match value {
        1 => format!("{} seconds", time as f64 / *value as f64),
        _ => format!("{:.2} {} ", time as f64 / *value as f64, unit),
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
