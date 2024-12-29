use serenity_discord_bot_derive::DiscordEmoji;

use crate::prelude::*;

/// Discord emojis are sent like this:
///
/// `<EmojiName:EmojiId>`
///
/// This macro aims to simplify the process by writing the display implementor like this:
///
/// ```rust
/// use serenity_discord_bot_derive::DiscordEmoji;
///
/// #[derive(DiscordEmoji)]
/// pub enum Emojis {
///     #[emoji_id = "123456789"]
///     EmojiOne,
///     #[emoji_id = "987654321"]
///     EmojiTwo,
/// }
///
/// assert_eq!(
///     Emojis::EmojiOne.to_string(),
///     "<:EmojiOne:123456789>".to_string()
/// );
/// assert_eq!(
///     Emojis::EmojiTwo.to_string(),
///     "<:EmojiTwo:987654321>".to_string()
/// );
/// assert_eq!(
///     Emojis::EmojiOne.get_variant_str(),
///     "EmojiOne"
/// );
/// assert_eq!(
///     Emojis::EmojiTwo.get_variant_str(),
///     "EmojiTwo"
/// );
/// assert_eq!(
///     Emojis::EmojiOne.get_id(),
///     "123456789"
/// );
/// assert_eq!(
///     Emojis::EmojiTwo.get_id(),
///     "987654321"
/// );
/// ```
///
/// ---
///
/// NOTE: This allows non-PascalCase because the emoji itself could have a non-PascalCase name. I'd
/// still try to have them all be PascalCase though.
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, IterateVariants, DiscordEmoji)]
pub enum Emojis {
    #[emoji_id = "1317920658021290097"]
    HuTaoHeh,
}
