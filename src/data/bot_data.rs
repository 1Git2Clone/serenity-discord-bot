use crate::utils::string_manipulation::upper_lowercase_permutations;

use lazy_static::lazy_static;
use regex::Regex;

pub const DEFAULT_XP: i64 = 0;
pub const DEFAULT_LEVEL: i64 = 1;

lazy_static! {
    #[derive(Debug)] // So it can be printed in main.rs (you shouldn't do it tho)
    pub(crate) static ref BOT_TOKEN: String =
        std::env::var("BOT_TOKEN").expect("Expected a token in the dotenv file.");
    pub(crate) static ref START_TIME: std::time::Instant = std::time::Instant::now();

    pub(crate) static ref XP_COOLDOWN_NUMBER_SECS: i64 = 60;
    pub(crate) static ref BOT_PREFIXES: Vec<String> = {
            let mut temp = vec![];
            temp.append(&mut upper_lowercase_permutations("hu"));
            temp.append(&mut upper_lowercase_permutations("ht"));

            temp
    };

    // https://regex101.com/r/aX8vec/5
    pub(crate) static ref EMOJIS_AND_EMBEDS_REGEX: Regex = Regex::new(r"(?<emoji>(:)([a-zA-Z0-9_]+)(:))|(?<embed>(\[)([a-zA-Z0-9_]+)(\])\([^()]*\))").unwrap();

}
