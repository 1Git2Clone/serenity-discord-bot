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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MentionsTable {}

pub type Mentions = i64;

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

impl MentionsTable {
    #[tracing::instrument(
        fields(
            category = "sql",
            db_pool = ?pool,
        )
    )]
    pub async fn fetch_mentions(pool: &PgPool) -> sqlx::Result<Mentions> {
        let row = sqlx::query!("SELECT mentions FROM bot_mentions LIMIT 1")
            .fetch_one(pool)
            .await?;

        Ok(row.mentions)
    }

    #[tracing::instrument(
        fields(
            category = "sql",
            db_pool = ?pool,
        )
    )]
    pub async fn update_mentions(pool: &PgPool, mentions: Mentions) -> sqlx::Result<()> {
        sqlx::query!("UPDATE bot_mentions SET mentions = $1", mentions)
            .execute(pool)
            .await?;
        Ok(())
    }
}
