#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The EmbedType is the Value from the HashMap containing a vector of all the URL links as strings
/// that correspond to the EmbedType variant.
#[derive(Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EmbedType {
    TieUp,
    Pat,
    Hug,
    Kiss,
    Slap,
    Punch,
    Bonk,
    RyanGoslingDrive, // heh...
    Nom,
    Kill,
    Kick,
    Bury,
    SelfBury,
    Chair, // you lack motivation.
    Peek,
}
