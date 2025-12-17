//! Cave generator using cellular automata
//!
//! Creates organic, natural-looking cave systems.

use rand::Rng;
use rand::rngs::StdRng;
use crate::ecs::Position;
use crate::world::{Map, Biome, TileType};

/// Generate a cave map using cellular automata
pub fn generate_caves(rng: &mut StdRng, floor: u32, biome: Biome) -> Map {
    let width = 80;
    let height = 50;
    let mut map = Map::new(width, height, floor, biome);

    // Initial random fill
    let fill_probability = 0.45;
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            if rng.gen_bool(fill_probability) {
                map.set_tile(x, y, TileType::Floor);
            }
        }
    }

    // Run cellular automata iterations
    for _ in 0..5 {
        let mut new_tiles = map.tiles.clone();

        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let wall_count = count_neighbors(&map, x, y);
                let idx = map.xy_to_idx(x, y);

                if wall_count > 4 {
                    new_tiles[idx].tile_type = TileType::Wall;
                } else if wall_count < 4 {
                    new_tiles[idx].tile_type = TileType::Floor;
                }
            }
        }

        map.tiles = new_tiles;
    }

    // Find connected regions and keep the largest
    ensure_connectivity(&mut map, rng);

    // Place start and exit
    place_start_and_exit(&mut map, rng);

    // Add decorations
    add_cave_decorations(rng, &mut map, biome);

    // Add shrines (multiple, different types)
    add_cave_shrines(rng, &mut map, floor);

    map
}

/// Count wall neighbors (8-directional)
fn count_neighbors(map: &Map, x: i32, y: i32) -> i32 {
    let mut count = 0;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x + dx;
            let ny = y + dy;
            if !map.in_bounds(nx, ny) || !map.is_walkable(nx, ny) {
                count += 1;
            }
        }
    }
    count
}

/// Ensure the cave is fully connected using flood fill
fn ensure_connectivity(map: &mut Map, rng: &mut StdRng) {
    // Find all floor tiles
    let mut floor_tiles: Vec<Position> = Vec::new();
    for y in 0..map.height {
        for x in 0..map.width {
            if map.is_walkable(x, y) {
                floor_tiles.push(Position::new(x, y));
            }
        }
    }

    if floor_tiles.is_empty() {
        // Fallback: create a simple cave
        for y in 10..40 {
            for x in 10..70 {
                map.set_tile(x, y, TileType::Floor);
            }
        }
        return;
    }

    // Flood fill from a random floor tile
    let start = floor_tiles[rng.gen_range(0..floor_tiles.len())];
    let mut visited = vec![false; (map.width * map.height) as usize];
    let mut stack = vec![start];
    let mut connected = Vec::new();

    while let Some(pos) = stack.pop() {
        let idx = map.xy_to_idx(pos.x, pos.y);
        if visited[idx] {
            continue;
        }
        visited[idx] = true;
        connected.push(pos);

        // Check neighbors
        for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
            let nx = pos.x + dx;
            let ny = pos.y + dy;
            if map.in_bounds(nx, ny) && map.is_walkable(nx, ny) {
                let nidx = map.xy_to_idx(nx, ny);
                if !visited[nidx] {
                    stack.push(Position::new(nx, ny));
                }
            }
        }
    }

    // If too few tiles are connected, carve tunnels to other regions
    if connected.len() < (floor_tiles.len() / 2) {
        // Find disconnected floor tiles and carve paths to them
        for tile in &floor_tiles {
            let idx = map.xy_to_idx(tile.x, tile.y);
            if !visited[idx] && map.is_walkable(tile.x, tile.y) {
                // Carve a path from this tile to the nearest connected tile
                if let Some(target) = connected.iter().min_by_key(|t| t.distance(tile)) {
                    carve_tunnel(map, *tile, *target);
                }
            }
        }
    }
}

/// Carve a tunnel between two points
fn carve_tunnel(map: &mut Map, from: Position, to: Position) {
    let mut x = from.x;
    let mut y = from.y;

    while x != to.x || y != to.y {
        if x < to.x {
            x += 1;
        } else if x > to.x {
            x -= 1;
        }

        map.set_tile(x, y, TileType::Corridor);

        if y < to.y {
            y += 1;
        } else if y > to.y {
            y -= 1;
        }

        map.set_tile(x, y, TileType::Corridor);
    }
}

/// Place start and exit positions
fn place_start_and_exit(map: &mut Map, rng: &mut StdRng) {
    let mut floor_tiles: Vec<Position> = Vec::new();
    for y in 0..map.height {
        for x in 0..map.width {
            if map.is_walkable(x, y) {
                floor_tiles.push(Position::new(x, y));
            }
        }
    }

    if floor_tiles.len() < 2 {
        map.start_pos = Position::new(map.width / 2, map.height / 2);
        return;
    }

    // Place start
    let start_idx = rng.gen_range(0..floor_tiles.len());
    map.start_pos = floor_tiles[start_idx];

    // Place exit far from start
    let exit = floor_tiles
        .iter()
        .max_by_key(|t| t.distance(&map.start_pos))
        .copied()
        .unwrap_or(map.start_pos);

    map.set_tile(exit.x, exit.y, TileType::StairsDown);
    map.exit_pos = Some(exit);
}

/// Add multiple shrines to the cave (not too close to start, exit, or each other)
fn add_cave_shrines(rng: &mut StdRng, map: &mut Map, floor: u32) {
    use rand::seq::SliceRandom;

    // Find floor tiles that are a good distance from start and exit
    let min_dist = 10;
    let min_shrine_dist = 15; // Minimum distance between shrines
    let mut candidates: Vec<Position> = Vec::new();

    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            if !map.is_walkable(x, y) {
                continue;
            }
            let pos = Position::new(x, y);
            let dist_from_start = pos.distance(&map.start_pos);
            let dist_from_exit = map.exit_pos
                .map(|e| pos.distance(&e))
                .unwrap_or(100);

            // Also check it's not in a narrow passage
            if dist_from_start >= min_dist && dist_from_exit >= min_dist && !map.is_narrow_passage(pos) {
                candidates.push(pos);
            }
        }
    }

    if candidates.is_empty() {
        return;
    }

    // Determine how many shrines (1-2 for caves, they're more open)
    let max_shrines = if candidates.len() > 50 { 2 } else { 1 };

    // Available shrine types (unique only)
    let mut available_types = vec![
        TileType::ShrineRest,
        TileType::ShrineSkill,
    ];
    // Enchanting shrines only appear on floor 5+ (players need gold first)
    if floor >= 5 {
        available_types.push(TileType::ShrineEnchant);
    }
    // Add corruption shrine chance on later floors
    if floor >= 3 && rng.gen_bool(0.3 + (floor as f64 * 0.02).min(0.3)) {
        available_types.push(TileType::ShrineCorruption);
    }
    available_types.shuffle(rng);

    let mut placed_positions: Vec<Position> = Vec::new();
    let mut shrines_placed = 0;

    // Shuffle candidates
    let mut shuffled_candidates = candidates.clone();
    shuffled_candidates.shuffle(rng);

    for pos in shuffled_candidates {
        if shrines_placed >= max_shrines || shrines_placed >= available_types.len() {
            break;
        }

        // Check distance from other shrines
        let too_close = placed_positions.iter().any(|p| pos.distance(p) < min_shrine_dist);
        if too_close {
            continue;
        }

        let shrine_type = available_types[shrines_placed];
        map.set_tile(pos.x, pos.y, shrine_type);
        placed_positions.push(pos);
        shrines_placed += 1;
    }
}

/// Check if a position is protected (start or exit)
fn is_protected_position(map: &Map, x: i32, y: i32) -> bool {
    let pos = Position::new(x, y);
    pos == map.start_pos || map.exit_pos == Some(pos)
}

/// Add biome-appropriate decorations to caves
fn add_cave_decorations(rng: &mut StdRng, map: &mut Map, biome: Biome) {
    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            if !map.is_walkable(x, y) {
                continue;
            }

            // Don't overwrite start or exit positions
            if is_protected_position(map, x, y) {
                continue;
            }

            match biome {
                Biome::BleedingCrypts => {
                    if rng.gen_bool(0.03) {
                        map.set_tile(x, y, TileType::BloodStain);
                    }
                    if rng.gen_bool(0.01) {
                        map.set_tile(x, y, TileType::Torch);
                    }
                }
                Biome::TheAbyss => {
                    if rng.gen_bool(0.02) {
                        map.set_tile(x, y, TileType::Rubble);
                    }
                }
                _ => {
                    if rng.gen_bool(0.01) {
                        map.set_tile(x, y, TileType::Torch);
                    }
                }
            }
        }
    }
}
