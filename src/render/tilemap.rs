//! Tile rendering with multiple backend support
//!
//! Renders the game map using ASCII, Unicode, or Kitty graphics.

use std::io::{self};
use ratatui::{
    buffer::Buffer,
    style::Color,
};

use super::{RenderMode, KittyGraphics, SpriteSheet, SpriteId};
use crate::world::TileType;

/// Tile renderer that supports multiple rendering modes
pub struct TileRenderer {
    /// Current rendering mode
    pub mode: RenderMode,
    /// Kitty graphics handler (only used in Kitty mode)
    kitty: Option<KittyGraphics>,
    /// Sprite sheet for Kitty mode
    sprites: Option<SpriteSheet>,
    /// Uploaded tile IDs for Kitty (tile_type -> kitty_image_id)
    uploaded_tiles: Vec<Option<u32>>,
    /// Whether tiles have been uploaded to terminal
    tiles_uploaded: bool,
}

impl TileRenderer {
    /// Create a new tile renderer with the given mode
    pub fn new(mode: RenderMode) -> Self {
        let kitty = if mode == RenderMode::Kitty {
            Some(KittyGraphics::new())
        } else {
            None
        };

        Self {
            mode,
            kitty,
            sprites: None,
            uploaded_tiles: Vec::new(),
            tiles_uploaded: false,
        }
    }

    /// Set the sprite sheet for Kitty mode
    pub fn set_sprites(&mut self, sprites: SpriteSheet) {
        self.sprites = Some(sprites);
        self.tiles_uploaded = false; // Need to re-upload
    }

    /// Load sprites and prepare for rendering
    pub fn initialize(&mut self) -> io::Result<()> {
        if self.mode != RenderMode::Kitty {
            return Ok(());
        }

        // Create default placeholders if no sprites loaded
        if self.sprites.is_none() {
            self.sprites = Some(SpriteSheet::default());
        }

        // Upload sprites to terminal
        self.upload_tiles()?;

        Ok(())
    }

    /// Upload tile sprites to the terminal (Kitty mode only)
    fn upload_tiles(&mut self) -> io::Result<()> {
        let kitty = match &mut self.kitty {
            Some(k) => k,
            None => return Ok(()),
        };

        let sprites = match &self.sprites {
            Some(s) => s,
            None => return Ok(()),
        };

        // Upload each tile type's sprite
        let tile_sprites = [
            (TileType::Floor, SpriteId::FLOOR),
            (TileType::Wall, SpriteId::WALL),
            (TileType::Corridor, SpriteId::CORRIDOR),
            (TileType::Lava, SpriteId::LAVA),
            (TileType::Pit, SpriteId::PIT),
            (TileType::DoorClosed, SpriteId::DOOR_CLOSED),
            (TileType::DoorOpen, SpriteId::DOOR_OPEN),
            (TileType::StairsDown, SpriteId::STAIRS_DOWN),
            (TileType::StairsUp, SpriteId::STAIRS_UP),
            (TileType::Rubble, SpriteId::RUBBLE),
            (TileType::Bones, SpriteId::BONES),
            (TileType::BloodStain, SpriteId::BLOOD),
            (TileType::Torch, SpriteId::TORCH),
            (TileType::Brazier, SpriteId::BRAZIER),
        ];

        // Initialize upload tracking
        self.uploaded_tiles = vec![None; 256]; // Max tile types

        for (tile_type, sprite_id) in &tile_sprites {
            if let Some(sprite) = sprites.get(*sprite_id) {
                let kitty_id = kitty.upload_image(&sprite.image)?;
                self.uploaded_tiles[*tile_type as usize] = Some(kitty_id);
            }
        }

        self.tiles_uploaded = true;
        log::info!("Uploaded {} tile sprites via Kitty protocol", tile_sprites.len());

        Ok(())
    }

    /// Get the character representation for a tile (ASCII/Unicode modes)
    pub fn tile_char(&self, tile_type: TileType) -> char {
        match self.mode {
            RenderMode::Ascii => Self::ascii_char(tile_type),
            RenderMode::Unicode => Self::unicode_char(tile_type),
            RenderMode::NerdFont => Self::nerd_char(tile_type),
            RenderMode::Kitty => Self::unicode_char(tile_type), // Fallback
        }
    }

    /// ASCII characters for tiles
    fn ascii_char(tile_type: TileType) -> char {
        match tile_type {
            TileType::Floor => '.',
            TileType::Wall => '#',
            TileType::Corridor => '.',
            TileType::Lava => '~',
            TileType::Pit => ' ',
            TileType::DoorClosed => '+',
            TileType::DoorOpen => '/',
            TileType::StairsDown => '>',
            TileType::StairsUp => '<',
            TileType::Rubble => ',',
            TileType::Bones => '%',
            TileType::BloodStain => '.',
            TileType::Cobweb => ';',
            TileType::Cracks => '=',
            TileType::Moss => '`',
            TileType::Ashes => ':',
            TileType::Grime => '.',
            TileType::Torch => '!',
            TileType::Brazier => '*',
            TileType::ShrineSkill => 'S',
            TileType::ShrineEnchant => 'E',
            TileType::ShrineRest => 'R',
            TileType::ShrineCorruption => 'C',
        }
    }

    /// Unicode characters for tiles (richer visuals)
    fn unicode_char(tile_type: TileType) -> char {
        match tile_type {
            TileType::Floor => '·',      // Middle dot
            TileType::Wall => '█',       // Full block
            TileType::Corridor => '∙',   // Bullet operator
            TileType::Lava => '≈',       // Wavy lava
            TileType::Pit => ' ',
            TileType::DoorClosed => '▮', // Black vertical rectangle
            TileType::DoorOpen => '▯',   // White vertical rectangle
            TileType::StairsDown => '▼', // Down triangle
            TileType::StairsUp => '▲',   // Up triangle
            TileType::Rubble => '░',     // Light shade
            TileType::Bones => '☠',      // Skull
            TileType::BloodStain => '•', // Bullet
            TileType::Cobweb => '░',     // Light shade for cobwebs
            TileType::Cracks => '≡',     // Triple bar
            TileType::Moss => '`',       // Backtick
            TileType::Ashes => '∴',      // Therefore
            TileType::Grime => '·',      // Middle dot
            TileType::Torch => '☀',      // Sun (light source)
            TileType::Brazier => '♨',    // Hot springs (fire)
            TileType::ShrineSkill => '⚝',   // Circled star
            TileType::ShrineEnchant => '✦', // Black four pointed star
            TileType::ShrineRest => '☥',    // Ankh
            TileType::ShrineCorruption => '☠', // Skull (corruption)
        }
    }

    /// Nerd Font characters (requires Nerd Font installed)
    fn nerd_char(tile_type: TileType) -> char {
        // Nerd Font icons - these require the user to have a Nerd Font
        match tile_type {
            TileType::Floor => '·',
            TileType::Wall => '█',
            TileType::Corridor => '·',
            TileType::Lava => '󰈸',   // Fire icon
            TileType::Pit => ' ',
            TileType::DoorClosed => '󰠲', // Door closed
            TileType::DoorOpen => '󰠳',   // Door open
            TileType::StairsDown => '󰁅', // Arrow down
            TileType::StairsUp => '󰁝',   // Arrow up
            TileType::Rubble => '󰟀',     // Debris
            TileType::Bones => '󰚌',      // Skull
            TileType::BloodStain => '󰗈', // Drop
            TileType::Cobweb => '󰜙',     // Web-like
            TileType::Cracks => '󰨃',     // Cracks
            TileType::Moss => '󰌪',       // Plant
            TileType::Ashes => '󰀘',      // Dust
            TileType::Grime => '·',      // Dot
            TileType::Torch => '󰛨',      // Torch/flame
            TileType::Brazier => '󱠇',    // Fire
            TileType::ShrineSkill => '󰓥',   // Magic wand
            TileType::ShrineEnchant => '󰂵', // Star
            TileType::ShrineRest => '󰒲',    // Sleep
            TileType::ShrineCorruption => '󰚌', // Skull (corruption)
        }
    }

    /// Get foreground color for a tile
    pub fn tile_fg_color(&self, tile_type: TileType, lit: bool) -> Color {
        let (r, g, b) = if lit {
            match tile_type {
                TileType::Floor => (80, 80, 80),
                TileType::Wall => (130, 110, 90),
                TileType::Corridor => (70, 70, 70),
                TileType::Lava => (255, 100, 0),
                TileType::Pit => (20, 20, 20),
                TileType::DoorClosed => (160, 120, 60),
                TileType::DoorOpen => (140, 100, 50),
                TileType::StairsDown => (220, 220, 200),
                TileType::StairsUp => (220, 220, 200),
                TileType::Rubble => (100, 90, 80),
                TileType::Bones => (220, 210, 190),
                TileType::BloodStain => (180, 40, 40),
                TileType::Cobweb => (180, 180, 170),
                TileType::Cracks => (90, 85, 75),
                TileType::Moss => (60, 100, 50),
                TileType::Ashes => (100, 95, 90),
                TileType::Grime => (70, 65, 50),
                TileType::Torch => (255, 220, 100),
                TileType::Brazier => (255, 180, 80),
                TileType::ShrineSkill => (200, 100, 255),
                TileType::ShrineEnchant => (100, 200, 255),
                TileType::ShrineRest => (100, 255, 100),
                TileType::ShrineCorruption => (200, 50, 100),
            }
        } else {
            // Dim colors for unexplored but seen tiles
            match tile_type {
                TileType::Floor => (30, 30, 30),
                TileType::Wall => (50, 45, 40),
                TileType::Corridor => (25, 25, 25),
                TileType::Lava => (80, 40, 0),
                TileType::Pit => (10, 10, 10),
                TileType::DoorClosed => (60, 45, 25),
                TileType::DoorOpen => (50, 40, 20),
                TileType::StairsDown => (80, 80, 70),
                TileType::StairsUp => (80, 80, 70),
                TileType::Rubble => (40, 35, 30),
                TileType::Bones => (80, 75, 65),
                TileType::BloodStain => (60, 20, 20),
                TileType::Cobweb => (60, 60, 55),
                TileType::Cracks => (35, 32, 28),
                TileType::Moss => (25, 40, 20),
                TileType::Ashes => (40, 38, 35),
                TileType::Grime => (28, 25, 20),
                TileType::Torch => (80, 70, 40),
                TileType::Brazier => (80, 60, 30),
                TileType::ShrineSkill => (80, 40, 100),
                TileType::ShrineEnchant => (40, 80, 100),
                TileType::ShrineRest => (40, 100, 40),
                TileType::ShrineCorruption => (80, 20, 40),
            }
        };

        Color::Rgb(r, g, b)
    }

    /// Get foreground color for a tile with biome tinting
    pub fn tile_fg_color_biome(&self, tile_type: TileType, lit: bool, ambient: (u8, u8, u8)) -> Color {
        let base = self.tile_fg_color(tile_type, lit);
        if let Color::Rgb(r, g, b) = base {
            // Blend with ambient color (subtle tint)
            let blend = 0.15;
            let nr = ((r as f32) * (1.0 - blend) + (ambient.0 as f32) * blend) as u8;
            let ng = ((g as f32) * (1.0 - blend) + (ambient.1 as f32) * blend) as u8;
            let nb = ((b as f32) * (1.0 - blend) + (ambient.2 as f32) * blend) as u8;
            Color::Rgb(nr, ng, nb)
        } else {
            base
        }
    }

    /// Get background color for a tile
    pub fn tile_bg_color(&self, tile_type: TileType, lit: bool) -> Color {
        let (r, g, b) = if lit {
            match tile_type {
                TileType::Floor => (20, 18, 15),
                TileType::Wall => (40, 35, 30),
                TileType::Corridor => (15, 13, 10),
                TileType::Lava => (80, 30, 0),
                TileType::Pit => (5, 5, 5),
                TileType::DoorClosed => (35, 28, 18),
                TileType::DoorOpen => (20, 18, 15),
                TileType::StairsDown => (25, 23, 20),
                TileType::StairsUp => (25, 23, 20),
                TileType::Rubble => (25, 22, 18),
                TileType::Bones => (22, 20, 17),
                TileType::BloodStain => (45, 15, 15),
                TileType::Cobweb => (22, 20, 18),
                TileType::Cracks => (18, 16, 14),
                TileType::Moss => (15, 25, 15),
                TileType::Ashes => (25, 24, 22),
                TileType::Grime => (20, 18, 12),
                TileType::Torch => (35, 28, 15),
                TileType::Brazier => (40, 30, 15),
                TileType::ShrineSkill => (30, 15, 40),
                TileType::ShrineEnchant => (15, 30, 40),
                TileType::ShrineRest => (15, 35, 15),
                TileType::ShrineCorruption => (40, 10, 25),
            }
        } else {
            // Very dark for unexplored
            match tile_type {
                TileType::Lava => (30, 10, 0), // Lava still glows slightly
                TileType::Torch => (15, 12, 8),
                TileType::Brazier => (15, 12, 8),
                TileType::ShrineSkill => (12, 6, 16),
                TileType::ShrineEnchant => (6, 12, 16),
                TileType::ShrineRest => (6, 14, 6),
                TileType::ShrineCorruption => (16, 4, 10),
                _ => (8, 7, 6),
            }
        };

        Color::Rgb(r, g, b)
    }

    /// Get background color for a tile with biome tinting
    pub fn tile_bg_color_biome(&self, tile_type: TileType, lit: bool, ambient: (u8, u8, u8)) -> Color {
        let base = self.tile_bg_color(tile_type, lit);
        if let Color::Rgb(r, g, b) = base {
            // Blend with ambient color (stronger tint for backgrounds)
            let blend = if lit { 0.25 } else { 0.15 };
            let nr = ((r as f32) * (1.0 - blend) + (ambient.0 as f32 * 0.5) * blend) as u8;
            let ng = ((g as f32) * (1.0 - blend) + (ambient.1 as f32 * 0.5) * blend) as u8;
            let nb = ((b as f32) * (1.0 - blend) + (ambient.2 as f32 * 0.5) * blend) as u8;
            Color::Rgb(nr, ng, nb)
        } else {
            base
        }
    }

    /// Render a single tile to a ratatui buffer (for ASCII/Unicode modes)
    pub fn render_tile_to_buffer(
        &self,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        tile_type: TileType,
        visible: bool,
        explored: bool,
    ) {
        if !explored {
            // Not explored - render nothing (black)
            let cell = buf.cell_mut((x, y)).unwrap();
            cell.set_char(' ');
            cell.set_bg(Color::Black);
            return;
        }

        let ch = self.tile_char(tile_type);
        let fg = self.tile_fg_color(tile_type, visible);
        let bg = self.tile_bg_color(tile_type, visible);

        let cell = buf.cell_mut((x, y)).unwrap();
        cell.set_char(ch);
        cell.set_fg(fg);
        cell.set_bg(bg);
    }

    /// Render a tile using Kitty graphics (directly to terminal)
    pub fn render_tile_kitty(
        &self,
        col: u16,
        row: u16,
        tile_type: TileType,
        _visible: bool,
        explored: bool,
    ) -> io::Result<()> {
        if !explored {
            return Ok(()); // Will be covered by black background
        }

        let kitty = match &self.kitty {
            Some(k) => k,
            None => return Ok(()),
        };

        if let Some(Some(image_id)) = self.uploaded_tiles.get(tile_type as usize) {
            // Apply visibility dimming via terminal colors if needed
            // For now, just display the sprite
            kitty.display_image_at(*image_id, col, row, 1, 1)?;
        }

        Ok(())
    }

    /// Clean up resources (call when switching modes or exiting)
    pub fn cleanup(&mut self) -> io::Result<()> {
        if let Some(kitty) = &mut self.kitty {
            kitty.clear_all()?;
        }
        self.tiles_uploaded = false;
        Ok(())
    }
}

impl Default for TileRenderer {
    fn default() -> Self {
        Self::new(RenderMode::Unicode)
    }
}

/// Entity rendering info
pub struct EntityGlyph {
    pub ascii: char,
    pub unicode: char,
    pub nerd: char,
    pub sprite_id: SpriteId,
    pub fg: (u8, u8, u8),
}

impl EntityGlyph {
    pub fn player() -> Self {
        Self {
            ascii: '@',
            unicode: '☺',
            nerd: '󰀄',
            sprite_id: SpriteId::PLAYER,
            fg: (255, 255, 200),
        }
    }

    pub fn skeleton() -> Self {
        Self {
            ascii: 's',
            unicode: '☠',
            nerd: '󰚌',
            sprite_id: SpriteId::SKELETON,
            fg: (200, 200, 180),
        }
    }

    pub fn zombie() -> Self {
        Self {
            ascii: 'z',
            unicode: '⚉',
            nerd: '󰮯',
            sprite_id: SpriteId::ZOMBIE,
            fg: (100, 140, 80),
        }
    }

    /// Get the character for the current render mode
    pub fn char_for_mode(&self, mode: RenderMode) -> char {
        match mode {
            RenderMode::Ascii => self.ascii,
            RenderMode::Unicode => self.unicode,
            RenderMode::NerdFont => self.nerd,
            RenderMode::Kitty => self.unicode, // Fallback for mixed rendering
        }
    }

    /// Get foreground color
    pub fn fg_color(&self) -> Color {
        Color::Rgb(self.fg.0, self.fg.1, self.fg.2)
    }
}
