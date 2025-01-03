use serenity_discord_bot_derive::DatabaseEnum;

use crate::prelude::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, DatabaseEnum)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LevelsSchema {
    UserId,
    GuildId,
    Level,
    ExperiencePoints,
    LastQueryTimestamp,
    Rank,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, DatabaseEnum)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MentionsSchema {
    Mentions,
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn correct_levels_schema() {
        assert_eq!(LevelsSchema::UserId.as_str(), "user_id");
        assert_eq!(LevelsSchema::GuildId.as_str(), "guild_id");
        assert_eq!(LevelsSchema::Level.as_str(), "level");
        assert_eq!(LevelsSchema::ExperiencePoints.as_str(), "experience_points");
        assert_eq!(
            LevelsSchema::LastQueryTimestamp.as_str(),
            "last_query_timestamp"
        );
        assert_eq!(LevelsSchema::Rank.as_str(), "rank");
    }

    #[test]
    fn correct_mentions_schema() {
        assert_eq!(MentionsSchema::Mentions.as_str(), "mentions");
    }
}
