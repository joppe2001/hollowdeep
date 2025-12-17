//! Biome definitions and configuration
//!
//! Each biome has distinct visual themes, enemy types, and generation parameters.

use crate::world::{Biome, TileType};

/// Configuration for a specific biome
#[derive(Debug, Clone)]
pub struct BiomeConfig {
    /// Display name
    pub name: &'static str,
    /// Flavor description
    pub description: &'static str,
    /// Primary wall color (RGB)
    pub wall_color: (u8, u8, u8),
    /// Secondary wall color for variety (RGB)
    pub wall_color_alt: (u8, u8, u8),
    /// Primary floor color (RGB)
    pub floor_color: (u8, u8, u8),
    /// Secondary floor color for variety (RGB)
    pub floor_color_alt: (u8, u8, u8),
    /// Ambient color tint (RGB) - applied to all tiles
    pub ambient_color: (u8, u8, u8),
    /// Corridor color (RGB)
    pub corridor_color: (u8, u8, u8),
    /// Generation style preference (0.0 = rooms, 1.0 = caves)
    pub cave_factor: f32,
    /// Light level modifier (1.0 = normal)
    pub light_modifier: f32,
    /// Chance of hazard tiles (lava, pits)
    pub hazard_chance: f32,
    /// Hazard type preference
    pub primary_hazard: HazardType,
    /// Primary decoration types for this biome
    pub decorations: &'static [TileType],
    /// Decoration density (chance per floor tile)
    pub decoration_density: f32,
    /// Wall glyph variations
    pub wall_glyphs: &'static [char],
    /// Floor glyph variations
    pub floor_glyphs: &'static [char],
}

/// Types of environmental hazards
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HazardType {
    None,
    Lava,
    Pit,
    Corruption,
}

impl Biome {
    /// Get the configuration for this biome
    pub fn config(&self) -> BiomeConfig {
        match self {
            Biome::SunkenCatacombs => BiomeConfig {
                name: "Sunken Catacombs",
                description: "Ancient burial chambers beneath the earth. The dead do not rest easy here.",
                wall_color: (70, 65, 55),
                wall_color_alt: (55, 50, 45),
                floor_color: (40, 38, 32),
                floor_color_alt: (35, 32, 28),
                ambient_color: (50, 45, 35),
                corridor_color: (32, 30, 25),
                cave_factor: 0.1,  // Mostly room-based for cleaner layouts
                light_modifier: 1.0,
                hazard_chance: 0.01,
                primary_hazard: HazardType::Pit,
                decorations: &[TileType::Bones, TileType::Rubble, TileType::Cobweb, TileType::Cracks],
                decoration_density: 0.04,
                wall_glyphs: &['#', '▓', '█', '▒'],
                floor_glyphs: &['.', '·', ',', '∙'],
            },
            Biome::BleedingCrypts => BiomeConfig {
                name: "Bleeding Crypts",
                description: "Crimson stains cover every surface. Blood cultists perform dark rituals in the depths.",
                wall_color: (80, 45, 45),
                wall_color_alt: (65, 35, 35),
                floor_color: (50, 30, 30),
                floor_color_alt: (40, 25, 25),
                ambient_color: (70, 25, 25),
                corridor_color: (35, 20, 20),
                cave_factor: 0.25,  // Some caves, but mostly rooms
                light_modifier: 0.9,
                hazard_chance: 0.03,
                primary_hazard: HazardType::Corruption,
                decorations: &[TileType::BloodStain, TileType::Bones, TileType::Grime],
                decoration_density: 0.06,
                wall_glyphs: &['#', '▓', '░', '▒'],
                floor_glyphs: &['.', '·', '∴', '•'],
            },
            Biome::HollowCathedral => BiomeConfig {
                name: "Hollow Cathedral",
                description: "Once a place of worship, now defiled by fallen angels. Grand halls echo with whispers.",
                wall_color: (90, 90, 105),
                wall_color_alt: (75, 75, 90),
                floor_color: (55, 55, 65),
                floor_color_alt: (45, 45, 55),
                ambient_color: (60, 60, 85),
                corridor_color: (40, 40, 50),
                cave_factor: 0.15,  // Large rooms with corridors
                light_modifier: 1.1,
                hazard_chance: 0.02,
                primary_hazard: HazardType::Pit,
                decorations: &[TileType::Rubble, TileType::Cracks, TileType::Cobweb],
                decoration_density: 0.03,
                wall_glyphs: &['#', '█', '▓', '╬'],
                floor_glyphs: &['.', '·', '○', '∙'],
            },
            Biome::TheAbyss => BiomeConfig {
                name: "The Abyss",
                description: "Reality itself breaks down here. Eldritch horrors lurk in the endless dark.",
                wall_color: (40, 30, 60),
                wall_color_alt: (30, 20, 50),
                floor_color: (28, 20, 45),
                floor_color_alt: (20, 15, 35),
                ambient_color: (35, 25, 60),
                corridor_color: (18, 12, 30),
                cave_factor: 0.3,  // Mixed - chaotic but navigable
                light_modifier: 0.7,
                hazard_chance: 0.05,
                primary_hazard: HazardType::Lava,
                decorations: &[TileType::Ashes, TileType::Cracks, TileType::Grime],
                decoration_density: 0.05,
                wall_glyphs: &['#', '▓', '█', '░'],
                floor_glyphs: &['.', '∙', '·', '°'],
            },
        }
    }

    /// Get the description
    pub fn description(&self) -> &'static str {
        self.config().description
    }

    /// Check if this biome prefers cave generation
    pub fn prefers_caves(&self) -> bool {
        self.config().cave_factor > 0.5
    }
}
