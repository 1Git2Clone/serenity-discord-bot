use crate::prelude::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LevelsTable {}

pub type UserId = i64;
pub type GuildId = i64;
pub type ExperiencePoints = i32;
pub type Level = i32;
pub type Rank = i64;

pub struct UserRank {
    pub user_id: i64,
    pub xp: i32,
    pub level: i32,
}

impl LevelsTable {
    #[tracing::instrument(
        fields(
            category = "sql",
            db_pool = ?pool,
            user_id = %user_id,
            guild_id = %guild_id
        )
    )]
    pub async fn add_user_level(
        pool: &PgPool,
        user_id: UserId,
        guild_id: GuildId,
        experience_points: ExperiencePoints,
        level: Level,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            "INSERT INTO user_stats (user_id, guild_id, experience_points, level)
             VALUES ($1, $2, $3, $4)",
            user_id,
            guild_id,
            experience_points,
            level
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    #[tracing::instrument(
        fields(
            category = "sql",
            db_pool = ?pool,
            user_id = %user_id,
            guild_id = %guild_id
        )
    )]
    pub async fn fetch_user_level(
        pool: &PgPool,
        user_id: UserId,
        guild_id: GuildId,
    ) -> sqlx::Result<(ExperiencePoints, Level)> {
        let row = sqlx::query!(
            "SELECT experience_points, level
             FROM user_stats
             WHERE user_id = $1 AND guild_id = $2",
            user_id,
            guild_id
        )
        .fetch_one(pool)
        .await?;

        Ok((row.experience_points, row.level))
    }

    #[tracing::instrument(
        fields(
            category = "sql",
            db_pool = ?pool,
            guild_id = %guild_id
        )
    )]
    pub async fn fetch_top_nine_users(pool: &PgPool, guild_id: i64) -> sqlx::Result<Vec<UserRank>> {
        // Use the struct here
        let rows = sqlx::query!(
            "SELECT user_id, experience_points, level
            FROM user_stats
            WHERE guild_id = $1
            ORDER BY level DESC, experience_points DESC
            LIMIT 9",
            guild_id
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| UserRank {
                user_id: r.user_id,
                xp: r.experience_points,
                level: r.level,
            })
            .collect())
    }

    #[tracing::instrument(
        fields(
            category = "sql",
            db_pool = ?pool,
            user_id = %user_id,
            guild_id = %guild_id
        )
    )]
    pub async fn update_user_level(
        pool: &PgPool,
        experience_points: ExperiencePoints,
        level: Level,
        user_id: UserId,
        guild_id: GuildId,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            "UPDATE user_stats
             SET experience_points = $1, level = $2
             WHERE user_id = $3 AND guild_id = $4",
            experience_points,
            level,
            user_id,
            guild_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument(
        fields(
            category = "sql",
            db_pool = ?pool,
            user_id = %user_id,
            guild_id = %guild_id
        )
    )]
    pub async fn fetch_user_level_and_rank(
        pool: &PgPool,
        user_id: UserId,
        guild_id: GuildId,
    ) -> sqlx::Result<(Level, ExperiencePoints, Rank)> {
        let row = sqlx::query!(
            "SELECT us.level, us.experience_points,
                (SELECT COUNT(*)
                 FROM user_stats AS inner_u
                 WHERE inner_u.guild_id = us.guild_id
                   AND (inner_u.level > us.level
                        OR (inner_u.level = us.level
                            AND inner_u.experience_points >= us.experience_points))
                ) AS rank
             FROM user_stats AS us
             WHERE us.user_id = $1 AND us.guild_id = $2",
            user_id,
            guild_id
        )
        .fetch_one(pool)
        .await?;

        Ok((row.level, row.experience_points, row.rank.unwrap_or(0)))
    }
}

#[cfg(feature = "ai")]
pub enum AiChannelsTable {}

#[cfg(feature = "ai")]
impl AiChannelsTable {
    #[tracing::instrument(
        fields(
            category = "sql",
            db_pool = ?pool,
            channel_id = %channel_id,
            guild_id = %guild_id
        )
    )]
    /// Returns `true` if the channel was newly registered.
    pub async fn register(pool: &PgPool, channel_id: i64, guild_id: i64) -> sqlx::Result<bool> {
        let res = sqlx::query!(
            "INSERT INTO ai_channels (channel_id, guild_id)
             VALUES ($1, $2)
             ON CONFLICT (channel_id) DO NOTHING",
            channel_id,
            guild_id
        )
        .execute(pool)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    #[tracing::instrument(
        fields(
            category = "sql",
            db_pool = ?pool,
            channel_id = %channel_id
        )
    )]
    /// Returns `true` if the channel was registered before.
    pub async fn unregister(pool: &PgPool, channel_id: i64) -> sqlx::Result<bool> {
        let res = sqlx::query!("DELETE FROM ai_channels WHERE channel_id = $1", channel_id)
            .execute(pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    #[tracing::instrument(
        fields(
            category = "sql",
            db_pool = ?pool,
        )
    )]
    pub async fn fetch_all(pool: &PgPool) -> sqlx::Result<Vec<i64>> {
        let rows = sqlx::query!("SELECT channel_id FROM ai_channels")
            .fetch_all(pool)
            .await?;

        Ok(rows.into_iter().map(|row| row.channel_id).collect())
    }
}

#[cfg(feature = "ai")]
pub enum AiReviewGuildsTable {}

#[cfg(feature = "ai")]
impl AiReviewGuildsTable {
    #[tracing::instrument(
        fields(
            category = "sql",
            db_pool = ?pool,
            guild_id = %guild_id
        )
    )]
    /// Returns `true` if the guild was newly registered.
    pub async fn register(pool: &PgPool, guild_id: i64) -> sqlx::Result<bool> {
        let res = sqlx::query!(
            "INSERT INTO ai_review_guilds (guild_id)
             VALUES ($1)
             ON CONFLICT (guild_id) DO NOTHING",
            guild_id
        )
        .execute(pool)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    #[tracing::instrument(
        fields(
            category = "sql",
            db_pool = ?pool,
            guild_id = %guild_id
        )
    )]
    /// Returns `true` if the guild was registered before.
    pub async fn unregister(pool: &PgPool, guild_id: i64) -> sqlx::Result<bool> {
        let res = sqlx::query!("DELETE FROM ai_review_guilds WHERE guild_id = $1", guild_id)
            .execute(pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    #[tracing::instrument(
        fields(
            category = "sql",
            db_pool = ?pool,
        )
    )]
    pub async fn fetch_all(pool: &PgPool) -> sqlx::Result<Vec<i64>> {
        let rows = sqlx::query!("SELECT guild_id FROM ai_review_guilds")
            .fetch_all(pool)
            .await?;

        Ok(rows.into_iter().map(|row| row.guild_id).collect())
    }
}

/// A row returned by `CustomReactionsTable::fetch_live` and `fetch_all_live`.
pub struct CustomReactionRow {
    pub id: i64,
    pub pattern: String,
    pub image_url: String,
    pub anywhere: bool,
}

pub enum CustomReactionsTable {}

impl CustomReactionsTable {
    #[tracing::instrument(
        fields(
            category = "sql",
            db_pool = ?pool,
            guild_id = %guild_id,
            pattern = %pattern,
        )
    )]
    /// Insert a new reaction. Returns the assigned `id`.
    pub async fn insert(
        pool: &PgPool,
        guild_id: i64,
        pattern: &str,
        image_url: &str,
        anywhere: bool,
    ) -> sqlx::Result<i64> {
        let row = sqlx::query!(
            "INSERT INTO custom_reactions (guild_id, pattern, image_url, anywhere)
             VALUES ($1, $2, $3, $4)
             RETURNING id",
            guild_id,
            pattern,
            image_url,
            anywhere,
        )
        .fetch_one(pool)
        .await?;
        Ok(row.id)
    }

    #[tracing::instrument(
        fields(
            category = "sql",
            db_pool = ?pool,
            id = %id,
            guild_id = %guild_id,
        )
    )]
    /// Soft-delete a reaction. Returns `true` if a live row was found and deleted.
    pub async fn soft_delete(pool: &PgPool, id: i64, guild_id: i64) -> sqlx::Result<bool> {
        let res = sqlx::query!(
            "UPDATE custom_reactions
             SET deleted_at = now()
             WHERE id = $1 AND guild_id = $2 AND deleted_at IS NULL",
            id,
            guild_id,
        )
        .execute(pool)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    #[tracing::instrument(
        fields(
            category = "sql",
            db_pool = ?pool,
            guild_id = %guild_id,
        )
    )]
    /// Fetch all live reactions for a single guild, ordered by id.
    pub async fn fetch_live(pool: &PgPool, guild_id: i64) -> sqlx::Result<Vec<CustomReactionRow>> {
        let rows = sqlx::query!(
            "SELECT id, pattern, image_url, anywhere
             FROM custom_reactions
             WHERE guild_id = $1 AND deleted_at IS NULL
             ORDER BY id",
            guild_id,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| CustomReactionRow {
                id: r.id,
                pattern: r.pattern,
                image_url: r.image_url,
                anywhere: r.anywhere,
            })
            .collect())
    }

    #[tracing::instrument(
        fields(
            category = "sql",
            db_pool = ?pool,
        )
    )]
    /// Fetch all live reactions across every guild, ordered by guild_id then id.
    /// Used for cold-start cache seeding.
    pub async fn fetch_all_live(pool: &PgPool) -> sqlx::Result<Vec<(i64, CustomReactionRow)>> {
        let rows = sqlx::query!(
            "SELECT id, guild_id, pattern, image_url, anywhere
             FROM custom_reactions
             WHERE deleted_at IS NULL
             ORDER BY guild_id, id",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                (
                    r.guild_id,
                    CustomReactionRow {
                        id: r.id,
                        pattern: r.pattern,
                        image_url: r.image_url,
                        anywhere: r.anywhere,
                    },
                )
            })
            .collect())
    }

    #[tracing::instrument(
        fields(
            category = "sql",
            db_pool = ?pool,
            guild_id = %guild_id,
        )
    )]
    /// Count live reactions for a guild.
    pub async fn count_live(pool: &PgPool, guild_id: i64) -> sqlx::Result<i64> {
        let row = sqlx::query!(
            "SELECT COUNT(*) AS count
             FROM custom_reactions
             WHERE guild_id = $1 AND deleted_at IS NULL",
            guild_id,
        )
        .fetch_one(pool)
        .await?;
        Ok(row.count.unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data::command_data::Error, tests::test_pool};

    type TestResult = Result<(), Error>;

    // Sentinel IDs in their own namespace, distinct from the guilds.rs tests.
    const USER_A: i64 = 0x5AFE_0002_0000_0001;
    const USER_B: i64 = 0x5AFE_0002_0000_0002;
    const USER_C: i64 = 0x5AFE_0002_0000_0003;
    const GUILD_ROUNDTRIP: i64 = 0x5AFE_0002_0000_0011;
    const GUILD_TOP_NINE: i64 = 0x5AFE_0002_0000_0012;
    #[cfg(feature = "ai")]
    const CHANNEL: i64 = 0x5AFE_0002_0000_0021;
    #[cfg(feature = "ai")]
    const GUILD_AI: i64 = 0x5AFE_0002_0000_0022;

    /// Unchecked query (not `query!`) so tests don't add entries to the
    /// offline `.sqlx` cache.
    async fn delete_guild_stats(pool: &PgPool, guild_id: i64) -> TestResult {
        sqlx::query("DELETE FROM user_stats WHERE guild_id = $1")
            .bind(guild_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn levels_add_fetch_update_rank_roundtrip() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        delete_guild_stats(&pool, GUILD_ROUNDTRIP).await?;

        LevelsTable::add_user_level(&pool, USER_A, GUILD_ROUNDTRIP, 50, 1).await?;
        assert_eq!(
            LevelsTable::fetch_user_level(&pool, USER_A, GUILD_ROUNDTRIP).await?,
            (50, 1)
        );

        LevelsTable::update_user_level(&pool, 25, 3, USER_A, GUILD_ROUNDTRIP).await?;
        assert_eq!(
            LevelsTable::fetch_user_level(&pool, USER_A, GUILD_ROUNDTRIP).await?,
            (25, 3)
        );

        let (level, xp, rank) =
            LevelsTable::fetch_user_level_and_rank(&pool, USER_A, GUILD_ROUNDTRIP).await?;
        assert_eq!((level, xp), (3, 25));
        assert_eq!(rank, 1);

        delete_guild_stats(&pool, GUILD_ROUNDTRIP).await?;
        Ok(())
    }

    #[tokio::test]
    async fn levels_top_nine_orders_by_level_then_xp() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        delete_guild_stats(&pool, GUILD_TOP_NINE).await?;

        LevelsTable::add_user_level(&pool, USER_A, GUILD_TOP_NINE, 10, 1).await?;
        LevelsTable::add_user_level(&pool, USER_B, GUILD_TOP_NINE, 90, 2).await?;
        LevelsTable::add_user_level(&pool, USER_C, GUILD_TOP_NINE, 10, 2).await?;

        let top = LevelsTable::fetch_top_nine_users(&pool, GUILD_TOP_NINE).await?;
        let ids: Vec<i64> = top.iter().map(|u| u.user_id).collect();
        assert_eq!(ids, vec![USER_B, USER_C, USER_A]);

        delete_guild_stats(&pool, GUILD_TOP_NINE).await?;
        Ok(())
    }

    /// One test for everything touching the bot_mentions singleton row, so
    /// parallel tests can't interleave writes to it. The row's value is
    /// restored at the end.
    #[tokio::test]
    /// One test for everything touching the bot_mentions singleton row, so
    /// parallel tests can't interleave writes to it. The row's value is
    /// restored at the end.
    async fn mentions_add_roundtrip() -> TestResult {
        use crate::database::bot_mentions::add_mentions;

        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        let before = add_mentions(&pool, 0).await?;
        assert_eq!(add_mentions(&pool, 5).await?, before + 5);
        assert_eq!(add_mentions(&pool, -5).await?, before);
        Ok(())
    }

    #[cfg(feature = "ai")]
    #[tokio::test]
    async fn ai_channels_register_unregister_fetch() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        AiChannelsTable::unregister(&pool, CHANNEL).await?;

        AiChannelsTable::register(&pool, CHANNEL, GUILD_AI).await?;
        // Registering twice is a no-op (ON CONFLICT DO NOTHING).
        AiChannelsTable::register(&pool, CHANNEL, GUILD_AI).await?;
        assert!(AiChannelsTable::fetch_all(&pool).await?.contains(&CHANNEL));

        AiChannelsTable::unregister(&pool, CHANNEL).await?;
        assert!(!AiChannelsTable::fetch_all(&pool).await?.contains(&CHANNEL));
        Ok(())
    }

    #[cfg(feature = "ai")]
    #[tokio::test]
    async fn ai_review_guilds_register_unregister_fetch() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        AiReviewGuildsTable::unregister(&pool, GUILD_AI).await?;

        AiReviewGuildsTable::register(&pool, GUILD_AI).await?;
        AiReviewGuildsTable::register(&pool, GUILD_AI).await?;
        assert!(
            AiReviewGuildsTable::fetch_all(&pool)
                .await?
                .contains(&GUILD_AI)
        );

        AiReviewGuildsTable::unregister(&pool, GUILD_AI).await?;
        assert!(
            !AiReviewGuildsTable::fetch_all(&pool)
                .await?
                .contains(&GUILD_AI)
        );
        Ok(())
    }

    const CR_GUILD: i64 = 0x5AFE_0004_0000_0001;

    async fn cleanup_cr(pool: &PgPool) -> TestResult {
        sqlx::query("DELETE FROM custom_reactions WHERE guild_id = $1")
            .bind(CR_GUILD)
            .execute(pool)
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn custom_reactions_insert_fetch_soft_delete_count() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        cleanup_cr(&pool).await?;

        let id = CustomReactionsTable::insert(
            &pool,
            CR_GUILD,
            "hello",
            "https://example.com/a.gif",
            false,
        )
        .await?;
        assert_eq!(CustomReactionsTable::count_live(&pool, CR_GUILD).await?, 1);

        let rows = CustomReactionsTable::fetch_live(&pool, CR_GUILD).await?;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, id);
        assert_eq!(rows[0].pattern, "hello");
        assert!(!rows[0].anywhere);

        let all = CustomReactionsTable::fetch_all_live(&pool).await?;
        assert!(all.iter().any(|(gid, r)| *gid == CR_GUILD && r.id == id));

        assert!(CustomReactionsTable::soft_delete(&pool, id, CR_GUILD).await?);
        // Second call: already deleted.
        assert!(!CustomReactionsTable::soft_delete(&pool, id, CR_GUILD).await?);
        assert_eq!(CustomReactionsTable::count_live(&pool, CR_GUILD).await?, 0);
        assert!(
            CustomReactionsTable::fetch_live(&pool, CR_GUILD)
                .await?
                .is_empty()
        );

        cleanup_cr(&pool).await?;
        Ok(())
    }
}
