//! Save/load system
//!
//! Handles game saving, loading, and player profiles.

pub mod save_game;
pub mod profile;

pub use save_game::{
    SaveData, SaveError, SaveSummary,
    save_game, load_game, delete_save,
    save_exists, list_saves, save_path,
};

pub use profile::{
    PlayerProfile, ProfileStats, ProfileSettings, Achievement,
    load_profile, save_profile, all_achievements,
};
