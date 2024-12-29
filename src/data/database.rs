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
            (DbSch::LastQueryTimestamp, "last_query_timestamp")
        ])
    };
    #[derive(Debug)]
    pub static ref MENTIONS_TABLE: HashMap<MentionsSchema, &'static str> = {
        use crate::enums::schemas::MentionsSchema as DbSch;

        HashMap::from([
            (DbSch::Mentions, "mentions"),
        ])
    };
}
