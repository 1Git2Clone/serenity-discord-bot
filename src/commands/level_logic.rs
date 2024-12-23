use std::collections::HashMap;

use crate::enums::schemas::LevelsSchema;

pub fn calculate_xp_to_level_up(level: u32) -> u32 {
    level * 100
}

/// Set the leveling condition and return the updated level with reset xp if true.
pub async fn update_level(experience: u32, level: u32) -> HashMap<LevelsSchema, u32> {
    use crate::enums::schemas::LevelsSchema as DbSch;
    let update_level = if experience >= calculate_xp_to_level_up(level) {
        level + 1
    } else {
        level
    };

    let update_experience = if update_level == level { experience } else { 0 };

    HashMap::from([
        (DbSch::ExperiencePoints, update_experience),
        (DbSch::Level, update_level),
    ])
}
