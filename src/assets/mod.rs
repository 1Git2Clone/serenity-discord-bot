use serenity_discord_bot_derive::Asset;

use crate::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, IterateVariants, Asset)]
pub enum Assets {
    #[filename = "hu_boom.jpg"]
    HuBoom,
}
