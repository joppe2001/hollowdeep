//! Difficulty settings and floor-based scaling
//!
//! Provides both global difficulty settings and per-floor scaling.

use serde::{Deserialize, Serialize};

/// Game difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
    Nightmare,
}

impl Difficulty {
    /// Damage multiplier for enemies
    pub fn enemy_damage_mult(&self) -> f32 {
        match self {
            Difficulty::Easy => 0.7,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.3,
            Difficulty::Nightmare => 1.6,
        }
    }

    /// Health multiplier for enemies
    pub fn enemy_health_mult(&self) -> f32 {
        match self {
            Difficulty::Easy => 0.8,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.25,
            Difficulty::Nightmare => 1.5,
        }
    }

    /// XP multiplier for rewards
    pub fn xp_mult(&self) -> f32 {
        match self {
            Difficulty::Easy => 0.8,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.2,
            Difficulty::Nightmare => 1.5,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
            Difficulty::Nightmare => "Nightmare",
        }
    }
}

// =============================================================================
// Floor-Based Scaling
// =============================================================================

/// Floor scaling configuration
#[derive(Clone)]
pub struct FloorScaling {
    /// Current floor number (1-indexed)
    pub floor: u32,
    /// Global difficulty setting
    pub difficulty: Difficulty,
    /// Elite bonus multiplier (for elite zones)
    pub elite_mult: f32,
}

impl FloorScaling {
    pub fn new(floor: u32, difficulty: Difficulty) -> Self {
        Self { floor, difficulty, elite_mult: 1.0 }
    }

    /// Create scaling for elite zone enemies (50% stronger, 100% more XP)
    pub fn elite_scaled(floor: u32, difficulty: Difficulty) -> Self {
        Self { floor, difficulty, elite_mult: 1.5 }
    }

    /// Get the floor scaling factor (1.0 at floor 1, increases per floor)
    /// Each floor adds ~5% to enemy strength
    fn floor_factor(&self) -> f32 {
        1.0 + (self.floor.saturating_sub(1) as f32 * 0.05)
    }

    /// Calculate scaled enemy HP
    /// Base formula: base_hp * floor_factor * difficulty_mult * elite_mult
    pub fn scale_enemy_hp(&self, base_hp: i32) -> i32 {
        let scaled = base_hp as f32 * self.floor_factor() * self.difficulty.enemy_health_mult() * self.elite_mult;
        scaled.round() as i32
    }

    /// Calculate scaled enemy damage (via stats)
    /// Returns a stat multiplier to apply to STR/INT
    pub fn stat_multiplier(&self) -> f32 {
        self.floor_factor() * self.difficulty.enemy_damage_mult() * self.elite_mult
    }

    /// Calculate scaled XP reward (elite zones give double XP)
    pub fn scale_xp(&self, base_xp: u32) -> u32 {
        let elite_xp_mult = if self.elite_mult > 1.0 { 2.0 } else { 1.0 };
        let scaled = base_xp as f32 * self.floor_factor() * self.difficulty.xp_mult() * elite_xp_mult;
        scaled.round() as u32
    }

    /// Scale a stat value (STR, DEX, etc)
    pub fn scale_stat(&self, base_stat: i32) -> i32 {
        let scaled = base_stat as f32 * self.stat_multiplier();
        scaled.round() as i32
    }

    /// Get enemy count multiplier for this floor
    /// Returns (min_add, max_add) to add to base enemy count
    pub fn enemy_count_bonus(&self) -> (usize, usize) {
        let base = (self.floor / 5) as usize;
        let diff_bonus = match self.difficulty {
            Difficulty::Easy => 0,
            Difficulty::Normal => 0,
            Difficulty::Hard => 1,
            Difficulty::Nightmare => 2,
        };
        (base, base + diff_bonus)
    }

    /// Check if this floor should have an elite enemy guaranteed
    pub fn has_guaranteed_elite(&self) -> bool {
        // Elite on every 5th floor, or always on Nightmare
        self.floor % 5 == 0 || self.difficulty == Difficulty::Nightmare
    }

    /// Get item drop quality bonus (affects rarity chances)
    /// Higher value = better drops
    pub fn loot_quality_bonus(&self) -> f32 {
        let floor_bonus = (self.floor as f32 - 1.0) * 0.02; // +2% per floor
        let diff_bonus = match self.difficulty {
            Difficulty::Easy => -0.1,
            Difficulty::Normal => 0.0,
            Difficulty::Hard => 0.1,
            Difficulty::Nightmare => 0.2,
        };
        floor_bonus + diff_bonus
    }
}

/// Convenience function for simple floor scaling
pub fn floor_hp_scale(base_hp: i32, floor: u32) -> i32 {
    FloorScaling::new(floor, Difficulty::Normal).scale_enemy_hp(base_hp)
}

/// Convenience function for simple floor XP scaling
pub fn floor_xp_scale(base_xp: u32, floor: u32) -> u32 {
    FloorScaling::new(floor, Difficulty::Normal).scale_xp(base_xp)
}

/// Convenience function for simple stat scaling
pub fn floor_stat_scale(base_stat: i32, floor: u32) -> i32 {
    FloorScaling::new(floor, Difficulty::Normal).scale_stat(base_stat)
}
