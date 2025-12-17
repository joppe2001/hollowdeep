//! Player profile and persistent progression
//!
//! Tracks unlocks, achievements, and statistics across runs.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Current profile version for compatibility
const PROFILE_VERSION: u32 = 1;

/// Persistent player profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    /// Version for compatibility checking
    pub version: u32,
    /// Player statistics
    pub stats: ProfileStats,
    /// Unlocked items (item IDs that can appear in loot pools)
    pub unlocked_items: HashSet<String>,
    /// Unlocked achievements
    pub achievements: HashSet<String>,
    /// Highest floor reached
    pub highest_floor: u32,
    /// Number of victories
    pub victories: u32,
    /// Settings preferences
    pub settings: ProfileSettings,
}

/// Profile statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileStats {
    /// Total runs attempted
    pub total_runs: u32,
    /// Total deaths
    pub total_deaths: u32,
    /// Total enemies killed
    pub enemies_killed: u32,
    /// Total gold collected
    pub gold_collected: u64,
    /// Total items found
    pub items_found: u32,
    /// Total floors descended
    pub floors_descended: u32,
    /// Bosses defeated
    pub bosses_defeated: u32,
    /// Total playtime in seconds
    pub playtime_seconds: u64,
}

/// Profile settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSettings {
    /// Message log verbosity (0 = minimal, 1 = normal, 2 = detailed)
    pub message_verbosity: u8,
    /// Show damage numbers
    pub show_damage_numbers: bool,
    /// Auto-pickup gold
    pub auto_pickup_gold: bool,
    /// Confirm before using shrines
    pub confirm_shrine_use: bool,
}

impl Default for ProfileSettings {
    fn default() -> Self {
        Self {
            message_verbosity: 1,
            show_damage_numbers: true,
            auto_pickup_gold: true,
            confirm_shrine_use: true,
        }
    }
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self {
            version: PROFILE_VERSION,
            stats: ProfileStats::default(),
            unlocked_items: HashSet::new(),
            achievements: HashSet::new(),
            highest_floor: 0,
            victories: 0,
            settings: ProfileSettings::default(),
        }
    }
}

impl PlayerProfile {
    /// Create a new profile
    pub fn new() -> Self {
        Self::default()
    }

    /// Record the start of a new run
    pub fn record_run_start(&mut self) {
        self.stats.total_runs += 1;
    }

    /// Record a death
    pub fn record_death(&mut self, floor: u32) {
        self.stats.total_deaths += 1;
        if floor > self.highest_floor {
            self.highest_floor = floor;
        }
    }

    /// Record a victory
    pub fn record_victory(&mut self) {
        self.victories += 1;
        self.check_victory_achievements();
    }

    /// Record floor descent
    pub fn record_floor_descent(&mut self, floor: u32) {
        self.stats.floors_descended += 1;
        if floor > self.highest_floor {
            self.highest_floor = floor;
        }
        self.check_floor_achievements(floor);
    }

    /// Record an enemy kill
    pub fn record_enemy_kill(&mut self, is_boss: bool) {
        self.stats.enemies_killed += 1;
        if is_boss {
            self.stats.bosses_defeated += 1;
        }
        self.check_kill_achievements();
    }

    /// Record gold collected
    pub fn record_gold(&mut self, amount: u32) {
        self.stats.gold_collected += amount as u64;
        self.check_gold_achievements();
    }

    /// Record item found
    pub fn record_item_found(&mut self, item_id: &str) {
        self.stats.items_found += 1;
        // Discovering certain items can unlock them for future runs
        if should_item_unlock(item_id) {
            self.unlocked_items.insert(item_id.to_string());
        }
    }

    /// Add playtime
    pub fn add_playtime(&mut self, seconds: u64) {
        self.stats.playtime_seconds += seconds;
    }

    /// Check if an item is unlocked
    pub fn is_item_unlocked(&self, item_id: &str) -> bool {
        self.unlocked_items.contains(item_id)
    }

    /// Check if an achievement is unlocked
    pub fn has_achievement(&self, achievement_id: &str) -> bool {
        self.achievements.contains(achievement_id)
    }

    /// Unlock an achievement
    pub fn unlock_achievement(&mut self, achievement_id: &str) -> bool {
        if !self.achievements.contains(achievement_id) {
            self.achievements.insert(achievement_id.to_string());
            log::info!("Achievement unlocked: {}", achievement_id);
            true
        } else {
            false
        }
    }

    // Achievement checking helpers
    fn check_floor_achievements(&mut self, floor: u32) {
        if floor >= 5 {
            self.unlock_achievement("reach_floor_5");
        }
        if floor >= 10 {
            self.unlock_achievement("reach_floor_10");
        }
        if floor >= 15 {
            self.unlock_achievement("reach_floor_15");
        }
        if floor >= 20 {
            self.unlock_achievement("reach_floor_20");
        }
    }

    fn check_kill_achievements(&mut self) {
        if self.stats.enemies_killed >= 100 {
            self.unlock_achievement("kill_100_enemies");
        }
        if self.stats.enemies_killed >= 500 {
            self.unlock_achievement("kill_500_enemies");
        }
        if self.stats.enemies_killed >= 1000 {
            self.unlock_achievement("kill_1000_enemies");
        }
        if self.stats.bosses_defeated >= 1 {
            self.unlock_achievement("defeat_first_boss");
        }
        if self.stats.bosses_defeated >= 4 {
            self.unlock_achievement("defeat_all_bosses");
        }
    }

    fn check_gold_achievements(&mut self) {
        if self.stats.gold_collected >= 1000 {
            self.unlock_achievement("collect_1000_gold");
        }
        if self.stats.gold_collected >= 10000 {
            self.unlock_achievement("collect_10000_gold");
        }
    }

    fn check_victory_achievements(&mut self) {
        if self.victories >= 1 {
            self.unlock_achievement("first_victory");
        }
        if self.victories >= 5 {
            self.unlock_achievement("five_victories");
        }
        if self.victories >= 10 {
            self.unlock_achievement("ten_victories");
        }
    }
}

/// Check if an item ID should be unlockable for future runs
fn should_item_unlock(item_id: &str) -> bool {
    // Legendary items are unlockable
    item_id.contains("legendary_") || item_id.contains("unique_")
}

// ============================================================================
// Profile Storage
// ============================================================================

/// Get the profile file path
fn profile_path() -> PathBuf {
    use directories::ProjectDirs;

    if let Some(proj_dirs) = ProjectDirs::from("com", "hollowdeep", "Hollowdeep") {
        let mut path = proj_dirs.data_local_dir().to_path_buf();
        path.push("profile.json");
        path
    } else {
        PathBuf::from("./profile.json")
    }
}

/// Load the player profile (or create default)
pub fn load_profile() -> PlayerProfile {
    let path = profile_path();

    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(data) => {
                match serde_json::from_str(&data) {
                    Ok(profile) => {
                        log::info!("Profile loaded from {:?}", path);
                        return profile;
                    }
                    Err(e) => {
                        log::warn!("Failed to parse profile: {}, creating new", e);
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to read profile: {}, creating new", e);
            }
        }
    }

    log::info!("Creating new profile");
    PlayerProfile::new()
}

/// Save the player profile
pub fn save_profile(profile: &PlayerProfile) -> Result<(), String> {
    let path = profile_path();

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let json = serde_json::to_string_pretty(profile)
        .map_err(|e| e.to_string())?;

    fs::write(&path, json).map_err(|e| e.to_string())?;

    log::info!("Profile saved to {:?}", path);
    Ok(())
}

// ============================================================================
// Achievement Definitions
// ============================================================================

/// Achievement definition
#[derive(Debug, Clone)]
pub struct Achievement {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub hidden: bool,
}

/// Get all achievement definitions
pub fn all_achievements() -> Vec<Achievement> {
    vec![
        // Floor achievements
        Achievement {
            id: "reach_floor_5",
            name: "Delver",
            description: "Reach floor 5",
            hidden: false,
        },
        Achievement {
            id: "reach_floor_10",
            name: "Explorer",
            description: "Reach floor 10",
            hidden: false,
        },
        Achievement {
            id: "reach_floor_15",
            name: "Spelunker",
            description: "Reach floor 15",
            hidden: false,
        },
        Achievement {
            id: "reach_floor_20",
            name: "Abyssal Diver",
            description: "Reach the final floor",
            hidden: false,
        },
        // Kill achievements
        Achievement {
            id: "kill_100_enemies",
            name: "Slayer",
            description: "Kill 100 enemies",
            hidden: false,
        },
        Achievement {
            id: "kill_500_enemies",
            name: "Executioner",
            description: "Kill 500 enemies",
            hidden: false,
        },
        Achievement {
            id: "kill_1000_enemies",
            name: "Genocide",
            description: "Kill 1000 enemies",
            hidden: false,
        },
        // Boss achievements
        Achievement {
            id: "defeat_first_boss",
            name: "Boss Slayer",
            description: "Defeat your first boss",
            hidden: false,
        },
        Achievement {
            id: "defeat_all_bosses",
            name: "Conqueror",
            description: "Defeat all four bosses",
            hidden: false,
        },
        // Gold achievements
        Achievement {
            id: "collect_1000_gold",
            name: "Wealthy",
            description: "Collect 1,000 gold total",
            hidden: false,
        },
        Achievement {
            id: "collect_10000_gold",
            name: "Treasure Hunter",
            description: "Collect 10,000 gold total",
            hidden: false,
        },
        // Victory achievements
        Achievement {
            id: "first_victory",
            name: "Victorious",
            description: "Complete the game for the first time",
            hidden: false,
        },
        Achievement {
            id: "five_victories",
            name: "Veteran",
            description: "Complete the game 5 times",
            hidden: false,
        },
        Achievement {
            id: "ten_victories",
            name: "Master",
            description: "Complete the game 10 times",
            hidden: false,
        },
        // Hidden achievements
        Achievement {
            id: "die_on_floor_1",
            name: "Humble Beginnings",
            description: "Die on the first floor",
            hidden: true,
        },
        Achievement {
            id: "no_damage_boss",
            name: "Untouchable",
            description: "Defeat a boss without taking damage",
            hidden: true,
        },
    ]
}
