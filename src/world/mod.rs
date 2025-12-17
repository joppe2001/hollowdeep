//! World module
//!
//! Contains map data structures, tiles, and procedural generation.

pub mod map;
pub mod tile;
pub mod fov;
pub mod generation;

pub use map::{Map, Biome};
pub use tile::{Tile, TileType};
pub use fov::compute_fov;
