//! Chest entity creation
//!
//! Chests spawn on floors and contain loot based on their rarity.

use hecs::{World, Entity};
use rand::Rng;

use crate::ecs::{Position, Renderable, Chest, ChestRarity};
use crate::items::{Item, loot};
use crate::world::Biome;

/// Roll a chest rarity based on floor depth
pub fn roll_chest_rarity(floor: u32, rng: &mut impl Rng) -> ChestRarity {
    // Use 1000 for finer granularity
    let roll = rng.gen_range(0..1000);

    // Floor tiers for progression
    let floor_tier = match floor {
        1..=5 => 0,
        6..=10 => 1,
        11..=15 => 2,
        16..=20 => 3,
        _ => 4,
    };

    // Legendary chests: 0% early, very rare even late game
    // Epic chests: 1% early, up to 10% late
    // Rare chests: 15% early, up to 30% late
    // Common: remainder

    let legendary_threshold = match floor_tier {
        0 => 1000,  // 0% - no legendary chests floors 1-5
        1 => 997,   // 0.3% - floors 6-10
        2 => 993,   // 0.7% - floors 11-15
        3 => 985,   // 1.5% - floors 16-20
        _ => 970,   // 3% - floors 21+
    };

    let epic_threshold = match floor_tier {
        0 => 990,   // 1% - early floors
        1 => 970,   // 3% - floors 6-10
        2 => 940,   // 6% - floors 11-15
        3 => 900,   // 10% - floors 16-20
        _ => 850,   // 15% - floors 21+
    };

    let rare_threshold = match floor_tier {
        0 => 850,   // 15% - early floors
        1 => 800,   // 20% - floors 6-10
        2 => 750,   // 25% - floors 11-15
        3 => 700,   // 30% - floors 16-20
        _ => 650,   // 35% - floors 21+
    };

    if roll >= legendary_threshold {
        ChestRarity::Legendary
    } else if roll >= epic_threshold {
        ChestRarity::Epic
    } else if roll >= rare_threshold {
        ChestRarity::Rare
    } else {
        ChestRarity::Common
    }
}

/// Spawn a chest at a position
pub fn spawn_chest(world: &mut World, pos: Position, rarity: ChestRarity) -> Entity {
    world.spawn((
        pos,
        Renderable::new(rarity.glyph(), rarity.color()).with_order(70),
        Chest { rarity, opened: false },
    ))
}

/// Spawn chests for a floor
pub fn spawn_chests_for_floor(
    world: &mut World,
    floor: u32,
    _biome: Biome,
    valid_positions: &[Position],
    rng: &mut impl Rng,
) -> Vec<Entity> {
    use rand::seq::SliceRandom;

    // Determine chest count based on floor
    let (min_chests, max_chests) = match floor {
        1 => (1, 2),      // Tutorial floor - fewer chests
        2..=5 => (2, 4),
        6..=10 => (3, 5),
        11..=15 => (3, 6),
        _ => (4, 7),
    };

    let count = rng.gen_range(min_chests..=max_chests);
    let count = count.min(valid_positions.len());

    // Choose random positions
    let mut positions = valid_positions.to_vec();
    positions.shuffle(rng);

    let mut spawned = Vec::with_capacity(count);

    for i in 0..count {
        let pos = positions[i];
        let rarity = roll_chest_rarity(floor, rng);
        let entity = spawn_chest(world, pos, rarity);
        spawned.push(entity);
    }

    spawned
}

/// Generate loot from a chest
pub fn generate_chest_loot(chest_rarity: ChestRarity, floor: u32, rng: &mut impl Rng) -> (Vec<Item>, u32) {
    let min_item_rarity = chest_rarity.min_item_rarity();
    let (min_items, max_items) = chest_rarity.item_count();
    let item_count = rng.gen_range(min_items..=max_items);

    let mut items = Vec::with_capacity(item_count);

    for _ in 0..item_count {
        // 60% weapon/armor, 40% consumable
        let item = if rng.gen_bool(0.6) {
            if rng.gen_bool(0.5) {
                loot::generate_weapon_with_min_rarity(floor, min_item_rarity, rng)
            } else {
                loot::generate_armor_with_min_rarity(floor, min_item_rarity, rng)
            }
        } else {
            loot::generate_consumable(rng)
        };
        items.push(item);
    }

    // Gold based on floor and chest rarity
    let base_gold = loot::generate_gold_drop(floor, rng);
    let gold = (base_gold as f32 * chest_rarity.gold_multiplier()) as u32;

    (items, gold)
}

/// Get chest at a position (if any)
pub fn get_chest_at(world: &World, pos: Position) -> Option<Entity> {
    for (entity, (chest_pos, chest)) in world.query::<(&Position, &Chest)>().iter() {
        if chest_pos.x == pos.x && chest_pos.y == pos.y && !chest.opened {
            return Some(entity);
        }
    }
    None
}

/// Mark a chest as opened
pub fn mark_chest_opened(world: &mut World, entity: Entity) {
    if let Ok(mut chest) = world.get::<&mut Chest>(entity) {
        chest.opened = true;
    }
    // Change visual to opened chest
    if let Ok(mut renderable) = world.get::<&mut Renderable>(entity) {
        renderable.glyph = '○'; // Opened chest glyph
        renderable.fg = (100, 100, 100); // Gray out
    }
}
