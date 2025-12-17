//! Map data structure
//!
//! The 2D grid representing a dungeon floor.

use super::tile::{Tile, TileType};
use crate::ecs::Position;
use serde::{Deserialize, Serialize};

/// A dungeon floor map
#[derive(Debug, Clone)]
pub struct Map {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Tile>,
    pub floor_number: u32,
    pub biome: Biome,
    /// Start position for player
    pub start_pos: Position,
    /// Exit position (stairs down)
    pub exit_pos: Option<Position>,
    /// Elite room positions (centers) - dangerous but rewarding
    pub elite_rooms: Vec<Position>,
}

/// Biome types for different dungeon zones
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Biome {
    SunkenCatacombs,
    BleedingCrypts,
    HollowCathedral,
    TheAbyss,
}

impl Map {
    /// Create a new map filled with walls
    pub fn new(width: i32, height: i32, floor_number: u32, biome: Biome) -> Self {
        let tiles = vec![Tile::default(); (width * height) as usize];
        Self {
            width,
            height,
            tiles,
            floor_number,
            biome,
            start_pos: Position::new(0, 0),
            exit_pos: None,
            elite_rooms: Vec::new(),
        }
    }

    /// Check if a position is in an elite zone (within 5 tiles of an elite room center)
    pub fn is_elite_zone(&self, pos: Position) -> bool {
        const ELITE_RADIUS: i32 = 5;
        self.elite_rooms.iter().any(|elite| elite.chebyshev_distance(&pos) <= ELITE_RADIUS)
    }

    /// Add an elite room at the given position
    pub fn add_elite_room(&mut self, pos: Position) {
        self.elite_rooms.push(pos);
    }

    /// Get all elite room center positions
    pub fn elite_rooms(&self) -> &[Position] {
        &self.elite_rooms
    }

    /// Convert 2D coordinates to 1D index
    #[inline]
    pub fn xy_to_idx(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    /// Convert 1D index to 2D coordinates
    #[inline]
    pub fn idx_to_xy(&self, idx: usize) -> (i32, i32) {
        let idx = idx as i32;
        (idx % self.width, idx / self.width)
    }

    /// Check if coordinates are within bounds
    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    /// Get tile at position
    pub fn get_tile(&self, x: i32, y: i32) -> Option<&Tile> {
        if self.in_bounds(x, y) {
            Some(&self.tiles[self.xy_to_idx(x, y)])
        } else {
            None
        }
    }

    /// Get mutable tile at position
    pub fn get_tile_mut(&mut self, x: i32, y: i32) -> Option<&mut Tile> {
        if self.in_bounds(x, y) {
            let idx = self.xy_to_idx(x, y);
            Some(&mut self.tiles[idx])
        } else {
            None
        }
    }

    /// Set tile type at position
    pub fn set_tile(&mut self, x: i32, y: i32, tile_type: TileType) {
        if self.in_bounds(x, y) {
            let idx = self.xy_to_idx(x, y);
            self.tiles[idx].tile_type = tile_type;
        }
    }

    /// Check if a position is walkable
    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        self.get_tile(x, y).map_or(false, |t| t.is_walkable())
    }

    /// Check if a position blocks line of sight
    pub fn is_opaque(&self, x: i32, y: i32) -> bool {
        self.get_tile(x, y).map_or(true, |t| !t.is_transparent())
    }

    /// Mark a tile as explored
    pub fn mark_explored(&mut self, x: i32, y: i32) {
        if let Some(tile) = self.get_tile_mut(x, y) {
            tile.explored = true;
        }
    }

    /// Set tile visibility
    pub fn set_visible(&mut self, x: i32, y: i32, visible: bool) {
        if let Some(tile) = self.get_tile_mut(x, y) {
            tile.visible = visible;
            if visible {
                tile.explored = true;
            }
        }
    }

    /// Clear all visibility (before recalculating FOV)
    pub fn clear_visibility(&mut self) {
        for tile in &mut self.tiles {
            tile.visible = false;
        }
    }

    /// Create a simple test map for development
    pub fn test_map() -> Self {
        let width = 80;
        let height = 50;
        let mut map = Map::new(width, height, 1, Biome::SunkenCatacombs);

        // Fill with floor
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                map.set_tile(x, y, TileType::Floor);
            }
        }

        // Add some rooms by carving walls
        // Room 1: Starting room
        for y in 5..15 {
            for x in 5..20 {
                map.set_tile(x, y, TileType::Floor);
            }
        }
        // Add a torch
        map.set_tile(7, 7, TileType::Torch);

        // Room 2
        for y in 5..20 {
            for x in 30..50 {
                map.set_tile(x, y, TileType::Floor);
            }
        }

        // Corridor connecting rooms
        for x in 20..31 {
            map.set_tile(x, 10, TileType::Corridor);
        }

        // Room 3 with stairs
        for y in 25..40 {
            for x in 40..60 {
                map.set_tile(x, y, TileType::Floor);
            }
        }
        map.set_tile(50, 32, TileType::StairsDown);

        // Corridor from room 2 to room 3
        for y in 19..26 {
            map.set_tile(45, y, TileType::Corridor);
        }

        // Add some decorations
        map.set_tile(35, 10, TileType::Bones);
        map.set_tile(42, 12, TileType::BloodStain);
        map.set_tile(55, 30, TileType::Rubble);
        map.set_tile(45, 35, TileType::Brazier);

        // Set start and exit
        map.start_pos = Position::new(10, 10);
        map.exit_pos = Some(Position::new(50, 32));

        map
    }
}

impl Map {
    /// Get all walkable positions (for spawning)
    pub fn get_walkable_positions(&self) -> Vec<Position> {
        self.tiles
            .iter()
            .enumerate()
            .filter(|(_, tile)| tile.is_walkable())
            .map(|(idx, _)| {
                let (x, y) = self.idx_to_xy(idx);
                Position::new(x, y)
            })
            .collect()
    }

    /// Get valid spawn positions (walkable, not too close to start)
    pub fn get_spawn_positions(&self, min_dist_from_start: i32) -> Vec<Position> {
        self.get_walkable_positions()
            .into_iter()
            .filter(|pos| pos.chebyshev_distance(&self.start_pos) >= min_dist_from_start)
            .collect()
    }

    /// Check if a position is in a narrow passage (would block movement if occupied)
    /// A narrow passage is:
    /// - A corridor tile type
    /// - Any tile with only 1-2 walkable cardinal neighbors (chokepoint)
    /// - Any tile where blocking it would prevent passage between neighbors
    pub fn is_narrow_passage(&self, pos: Position) -> bool {
        // Check if tile is a corridor type
        if let Some(tile) = self.get_tile(pos.x, pos.y) {
            if tile.tile_type == TileType::Corridor {
                return true;
            }
        }

        // Count walkable neighbors in cardinal directions
        let cardinal = [
            (pos.x - 1, pos.y),  // West
            (pos.x + 1, pos.y),  // East
            (pos.x, pos.y - 1),  // North
            (pos.x, pos.y + 1),  // South
        ];

        let walkable_cardinal: Vec<(i32, i32)> = cardinal.iter()
            .filter(|(x, y)| {
                self.get_tile(*x, *y)
                    .map(|t| t.is_walkable())
                    .unwrap_or(false)
            })
            .copied()
            .collect();

        // Dead end or very narrow - definitely a chokepoint
        if walkable_cardinal.len() <= 1 {
            return true;
        }

        // If only 2 walkable neighbors and they're opposite, it's a narrow passage
        if walkable_cardinal.len() == 2 {
            let (x1, y1) = walkable_cardinal[0];
            let (x2, y2) = walkable_cardinal[1];
            // Check if they're on opposite sides (straight line through)
            let is_horizontal = y1 == y2 && y1 == pos.y;
            let is_vertical = x1 == x2 && x1 == pos.x;
            if is_horizontal || is_vertical {
                return true;
            }
        }

        // Also check diagonal neighbors - if tile has few total neighbors, it might be important
        let diagonals = [
            (pos.x - 1, pos.y - 1),
            (pos.x + 1, pos.y - 1),
            (pos.x - 1, pos.y + 1),
            (pos.x + 1, pos.y + 1),
        ];

        let walkable_diag_count = diagonals.iter()
            .filter(|(x, y)| {
                self.get_tile(*x, *y)
                    .map(|t| t.is_walkable())
                    .unwrap_or(false)
            })
            .count();

        // If very few total neighbors, this is a cramped space - avoid spawning here
        let total_neighbors = walkable_cardinal.len() + walkable_diag_count;
        if total_neighbors <= 3 {
            return true;
        }

        false
    }

    /// Get spawn positions suitable for NPCs (not in narrow passages)
    pub fn get_npc_spawn_positions(&self, min_dist_from_start: i32) -> Vec<Position> {
        self.get_walkable_positions()
            .into_iter()
            .filter(|pos| {
                pos.chebyshev_distance(&self.start_pos) >= min_dist_from_start
                    && !self.is_narrow_passage(*pos)
            })
            .collect()
    }
}

impl Biome {
    /// Get the biome name for display
    pub fn name(&self) -> &'static str {
        match self {
            Biome::SunkenCatacombs => "Sunken Catacombs",
            Biome::BleedingCrypts => "Bleeding Crypts",
            Biome::HollowCathedral => "Hollow Cathedral",
            Biome::TheAbyss => "The Abyss",
        }
    }

    /// Get ambient color tint for the biome
    pub fn ambient_color(&self) -> (u8, u8, u8) {
        match self {
            Biome::SunkenCatacombs => (30, 25, 20),
            Biome::BleedingCrypts => (40, 15, 15),
            Biome::HollowCathedral => (25, 25, 35),
            Biome::TheAbyss => (10, 10, 20),
        }
    }
}
