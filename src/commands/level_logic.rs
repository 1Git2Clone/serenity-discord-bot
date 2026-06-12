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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xp_to_level_up_scales_linearly() {
        assert_eq!(calculate_xp_to_level_up(0), 0);
        assert_eq!(calculate_xp_to_level_up(1), 100);
        assert_eq!(calculate_xp_to_level_up(5), 500);
        assert_eq!(calculate_xp_to_level_up(10), 1000);
    }

    #[tokio::test]
    async fn update_level_advances_when_xp_meets_threshold() {
        let stats = update_level(100, 1).await;
        assert_eq!(stats.updated_level, 2);
        assert_eq!(stats.updated_experience, 0);
    }

    #[tokio::test]
    async fn update_level_advances_when_xp_exceeds_threshold() {
        let stats = update_level(150, 1).await;
        assert_eq!(stats.updated_level, 2);
        assert_eq!(stats.updated_experience, 0);
    }

    #[tokio::test]
    async fn update_level_no_change_when_xp_below_threshold() {
        let stats = update_level(50, 1).await;
        assert_eq!(stats.updated_level, 1);
        assert_eq!(stats.updated_experience, 50);
    }

    #[tokio::test]
    async fn update_level_no_change_when_xp_is_zero() {
        let stats = update_level(0, 1).await;
        assert_eq!(stats.updated_level, 1);
        assert_eq!(stats.updated_experience, 0);
    }

    #[tokio::test]
    async fn update_level_zero_level_always_advances() {
        // calculate_xp_to_level_up(0) == 0, so any xp >= 0 causes a level-up
        let stats = update_level(0, 0).await;
        assert_eq!(stats.updated_level, 1);
        assert_eq!(stats.updated_experience, 0);
    }
}
