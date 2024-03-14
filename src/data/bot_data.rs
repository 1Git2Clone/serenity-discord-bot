use crate::enums::{
    command_enums::CmdPrefixes,
    command_enums::CmdPrefixes::*,
    schemas::{DatabaseSchema, DatabaseSchema::*},
};
use std::collections::HashMap;

use lazy_static::lazy_static;

lazy_static! {
    #[derive(Debug)] // So it can be printed in main.rs (you shouldn't do it tho)
    pub(crate) static ref BOT_TOKEN: String =
        std::env::var("BOT_TOKEN").expect("Expected a token in the dotenv file.");
    pub(crate) static ref START_TIME: std::time::Instant = std::time::Instant::now();
    pub(crate) static ref DATABASE_FILENAME: &'static str = "database/bot_database.sqlite";
    #[derive(Debug)]
    pub(crate) static ref DATABASE_USERS: &'static str = "user_stats";
    pub(crate) static ref DATABASE_COLUMNS: HashMap<DatabaseSchema, &'static str> = HashMap::from([
        (UserId, "user_id"),
        (GuildId, "guild_id"),
        (ExperiencePoints, "experience_points"),
        (Level, "level"),
        (LastQueryTimestamp, "last_query_timestamp")
    ]);
    pub(crate) static ref XP_COOLDOWN_NUMBER_SECS: i64 = 60;
    pub(crate) static ref BOT_PREFIXES: HashMap<CmdPrefixes, &'static str> = HashMap::from([
        (Hu, "hu"),
        (HT, "ht"),
        (ExclaimationMark, "!")
    ]);
}
