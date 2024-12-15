/// Discord emojis are sent like this:
///
/// `<EmojiName:EmojiId>`
///
/// This macro aims to simplify the process by writing the display implementor like this:
///
/// ```rust
/// use serenity_discord_bot::display_emoji_impl;
///
/// pub enum Emojis {
///     EmojiOne,
///     EmojiTwo,
/// }
///
/// display_emoji_impl! {
///     Emojis {
///         EmojiOne => "123456789",
///         EmojiTwo => "987654321",
///     }
/// }
///
/// assert_eq!(Emojis::EmojiOne.to_string(), "<:EmojiOne:123456789>".to_string());
/// assert_eq!(Emojis::EmojiTwo.to_string(), "<:EmojiTwo:987654321>".to_string());
/// ```
#[macro_export]
macro_rules! display_emoji_impl {
    ($enum_name:ident { $($variant:ident => $id:expr),* $(,)? }) => {
        impl std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$variant => {
                            write!(
                                f,
                                concat!(
                                    "<:",
                                    stringify!($variant),
                                    ":",
                                    $id,
                                    ">"
                                )
                            )
                        }
                    )*
                }
            }
        }
    };
}

/// NOTE: This allows non-PascalCase because the emoji itself could have a non-PascalCase name. I'd
/// still try to have them all be PascalCase though.
#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub enum Emojis {
    HuTaoHeh,
}

display_emoji_impl! {
    Emojis {
        HuTaoHeh => "1317920658021290097",
    }
}
