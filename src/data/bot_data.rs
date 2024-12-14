use crate::{
    enums::schemas::DatabaseSchema, utils::string_manipulation::upper_lowercase_permutations,
};
use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;

pub const DATABASE_FILENAME: &str = "database/bot_database.sqlite";
pub const DATABASE_USERS: &str = "user_stats";

pub const DEFAULT_XP: i64 = 0;
pub const DEFAULT_LEVEL: i64 = 1;

lazy_static! {
    #[derive(Debug)] // So it can be printed in main.rs (you shouldn't do it tho)
    pub(crate) static ref BOT_TOKEN: String =
        std::env::var("BOT_TOKEN").expect("Expected a token in the dotenv file.");
    pub(crate) static ref START_TIME: std::time::Instant = std::time::Instant::now();
    #[derive(Debug)]
    pub(crate) static ref DATABASE_COLUMNS: HashMap<DatabaseSchema, &'static str> = {
        use crate::enums::schemas::DatabaseSchema as DbSch;

        HashMap::from([
            (DbSch::UserId, "user_id"),
            (DbSch::GuildId, "guild_id"),
            (DbSch::ExperiencePoints, "experience_points"),
            (DbSch::Level, "level"),
            (DbSch::LastQueryTimestamp, "last_query_timestamp")
        ])
    };
    pub(crate) static ref XP_COOLDOWN_NUMBER_SECS: i64 = 60;
    pub(crate) static ref BOT_PREFIXES: Vec<String> = {
            let mut temp = vec![];
            temp.append(&mut upper_lowercase_permutations("hu"));
            temp.append(&mut upper_lowercase_permutations("ht"));

            temp
    };

    // https://regex101.com/r/aX8vec/3
    pub(crate) static ref EMOJIS_AND_IMAGE_EMOJIS_REGEX: Regex = Regex::new(r"((:|\[)([a-zA-Z0-9_]+)(:|\]))|\([^()]*\)").unwrap();
}
