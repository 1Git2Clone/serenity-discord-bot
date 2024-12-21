use std::borrow::Cow;

use crate::data::{
    bot_data::{BOT_PREFIXES, EMOJIS_AND_EMBEDS_REGEX},
    command_data::Error,
};
use poise::serenity_prelude as serenity;

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

pub fn levenshtein_core<'a>(msg: &'a str, commands: &'a [String]) -> LevenshteinCommandData<'a> {
    let lower = msg.to_lowercase();
    let mut data = LevenshteinCommandData::new();
    for prefix in BOT_PREFIXES.iter() {
        if lower.starts_with(prefix) {
            data.prefix = prefix.as_str();
            break;
        }
    }
    if data.prefix.is_empty() {
        return data;
    }
    for command in commands {
        // The message is indeed a valid command.
        let cmd = format!("{}{}", data.prefix, command);
        if cmd == lower {
            return data;
        }
        if strsim::levenshtein(&cmd, &lower) <= 2 {
            data.command_matches.push(cmd);
        }
    }
    data
}

/// Check for typos in msg commands.
pub async fn levenshtein_cmd(
    ctx: &serenity::Context,
    msg: &serenity::Message,
    commands: &[String],
) -> Result<(), Error> {
    let levenshtein_results = levenshtein_core(&msg.content, commands);
    if levenshtein_results.command_matches.is_empty() || levenshtein_results.prefix.is_empty() {
        return Ok(());
    }

    let formatted_command_list = {
        let mut tmp = String::new();
        for c in levenshtein_results.command_matches {
            tmp.push_str(format!("- `{c}`\n").as_str());
        }
        tmp
    };
    let reply = format!(
        "Message starts with the bot prefix: `{}`",
        levenshtein_results.prefix
    ) + " "
        + &format!(
            "but it's not a valid command. Perhaps you meant one of the following:\n{}",
            formatted_command_list
        );

    msg.reply(ctx, reply).await?;

    Ok(())
}
