use std::borrow::Cow;

use crate::data::bot_data::EMOJIS_AND_EMBEDS_REGEX;

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

/// Removes the emojis from a string.
///
/// ```rust
/// use serenity_discord_bot::utils::string_manipulation::remove_emojis_and_embeds_from_str;
///
/// assert_eq!(
///     remove_emojis_and_embeds_from_str(":hutao:"),
///     ""
/// );
/// assert_eq!(
///     remove_emojis_and_embeds_from_str(":hutao"),
///     ":hutao"
/// );
/// assert_eq!(
///     remove_emojis_and_embeds_from_str(
///         "Some longer example : messsage hutao: :hutao:"
///     ),
///     "Some longer example : messsage hutao: "
/// );
/// ```
pub fn remove_emojis_and_embeds_from_str(whole_str: &str) -> Cow<'_, str> {
    EMOJIS_AND_EMBEDS_REGEX.replace_all(whole_str, "")
}
