use chrono::{DateTime, Datelike, FixedOffset, LocalResult, Months, NaiveDate, TimeZone, Utc};
use chrono_tz::{TZ_VARIANTS, Tz};

use crate::prelude::*;

/// Max reminders a single user may have pending at once.
const MAX_PENDING_PER_USER: i64 = 50;
/// Discord caps autocomplete responses at 25 entries.
const MAX_AUTOCOMPLETE: usize = 25;
/// Max characters allowed in a reminder message.
const MAX_MESSAGE_CHARS: usize = 500;
/// Reminders shown per page in `list`.
const PAGE_SIZE: usize = 6;
/// Message chars shown per list row before truncating, so long ones don't flood the page.
const ROW_PREVIEW_CHARS: usize = 120;
/// How long the list paginator stays interactive.
const PAGINATE_TIMEOUT_SECS: u64 = 300;

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
    rename = "reminder",
    subcommands("create", "list", "search", "delete", "timezone"),
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
    // `max_length` is enforced by Discord for the slash option; guard anyway.
    if message.chars().count() > MAX_MESSAGE_CHARS {
        ctx.say(format!(
            "Reminders are capped at {MAX_MESSAGE_CHARS} characters."
        ))
        .await?;
        return Ok(());
    }

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

/// Which reminders `list` shows.
#[derive(Debug, poise::ChoiceParameter, PartialEq)]
enum StatusFilter {
    #[name = "All"]
    All,
    #[name = "Pending"]
    Pending,
    #[name = "Finished"]
    Finished,
}

/// Escape LIKE wildcards so a user's `%`/`_` match literally (paired with
/// `ESCAPE '\'` in the query).
fn escape_like(term: &str) -> String {
    term.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// Render one reminder row into a list line. No DB id — it's a global serial
/// that means nothing to the user and leaks total volume.
fn render_row(remind_at: DateTime<Utc>, finished_at: Option<DateTime<Utc>>, message: &str) -> String {
    let ts = remind_at.timestamp();
    let msg = if message.chars().count() > ROW_PREVIEW_CHARS {
        format!("{}…", message.chars().take(ROW_PREVIEW_CHARS).collect::<String>())
    } else {
        message.to_string()
    };
    match finished_at {
        None => format!("⏳ <t:{ts}:F> (<t:{ts}:R>) — {msg}"),
        Some(_) => format!("✅ <t:{ts}:F> — {msg}"),
    }
}

/// Query the user's reminders for a status (and optional search term) and show
/// them in the paginated embed. Shared by `list` and `search`.
async fn show_reminders(
    ctx: Context<'_>,
    status: StatusFilter,
    search: Option<&str>,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;

    // `search` is NULL -> no filter; otherwise a literal-escaped ILIKE pattern.
    let pattern = search.map(|t| format!("%{}%", escape_like(t)));

    let rows = sqlx::query!(
        r#"SELECT remind_at, finished_at, message
           FROM reminders
           WHERE user_id = $1
             AND ($2::bool OR (finished_at IS NULL) = $3::bool)
             AND ($4::text IS NULL OR message ILIKE $4 ESCAPE '\')
           ORDER BY (finished_at IS NULL) DESC, COALESCE(finished_at, remind_at)"#,
        user_id,
        status == StatusFilter::All,
        status == StatusFilter::Pending,
        pattern,
    )
    .fetch_all(&*ctx.data().pool)
    .await?;

    if rows.is_empty() {
        let what = match status {
            StatusFilter::All => "reminders",
            StatusFilter::Pending => "pending reminders",
            StatusFilter::Finished => "finished reminders",
        };
        let suffix = if pattern.is_some() { " matching that search" } else { "" };
        ctx.say(format!("You have no {what}{suffix}.")).await?;
        return Ok(());
    }

    let lines: Vec<String> = rows
        .iter()
        .map(|r| render_row(r.remind_at, r.finished_at, &r.message))
        .collect();

    let total = lines.len();
    let pages: Vec<String> = lines
        .chunks(PAGE_SIZE)
        .map(|chunk| chunk.join("\n\n"))
        .collect();
    let header = match search {
        Some(term) => format!("Reminders matching “{term}” ({total})"),
        None => format!("Your reminders ({total})"),
    };

    paginate(ctx, &header, &pages).await
}

/// List your reminders, optionally filtered by status.
#[poise::command(slash_command, rename = "list")]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().qualified_name,
        author = %ctx.author().id,
    )
)]
pub async fn list(
    ctx: Context<'_>,
    #[description = "Which reminders to show (default All)"] status: Option<StatusFilter>,
) -> Result<(), Error> {
    show_reminders(ctx, status.unwrap_or(StatusFilter::All), None).await
}

/// Search your reminders by message text.
#[poise::command(slash_command, rename = "search")]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().qualified_name,
        author = %ctx.author().id,
    )
)]
pub async fn search(
    ctx: Context<'_>,
    #[description = "Text to look for in the reminder message"]
    #[max_length = 100]
    query: String,
    #[description = "Which reminders to search (default All)"] status: Option<StatusFilter>,
) -> Result<(), Error> {
    show_reminders(ctx, status.unwrap_or(StatusFilter::All), Some(&query)).await
}

/// Autocomplete for `/reminder delete`: returns pending reminders as `"<id> <time> — <preview>"`.
/// The id prefix is parsed back in the handler; the rest is shown to the user in the dropdown.
async fn autocomplete_pending_reminder(ctx: Context<'_>, partial: &str) -> Vec<String> {
    let user_id = ctx.author().id.get() as i64;
    let rows = sqlx::query!(
        "SELECT id, remind_at, message FROM reminders \
         WHERE user_id = $1 AND finished_at IS NULL \
         ORDER BY remind_at \
         LIMIT 50",
        user_id,
    )
    .fetch_all(&*ctx.data().pool)
    .await
    .unwrap_or_default();

    let needle = partial.to_lowercase();
    rows.into_iter()
        .filter(|r| needle.is_empty() || r.message.to_lowercase().contains(&needle))
        .take(MAX_AUTOCOMPLETE)
        .map(|r| {
            let ts = r.remind_at.format("%Y-%m-%d %H:%M UTC");
            let preview: String = r.message.chars().take(80).collect();
            format!("{} {ts} — {preview}", r.id)
        })
        .collect()
}

/// Delete a pending reminder you set.
#[poise::command(slash_command, rename = "delete")]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().qualified_name,
        author = %ctx.author().id,
    )
)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "Which pending reminder to delete (pick from the list)"]
    #[autocomplete = "autocomplete_pending_reminder"]
    reminder: String,
) -> Result<(), Error> {
    // The autocomplete value is "<id> <time> — <preview>"; parse the leading id.
    let reminder_id: i64 = match reminder.split_whitespace().next().and_then(|s| s.parse().ok()) {
        Some(id) => id,
        None => {
            ctx.say("Couldn't parse that selection — please pick one from the autocomplete list.")
                .await?;
            return Ok(());
        }
    };

    let user_id = ctx.author().id.get() as i64;
    let deleted = sqlx::query!(
        "DELETE FROM reminders \
         WHERE id = $1 AND user_id = $2 AND finished_at IS NULL \
         RETURNING remind_at, message",
        reminder_id,
        user_id,
    )
    .fetch_optional(&*ctx.data().pool)
    .await?;

    match deleted {
        Some(r) => {
            let ts = r.remind_at.timestamp();
            ctx.say(format!("Deleted reminder for <t:{ts}:F>: {}", r.message))
                .await?;
        }
        None => {
            ctx.say("No matching pending reminder found — it may have already fired or been deleted.")
                .await?;
        }
    }
    Ok(())
}

/// Ephemeral, button-driven embed pager: ⏮ ◀ [page x/y → modal] ▶ ⏭.
async fn paginate(ctx: Context<'_>, header: &str, pages: &[String]) -> Result<(), Error> {
    let total = pages.len();
    let id = ctx.id();
    let (first, prev, goto, next, last) = (
        format!("{id}first"),
        format!("{id}prev"),
        format!("{id}goto"),
        format!("{id}next"),
        format!("{id}last"),
    );

    let render = |page: usize| {
        serenity::CreateEmbed::new()
            .title(header)
            .description(&pages[page])
            .footer(serenity::CreateEmbedFooter::new(format!("Page {}/{total}", page + 1)))
    };
    let buttons = |page: usize| {
        let at_start = page == 0;
        let at_end = page + 1 >= total;
        serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(&first).emoji('⏮').disabled(at_start),
            serenity::CreateButton::new(&prev).emoji('◀').disabled(at_start),
            serenity::CreateButton::new(&goto)
                .label(format!("{}/{total}", page + 1))
                .style(serenity::ButtonStyle::Secondary),
            serenity::CreateButton::new(&next).emoji('▶').disabled(at_end),
            serenity::CreateButton::new(&last).emoji('⏭').disabled(at_end),
        ])
    };

    let mut reply = poise::CreateReply::default().ephemeral(true).embed(render(0));
    if total > 1 {
        reply = reply.components(vec![buttons(0)]);
    }
    let handle = ctx.send(reply).await?;
    if total <= 1 {
        return Ok(());
    }

    let mut page = 0usize;
    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        .filter(move |p| p.data.custom_id.starts_with(&id.to_string()))
        .timeout(std::time::Duration::from_secs(PAGINATE_TIMEOUT_SECS))
        .await
    {
        let cid = &press.data.custom_id;
        if *cid == goto {
            // The modal IS this interaction's response; the page change is then
            // applied via the modal-submit interaction.
            let modal = serenity::CreateQuickModal::new("Go to page")
                .timeout(std::time::Duration::from_secs(60))
                .short_field(format!("Page number (1–{total})"));
            if let Some(resp) = press.quick_modal(ctx.serenity_context(), modal).await? {
                if let Ok(n) = resp.inputs[0].trim().parse::<usize>() {
                    page = n.saturating_sub(1).min(total - 1);
                }
                resp.interaction
                    .create_response(
                        ctx.serenity_context(),
                        serenity::CreateInteractionResponse::UpdateMessage(
                            serenity::CreateInteractionResponseMessage::new()
                                .embed(render(page))
                                .components(vec![buttons(page)]),
                        ),
                    )
                    .await?;
            }
            continue;
        }

        page = if *cid == first {
            0
        } else if *cid == prev {
            page.saturating_sub(1)
        } else if *cid == next {
            (page + 1).min(total - 1)
        } else if *cid == last {
            total - 1
        } else {
            continue;
        };

        press
            .create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .embed(render(page))
                        .components(vec![buttons(page)]),
                ),
            )
            .await?;
    }

    // Drop the buttons once navigation times out so stale controls don't linger.
    handle
        .edit(ctx, poise::CreateReply::default().embed(render(page)).components(vec![]))
        .await?;
    Ok(())
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
