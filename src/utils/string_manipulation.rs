use crate::prelude::*;

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

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct LevenshteinCommandData<'a> {
    pub prefix: &'a str,
    pub command_matches: Vec<String>,
}
impl LevenshteinCommandData<'_> {
    pub fn new() -> Self {
        Self::default()
    }
}

#[tracing::instrument(
    skip(msg, commands),
    fields(
        category = "levenshtein",
        msg = %msg,
        commands = ?commands,
    )
)]
pub fn levenshtein_core<'a>(msg: &'a str, commands: &'a [String]) -> LevenshteinCommandData<'a> {
    let lower = msg.to_lowercase();
    let mut data = LevenshteinCommandData::new();
    for prefix in BOT_PREFIXES.iter() {
        if lower.starts_with(prefix) {
            data.prefix = prefix;
            break;
        }
    }
    if data.prefix.is_empty() {
        return data;
    }
    data.command_matches = {
        let mut tmp = Vec::new();
        for command in commands {
            // The message is indeed a valid command.
            let cmd = format!("{}{}", data.prefix, command);
            if cmd == lower {
                return data;
            }
            if strsim::levenshtein(&cmd, &lower) <= 1 {
                tmp.push(cmd);
            }
        }
        tmp
    };
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_emojis_strips_colon_wrapped_word() {
        assert_eq!(remove_emojis_and_embeds_from_str(":hutao:"), "");
    }

    #[test]
    fn remove_emojis_preserves_unclosed_colon() {
        assert_eq!(remove_emojis_and_embeds_from_str(":hutao"), ":hutao");
    }

    #[test]
    fn remove_emojis_strips_only_emoji_portion() {
        assert_eq!(
            remove_emojis_and_embeds_from_str("Some longer example : messsage hutao: :hutao:"),
            "Some longer example : messsage hutao: "
        );
    }

    #[test]
    fn remove_emojis_strips_markdown_link() {
        assert_eq!(remove_emojis_and_embeds_from_str("[text](url)"), "");
        assert_eq!(
            remove_emojis_and_embeds_from_str("hello [text](url) world"),
            "hello  world"
        );
    }

    #[test]
    fn levenshtein_command_data_new_is_default() {
        let data = LevenshteinCommandData::new();
        assert_eq!(data.prefix, "");
        assert!(data.command_matches.is_empty());
    }

    #[test]
    fn levenshtein_core_no_prefix_returns_empty() {
        let cmds = vec!["hello".to_string()];
        let result = levenshtein_core("world", &cmds);
        assert!(result.prefix.is_empty());
        assert!(result.command_matches.is_empty());
    }

    #[test]
    fn levenshtein_core_exact_match_no_suggestions() {
        let cmds = vec!["hello".to_string()];
        let result = levenshtein_core("huhello", &cmds);
        assert_eq!(result.prefix, "hu");
        assert!(result.command_matches.is_empty());
    }

    #[test]
    fn levenshtein_core_one_edit_away_returns_suggestion() {
        // "huhelln" vs command "huhello" — differ only in last character (o→n)
        let cmds = vec!["hello".to_string()];
        let result = levenshtein_core("huhelln", &cmds);
        assert_eq!(result.prefix, "hu");
        assert_eq!(result.command_matches, vec!["huhello".to_string()]);
    }

    #[test]
    fn levenshtein_core_far_from_any_command_no_suggestions() {
        let cmds = vec!["hello".to_string()];
        let result = levenshtein_core("huxyz", &cmds);
        assert_eq!(result.prefix, "hu");
        assert!(result.command_matches.is_empty());
    }

    #[test]
    fn levenshtein_core_ht_prefix_recognized() {
        let cmds = vec!["ping".to_string()];
        let result = levenshtein_core("htping", &cmds);
        assert_eq!(result.prefix, "ht");
        assert!(result.command_matches.is_empty());
    }

    #[test]
    fn levenshtein_core_empty_commands_slice() {
        let result = levenshtein_core("huhello", &[]);
        assert_eq!(result.prefix, "hu");
        assert!(result.command_matches.is_empty());
    }
}
