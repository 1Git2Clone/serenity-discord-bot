pub struct LevelStats {
    pub updated_level: i32,
    pub updated_experience: i32,
}

pub fn calculate_xp_to_level_up(level: i32) -> i32 {
    level * 100
}

/// Set the leveling condition and return the updated level with reset xp if true.
#[tracing::instrument(
    fields(
        category = "discord_command_utility",
        eperience = %experience,
        level = %level,
        level_up = %(experience >= calculate_xp_to_level_up(level))
    )
)]
pub async fn update_level(experience: i32, level: i32) -> LevelStats {
    let update_level = if experience >= calculate_xp_to_level_up(level) {
        level + 1
    } else {
        level
    };

    let update_experience = if update_level == level { experience } else { 0 };

    LevelStats {
        updated_level: update_level,
        updated_experience: update_experience,
    }
}
