use chrono::{DateTime, Datelike, FixedOffset, LocalResult, Months, NaiveDate, TimeZone, Utc};
use chrono_tz::{TZ_VARIANTS, Tz};

use crate::prelude::*;

/// Max reminders a single user may have pending at once.
const MAX_PENDING_PER_USER: i64 = 50;
/// Discord caps autocomplete responses at 25 entries.
const MAX_AUTOCOMPLETE: usize = 25;

/// A user-supplied timezone: either a named IANA zone (`Europe/Sofia`) or a
/// fixed offset from GMT (`GMT+2`, `+02:00`). Both implement `TimeZone`, but
/// they're distinct types, so this enum unifies them.
enum UserTz {
    Named(Tz),
    Fixed(FixedOffset),
}

impl UserTz {
    /// Interpret wall-clock fields in this zone and convert to a UTC instant.
    /// Returns the daylight-saving disposition so the caller can report a gap.
    fn to_utc(&self, y: i32, mo: u32, d: u32, h: u32, mi: u32) -> LocalResult<DateTime<Utc>> {
        match self {
            Self::Named(tz) => tz.with_ymd_and_hms(y, mo, d, h, mi, 0).map(|dt| dt.to_utc()),
            Self::Fixed(off) => off.with_ymd_and_hms(y, mo, d, h, mi, 0).map(|dt| dt.to_utc()),
        }
    }

    /// "Now" wall-clock fields in this zone, for filling unspecified inputs.
    fn now_parts(&self) -> (i32, u32, u32) {
        let now = Utc::now();
        let date = match self {
            Self::Named(tz) => now.with_timezone(tz).date_naive(),
            Self::Fixed(off) => now.with_timezone(off).date_naive(),
        };
        (date.year(), date.month(), date.day())
    }
}

/// Resolve a timezone string into a zone plus the canonical name to echo back.
/// Accepts (in order): empty -> UTC; an exact IANA/abbrev name (`Europe/Sofia`,
/// `CET`, `UTC`); a GMT offset (`GMT+2`, `UTC-5`, `+02:00`); or a bare city
/// (`Sofia`). The slash command's autocomplete narrows names to valid zones,
/// so this only has to cover what someone free-types or sends via prefix.
fn resolve_tz(input: Option<&str>) -> Result<(UserTz, String), String> {
    let s = input.unwrap_or("").trim();
    if s.is_empty() {
        return Ok((UserTz::Named(Tz::UTC), "UTC".to_string()));
    }

    // Exact name: UTC, GMT, CET, Europe/Sofia, ...
    if let Ok(tz) = s.parse::<Tz>() {
        return Ok((UserTz::Named(tz), tz.name().to_string()));
    }

    // GMT/UTC offset: GMT+2, UTC-05:00, +02:00, -0530
    if let Some(off) = parse_offset(s) {
        return Ok((UserTz::Fixed(off), format!("UTC{off}")));
    }

    // Case-insensitive full name, then a bare unique city.
    let tz = resolve_named(s)?;
    Ok((UserTz::Named(tz), tz.name().to_string()))
}

/// Filter zone names by the partial input for the slash command's timezone
/// dropdown. `contains` matches the city segment too (`sofia` -> `Europe/Sofia`).
/// `Etc/GMT±N` zones are hidden because their sign is inverted (`Etc/GMT+2` is
/// UTC−2) — a free-text `GMT+2` offset goes through the correct parser instead.
async fn autocomplete_timezone(_ctx: Context<'_>, partial: &str) -> Vec<String> {
    let needle = partial.to_lowercase();
    TZ_VARIANTS
        .iter()
        .map(|tz| tz.name())
        .filter(|name| !name.starts_with("Etc/"))
        .filter(|name| name.to_lowercase().contains(&needle))
        .take(MAX_AUTOCOMPLETE)
        .map(str::to_string)
        .collect()
}

/// Parse a GMT offset: an optional `GMT`/`UTC` prefix then `±HH`, `±HH:MM`, or
/// `±HHMM`. Returns `None` for anything that isn't clearly an offset.
fn parse_offset(s: &str) -> Option<FixedOffset> {
    let body = ["GMT", "UTC", "gmt", "utc"]
        .iter()
        .find_map(|p| s.strip_prefix(p))
        .unwrap_or(s)
        .trim();

    let (sign, rest) = match body.split_at(1) {
        ("+", r) => (1, r),
        ("-", r) => (-1, r),
        _ => return None,
    };

    let (h, m) = if let Some((h, m)) = rest.split_once(':') {
        (h.parse::<i32>().ok()?, m.parse::<i32>().ok()?)
    } else if rest.len() == 4 {
        (rest[..2].parse().ok()?, rest[2..].parse().ok()?)
    } else {
        (rest.parse::<i32>().ok()?, 0)
    };

    if !(0..=14).contains(&h) || !(0..=59).contains(&m) {
        return None;
    }
    FixedOffset::east_opt(sign * (h * 3600 + m * 60))
}

/// Match a name case-insensitively, then by bare city (e.g. `sofia`).
fn resolve_named(input: &str) -> Result<Tz, String> {
    let lower = input.to_lowercase();
    let city_of = |tz: &Tz| tz.name().rsplit('/').next().unwrap_or("").to_lowercase();

    // Case-insensitive full name (so `europe/sofia` works).
    if let Some(tz) = TZ_VARIANTS.iter().find(|tz| tz.name().to_lowercase() == lower) {
        return Ok(*tz);
    }

    // Bare city: unique -> accept, several -> ask which.
    let city_hits: Vec<&Tz> = TZ_VARIANTS.iter().filter(|tz| city_of(tz) == lower).collect();
    match city_hits.as_slice() {
        [tz] => Ok(**tz),
        [] => Err(format!(
            "Unknown timezone `{input}`. Pick one from the autocomplete list, \
             or use a name like `Europe/Sofia` or a GMT offset like `GMT+2`. \
             Summer-time abbreviations (EEST, CEST, BST…) aren't zones — pick \
             your city and daylight saving is applied automatically."
        )),
        many => {
            let names: Vec<&str> = many.iter().map(|tz| tz.name()).collect();
            Err(format!(
                "`{input}` matches several zones: {}. Use the full name.",
                names.join(", ")
            ))
        }
    }
}

/// Days in a given month, leap years included.
fn days_in_month(y: i32, m: u32) -> u32 {
    let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    NaiveDate::from_ymd_opt(ny, nm, 1)
        .and_then(|d| d.pred_opt())
        .map(|d| d.day())
        .unwrap_or(31)
}

/// Validate the fields against the calendar and zone, returning a precise error
/// for each way it can be wrong.
fn build_remind_at(tz: &UserTz, y: i32, mo: u32, d: u32, h: u32, mi: u32) -> Result<DateTime<Utc>, String> {
    if !(1..=12).contains(&mo) {
        return Err(format!("Month must be 1–12 (you gave {mo})."));
    }
    let dim = days_in_month(y, mo);
    if !(1..=dim).contains(&d) {
        let month_name = NaiveDate::from_ymd_opt(y, mo, 1)
            .map(|date| date.format("%B %Y").to_string())
            .unwrap_or_default();
        return Err(format!("{month_name} has {dim} days — day {d} doesn't exist."));
    }
    if h > 23 {
        return Err(format!("Hour must be 0–23 (you gave {h})."));
    }
    if mi > 59 {
        return Err(format!("Minute must be 0–59 (you gave {mi})."));
    }

    match tz.to_utc(y, mo, d, h, mi) {
        LocalResult::Single(dt) => Ok(dt),
        // Fall-back overlap: the wall-clock happens twice. Pick the earlier one.
        LocalResult::Ambiguous(earlier, _) => Ok(earlier),
        // Spring-forward gap: the wall-clock never happens.
        LocalResult::None => Err(format!(
            "{h:02}:{mi:02} doesn't exist on that date in this timezone \
             (daylight-saving jump). Pick a different time."
        )),
    }
}

/// Reminders that DM you at a set time.
#[poise::command(
    slash_command,
    prefix_command,
    rename = "reminder",
    subcommands("create", "list", "timezone"),
    subcommand_required
)]
pub async fn reminder(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// The user's stored default timezone name for this context. A server-specific
/// setting wins over the global one (`guild_id = 0`); `None` if neither is set.
async fn fetch_default_tz(
    pool: &PgPool,
    user_id: i64,
    guild_id: Option<u64>,
) -> Result<Option<String>, Error> {
    // Real guild ids are large, so DESC orders a server setting ahead of global.
    let scopes: Vec<i64> = match guild_id {
        Some(g) => vec![g as i64, 0],
        None => vec![0],
    };
    let tz = sqlx::query_scalar!(
        "SELECT timezone FROM user_timezones \
         WHERE user_id = $1 AND guild_id = ANY($2) \
         ORDER BY guild_id DESC LIMIT 1",
        user_id,
        &scopes,
    )
    .fetch_optional(pool)
    .await?;
    Ok(tz)
}

/// Set a reminder — the bot DMs you at the given time (fires within a minute).
// Polling runs at minute resolution, so finer input precision wouldn't help.
#[poise::command(slash_command, prefix_command, rename = "create")]
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
    #[description = "What to remind you of"] message: String,
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
    // Explicit arg wins; otherwise fall back to the user's saved default.
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

    // No time fields at all -> one hour from now.
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
    let max_at = now.checked_add_months(Months::new(1)).unwrap_or(now);
    if remind_at > max_at {
        ctx.say("Reminders can be set at most one month in advance.")
            .await?;
        return Ok(());
    }

    let user_id = ctx.author().id.get() as i64;
    let pending = sqlx::query_scalar!("SELECT COUNT(*) FROM reminders WHERE user_id = $1", user_id)
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
        "INSERT INTO reminders (user_id, remind_at, message) VALUES ($1, $2, $3)",
        user_id,
        remind_at,
        message,
    )
    .execute(&*ctx.data().pool)
    .await?;

    // Discord renders <t:..> in each viewer's own locale, so no UTC math needed.
    let ts = remind_at.timestamp();
    ctx.say(format!(
        "Got it! I'll DM you <t:{ts}:F> (<t:{ts}:R>) — timezone `{tz_name}`."
    ))
    .await?;

    Ok(())
}

/// List your pending reminders.
#[poise::command(slash_command, prefix_command, rename = "list")]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().qualified_name,
        author = %ctx.author().id,
    )
)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let rows = sqlx::query!(
        "SELECT id, remind_at, message FROM reminders WHERE user_id = $1 ORDER BY remind_at",
        user_id,
    )
    .fetch_all(&*ctx.data().pool)
    .await?;

    if rows.is_empty() {
        ctx.say("You have no pending reminders.").await?;
        return Ok(());
    }

    let mut out = format!("**Your pending reminders ({}):**\n", rows.len());
    for row in &rows {
        let ts = row.remind_at.timestamp();
        let msg = if row.message.chars().count() > 80 {
            format!("{}…", row.message.chars().take(80).collect::<String>())
        } else {
            row.message.clone()
        };
        let line = format!("`#{}` <t:{ts}:F> (<t:{ts}:R>) — {msg}\n", row.id);
        // Discord caps messages at 2000 chars.
        if out.len() + line.len() > 1900 {
            out.push_str("…and more.");
            break;
        }
        out.push_str(&line);
    }

    ctx.say(out).await?;
    Ok(())
}

/// Set your default timezone — for this server, or everywhere with `global`.
#[poise::command(slash_command, prefix_command, rename = "timezone")]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().qualified_name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
    )
)]
pub async fn timezone(
    ctx: Context<'_>,
    #[description = "City (Europe/Sofia, Sofia) or GMT offset (GMT+2, +02:00)"]
    #[autocomplete = "autocomplete_timezone"]
    timezone: String,
    #[description = "Apply everywhere instead of just this server"] global: Option<bool>,
) -> Result<(), Error> {
    // Validate and canonicalize so what we store round-trips through resolve_tz.
    let tz_name = match resolve_tz(Some(&timezone)) {
        Ok((_, name)) => name,
        Err(msg) => {
            ctx.say(msg).await?;
            return Ok(());
        }
    };

    // No per-server scope exists in DMs, so default those to global.
    let is_dm = ctx.guild_id().is_none();
    let global = global.unwrap_or(false) || is_dm;
    let scope_guild = if global {
        0
    } else {
        ctx.guild_id().map(|g| g.get() as i64).unwrap_or(0)
    };

    sqlx::query!(
        "INSERT INTO user_timezones (user_id, guild_id, timezone) VALUES ($1, $2, $3) \
         ON CONFLICT (user_id, guild_id) DO UPDATE SET timezone = EXCLUDED.timezone",
        ctx.author().id.get() as i64,
        scope_guild,
        tz_name,
    )
    .execute(&*ctx.data().pool)
    .await?;

    let scope = if global { "everywhere" } else { "this server" };
    ctx.say(format!("Default timezone set to `{tz_name}` for {scope}."))
        .await?;
    Ok(())
}
