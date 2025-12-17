//! Hollowdeep - A grimdark terminal roguelike RPG
//!
//! Descend into the cursed depths, face eldritch horrors,
//! and forge your path through corruption and darkness.

pub mod game;
pub mod ecs;
pub mod world;
pub mod entities;
pub mod combat;
pub mod items;
pub mod progression;
pub mod ui;
pub mod render;
pub mod audio;
pub mod save;
pub mod mods;
pub mod data;

// Re-export commonly used types
pub use game::{Game, GameState};
pub use ecs::components::*;
pub use world::map::Map;
