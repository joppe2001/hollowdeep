//! Room and corridor dungeon generator
//!
//! Classic roguelike dungeon with rectangular rooms connected by corridors.

use rand::Rng;
use rand::rngs::StdRng;
use crate::ecs::Position;
use crate::world::{Map, Biome, TileType};

/// A rectangular room
#[derive(Debug, Clone)]
struct Room {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Room {
    fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x1: x,
            y1: y,
            x2: x + width,
            y2: y + height,
        }
    }

    fn center(&self) -> Position {
        Position::new((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }

    fn intersects(&self, other: &Room) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }
}

/// Generate a dungeon with rooms and corridors
pub fn generate_dungeon(rng: &mut StdRng, floor: u32, biome: Biome) -> Map {
    let width = 80;
    let height = 50;
    let mut map = Map::new(width, height, floor, biome);

    let min_room_size = 4;  // Reduced from 6 for tighter spaces
    let max_room_size = 8;  // Reduced from 12 for smaller rooms
    let max_rooms = 18 + (floor as i32 / 2).min(10); // More rooms since they're smaller

    let mut rooms: Vec<Room> = Vec::new();

    for _ in 0..100 {
        // Try up to 100 times to place rooms
        if rooms.len() >= max_rooms as usize {
            break;
        }

        let w = rng.gen_range(min_room_size..=max_room_size);
        let h = rng.gen_range(min_room_size..=max_room_size);
        let x = rng.gen_range(1..width - w - 1);
        let y = rng.gen_range(1..height - h - 1);

        let new_room = Room::new(x, y, w, h);

        // Check for overlaps
        let overlaps = rooms.iter().any(|r| new_room.intersects(r));

        if !overlaps {
            carve_room(&mut map, &new_room);

            if !rooms.is_empty() {
                // Connect to previous room
                let prev_center = rooms.last().unwrap().center();
                let new_center = new_room.center();

                if rng.gen_bool(0.5) {
                    carve_h_corridor(&mut map, prev_center.x, new_center.x, prev_center.y);
                    carve_v_corridor(&mut map, prev_center.y, new_center.y, new_center.x);
                    // Add corner fill to prevent diagonal-only passage
                    fill_corner(&mut map, new_center.x, prev_center.y);
                } else {
                    carve_v_corridor(&mut map, prev_center.y, new_center.y, prev_center.x);
                    carve_h_corridor(&mut map, prev_center.x, new_center.x, new_center.y);
                    // Add corner fill to prevent diagonal-only passage
                    fill_corner(&mut map, prev_center.x, new_center.y);
                }
            }

            rooms.push(new_room);
        }
    }

    // Set start position in first room
    if let Some(first) = rooms.first() {
        map.start_pos = first.center();
    }

    // Set exit in last room
    if let Some(last) = rooms.last() {
        let exit = last.center();
        map.set_tile(exit.x, exit.y, TileType::StairsDown);
        map.exit_pos = Some(exit);
    }

    // Add elite rooms (risk/reward areas) - not first or last room
    if rooms.len() > 3 {
        // 30-40% of rooms become elite on higher floors
        let elite_chance = 0.20 + (floor as f64 * 0.02).min(0.20);
        for room in rooms.iter().skip(1).rev().skip(1) {
            if rng.gen_bool(elite_chance) {
                map.add_elite_room(room.center());
                // Mark elite room with special floor (slightly different color)
                mark_elite_room(&mut map, room);
            }
        }
    }

    // Add decorations
    add_decorations(rng, &mut map, &rooms, biome);

    // Add shrines (multiple, different types, not in first or last room)
    if rooms.len() > 2 {
        add_shrines(rng, &mut map, &rooms, floor);
    }

    map
}

/// Mark a room as elite with special floor markers
fn mark_elite_room(map: &mut Map, room: &Room) {
    // Mark corners of elite rooms
    let corners = [
        (room.x1 + 1, room.y1 + 1),
        (room.x2 - 1, room.y1 + 1),
        (room.x1 + 1, room.y2 - 1),
        (room.x2 - 1, room.y2 - 1),
    ];
    for (x, y) in corners {
        if let Some(tile) = map.get_tile_mut(x, y) {
            // Mark tile as special (for rendering)
            tile.glyph = Some('✧'); // Elite marker
        }
    }
}

/// Carve out a room
fn carve_room(map: &mut Map, room: &Room) {
    for y in room.y1 + 1..room.y2 {
        for x in room.x1 + 1..room.x2 {
            map.set_tile(x, y, TileType::Floor);
        }
    }
}

/// Carve a horizontal corridor (2 tiles wide to avoid diagonal-only movement)
fn carve_h_corridor(map: &mut Map, x1: i32, x2: i32, y: i32) {
    let (start, end) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
    for x in start..=end {
        map.set_tile(x, y, TileType::Corridor);
        // Add second row for width (check bounds)
        if y + 1 < map.height as i32 {
            map.set_tile(x, y + 1, TileType::Corridor);
        }
    }
}

/// Carve a vertical corridor (2 tiles wide to avoid diagonal-only movement)
fn carve_v_corridor(map: &mut Map, y1: i32, y2: i32, x: i32) {
    let (start, end) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
    for y in start..=end {
        map.set_tile(x, y, TileType::Corridor);
        // Add second column for width (check bounds)
        if x + 1 < map.width as i32 {
            map.set_tile(x + 1, y, TileType::Corridor);
        }
    }
}

/// Fill corner intersection to prevent diagonal-only passage
/// Creates a 3x3 area at the corner point where corridors meet
fn fill_corner(map: &mut Map, x: i32, y: i32) {
    // Carve a 3x3 area around the corner point
    for dy in -1..=1 {
        for dx in -1..=1 {
            let nx = x + dx;
            let ny = y + dy;
            if nx > 0 && nx < map.width as i32 - 1 && ny > 0 && ny < map.height as i32 - 1 {
                map.set_tile(nx, ny, TileType::Corridor);
            }
        }
    }
}

/// Get all valid shrine positions in a room (walkable tiles, not center of doorways, not start/exit)
fn get_valid_shrine_positions(map: &Map, room: &Room) -> Vec<Position> {
    let mut positions = Vec::new();
    // Check tiles in the inner area of the room (avoiding edges)
    for y in room.y1 + 2..room.y2 - 1 {
        for x in room.x1 + 2..room.x2 - 1 {
            let pos = Position::new(x, y);
            // Don't place shrines on start, exit, or narrow passages
            if map.is_walkable(x, y)
                && !map.is_narrow_passage(pos)
                && pos != map.start_pos
                && map.exit_pos != Some(pos)
            {
                positions.push(pos);
            }
        }
    }
    positions
}

/// Add multiple shrines to middle rooms (different types only)
fn add_shrines(rng: &mut StdRng, map: &mut Map, rooms: &[Room], floor: u32) {
    use rand::seq::SliceRandom;

    // Skip first and last rooms
    let middle_rooms: Vec<_> = rooms[1..rooms.len().saturating_sub(1)].to_vec();
    if middle_rooms.is_empty() {
        return;
    }

    // Determine how many shrines (1-3 based on floor and room count)
    let max_shrines = if middle_rooms.len() >= 6 {
        3
    } else if middle_rooms.len() >= 3 {
        2
    } else {
        1
    };

    // Available shrine types (we'll pick unique ones)
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

    // Shuffle rooms and shrine types
    let mut shuffled_rooms = middle_rooms.clone();
    shuffled_rooms.shuffle(rng);
    available_types.shuffle(rng);

    let mut shrines_placed = 0;
    let mut used_rooms: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for (i, room) in shuffled_rooms.iter().enumerate() {
        if shrines_placed >= max_shrines || shrines_placed >= available_types.len() {
            break;
        }

        // Find valid positions in this room
        let valid_positions = get_valid_shrine_positions(map, room);
        if valid_positions.is_empty() {
            continue;
        }

        // Pick a random valid position
        let shrine_pos = *valid_positions.choose(rng).unwrap();

        // Use the next unique shrine type
        let shrine_type = available_types[shrines_placed];

        // Check if this room is in an elite zone - override with corruption chance
        let is_elite = map.is_elite_zone(shrine_pos);
        let final_shrine_type = if is_elite && rng.gen_bool(0.4) && shrine_type != TileType::ShrineCorruption {
            TileType::ShrineCorruption
        } else {
            shrine_type
        };

        map.set_tile(shrine_pos.x, shrine_pos.y, final_shrine_type);
        used_rooms.insert(i);
        shrines_placed += 1;
    }
}

/// Check if a position is protected (start or exit)
fn is_protected_position(map: &Map, x: i32, y: i32) -> bool {
    let pos = Position::new(x, y);
    pos == map.start_pos || map.exit_pos == Some(pos)
}

/// Add biome-appropriate decorations
fn add_decorations(rng: &mut StdRng, map: &mut Map, rooms: &[Room], biome: Biome) {
    for room in rooms {
        // Add torches to some rooms
        if rng.gen_bool(0.6) {
            let x = rng.gen_range(room.x1 + 2..room.x2 - 1);
            let y = rng.gen_range(room.y1 + 2..room.y2 - 1);
            // Don't overwrite start or exit positions
            if !is_protected_position(map, x, y) {
                map.set_tile(x, y, TileType::Torch);
            }
        }

        // Biome-specific decorations
        match biome {
            Biome::SunkenCatacombs => {
                if rng.gen_bool(0.3) {
                    let x = rng.gen_range(room.x1 + 2..room.x2 - 1);
                    let y = rng.gen_range(room.y1 + 2..room.y2 - 1);
                    if !is_protected_position(map, x, y) {
                        map.set_tile(x, y, TileType::Bones);
                    }
                }
            }
            Biome::BleedingCrypts => {
                if rng.gen_bool(0.4) {
                    let x = rng.gen_range(room.x1 + 2..room.x2 - 1);
                    let y = rng.gen_range(room.y1 + 2..room.y2 - 1);
                    if !is_protected_position(map, x, y) {
                        map.set_tile(x, y, TileType::BloodStain);
                    }
                }
            }
            Biome::HollowCathedral => {
                if rng.gen_bool(0.5) {
                    let x = rng.gen_range(room.x1 + 2..room.x2 - 1);
                    let y = rng.gen_range(room.y1 + 2..room.y2 - 1);
                    if !is_protected_position(map, x, y) {
                        map.set_tile(x, y, TileType::Brazier);
                    }
                }
            }
            Biome::TheAbyss => {
                if rng.gen_bool(0.2) {
                    let x = rng.gen_range(room.x1 + 2..room.x2 - 1);
                    let y = rng.gen_range(room.y1 + 2..room.y2 - 1);
                    if !is_protected_position(map, x, y) {
                        map.set_tile(x, y, TileType::Rubble);
                    }
                }
            }
        }
    }
}
