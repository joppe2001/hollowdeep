//! Experience and leveling
//!
//! XP calculations, leveling formulas, and progression helpers.

/// Calculate XP needed to reach a specific level
pub fn xp_for_level(level: u32) -> u32 {
    if level <= 1 {
        0
    } else {
        // Base 100 XP for level 2, +50 per level after
        100 + (level - 2) * 50
    }
}

/// Calculate total XP needed from level 1 to reach a given level
pub fn total_xp_for_level(level: u32) -> u32 {
    (1..level).map(|l| xp_for_level(l + 1)).sum()
}

/// Get a title/rank based on level
pub fn level_title(level: u32) -> &'static str {
    match level {
        1..=2 => "Novice",
        3..=4 => "Apprentice",
        5..=7 => "Journeyman",
        8..=10 => "Adept",
        11..=14 => "Expert",
        15..=18 => "Master",
        19..=24 => "Grandmaster",
        _ => "Legend",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_for_level() {
        assert_eq!(xp_for_level(1), 0);
        assert_eq!(xp_for_level(2), 100); // Need 100 to go from 1 -> 2
        assert_eq!(xp_for_level(3), 150); // Need 150 to go from 2 -> 3
        assert_eq!(xp_for_level(4), 200);
    }

    #[test]
    fn test_level_title() {
        assert_eq!(level_title(1), "Novice");
        assert_eq!(level_title(5), "Journeyman");
        assert_eq!(level_title(25), "Legend");
    }
}
