#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DatabaseSchema {
    UserId,
    GuildId,
    Level,
    ExperiencePoints,
    LastQueryTimestamp,
}
