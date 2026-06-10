use chrono::{DateTime, Utc};
use chrono_tz::{TZ_VARIANTS, Tz};

use crate::prelude::*;

use super::MAX_AUTOCOMPLETE;

/// A user-supplied timezone: either a named IANA zone (`Europe/Sofia`) or a
/// fixed offset from GMT (`GMT+2`, `+02:00`). Both implement `TimeZone`, but
/// they're distinct types, so this enum unifies them.
pub(super) enum UserTz {
    Named(Tz),
    Fixed(chrono::FixedOffset),
}

impl UserTz {
    /// Interpret wall-clock fields in this zone and convert to a UTC instant.
    /// Returns the daylight-saving disposition so the caller can report a gap.
    fn to_utc(
        &self,
        y: i32,
        mo: u32,
        d: u32,
        h: u32,
        mi: u32,
    ) -> chrono::LocalResult<DateTime<Utc>> {
        use chrono::TimeZone as _;
        match self {
            Self::Named(tz) => tz.with_ymd_and_hms(y, mo, d, h, mi, 0).map(|dt| dt.to_utc()),
            Self::Fixed(off) => off.with_ymd_and_hms(y, mo, d, h, mi, 0).map(|dt| dt.to_utc()),
        }
    }

    /// Convert a UTC instant to a local `NaiveDateTime` in this zone.
    pub(super) fn to_local(&self, utc: DateTime<Utc>) -> chrono::NaiveDateTime {
        match self {
            Self::Named(tz) => utc.with_timezone(tz).naive_local(),
            Self::Fixed(off) => utc.with_timezone(off).naive_local(),
        }
    }

    /// "Now" wall-clock fields in this zone, for filling unspecified inputs.
    pub(super) fn now_parts(&self) -> (i32, u32, u32) {
        use chrono::Datelike as _;
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
pub(super) fn resolve_tz(input: Option<&str>) -> Result<(UserTz, String), String> {
    let s = input.unwrap_or("").trim();
    if s.is_empty() {
        return Ok((UserTz::Named(Tz::UTC), "UTC".to_string()));
    }

    if let Ok(tz) = s.parse::<Tz>() {
        return Ok((UserTz::Named(tz), tz.name().to_string()));
    }

    if let Some(off) = parse_offset(s) {
        return Ok((UserTz::Fixed(off), format!("UTC{off}")));
    }

    let tz = resolve_named(s)?;
    Ok((UserTz::Named(tz), tz.name().to_string()))
}

/// Filter zone names by the partial input for the slash command's timezone
/// dropdown. `contains` matches the city segment too (`sofia` -> `Europe/Sofia`).
/// `Etc/GMT±N` zones are hidden because their sign is inverted (`Etc/GMT+2` is
/// UTC−2) — a free-text `GMT+2` offset goes through the correct parser instead.
pub(super) async fn autocomplete_timezone(_ctx: Context<'_>, partial: &str) -> Vec<String> {
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
fn parse_offset(s: &str) -> Option<chrono::FixedOffset> {
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
    chrono::FixedOffset::east_opt(sign * (h * 3600 + m * 60))
}

/// Match a name case-insensitively, then by bare city (e.g. `sofia`).
fn resolve_named(input: &str) -> Result<Tz, String> {
    let lower = input.to_lowercase();
    let city_of = |tz: &Tz| tz.name().rsplit('/').next().unwrap_or("").to_lowercase();

    if let Some(tz) = TZ_VARIANTS.iter().find(|tz| tz.name().to_lowercase() == lower) {
        return Ok(*tz);
    }

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
    use chrono::Datelike as _;
    let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    chrono::NaiveDate::from_ymd_opt(ny, nm, 1)
        .and_then(|d| d.pred_opt())
        .map(|d| d.day())
        .unwrap_or(31)
}

/// Validate the fields against the calendar and zone, returning a precise error
/// for each way it can be wrong.
pub(super) fn build_remind_at(
    tz: &UserTz,
    y: i32,
    mo: u32,
    d: u32,
    h: u32,
    mi: u32,
) -> Result<DateTime<Utc>, String> {
    use chrono::LocalResult;

    if !(1..=12).contains(&mo) {
        return Err(format!("Month must be 1–12 (you gave {mo})."));
    }
    let dim = days_in_month(y, mo);
    if !(1..=dim).contains(&d) {
        let month_name = chrono::NaiveDate::from_ymd_opt(y, mo, 1)
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
        LocalResult::Ambiguous(earlier, _) => Ok(earlier),
        LocalResult::None => Err(format!(
            "{h:02}:{mi:02} doesn't exist on that date in this timezone \
             (daylight-saving jump). Pick a different time."
        )),
    }
}

/// The user's stored default timezone name for this context. A server-specific
/// setting wins over the global one (`guild_id = 0`); `None` if neither is set.
pub(super) async fn fetch_default_tz(
    pool: &PgPool,
    user_id: i64,
    guild_id: Option<u64>,
) -> Result<Option<String>, Error> {
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

/// Set your default timezone — for this server, or everywhere with `global`.
#[poise::command(slash_command, rename = "timezone")]
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
    let tz_name = match resolve_tz(Some(&timezone)) {
        Ok((_, name)) => name,
        Err(msg) => {
            ctx.say(msg).await?;
            return Ok(());
        }
    };

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
