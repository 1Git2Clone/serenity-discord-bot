use std::collections::HashSet;
use std::sync::Arc;

use poise::serenity_prelude::{Http, UserId};
use sqlx::PgPool;
use tracing::Instrument;

/// Most finished reminders kept per user; older ones are pruned on each fire.
const HISTORY_LIMIT: i64 = 100;

/// Polls the database every 60 seconds for due reminders, sends each user a DM,
/// and marks them finished (kept as history, capped per user). Minute resolution
/// matches the command's input granularity — finer polling would only add load
/// for accuracy the schedule can't promise. Runs for the lifetime of the process.
#[allow(clippy::infinite_loop)]
pub async fn reminder_polling_loop(http: Arc<Http>, pool: Arc<PgPool>) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

        async {
            // Atomically claim due reminders across instances with FOR UPDATE
            // SKIP LOCKED — each instance gets a disjoint subset of rows.
            let claimed = match sqlx::query!(
                "WITH claimed AS ( \
                     SELECT id, user_id, message FROM reminders \
                     WHERE finished_at IS NULL AND remind_at <= NOW() \
                     ORDER BY remind_at \
                     FOR UPDATE SKIP LOCKED \
                 ) \
                 UPDATE reminders SET finished_at = NOW() \
                 FROM claimed \
                 WHERE reminders.id = claimed.id \
                 RETURNING claimed.id, claimed.user_id, claimed.message"
            )
            .fetch_all(&*pool)
            .await
            {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(error = %e, "Failed to claim due reminders");
                    return;
                }
            };

            let mut fired_users: HashSet<i64> = HashSet::new();

            for row in &claimed {
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

                // Already marked finished by the atomic claim above;
                // just record the user for history pruning.
                fired_users.insert(row.user_id);
            }

            // Cap each affected user's finished history to the newest entries.
            for user_id in fired_users {
                if let Err(e) = sqlx::query!(
                    "DELETE FROM reminders WHERE user_id = $1 AND finished_at IS NOT NULL \
                     AND id NOT IN ( \
                         SELECT id FROM reminders \
                         WHERE user_id = $1 AND finished_at IS NOT NULL \
                         ORDER BY finished_at DESC LIMIT $2 \
                     )",
                    user_id,
                    HISTORY_LIMIT,
                )
                .execute(&*pool)
                .await
                {
                    tracing::error!(user_id, error = %e, "Failed to prune reminder history");
                }
            }
        }
        .instrument(tracing::info_span!("reminder_poll", category = "reminders"))
        .await;
    }
}
