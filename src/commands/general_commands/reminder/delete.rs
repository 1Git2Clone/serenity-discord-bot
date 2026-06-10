use crate::prelude::*;

use super::MAX_AUTOCOMPLETE;

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
