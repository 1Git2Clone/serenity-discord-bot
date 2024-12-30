use crate::prelude::*;

pub const DATABASE_FILENAME: &str = "database/bot_database.sqlite";
pub const DATABASE_USERS: &str = "user_stats";
pub const MENTIONS_TABLE_NAME: &str = "bot_mentions";

lazy_static! {
    #[derive(Debug)]
    pub static ref LEVELS_TABLE: HashMap<LevelsSchema, &'static str> = {
        use crate::enums::schemas::LevelsSchema as DbSch;

        HashMap::from([
            (DbSch::UserId, "user_id"),
            (DbSch::GuildId, "guild_id"),
            (DbSch::ExperiencePoints, "experience_points"),
            (DbSch::Level, "level"),
            (DbSch::LastQueryTimestamp, "last_query_timestamp"),
            (DbSch::RankSelector, "rank")
        ])
    };

    pub static ref ADD_USER_LEVEL_QUERY: String = format!(
        "INSERT INTO `{}` (`{}`, `{}`, `{}`, `{}`)
         VALUES (?, ?, ?, ?)",
        DATABASE_USERS,
        LEVELS_TABLE[&LevelsSchema::UserId],
        LEVELS_TABLE[&LevelsSchema::GuildId],
        LEVELS_TABLE[&LevelsSchema::ExperiencePoints],
        LEVELS_TABLE[&LevelsSchema::Level],
    );

    pub static ref FETCH_USER_LEVEL_QUERY: String = format!(
        "SELECT `{}`, `{}`, `{}`
         FROM `{}`
         WHERE `{}` = ? AND `{}` = ?",
        LEVELS_TABLE[&LevelsSchema::UserId],
        LEVELS_TABLE[&LevelsSchema::ExperiencePoints],
        LEVELS_TABLE[&LevelsSchema::Level],
        //
        DATABASE_USERS,
        //
        LEVELS_TABLE[&LevelsSchema::UserId],
        LEVELS_TABLE[&LevelsSchema::GuildId]
    );

    pub static ref FETCH_TOP_NINE_USERS_IN_GUILD_QUERY: String = format!(
        "SELECT
         COALESCE(`{}`, 'Unknown user') AS `{}`,
         COALESCE(`{}`, 0) AS `{}`,
         COALESCE(`{}`, 0) AS `{}`
         FROM `{}`
         WHERE `{}` = ?
         ORDER BY {} DESC, {} DESC
         LIMIT 9",
        LEVELS_TABLE[&LevelsSchema::UserId],
        LEVELS_TABLE[&LevelsSchema::UserId],
        //
        LEVELS_TABLE[&LevelsSchema::ExperiencePoints],
        LEVELS_TABLE[&LevelsSchema::ExperiencePoints],
        //
        LEVELS_TABLE[&LevelsSchema::Level],
        LEVELS_TABLE[&LevelsSchema::Level],
        //
        DATABASE_USERS,
        LEVELS_TABLE[&LevelsSchema::GuildId],
        LEVELS_TABLE[&LevelsSchema::Level],
        LEVELS_TABLE[&LevelsSchema::ExperiencePoints],
    );

    pub static ref UPDATE_USER_LEVEL_QUERY: String = format!(
        "UPDATE `{}`
         SET `{}` = ?, `{}` = ?
         WHERE `{}` = ? AND `{}` = ?",
        DATABASE_USERS,
        //
        LEVELS_TABLE[&LevelsSchema::ExperiencePoints],
        LEVELS_TABLE[&LevelsSchema::Level],
        //
        LEVELS_TABLE[&LevelsSchema::UserId],
        LEVELS_TABLE[&LevelsSchema::GuildId],
    );

    // NOTE: Don't touch this.
    pub static ref FETCH_USER_LEVEL_AND_RANK_QUERY: String = format!(
        "SELECT {0}.*,
             (SELECT COUNT(*)
                 FROM {1} AS {2}
                 WHERE {2}.{3} = {0}.{3}
                     AND ({2}.{4} > {0}.{4} OR
                         ({2}.{4} = {0}.{4} AND {2}.{5} >= {0}.{5}))
             ) AS {7}
         FROM {1} AS {0}
         WHERE {0}.{6} = ? AND {0}.{3} = ?
         ORDER BY {4} DESC, {5} DESC",
        "us",
        DATABASE_USERS,
        "inner_u",
        LEVELS_TABLE[&LevelsSchema::GuildId],
        LEVELS_TABLE[&LevelsSchema::Level],
        LEVELS_TABLE[&LevelsSchema::ExperiencePoints],
        LEVELS_TABLE[&LevelsSchema::UserId],
        LEVELS_TABLE[&LevelsSchema::RankSelector]
    );

    #[derive(Debug)]
    pub static ref MENTIONS_TABLE: HashMap<MentionsSchema, &'static str> = {
        use crate::enums::schemas::MentionsSchema as DbSch;

        HashMap::from([
            (DbSch::Mentions, "mentions"),
        ])
    };
}
