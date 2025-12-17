//! NPC entity creation
//!
//! NPCs are non-hostile entities that provide services like shops, healing, and quests.

use hecs::{World, Entity};
use rand::Rng;
use rand::rngs::StdRng;
use crate::ecs::{Position, Renderable};
use crate::items::{Item, ItemId, Rarity, generate_weapon, generate_armor};
use crate::items::item::templates;
use crate::world::Biome;

/// Types of NPCs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NpcType {
    /// Sells weapons, armor, consumables
    Merchant,
    /// Repairs and enchants equipment
    Blacksmith,
    /// Heals and removes curses
    Healer,
    /// Provides lore and hints
    Storyteller,
    /// Mysterious figure with risky trades
    Collector,
}

impl NpcType {
    pub fn name(&self) -> &'static str {
        match self {
            NpcType::Merchant => "Wandering Merchant",
            NpcType::Blacksmith => "Traveling Blacksmith",
            NpcType::Healer => "Field Healer",
            NpcType::Storyteller => "Storyteller",
            NpcType::Collector => "Strange Collector",
        }
    }

    pub fn glyph(&self) -> char {
        match self {
            NpcType::Merchant => '$',
            NpcType::Blacksmith => '&',
            NpcType::Healer => '+',
            NpcType::Storyteller => '?',
            NpcType::Collector => '%',
        }
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            NpcType::Merchant => (255, 215, 0),   // Gold
            NpcType::Blacksmith => (180, 100, 60), // Bronze
            NpcType::Healer => (100, 255, 100),    // Green
            NpcType::Storyteller => (180, 180, 255), // Light blue
            NpcType::Collector => (200, 100, 200), // Purple
        }
    }

    pub fn greeting(&self) -> &'static str {
        match self {
            NpcType::Merchant => "Care to see my wares, traveler?",
            NpcType::Blacksmith => "I can mend your gear, for a price.",
            NpcType::Healer => "Let me tend to your wounds.",
            NpcType::Storyteller => "Ah, another soul braving the depths...",
            NpcType::Collector => "I seek... unusual items. Perhaps we can trade.",
        }
    }

    pub fn biome_affinity(&self, biome: Biome) -> f32 {
        match (self, biome) {
            // Merchants appear everywhere but more in early areas
            (NpcType::Merchant, Biome::SunkenCatacombs) => 1.0,
            (NpcType::Merchant, _) => 0.7,
            // Blacksmiths in structured areas
            (NpcType::Blacksmith, Biome::SunkenCatacombs) => 0.8,
            (NpcType::Blacksmith, Biome::HollowCathedral) => 0.9,
            (NpcType::Blacksmith, _) => 0.5,
            // Healers more common in dangerous areas
            (NpcType::Healer, Biome::BleedingCrypts) => 0.9,
            (NpcType::Healer, Biome::TheAbyss) => 1.0,
            (NpcType::Healer, _) => 0.6,
            // Storytellers everywhere
            (NpcType::Storyteller, _) => 0.5,
            // Collectors in weird places
            (NpcType::Collector, Biome::BleedingCrypts) => 0.8,
            (NpcType::Collector, Biome::TheAbyss) => 1.0,
            (NpcType::Collector, _) => 0.3,
        }
    }
}

/// NPC component for entities
#[derive(Debug, Clone)]
pub struct NpcComponent {
    pub npc_type: NpcType,
    /// Shop inventory (for merchants)
    pub shop_items: Vec<ShopItem>,
    /// Gold the NPC has for buying
    pub gold: u32,
    /// Has the player interacted with this NPC?
    pub interacted: bool,
    /// Unique dialogue state
    pub dialogue_state: u8,
}

/// Item for sale in a shop
#[derive(Debug, Clone)]
pub struct ShopItem {
    pub item: Item,
    pub buy_price: u32,
    pub sell_price: u32,
}

impl ShopItem {
    pub fn new(item: Item) -> Self {
        let base_value = item.value;
        let rarity_mult = match item.rarity {
            Rarity::Common => 1.0,
            Rarity::Uncommon => 1.5,
            Rarity::Rare => 2.5,
            Rarity::Epic => 4.0,
            Rarity::Legendary => 8.0,
            Rarity::Mythic => 15.0, // Extremely valuable
        };

        let buy_price = ((base_value as f32) * rarity_mult * 1.2) as u32;
        let sell_price = ((base_value as f32) * 0.4) as u32;

        Self {
            item,
            buy_price: buy_price.max(5),
            sell_price: sell_price.max(1),
        }
    }
}

/// Merchant specialization types for inventory variety
#[derive(Debug, Clone, Copy)]
pub enum MerchantType {
    GeneralStore,   // Mix of everything
    Weaponsmith,    // Mostly weapons
    Armorer,        // Mostly armor
    Alchemist,      // Mostly consumables
}

/// Generate shop inventory for a merchant
pub fn generate_shop_inventory(
    rng: &mut StdRng,
    floor: u32,
    biome: Biome,
    item_id_counter: &mut u64,
) -> Vec<ShopItem> {
    let mut items = Vec::new();

    // Merchant type is determined by floor number to ensure variety
    // Each floor gets a different merchant type in a cycle
    let merchant_type = match floor % 4 {
        0 => MerchantType::GeneralStore,
        1 => MerchantType::Weaponsmith,
        2 => MerchantType::Armorer,
        _ => MerchantType::Alchemist,
    };

    // Number of items scales with floor
    let num_items = 4 + (floor / 3).min(4) as usize;

    for _ in 0..num_items {
        let item = match merchant_type {
            MerchantType::GeneralStore => {
                // Mix of everything
                match rng.gen_range(0..10) {
                    0..=3 => generate_consumable_for_shop(rng, *item_id_counter),
                    4..=6 => generate_weapon(floor, rng),
                    _ => generate_armor(floor, rng),
                }
            }
            MerchantType::Weaponsmith => {
                // 70% weapons, 20% armor, 10% consumables
                match rng.gen_range(0..10) {
                    0..=6 => generate_weapon(floor, rng),
                    7..=8 => generate_armor(floor, rng),
                    _ => generate_consumable_for_shop(rng, *item_id_counter),
                }
            }
            MerchantType::Armorer => {
                // 70% armor, 20% weapons, 10% consumables
                match rng.gen_range(0..10) {
                    0..=6 => generate_armor(floor, rng),
                    7..=8 => generate_weapon(floor, rng),
                    _ => generate_consumable_for_shop(rng, *item_id_counter),
                }
            }
            MerchantType::Alchemist => {
                // 80% consumables, 10% each weapons/armor
                match rng.gen_range(0..10) {
                    0..=7 => generate_consumable_for_shop(rng, *item_id_counter),
                    8 => generate_weapon(floor, rng),
                    _ => generate_armor(floor, rng),
                }
            }
        };
        *item_id_counter += 1;
        items.push(ShopItem::new(item));
    }

    // Always have at least one health potion
    items.push(ShopItem::new(templates::health_potion(*item_id_counter)));
    *item_id_counter += 1;

    // Alchemists have more potions
    if matches!(merchant_type, MerchantType::Alchemist) {
        items.push(ShopItem::new(templates::mana_potion(*item_id_counter)));
        *item_id_counter += 1;
        items.push(ShopItem::new(templates::health_potion(*item_id_counter)));
        *item_id_counter += 1;
    }

    // Biome-specific items - alternate based on floor to ensure variety
    // Odd floors get one item, even floors get the other (within same biome)
    let floor_is_even = floor % 2 == 0;

    match biome {
        Biome::BleedingCrypts => {
            if floor_is_even {
                items.push(ShopItem::new(templates::cultist_robe(*item_id_counter)));
                *item_id_counter += 1;
            } else {
                items.push(ShopItem::new(templates::cultist_dagger(*item_id_counter)));
                *item_id_counter += 1;
            }
            // Random chance for second biome item
            if rng.gen_bool(0.25) {
                if floor_is_even {
                    items.push(ShopItem::new(templates::cultist_dagger(*item_id_counter)));
                } else {
                    items.push(ShopItem::new(templates::cultist_robe(*item_id_counter)));
                }
                *item_id_counter += 1;
            }
        }
        Biome::HollowCathedral => {
            if floor_is_even {
                items.push(ShopItem::new(templates::knight_helm(*item_id_counter)));
                *item_id_counter += 1;
            } else {
                items.push(ShopItem::new(templates::knight_plate(*item_id_counter)));
                *item_id_counter += 1;
            }
            // Random chance for second biome item
            if rng.gen_bool(0.25) {
                if floor_is_even {
                    items.push(ShopItem::new(templates::knight_plate(*item_id_counter)));
                } else {
                    items.push(ShopItem::new(templates::knight_helm(*item_id_counter)));
                }
                *item_id_counter += 1;
            }
        }
        Biome::TheAbyss => {
            if floor_is_even {
                items.push(ShopItem::new(templates::corrupted_gauntlets(*item_id_counter)));
                *item_id_counter += 1;
            } else {
                items.push(ShopItem::new(templates::shadow_cloak(*item_id_counter)));
                *item_id_counter += 1;
            }
            // Random chance for second biome item
            if rng.gen_bool(0.25) {
                if floor_is_even {
                    items.push(ShopItem::new(templates::shadow_cloak(*item_id_counter)));
                } else {
                    items.push(ShopItem::new(templates::corrupted_gauntlets(*item_id_counter)));
                }
                *item_id_counter += 1;
            }
        }
        Biome::SunkenCatacombs => {
            // Basic gear - alternate between weapon and armor focus
            if floor_is_even {
                items.push(ShopItem::new(templates::iron_sword(*item_id_counter)));
                *item_id_counter += 1;
            } else {
                items.push(ShopItem::new(templates::leather_armor(*item_id_counter)));
                *item_id_counter += 1;
            }
            // Random chance for second item
            if rng.gen_bool(0.3) {
                if floor_is_even {
                    items.push(ShopItem::new(templates::leather_armor(*item_id_counter)));
                } else {
                    items.push(ShopItem::new(templates::iron_sword(*item_id_counter)));
                }
                *item_id_counter += 1;
            }
        }
    }

    // Elemental weapons based on floor - each floor focuses on different elements
    // This creates variety: floor 3 might have fire, floor 4 frost, floor 5 venom, etc.
    let elemental_offset = floor % 3;

    if floor >= 3 {
        match elemental_offset {
            0 => {
                items.push(ShopItem::new(templates::flame_sword(*item_id_counter)));
                *item_id_counter += 1;
            }
            1 => {
                items.push(ShopItem::new(templates::frost_dagger(*item_id_counter)));
                *item_id_counter += 1;
            }
            _ => {
                if floor >= 5 {
                    items.push(ShopItem::new(templates::venom_blade(*item_id_counter)));
                    *item_id_counter += 1;
                }
            }
        }
    }

    // Higher floors may have a second elemental weapon
    if floor >= 8 && rng.gen_bool(0.4) {
        let second_element = (elemental_offset + 1) % 3;
        match second_element {
            0 => {
                items.push(ShopItem::new(templates::flame_sword(*item_id_counter)));
                *item_id_counter += 1;
            }
            1 => {
                items.push(ShopItem::new(templates::frost_dagger(*item_id_counter)));
                *item_id_counter += 1;
            }
            _ => {
                items.push(ShopItem::new(templates::venom_blade(*item_id_counter)));
                *item_id_counter += 1;
            }
        }
    }

    items
}

fn generate_consumable_for_shop(rng: &mut StdRng, id: ItemId) -> Item {
    // More variety in consumables based on random roll
    match rng.gen_range(0..10) {
        0..=4 => templates::health_potion(id),  // 50% health potions
        5..=7 => templates::mana_potion(id),    // 30% mana potions
        _ => templates::health_potion(id),      // 20% fallback (could add more potion types later)
    }
}

/// Spawn an NPC at the given position
pub fn spawn_npc(
    world: &mut World,
    npc_type: NpcType,
    pos: Position,
    rng: &mut StdRng,
    floor: u32,
    biome: Biome,
    item_id_counter: &mut u64,
) -> Entity {
    let shop_items = if matches!(npc_type, NpcType::Merchant) {
        generate_shop_inventory(rng, floor, biome, item_id_counter)
    } else {
        Vec::new()
    };

    let npc = NpcComponent {
        npc_type,
        shop_items,
        gold: 200 + floor * 50,
        interacted: false,
        dialogue_state: 0,
    };

    let color = npc_type.color();
    let renderable = Renderable {
        glyph: npc_type.glyph(),
        fg: color,
        bg: None,
        render_order: 4, // Above items, below player
    };

    world.spawn((
        pos,
        npc,
        NpcMarker,
        renderable,
    ))
}

/// Marker component for NPCs
#[derive(Debug, Clone, Copy)]
pub struct NpcMarker;

/// Choose which NPC type to spawn based on biome and randomness
pub fn choose_npc_for_biome(rng: &mut StdRng, biome: Biome) -> NpcType {
    let types = [
        NpcType::Merchant,
        NpcType::Blacksmith,
        NpcType::Healer,
        NpcType::Storyteller,
        NpcType::Collector,
    ];

    // Weight by affinity
    let weights: Vec<f32> = types.iter()
        .map(|t| t.biome_affinity(biome))
        .collect();

    let total: f32 = weights.iter().sum();
    let mut roll = rng.gen::<f32>() * total;

    for (i, &weight) in weights.iter().enumerate() {
        roll -= weight;
        if roll <= 0.0 {
            return types[i];
        }
    }

    NpcType::Merchant // Default
}

/// Spawn NPCs for a floor (multiple different types, no duplicates)
pub fn spawn_npcs_for_floor(
    world: &mut World,
    biome: Biome,
    floor: u32,
    spawn_positions: &[Position],
    rng: &mut StdRng,
    item_id_counter: &mut u64,
) -> Vec<Entity> {
    use rand::seq::SliceRandom;

    let mut npcs = Vec::new();

    if spawn_positions.is_empty() {
        return npcs;
    }

    // 80% chance of having NPCs on the floor (increased since we can have multiple)
    if !rng.gen_bool(0.8) {
        return npcs;
    }

    // Determine how many NPCs (1-3 based on floor and available positions)
    let max_npcs = if spawn_positions.len() >= 6 && floor >= 3 {
        3
    } else if spawn_positions.len() >= 3 {
        2
    } else {
        1
    };

    // Get all NPC types weighted by biome affinity
    let all_types = [
        NpcType::Merchant,
        NpcType::Blacksmith,
        NpcType::Healer,
        NpcType::Storyteller,
        NpcType::Collector,
    ];

    // Sort by biome affinity (higher affinity = more likely to be picked first)
    let mut weighted_types: Vec<(NpcType, f32)> = all_types.iter()
        .map(|&t| (t, t.biome_affinity(biome)))
        .collect();
    weighted_types.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Shuffle positions
    let mut available_positions = spawn_positions.to_vec();
    available_positions.shuffle(rng);

    let mut spawned_types: std::collections::HashSet<NpcType> = std::collections::HashSet::new();
    let min_npc_dist = 10; // Minimum distance between NPCs

    for (npc_type, affinity) in weighted_types {
        if npcs.len() >= max_npcs {
            break;
        }

        // Check if we should spawn this NPC type (based on affinity chance)
        if !rng.gen_bool(affinity as f64 * 0.8) {
            continue;
        }

        // Don't spawn duplicate types
        if spawned_types.contains(&npc_type) {
            continue;
        }

        // Find a position that's not too close to other NPCs
        let mut found_pos = None;
        for pos in &available_positions {
            let too_close = npcs.iter().any(|&npc_entity| {
                if let Ok(npc_pos) = world.get::<&Position>(npc_entity) {
                    pos.distance(&npc_pos) < min_npc_dist
                } else {
                    false
                }
            });

            if !too_close {
                found_pos = Some(*pos);
                break;
            }
        }

        if let Some(pos) = found_pos {
            let npc = spawn_npc(world, npc_type, pos, rng, floor, biome, item_id_counter);
            npcs.push(npc);
            spawned_types.insert(npc_type);
            // Remove this position from available
            available_positions.retain(|p| p != &pos);
            log::info!("Spawned {} on floor {}", npc_type.name(), floor);
        }
    }

    npcs
}

/// Get NPC at position
pub fn get_npc_at(world: &World, pos: Position) -> Option<Entity> {
    for (entity, (npc_pos, _)) in world.query::<(&Position, &NpcMarker)>().iter() {
        if npc_pos.x == pos.x && npc_pos.y == pos.y {
            return Some(entity);
        }
    }
    None
}
