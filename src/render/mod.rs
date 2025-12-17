//! Rendering abstraction layer
//!
//! Supports multiple rendering modes:
//! - ASCII: Classic roguelike characters
//! - Unicode: Rich unicode symbols
//! - Kitty: Full image/sprite rendering via Kitty graphics protocol

pub mod mode;
pub mod kitty;
pub mod sprites;
pub mod tilemap;

pub use mode::{RenderMode, detect_render_mode};
pub use kitty::KittyGraphics;
pub use sprites::{SpriteSheet, Sprite, SpriteId};
pub use tilemap::TileRenderer;
