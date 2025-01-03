use crate::prelude::*;

pub const DATABASE_FILENAME: &str = "database/bot_database.sqlite";
pub const DATABASE_USERS: &str = "user_stats";
pub const MENTIONS_TABLE_NAME: &str = "bot_mentions";

lazy_static! {
    pub static ref ADD_USER_LEVEL_QUERY: String = format!(
        "INSERT INTO `{}` (`{}`, `{}`, `{}`, `{}`)
         VALUES (?, ?, ?, ?)",
        DATABASE_USERS,
        LevelsSchema::UserId.as_str(),
        LevelsSchema::GuildId.as_str(),
        LevelsSchema::ExperiencePoints.as_str(),
        LevelsSchema::Level.as_str(),
    );

    pub static ref FETCH_USER_LEVEL_QUERY: String = format!(
        "SELECT `{}`, `{}`, `{}`
         FROM `{}`
         WHERE `{}` = ? AND `{}` = ?",
        LevelsSchema::UserId.as_str(),
        LevelsSchema::ExperiencePoints.as_str(),
        LevelsSchema::Level.as_str(),
        //
        DATABASE_USERS,
        //
        LevelsSchema::UserId.as_str(),
        LevelsSchema::GuildId.as_str()
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
        LevelsSchema::UserId.as_str(),
        LevelsSchema::UserId.as_str(),
        //
        LevelsSchema::ExperiencePoints.as_str(),
        LevelsSchema::ExperiencePoints.as_str(),
        //
        LevelsSchema::Level.as_str(),
        LevelsSchema::Level.as_str(),
        //
        DATABASE_USERS,
        LevelsSchema::GuildId.as_str(),
        LevelsSchema::Level.as_str(),
        LevelsSchema::ExperiencePoints.as_str(),
    );

    pub static ref UPDATE_USER_LEVEL_QUERY: String = format!(
        "UPDATE `{}`
         SET `{}` = ?, `{}` = ?
         WHERE `{}` = ? AND `{}` = ?",
        DATABASE_USERS,
        //
        LevelsSchema::ExperiencePoints.as_str(),
        LevelsSchema::Level.as_str(),
        //
        LevelsSchema::UserId.as_str(),
        LevelsSchema::GuildId.as_str(),
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
        LevelsSchema::GuildId.as_str(),
        LevelsSchema::Level.as_str(),
        LevelsSchema::ExperiencePoints.as_str(),
        LevelsSchema::UserId.as_str(),
        LevelsSchema::Rank.as_str()
    );
}
