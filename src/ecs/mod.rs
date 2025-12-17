//! Entity Component System module
//!
//! Defines all components and systems for the game.

pub mod components;
pub mod systems;
pub mod resources;

pub use components::*;
pub use systems::{run_enemy_ai, execute_ai_actions, AIAction};
