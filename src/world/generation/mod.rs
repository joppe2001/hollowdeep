//! Procedural map generation
//!
//! Different generators for various biomes.

pub mod rooms;
pub mod caves;
pub mod biomes;
pub mod templates;

pub use biomes::{BiomeConfig, HazardType};

use rand::Rng;
use rand::rngs::StdRng;
use super::{Map, Biome};

/// Generate a floor based on biome type
pub fn generate_floor(rng: &mut StdRng, floor: u32, biome: Biome) -> Map {
    let config = biome.config();

    // Use cave_factor to probabilistically choose generator
    // This creates variety within biomes
    let use_caves = rng.gen_bool(config.cave_factor as f64);

    let mut map = if use_caves {
        caves::generate_caves(rng, floor, biome)
    } else {
        rooms::generate_dungeon(rng, floor, biome)
    };

    // SAFETY: Ensure stairs always exist
    // If no exit was placed, find a valid position far from start
    ensure_stairs_exist(&mut map);

    // Add biome-specific hazards
    add_hazards(rng, &mut map, &config);

    // Add biome-specific decorations for visual variety
    add_biome_decorations(rng, &mut map, &config);

    // SAFETY: Double-check stairs weren't overwritten by hazards/decorations
    ensure_stairs_exist(&mut map);

    map
}

/// Ensure stairs down exist on the map
/// If exit_pos is None or the tile at exit_pos is not StairsDown, fix it
fn ensure_stairs_exist(map: &mut Map) {
    use super::TileType;
    use crate::ecs::Position;

    // Check if stairs already exist at exit_pos
    if let Some(exit) = map.exit_pos {
        if let Some(tile) = map.get_tile(exit.x, exit.y) {
            if tile.tile_type == TileType::StairsDown {
                return; // Stairs exist, all good
            }
        }
        // Exit pos is set but tile is wrong - restore it
        map.set_tile(exit.x, exit.y, TileType::StairsDown);
        return;
    }

    // No exit_pos set - find the best position for stairs
    // Collect all walkable floor tiles
    let mut floor_tiles: Vec<Position> = Vec::new();
    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            if map.is_walkable(x, y) {
                let pos = Position::new(x, y);
                // Don't place at start position
                if pos != map.start_pos {
                    floor_tiles.push(pos);
                }
            }
        }
    }

    if floor_tiles.is_empty() {
        // Extremely broken map - just place stairs near center
        let center = Position::new(map.width / 2, map.height / 2);
        map.set_tile(center.x, center.y, TileType::StairsDown);
        map.exit_pos = Some(center);
        return;
    }

    // Find the tile farthest from start
    let best_exit = floor_tiles
        .iter()
        .max_by_key(|t| t.distance(&map.start_pos))
        .copied()
        .unwrap_or(floor_tiles[0]);

    map.set_tile(best_exit.x, best_exit.y, TileType::StairsDown);
    map.exit_pos = Some(best_exit);
}

/// Add environmental hazards based on biome config
fn add_hazards(rng: &mut StdRng, map: &mut Map, config: &BiomeConfig) {
    use super::TileType;

    if config.hazard_chance <= 0.0 || config.primary_hazard == HazardType::None {
        return;
    }

    let hazard_tile = match config.primary_hazard {
        HazardType::Lava => TileType::Lava,
        HazardType::Pit => TileType::Pit,
        HazardType::Corruption => TileType::BloodStain,
        HazardType::None => return,
    };

    // Add hazards to random floor tiles (not start/exit)
    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let pos = crate::ecs::Position::new(x, y);

            // Skip start, exit, and non-walkable tiles
            if pos == map.start_pos {
                continue;
            }
            if Some(pos) == map.exit_pos {
                continue;
            }
            if !map.is_walkable(x, y) {
                continue;
            }

            // Random chance to place hazard
            if rng.gen_bool(config.hazard_chance as f64) {
                map.set_tile(x, y, hazard_tile);
            }
        }
    }
}

/// Add biome-specific decorations for visual variety
fn add_biome_decorations(rng: &mut StdRng, map: &mut Map, config: &BiomeConfig) {
    use rand::seq::SliceRandom;
    use super::TileType;

    if config.decorations.is_empty() || config.decoration_density <= 0.0 {
        return;
    }

    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let pos = crate::ecs::Position::new(x, y);

            // Only decorate floor tiles, not special tiles
            if let Some(tile) = map.get_tile(x, y) {
                if tile.tile_type != TileType::Floor && tile.tile_type != TileType::Corridor {
                    continue;
                }
            } else {
                continue;
            }

            // Skip protected positions
            if pos == map.start_pos || Some(pos) == map.exit_pos {
                continue;
            }

            // Random chance to place decoration
            if rng.gen_bool(config.decoration_density as f64) {
                if let Some(decoration) = config.decorations.choose(rng) {
                    map.set_tile(x, y, *decoration);
                }
            }
        }
    }
}

/// Get the biome for a given floor number
pub fn biome_for_floor(floor: u32) -> Biome {
    match floor {
        1..=5 => Biome::SunkenCatacombs,
        6..=10 => Biome::BleedingCrypts,
        11..=15 => Biome::HollowCathedral,
        _ => Biome::TheAbyss,
    }
}
