use serenity_discord_bot_derive::Asset;

use crate::prelude::*;

/// Due to GitHub gracefully handling URLs with `./` paths, you can set the `src_path` with
/// `./<file>` which means your text editor can make use of file path autocompletion.
///
/// ```rust
/// use serenity_discord_bot_derive::Asset;
///
/// #[derive(Asset)]
/// #[base_url = "foo"]
/// pub enum Assets {
///     #[src_path = "bar.ext"]
///     Bar,
///     #[src_path = "baz.ext"]
///     Baz,
/// }
///
/// assert_eq!(Assets::Bar.to_string(), "foo/bar.ext");
/// assert_eq!(Assets::Baz.to_string(), "foo/baz.ext");
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, IterateVariants, Asset)]
#[base_url = "https://raw.githubusercontent.com/1Git2Clone/serenity-discord-bot/main/src/assets/"]
pub enum Assets {
    #[src_path = "./hu_boom.jpg"]
    HuBoom,
}
