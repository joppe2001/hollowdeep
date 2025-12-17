//! Data loading and external game content
//!
//! This module handles loading game data from external RON files,
//! allowing for data-driven content and easy modding.

pub mod loader;
pub mod items;
pub mod enemies;
pub mod synergies;

pub use loader::DataManager;
pub use items::ItemTemplate;
pub use enemies::EnemyTemplate;
pub use synergies::SynergyDef;
