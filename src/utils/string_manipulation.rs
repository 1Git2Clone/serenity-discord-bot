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
/// use serenity_discord_bot::utils::string_manipulation::is_in_emoji;
///
/// assert_eq!(is_in_emoji(":hutao:", "hutao"), Some(true));
/// assert_eq!(is_in_emoji(":hutao", "hutao"), Some(false));
/// assert_eq!(is_in_emoji("hutao:", "hutao"), Some(false));
/// assert_eq!(is_in_emoji("Some longer example : messsage hutao:", "hutao"), Some(false));
/// assert_eq!(is_in_emoji(":message hutao:", "hutao"), Some(false));
/// assert_eq!(is_in_emoji(":hutaoemoji:", "hutao"), Some(true));
/// assert_eq!(is_in_emoji(":htaoemoji:", "hutao"), None);
/// ```
pub fn is_in_emoji(whole_str: &str, lookup_pattern: &str) -> Option<bool> {
    whole_str.find(lookup_pattern).map(|pos| {
        let mut before = false;
        for char in whole_str.chars().take(pos) {
            match char {
                ' ' => before = false,
                ':' => before = true,
                _ => (),
            }
        }
        let mut after = false;
        for char in whole_str.chars().skip(pos) {
            match char {
                ' ' => break,
                ':' => after = true,
                _ => (),
            }
        }

        before && after
    })
}
