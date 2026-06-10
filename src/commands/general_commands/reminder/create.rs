use chrono::Utc;

use crate::prelude::*;

use super::tz::{autocomplete_timezone, build_remind_at, fetch_default_tz, resolve_tz};

/// Max reminders a single user may have pending at once.
const MAX_PENDING_PER_USER: i64 = 50;
/// Max characters allowed in a reminder message.
const MAX_MESSAGE_CHARS: usize = 500;

/// Set a reminder — the bot DMs you at the given time (fires within a minute).
// Polling runs at minute resolution, so finer input precision wouldn't help.
#[poise::command(slash_command, rename = "create")]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().qualified_name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
    )
)]
#[allow(clippy::too_many_arguments)]
pub async fn create(
    ctx: Context<'_>,
    #[description = "What to remind you of"]
    #[max_length = 500]
    message: String,
    #[description = "Hour 0–23 (default 0 = midnight, in your timezone)"]
    #[min = 0]
    #[max = 23]
    hour: Option<u8>,
    #[description = "Minute 0–59 (default 0)"]
    #[min = 0]
    #[max = 59]
    minute: Option<u8>,
    #[description = "Day of month 1–31 (default today)"]
    #[min = 1]
    #[max = 31]
    day: Option<u8>,
    #[description = "Month 1–12 (default current)"]
    #[min = 1]
    #[max = 12]
    month: Option<u8>,
    #[description = "Year (default current)"] year: Option<i32>,
    #[description = "City/offset for this reminder. Defaults to your saved zone, else UTC"]
    #[autocomplete = "autocomplete_timezone"]
    timezone: Option<String>,
) -> Result<(), Error> {
    if message.chars().count() > MAX_MESSAGE_CHARS {
        ctx.say(format!(
            "Reminders are capped at {MAX_MESSAGE_CHARS} characters."
        ))
        .await?;
        return Ok(());
    }

    let tz_string = match timezone {
        Some(t) => Some(t),
        None => {
            fetch_default_tz(
                &ctx.data().pool,
                ctx.author().id.get() as i64,
                ctx.guild_id().map(GuildId::get),
            )
            .await?
        }
    };
    let (tz, tz_name) = match resolve_tz(tz_string.as_deref()) {
        Ok(v) => v,
        Err(msg) => {
            ctx.say(msg).await?;
            return Ok(());
        }
    };

    let now = Utc::now();

    let remind_at = if [hour.is_some(), minute.is_some(), day.is_some(), month.is_some(), year.is_some()]
        .iter()
        .all(|set| !set)
    {
        now + chrono::Duration::hours(1)
    } else {
        let (cur_y, cur_mo, cur_d) = tz.now_parts();
        let y = year.unwrap_or(cur_y);
        let mo = month.map(u32::from).unwrap_or(cur_mo);
        let d = day.map(u32::from).unwrap_or(cur_d);
        let h = hour.map(u32::from).unwrap_or(0);
        let mi = minute.map(u32::from).unwrap_or(0);

        match build_remind_at(&tz, y, mo, d, h, mi) {
            Ok(dt) => dt,
            Err(msg) => {
                ctx.say(msg).await?;
                return Ok(());
            }
        }
    };

    if remind_at <= now {
        ctx.say("That time is already in the past!").await?;
        return Ok(());
    }
    let max_at = now.checked_add_months(chrono::Months::new(1)).unwrap_or(now);
    if remind_at > max_at {
        ctx.say("Reminders can be set at most one month in advance.")
            .await?;
        return Ok(());
    }

    let user_id = ctx.author().id.get() as i64;
    let pending = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM reminders WHERE user_id = $1 AND finished_at IS NULL",
        user_id,
    )
    .fetch_one(&*ctx.data().pool)
    .await?
    .unwrap_or(0);
    if pending >= MAX_PENDING_PER_USER {
        ctx.say(format!(
            "You already have {MAX_PENDING_PER_USER} pending reminders — wait for some to fire first."
        ))
        .await?;
        return Ok(());
    }

    sqlx::query!(
        "INSERT INTO reminders (user_id, remind_at, message, timezone) VALUES ($1, $2, $3, $4)",
        user_id,
        remind_at,
        message,
        tz_name,
    )
    .execute(&*ctx.data().pool)
    .await?;

    let ts = remind_at.timestamp();
    ctx.say(format!(
        "Got it! I'll DM you <t:{ts}:F> (<t:{ts}:R>) — timezone `{tz_name}`."
    ))
    .await?;

    Ok(())
}
