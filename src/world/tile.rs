//! Tile definitions
//!
//! Different tile types and their properties.

use serde::{Deserialize, Serialize};

/// A single tile in the map
#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub tile_type: TileType,
    pub explored: bool,
    pub visible: bool,
    pub light_level: u8,
    /// Optional glyph override (for elite markers, etc.)
    pub glyph: Option<char>,
}

impl Tile {
    pub fn new(tile_type: TileType) -> Self {
        Self {
            tile_type,
            explored: false,
            visible: false,
            light_level: 0,
            glyph: None,
        }
    }

    pub fn is_walkable(&self) -> bool {
        self.tile_type.is_walkable()
    }

    pub fn is_transparent(&self) -> bool {
        self.tile_type.is_transparent()
    }

    pub fn glyph(&self) -> char {
        self.glyph.unwrap_or_else(|| self.tile_type.glyph())
    }

    pub fn fg_color(&self, lit: bool) -> (u8, u8, u8) {
        if lit {
            self.tile_type.fg_color()
        } else {
            // Dimmed color for unexplored
            let (r, g, b) = self.tile_type.fg_color();
            (r / 3, g / 3, b / 3)
        }
    }

    pub fn bg_color(&self, lit: bool) -> (u8, u8, u8) {
        if lit {
            self.tile_type.bg_color()
        } else {
            let (r, g, b) = self.tile_type.bg_color();
            (r / 3, g / 3, b / 3)
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self::new(TileType::Wall)
    }
}

/// Types of tiles in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileType {
    // Basic terrain
    Floor,
    Wall,

    // Special floor types
    Corridor,
    Lava,
    Pit,

    // Interactables
    DoorClosed,
    DoorOpen,
    StairsDown,
    StairsUp,

    // Decorative (biome-specific floor variations)
    Rubble,
    Bones,
    BloodStain,
    Cobweb,
    Cracks,
    Moss,
    Ashes,
    Grime,

    // Light sources
    Torch,
    Brazier,

    // Shrines
    ShrineSkill,
    ShrineEnchant,
    ShrineRest,
    ShrineCorruption, // Risk/reward: curse for power
}

impl TileType {
    pub fn is_walkable(&self) -> bool {
        matches!(
            self,
            TileType::Floor
                | TileType::Corridor
                | TileType::DoorOpen
                | TileType::StairsDown
                | TileType::StairsUp
                | TileType::Rubble
                | TileType::Bones
                | TileType::BloodStain
                | TileType::Cobweb
                | TileType::Cracks
                | TileType::Moss
                | TileType::Ashes
                | TileType::Grime
                | TileType::Torch
                | TileType::Brazier
                | TileType::ShrineSkill
                | TileType::ShrineEnchant
                | TileType::ShrineRest
                | TileType::ShrineCorruption
        )
    }

    pub fn is_transparent(&self) -> bool {
        !matches!(self, TileType::Wall | TileType::DoorClosed)
    }

    pub fn glyph(&self) -> char {
        match self {
            TileType::Floor => '.',
            TileType::Wall => '#',
            TileType::Corridor => '.',
            TileType::Lava => '≈',
            TileType::Pit => ' ',
            TileType::DoorClosed => '+',
            TileType::DoorOpen => '/',
            TileType::StairsDown => '>',
            TileType::StairsUp => '<',
            TileType::Rubble => ',',
            TileType::Bones => '%',
            TileType::BloodStain => '·',
            TileType::Cobweb => '░',
            TileType::Cracks => '≡',
            TileType::Moss => '`',
            TileType::Ashes => '∴',
            TileType::Grime => '·',
            TileType::Torch => '☀',
            TileType::Brazier => '♨',
            TileType::ShrineSkill => '⚝',
            TileType::ShrineEnchant => '✦',
            TileType::ShrineRest => '☥',
            TileType::ShrineCorruption => '☠',
        }
    }

    pub fn fg_color(&self) -> (u8, u8, u8) {
        match self {
            TileType::Floor => (80, 80, 80),
            TileType::Wall => (130, 110, 90),
            TileType::Corridor => (70, 70, 70),
            TileType::Lava => (255, 100, 0),
            TileType::Pit => (20, 20, 20),
            TileType::DoorClosed => (139, 90, 43),
            TileType::DoorOpen => (139, 90, 43),
            TileType::StairsDown => (200, 200, 200),
            TileType::StairsUp => (200, 200, 200),
            TileType::Rubble => (100, 90, 80),
            TileType::Bones => (200, 200, 180),
            TileType::BloodStain => (150, 30, 30),
            TileType::Cobweb => (180, 180, 170),
            TileType::Cracks => (90, 85, 75),
            TileType::Moss => (60, 100, 50),
            TileType::Ashes => (100, 95, 90),
            TileType::Grime => (70, 65, 50),
            TileType::Torch => (255, 200, 50),
            TileType::Brazier => (255, 150, 50),
            TileType::ShrineSkill => (200, 100, 255),   // Purple for skill shrine
            TileType::ShrineEnchant => (100, 200, 255), // Cyan for enchant shrine
            TileType::ShrineRest => (100, 255, 100),    // Green for rest shrine
            TileType::ShrineCorruption => (180, 50, 100), // Dark red/magenta for corruption
        }
    }

    pub fn bg_color(&self) -> (u8, u8, u8) {
        match self {
            TileType::Floor => (20, 18, 15),
            TileType::Wall => (40, 35, 30),
            TileType::Corridor => (15, 13, 10),
            TileType::Lava => (80, 20, 0),
            TileType::Pit => (5, 5, 5),
            TileType::DoorClosed => (30, 25, 20),
            TileType::DoorOpen => (20, 18, 15),
            TileType::StairsDown => (20, 18, 15),
            TileType::StairsUp => (20, 18, 15),
            TileType::Rubble => (25, 22, 18),
            TileType::Bones => (20, 18, 15),
            TileType::BloodStain => (40, 15, 15),
            TileType::Cobweb => (22, 20, 18),
            TileType::Cracks => (18, 16, 14),
            TileType::Moss => (15, 25, 15),
            TileType::Ashes => (25, 24, 22),
            TileType::Grime => (20, 18, 12),
            TileType::Torch => (30, 25, 15),
            TileType::Brazier => (35, 25, 15),
            TileType::ShrineSkill => (30, 15, 40),
            TileType::ShrineEnchant => (15, 30, 40),
            TileType::ShrineRest => (15, 35, 15),
            TileType::ShrineCorruption => (40, 10, 25), // Dark ominous background
        }
    }

    /// Is this a light source?
    pub fn light_radius(&self) -> Option<i32> {
        match self {
            TileType::Torch => Some(4),
            TileType::Brazier => Some(6),
            TileType::Lava => Some(3),
            TileType::ShrineSkill => Some(3),
            TileType::ShrineEnchant => Some(3),
            TileType::ShrineRest => Some(3),
            TileType::ShrineCorruption => Some(4), // Eerie glow
            _ => None,
        }
    }

    /// Is this a shrine?
    pub fn is_shrine(&self) -> bool {
        matches!(self, TileType::ShrineSkill | TileType::ShrineEnchant | TileType::ShrineRest | TileType::ShrineCorruption)
    }
}
