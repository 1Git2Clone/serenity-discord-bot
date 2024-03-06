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
    pub(crate) static ref DATABASE_COLUMNS: HashMap<&'static str, &'static str> = HashMap::from([
        ("user_id", "user_id"),
        ("guild_id", "guild_id"),
        ("experience_points", "experience_points"),
        ("level", "level"),
        ("last_query_timestamp", "last_query_timestamp")
    ]);
}
pub static BOT_PREFIX: &str = "!";
