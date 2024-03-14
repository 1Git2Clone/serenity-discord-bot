#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CmdPrefixes {
    Hu,
    HT,
    ExclaimationMark,
}

fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
#[test]
fn normal_types() {
    _is_normal::<EmbedType>();
    _is_normal::<CmdPrefixes>();
}
