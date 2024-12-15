use std::collections::HashMap;

use crate::enums::schemas::LevelsSchema;

/// Set the leveling condition and return the updated level with reset xp if true.
pub async fn update_level(experience: i32, level: i32) -> HashMap<LevelsSchema, i32> {
    use crate::enums::schemas::LevelsSchema as DbSch;
    let update_level = if experience >= level * 100 {
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
