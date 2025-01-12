use crate::prelude::*;

pub const DEFAULT_XP: i64 = 0;
pub const DEFAULT_LEVEL: i64 = 1;

pub const MIN_XP: u32 = 5;
pub const MAX_XP: u32 = 15;
pub const XP_RANGE: std::ops::RangeInclusive<u32> = MIN_XP..=MAX_XP;

lazy_static! {
    pub static ref BOT_TOKEN: String = {
        #[allow(
            clippy::expect_used,
            reason = "If anything fails here, it should fail."
        )]
        std::env::var("BOT_TOKEN").expect("Expected a token in the dotenv file.")
    };
    pub static ref START_TIME: std::time::Instant = std::time::Instant::now();

    pub static ref XP_COOLDOWN_NUMBER_SECS: i64 = 60;
    pub static ref BOT_PREFIXES: [&'static str; 2] = ["hu", "ht"];

    pub static ref VALID_MENTION_COUNT_PATTERNS: [&'static str; 2] = ["hutao", "hu tao"];


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
    pub static ref EMOJIS_AND_EMBEDS_REGEX: Regex = {
        #[allow(
            clippy::unwrap_used,
            reason = "If anything fails here, it should fail."
        )]
        Regex::new(
            concat!(
                "(?<emoji>",
                    r":[^:\s]*:",
                ")|(?<embed_emoji>",
                    r"\[[^\[\]]*\]\([^()]*\)",
                ")",
            )
        ).unwrap()
    };


}
