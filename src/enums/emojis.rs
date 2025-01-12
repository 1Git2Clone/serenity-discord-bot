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
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, IterateVariants, DiscordEmoji)]
pub enum Emojis {
    #[emoji_id = "1317920658021290097"]
    HuTaoHeh,
    #[emoji_id = "1327955257744953415"]
    HuTaoSmug,
    #[emoji_id = "1327955243962470451"]
    HuTaoHug,
    #[emoji_id = "1327957502092120074"]
    HuTaoJuice,
    #[emoji_id = "1327957849493737522"]
    HuTaoEvilLaugh,
    #[emoji_id = "1327958620872376381"]
    A_HuTaoNote,
    #[emoji_id = "1327959223237218344"]
    HuTaoHeheNote,
}
