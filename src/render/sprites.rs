//! Sprite and tileset management
//!
//! Load sprite sheets, extract individual sprites, and manage sprite IDs.

use std::collections::HashMap;
use std::path::Path;
use image::{DynamicImage, GenericImageView, RgbaImage};
use serde::{Deserialize, Serialize};

/// Unique identifier for a sprite
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpriteId(pub u32);

impl SpriteId {
    // Predefined sprite IDs for game elements
    // Terrain
    pub const FLOOR: SpriteId = SpriteId(0);
    pub const WALL: SpriteId = SpriteId(1);
    pub const CORRIDOR: SpriteId = SpriteId(2);
    pub const LAVA: SpriteId = SpriteId(4);
    pub const PIT: SpriteId = SpriteId(5);
    pub const DOOR_CLOSED: SpriteId = SpriteId(6);
    pub const DOOR_OPEN: SpriteId = SpriteId(7);
    pub const STAIRS_DOWN: SpriteId = SpriteId(8);
    pub const STAIRS_UP: SpriteId = SpriteId(9);

    // Decorations
    pub const RUBBLE: SpriteId = SpriteId(10);
    pub const BONES: SpriteId = SpriteId(11);
    pub const BLOOD: SpriteId = SpriteId(12);
    pub const TORCH: SpriteId = SpriteId(13);
    pub const BRAZIER: SpriteId = SpriteId(14);

    // Entities
    pub const PLAYER: SpriteId = SpriteId(100);
    pub const SKELETON: SpriteId = SpriteId(101);
    pub const ZOMBIE: SpriteId = SpriteId(102);
    pub const GHOST: SpriteId = SpriteId(103);
    pub const CULTIST: SpriteId = SpriteId(104);
    pub const DEMON: SpriteId = SpriteId(105);
    pub const BOSS_CATACOMBS: SpriteId = SpriteId(150);
    pub const BOSS_CRYPTS: SpriteId = SpriteId(151);
    pub const BOSS_CATHEDRAL: SpriteId = SpriteId(152);
    pub const BOSS_ABYSS: SpriteId = SpriteId(153);

    // Items
    pub const SWORD: SpriteId = SpriteId(200);
    pub const AXE: SpriteId = SpriteId(201);
    pub const STAFF: SpriteId = SpriteId(202);
    pub const DAGGER: SpriteId = SpriteId(203);
    pub const SHIELD: SpriteId = SpriteId(204);
    pub const ARMOR: SpriteId = SpriteId(205);
    pub const HELMET: SpriteId = SpriteId(206);
    pub const POTION_RED: SpriteId = SpriteId(220);
    pub const POTION_BLUE: SpriteId = SpriteId(221);
    pub const POTION_GREEN: SpriteId = SpriteId(222);
    pub const SCROLL: SpriteId = SpriteId(230);
    pub const KEY: SpriteId = SpriteId(231);
    pub const GOLD: SpriteId = SpriteId(232);
    pub const CHEST: SpriteId = SpriteId(233);

    // UI elements
    pub const HEART_FULL: SpriteId = SpriteId(300);
    pub const HEART_HALF: SpriteId = SpriteId(301);
    pub const HEART_EMPTY: SpriteId = SpriteId(302);
    pub const MANA_FULL: SpriteId = SpriteId(303);
    pub const MANA_EMPTY: SpriteId = SpriteId(304);

    // Effects
    pub const EFFECT_FIRE: SpriteId = SpriteId(400);
    pub const EFFECT_ICE: SpriteId = SpriteId(401);
    pub const EFFECT_POISON: SpriteId = SpriteId(402);
    pub const EFFECT_HEAL: SpriteId = SpriteId(403);
}

/// A single sprite image
#[derive(Clone)]
pub struct Sprite {
    /// The image data
    pub image: DynamicImage,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Optional Kitty image ID (set after upload)
    pub kitty_id: Option<u32>,
}

impl Sprite {
    /// Create a sprite from an image
    pub fn new(image: DynamicImage) -> Self {
        let (width, height) = image.dimensions();
        Self {
            image,
            width,
            height,
            kitty_id: None,
        }
    }

    /// Create a solid colored sprite (for fallback/placeholders)
    pub fn solid_color(width: u32, height: u32, r: u8, g: u8, b: u8, a: u8) -> Self {
        let mut img = RgbaImage::new(width, height);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgba([r, g, b, a]);
        }
        Self::new(DynamicImage::ImageRgba8(img))
    }
}

/// A sprite sheet containing multiple sprites in a grid
pub struct SpriteSheet {
    /// All loaded sprites by ID
    sprites: HashMap<SpriteId, Sprite>,
    /// Sprite size (all sprites same size)
    pub sprite_width: u32,
    pub sprite_height: u32,
}

impl SpriteSheet {
    /// Create an empty sprite sheet
    pub fn new(sprite_width: u32, sprite_height: u32) -> Self {
        Self {
            sprites: HashMap::new(),
            sprite_width,
            sprite_height,
        }
    }

    /// Load a sprite sheet from an image file
    pub fn from_file<P: AsRef<Path>>(
        path: P,
        sprite_width: u32,
        sprite_height: u32,
    ) -> Result<Self, image::ImageError> {
        let img = image::open(path)?;
        Ok(Self::from_image(img, sprite_width, sprite_height))
    }

    /// Load from an already-loaded image
    pub fn from_image(
        image: DynamicImage,
        sprite_width: u32,
        sprite_height: u32,
    ) -> Self {
        let mut sheet = Self::new(sprite_width, sprite_height);
        sheet.extract_sprites_grid(&image);
        sheet
    }

    /// Extract all sprites from a grid layout
    /// Sprites are numbered left-to-right, top-to-bottom
    fn extract_sprites_grid(&mut self, image: &DynamicImage) {
        let (img_width, img_height) = image.dimensions();
        let cols = img_width / self.sprite_width;
        let rows = img_height / self.sprite_height;

        let mut id = 0u32;
        for row in 0..rows {
            for col in 0..cols {
                let x = col * self.sprite_width;
                let y = row * self.sprite_height;

                let sprite_img = image.crop_imm(x, y, self.sprite_width, self.sprite_height);
                self.sprites.insert(SpriteId(id), Sprite::new(sprite_img));
                id += 1;
            }
        }

        log::info!(
            "Extracted {} sprites ({}x{} grid) from sprite sheet",
            id,
            cols,
            rows
        );
    }

    /// Add a sprite manually
    pub fn add_sprite(&mut self, id: SpriteId, sprite: Sprite) {
        self.sprites.insert(id, sprite);
    }

    /// Get a sprite by ID
    pub fn get(&self, id: SpriteId) -> Option<&Sprite> {
        self.sprites.get(&id)
    }

    /// Get a mutable sprite by ID
    pub fn get_mut(&mut self, id: SpriteId) -> Option<&mut Sprite> {
        self.sprites.get_mut(&id)
    }

    /// Check if a sprite exists
    pub fn has_sprite(&self, id: SpriteId) -> bool {
        self.sprites.contains_key(&id)
    }

    /// Get all sprite IDs
    pub fn sprite_ids(&self) -> impl Iterator<Item = &SpriteId> {
        self.sprites.keys()
    }

    /// Number of sprites loaded
    pub fn len(&self) -> usize {
        self.sprites.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.sprites.is_empty()
    }

    /// Create placeholder sprites for all standard IDs
    /// Useful when no sprite sheet is loaded yet
    pub fn create_placeholders(&mut self) {
        let w = self.sprite_width;
        let h = self.sprite_height;

        // Terrain placeholders
        self.add_sprite(SpriteId::FLOOR, Sprite::solid_color(w, h, 40, 35, 30, 255));
        self.add_sprite(SpriteId::WALL, Sprite::solid_color(w, h, 80, 70, 60, 255));
        self.add_sprite(SpriteId::CORRIDOR, Sprite::solid_color(w, h, 30, 25, 20, 255));
        self.add_sprite(SpriteId::LAVA, Sprite::solid_color(w, h, 200, 80, 20, 255));
        self.add_sprite(SpriteId::PIT, Sprite::solid_color(w, h, 10, 10, 10, 255));
        self.add_sprite(SpriteId::DOOR_CLOSED, Sprite::solid_color(w, h, 100, 70, 40, 255));
        self.add_sprite(SpriteId::DOOR_OPEN, Sprite::solid_color(w, h, 60, 40, 25, 255));
        self.add_sprite(SpriteId::STAIRS_DOWN, Sprite::solid_color(w, h, 150, 150, 150, 255));
        self.add_sprite(SpriteId::STAIRS_UP, Sprite::solid_color(w, h, 180, 180, 180, 255));

        // Decoration placeholders
        self.add_sprite(SpriteId::RUBBLE, Sprite::solid_color(w, h, 70, 65, 55, 255));
        self.add_sprite(SpriteId::BONES, Sprite::solid_color(w, h, 200, 190, 170, 255));
        self.add_sprite(SpriteId::BLOOD, Sprite::solid_color(w, h, 120, 20, 20, 255));
        self.add_sprite(SpriteId::TORCH, Sprite::solid_color(w, h, 255, 180, 50, 255));
        self.add_sprite(SpriteId::BRAZIER, Sprite::solid_color(w, h, 255, 140, 40, 255));

        // Entity placeholders
        self.add_sprite(SpriteId::PLAYER, Sprite::solid_color(w, h, 255, 255, 200, 255));
        self.add_sprite(SpriteId::SKELETON, Sprite::solid_color(w, h, 200, 200, 180, 255));
        self.add_sprite(SpriteId::ZOMBIE, Sprite::solid_color(w, h, 100, 140, 80, 255));
        self.add_sprite(SpriteId::GHOST, Sprite::solid_color(w, h, 180, 200, 220, 200));
        self.add_sprite(SpriteId::CULTIST, Sprite::solid_color(w, h, 100, 30, 30, 255));
        self.add_sprite(SpriteId::DEMON, Sprite::solid_color(w, h, 180, 40, 40, 255));

        // Item placeholders
        self.add_sprite(SpriteId::SWORD, Sprite::solid_color(w, h, 180, 180, 200, 255));
        self.add_sprite(SpriteId::POTION_RED, Sprite::solid_color(w, h, 200, 50, 50, 255));
        self.add_sprite(SpriteId::POTION_BLUE, Sprite::solid_color(w, h, 50, 100, 200, 255));
        self.add_sprite(SpriteId::GOLD, Sprite::solid_color(w, h, 255, 215, 0, 255));
        self.add_sprite(SpriteId::CHEST, Sprite::solid_color(w, h, 139, 90, 43, 255));

        log::info!("Created {} placeholder sprites", self.sprites.len());
    }
}

impl Default for SpriteSheet {
    fn default() -> Self {
        let mut sheet = Self::new(16, 16); // Default 16x16 sprites
        sheet.create_placeholders();
        sheet
    }
}

/// Sprite mapping configuration (loaded from RON file)
#[derive(Debug, Serialize, Deserialize)]
pub struct SpriteMapping {
    pub sprite_width: u32,
    pub sprite_height: u32,
    pub mappings: HashMap<String, (u32, u32)>, // name -> (col, row) in sheet
}

impl SpriteMapping {
    /// Load from a RON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let mapping: SpriteMapping = ron::from_str(&content)?;
        Ok(mapping)
    }
}
