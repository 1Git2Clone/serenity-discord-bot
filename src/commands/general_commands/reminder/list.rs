use chrono::{DateTime, Utc};

use crate::prelude::*;

/// Reminders shown per page in `list`.
const PAGE_SIZE: usize = 6;
/// Message chars shown per list row before truncating, so long ones don't flood the page.
const ROW_PREVIEW_CHARS: usize = 120;
/// How long the list paginator stays interactive.
const PAGINATE_TIMEOUT_SECS: u64 = 300;

/// Which reminders `list` shows.
#[derive(Debug, poise::ChoiceParameter, PartialEq)]
pub enum StatusFilter {
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
        Some(term) => format!("Reminders matching \u{201C}{term}\u{201D} ({total})"),
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

    handle
        .edit(ctx, poise::CreateReply::default().embed(render(page)).components(vec![]))
        .await?;
    Ok(())
}
