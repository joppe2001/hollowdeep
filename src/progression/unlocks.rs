//! Meta-progression unlocks
//!
//! Persistent unlocks that carry between runs.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Types of unlockable content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnlockType {
    /// A new skill becomes available at shrines
    Skill(u32),
    /// A new item can spawn in the world
    Item(u32),
    /// A new enemy type can spawn
    Enemy(u32),
    /// A floor/biome is accessible
    Floor(u32),
    /// A special game mode
    GameMode(u32),
}

/// Player's unlock progress
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Unlocks {
    /// Set of all unlocked content
    pub unlocked: HashSet<UnlockType>,
    /// Highest floor reached
    pub highest_floor: u32,
    /// Total enemies killed (across all runs)
    pub total_kills: u32,
    /// Total gold collected (across all runs)
    pub total_gold: u32,
    /// Number of completed runs
    pub completed_runs: u32,
}

impl Unlocks {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if something is unlocked
    pub fn is_unlocked(&self, unlock: UnlockType) -> bool {
        self.unlocked.contains(&unlock)
    }

    /// Unlock something
    pub fn unlock(&mut self, unlock: UnlockType) -> bool {
        self.unlocked.insert(unlock)
    }

    /// Update stats after a run
    pub fn record_run(&mut self, floor: u32, kills: u32, gold: u32, won: bool) {
        if floor > self.highest_floor {
            self.highest_floor = floor;
        }
        self.total_kills += kills;
        self.total_gold += gold;
        if won {
            self.completed_runs += 1;
        }
    }
}

/// Achievements that can trigger unlocks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Achievement {
    /// Reached a specific floor for the first time
    ReachedFloor(u32),
    /// Killed a specific boss
    KilledBoss(u32),
    /// Found a legendary item
    FoundLegendary,
    /// Completed a run
    CompletedRun,
    /// Killed 100 enemies total
    KillCount100,
    /// Collected 1000 gold total
    GoldCount1000,
}

/// Check and apply achievements
pub fn check_achievements(unlocks: &mut Unlocks) -> Vec<String> {
    let mut messages = Vec::new();

    // Check kill count achievements
    if unlocks.total_kills >= 100 && !unlocks.is_unlocked(UnlockType::Skill(3)) {
        unlocks.unlock(UnlockType::Skill(3)); // Unlock Envenom skill
        messages.push("Achievement: 100 kills! Envenom skill unlocked.".to_string());
    }

    // Check gold achievements
    if unlocks.total_gold >= 1000 && !unlocks.is_unlocked(UnlockType::Skill(5)) {
        unlocks.unlock(UnlockType::Skill(5)); // Unlock Whirlwind skill
        messages.push("Achievement: 1000 gold! Whirlwind skill unlocked.".to_string());
    }

    // Check floor achievements
    if unlocks.highest_floor >= 5 && !unlocks.is_unlocked(UnlockType::Floor(6)) {
        unlocks.unlock(UnlockType::Floor(6));
        messages.push("Achievement: Reached floor 5! Bleeding Crypts unlocked.".to_string());
    }

    messages
}
