use crate::prelude::*;

pub const DEFAULT_XP: i32 = 0;
pub const DEFAULT_LEVEL: i32 = 1;

pub const MIN_XP: i32 = 5;
pub const MAX_XP: i32 = 15;
pub const XP_RANGE: std::ops::RangeInclusive<i32> = MIN_XP..=MAX_XP;

pub const XP_COOLDOWN_NUMBER_SECS: i64 = 60;
pub const BOT_PREFIXES: [&str; 2] = ["hu", "ht"];

pub const VALID_MENTION_COUNT_PATTERNS: [&str; 2] = ["hutao", "hu tao"];

pub static BOT_TOKEN: LazyLock<String> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "If anything fails here, it should fail."
    )]
    std::env::var("BOT_TOKEN").expect("Expected a token in the dotenv file.")
});

pub static START_TIME: LazyLock<std::time::Instant> = LazyLock::new(std::time::Instant::now);

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
pub static EMOJIS_AND_EMBEDS_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    #[allow(
        clippy::unwrap_used,
        reason = "If anything fails here, it should fail."
    )]
    Regex::new(concat!(
        "(?<emoji>",
        r":[^:\s]*:",
        ")|(?<embed_emoji>",
        r"\[[^\[\]]*\]\([^()]*\)",
        ")",
    ))
    .unwrap()
});
