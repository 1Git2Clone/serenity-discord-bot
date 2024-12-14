use regex::Regex;

pub fn upper_lowercase_permutations(data: &str) -> Vec<String> {
    if data.is_empty() {
        return vec![String::new()];
    }

    let first = data.chars().next().unwrap();
    let rest = &data[1..];

    let permutations = upper_lowercase_permutations(rest);

    let mut result: Vec<String> = Vec::new();

    for perm in permutations {
        result.push(format!("{}{}", first.to_ascii_lowercase(), perm));
        result.push(format!("{}{}", first.to_ascii_uppercase(), perm));
    }

    result
}

/// Checks if the lookup_pattern is not in a discord emoji (wrapped around `:`).
///
///
/// ```rust
/// use serenity_discord_bot::utils::string_manipulation::non_emoji_match;
///
/// assert_eq!(non_emoji_match(":hutao:", "hutao"), false);
/// assert_eq!(non_emoji_match(":hutao", "hutao"), true);
/// assert_eq!(non_emoji_match("hutao:", "hutao"), true);
/// assert_eq!(non_emoji_match("Some longer example : messsage hutao:", "hutao"), true);
/// assert_eq!(non_emoji_match(":message hutao:", "hutao"), true);
/// assert_eq!(non_emoji_match(":hutaoemoji:", "hutao"), false);
/// assert_eq!(non_emoji_match(":htaoemoji:", "hutao"), false);
/// ```
pub fn non_emoji_match(whole_str: &str, lookup_pattern: &str) -> bool {
    // https://regex101.com/r/aX8vec/1
    let re = Regex::new(r"(^|\s):([a-zA-Z0-9_]+):(\s|$)").unwrap();
    let trimmed_emojis = re.replace_all(whole_str, "");
    trimmed_emojis.contains(lookup_pattern)
}
