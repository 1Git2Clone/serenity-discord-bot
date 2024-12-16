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

    /// # 2 groups
    ///
    /// *(matched via bitwise or `|`)*
    ///
    /// 1. emoji
    /// - `:UTF-8:`
    ///   - Exceptions for the `UTF-8` group:
    ///     - `:` & `<any-whitespace>` at both ends
    /// 2. embed_emoji
    /// - `[UTF-8](<any-pattern/nothing>)`
    ///   - Exceptions for the `UTF-8` group:
    ///     - `<any-whitespace>` at both ends
    ///     - `[` at the start
    ///     - `]` at the end
    ///
    /// ---
    ///
    /// <https://regex101.com/r/Yi782B/2>
    pub(crate) static ref EMOJIS_AND_EMBEDS_REGEX: Regex = Regex::new(
        concat!(
            "(?<emoji>",
                r":[^:\s]*:",
            ")|(?<embed_emoji>",
                r"\[[^\[\]\s]*\]\([^()]*\)",
            ")",
        )
    ).unwrap();

}
