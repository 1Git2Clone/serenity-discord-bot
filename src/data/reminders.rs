use std::sync::Arc;

use poise::serenity_prelude::{Http, UserId};
use sqlx::PgPool;
use tracing::Instrument;

/// Polls the database every 60 seconds for due reminders, sends each user a DM,
/// and deletes them. Minute resolution matches the command's input granularity —
/// finer polling would only add load for accuracy the schedule can't promise.
/// Runs for the lifetime of the process.
pub async fn reminder_polling_loop(http: Arc<Http>, pool: Arc<PgPool>) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

        async {
            let rows = match sqlx::query!(
                "SELECT id, user_id, message FROM reminders WHERE remind_at <= NOW()"
            )
            .fetch_all(&*pool)
            .await
            {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(error = %e, "Failed to query due reminders");
                    return;
                }
            };

            for row in rows {
                let user_id = UserId::new(row.user_id as u64);

                match user_id.create_dm_channel(&http).await {
                    Ok(channel) => {
                        if let Err(e) = channel.say(&http, &row.message).await {
                            tracing::warn!(
                                user_id = row.user_id,
                                error = %e,
                                "Failed to send reminder DM",
                            );
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            user_id = row.user_id,
                            error = %e,
                            "Failed to open DM channel for reminder",
                        );
                    }
                }

                // Delete regardless — if the DM failed the user has closed DMs
                // and retrying every minute would be spam.
                if let Err(e) = sqlx::query!("DELETE FROM reminders WHERE id = $1", row.id)
                    .execute(&*pool)
                    .await
                {
                    tracing::error!(id = row.id, error = %e, "Failed to delete fired reminder");
                }
            }
        }
        .instrument(tracing::info_span!("reminder_poll", category = "reminders"))
        .await;
    }
}
