//! Color definitions for the graphical frontend

use macroquad::prelude::Color;

/// Convert RGB tuple to macroquad Color
pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba(r, g, b, 255)
}

/// Convert RGBA tuple to macroquad Color
pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba(r, g, b, a)
}

// UI Colors
pub const BACKGROUND: Color = Color::new(0.05, 0.05, 0.08, 1.0);
pub const PANEL_BG: Color = Color::new(0.08, 0.08, 0.12, 0.95);
pub const PANEL_BORDER: Color = Color::new(0.3, 0.3, 0.4, 1.0);
pub const TEXT_PRIMARY: Color = Color::new(0.9, 0.9, 0.85, 1.0);
pub const TEXT_SECONDARY: Color = Color::new(0.6, 0.6, 0.55, 1.0);
pub const TEXT_MUTED: Color = Color::new(0.4, 0.4, 0.35, 1.0);

// Health/Resource colors
pub const HEALTH_HIGH: Color = Color::new(0.2, 0.8, 0.2, 1.0);
pub const HEALTH_MED: Color = Color::new(0.9, 0.7, 0.1, 1.0);
pub const HEALTH_LOW: Color = Color::new(0.9, 0.2, 0.2, 1.0);
pub const MANA: Color = Color::new(0.3, 0.5, 0.9, 1.0);
pub const STAMINA: Color = Color::new(0.9, 0.8, 0.2, 1.0);
pub const XP: Color = Color::new(0.3, 0.8, 0.8, 1.0);

// Entity colors
pub const PLAYER: Color = Color::new(1.0, 1.0, 0.8, 1.0);
pub const ENEMY: Color = Color::new(0.9, 0.3, 0.3, 1.0);
pub const ENEMY_ELITE: Color = Color::new(1.0, 0.5, 0.0, 1.0);
pub const NPC: Color = Color::new(0.5, 0.8, 0.5, 1.0);
pub const ITEM: Color = Color::new(0.9, 0.8, 0.3, 1.0);

// Tile colors (fallbacks, biomes override these)
pub const FLOOR: Color = Color::new(0.15, 0.14, 0.12, 1.0);
pub const WALL: Color = Color::new(0.3, 0.28, 0.22, 1.0);
pub const CORRIDOR: Color = Color::new(0.12, 0.11, 0.09, 1.0);
pub const STAIRS: Color = Color::new(0.6, 0.6, 0.7, 1.0);
pub const DOOR: Color = Color::new(0.5, 0.35, 0.2, 1.0);

// Hazard colors
pub const LAVA: Color = Color::new(0.9, 0.4, 0.1, 1.0);
pub const PIT: Color = Color::new(0.02, 0.02, 0.02, 1.0);
pub const CORRUPTION: Color = Color::new(0.4, 0.1, 0.4, 1.0);

// Decoration colors
pub const TORCH: Color = Color::new(1.0, 0.7, 0.3, 1.0);
pub const BRAZIER: Color = Color::new(1.0, 0.5, 0.2, 1.0);
pub const BLOOD: Color = Color::new(0.5, 0.1, 0.1, 1.0);
pub const BONES: Color = Color::new(0.8, 0.75, 0.65, 1.0);

// Rarity colors
pub const RARITY_COMMON: Color = Color::new(0.8, 0.8, 0.8, 1.0);
pub const RARITY_UNCOMMON: Color = Color::new(0.4, 1.0, 0.4, 1.0);
pub const RARITY_RARE: Color = Color::new(0.4, 0.6, 1.0, 1.0);
pub const RARITY_EPIC: Color = Color::new(0.8, 0.4, 1.0, 1.0);
pub const RARITY_LEGENDARY: Color = Color::new(1.0, 0.7, 0.2, 1.0);

// Message colors
pub const MSG_COMBAT: Color = Color::new(0.9, 0.3, 0.3, 1.0);
pub const MSG_ITEM: Color = Color::new(0.9, 0.8, 0.3, 1.0);
pub const MSG_SYSTEM: Color = Color::new(0.3, 0.8, 0.9, 1.0);
pub const MSG_LORE: Color = Color::new(0.8, 0.4, 0.8, 1.0);
pub const MSG_WARNING: Color = Color::new(1.0, 0.5, 0.5, 1.0);

// Fog of war
pub const FOG_EXPLORED: Color = Color::new(0.15, 0.15, 0.18, 1.0);
pub const FOG_HIDDEN: Color = Color::new(0.0, 0.0, 0.0, 1.0);
