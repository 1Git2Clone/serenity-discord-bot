use crate::prelude::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LevelsSchema {
    UserId,
    GuildId,
    Level,
    ExperiencePoints,
    LastQueryTimestamp,
    RankSelector,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MentionsSchema {
    Mentions,
}
