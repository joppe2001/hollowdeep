//! Loot generation system
//!
//! Handles random item generation, affixes, and drop tables.

use rand::Rng;
use super::item::{Item, ItemId, Rarity, Affix, AffixType, templates};

/// Counter for generating unique item IDs
static mut NEXT_ITEM_ID: ItemId = 1;

/// Get next unique item ID
pub fn next_item_id() -> ItemId {
    unsafe {
        let id = NEXT_ITEM_ID;
        NEXT_ITEM_ID += 1;
        id
    }
}

/// Generate a random rarity based on floor depth
pub fn roll_rarity(floor: u32, rng: &mut impl Rng) -> Rarity {
    // Use 1000 for finer granularity on rare drops
    let roll = rng.gen_range(0..1000);

    // Floor bonus scales slowly - meaningful progression requires deeper floors
    // Floors 1-5: early game, floors 6-15: mid game, floors 16+: late game
    let floor_tier = match floor {
        1..=5 => 0,
        6..=10 => 1,
        11..=15 => 2,
        16..=20 => 3,
        21..=30 => 4,
        _ => 5, // Floor 31+ (deep endgame)
    };

    // Mythic: Only floor 20+, starts at 0.1%, scales to 0.5%
    // Legendary: 0% base, only starts appearing floor 6+, maxes at ~2% floor 20+
    // Epic: 0.5% base, scales to ~7% at high floors
    // Rare: 5% base, scales to ~20%
    // Uncommon: 25% base, scales to ~45%
    // Common: remainder

    // Mythic threshold - only floor 20+ can drop mythic items
    let mythic_threshold = match floor_tier {
        0..=2 => 1000,  // 0% - no mythics before floor 16
        3 => 999,       // 0.1% - floors 16-20 (very rare preview)
        4 => 997,       // 0.3% - floors 21-30
        _ => 995,       // 0.5% - floors 31+
    };

    let legendary_threshold = match floor_tier {
        0 => 1000,  // 0% - no legendaries floors 1-5
        1 => 998,   // 0.2% - floors 6-10
        2 => 995,   // 0.5% - floors 11-15
        3 => 990,   // 1% - floors 16-20
        4 => 980,   // 2% - floors 21-30
        _ => 970,   // 3% - floors 31+
    };

    let epic_threshold = match floor_tier {
        0 => 995,   // 0.5% - early floors
        1 => 985,   // 1.5% - floors 6-10
        2 => 970,   // 3% - floors 11-15
        3 => 950,   // 5% - floors 16-20
        4 => 930,   // 7% - floors 21-30
        _ => 910,   // 9% - floors 31+
    };

    let rare_threshold = match floor_tier {
        0 => 950,   // 5% - early floors
        1 => 920,   // 8% - floors 6-10
        2 => 880,   // 12% - floors 11-15
        3 => 850,   // 15% - floors 16-20
        4 => 800,   // 20% - floors 21-30
        _ => 750,   // 25% - floors 31+
    };

    let uncommon_threshold = match floor_tier {
        0 => 750,   // 25% - early floors
        1 => 700,   // 30% - floors 6-10
        2 => 650,   // 35% - floors 11-15
        3 => 600,   // 40% - floors 16-20
        4 => 550,   // 45% - floors 21-30
        _ => 500,   // 50% - floors 31+
    };

    if roll >= mythic_threshold {
        Rarity::Mythic
    } else if roll >= legendary_threshold {
        Rarity::Legendary
    } else if roll >= epic_threshold {
        Rarity::Epic
    } else if roll >= rare_threshold {
        Rarity::Rare
    } else if roll >= uncommon_threshold {
        Rarity::Uncommon
    } else {
        Rarity::Common
    }
}

/// Get number of affixes for a rarity
pub fn affixes_for_rarity(rarity: Rarity) -> usize {
    match rarity {
        Rarity::Common => 0,
        Rarity::Uncommon => 1,
        Rarity::Rare => 2,
        Rarity::Epic => 4,      // Increased from 3
        Rarity::Legendary => 5, // Increased from 3 - truly powerful items
        Rarity::Mythic => 6,    // 4 regular + 2 mythic-only affixes
    }
}

/// Get base stat bonus for a rarity (for weapons: damage, for armor: armor)
/// Higher rarities now give significantly more impactful bonuses
pub fn rarity_stat_bonus(rarity: Rarity, rng: &mut impl Rng) -> i32 {
    match rarity {
        Rarity::Common => 0,
        Rarity::Uncommon => rng.gen_range(2..=4),    // +2-4 (was 1-2)
        Rarity::Rare => rng.gen_range(5..=8),        // +5-8 (was 2-4)
        Rarity::Epic => rng.gen_range(9..=14),       // +9-14 (was 4-6)
        Rarity::Legendary => rng.gen_range(15..=22), // +15-22 (was 6-10)
        Rarity::Mythic => rng.gen_range(25..=35),    // +25-35 for mythic
    }
}

/// Generate a random affix with value scaled by rarity
pub fn roll_affix_with_rarity(rng: &mut impl Rng, for_weapon: bool, rarity: Rarity) -> Affix {
    let possible_affixes = if for_weapon {
        vec![
            (AffixType::BonusDamage, 2, 8),
            (AffixType::BonusCritChance, 3, 10),
            (AffixType::BonusCritDamage, 10, 30),
            (AffixType::FireDamage, 2, 6),
            (AffixType::IceDamage, 2, 6),
            (AffixType::LightningDamage, 2, 6),
            (AffixType::PoisonDamage, 1, 4),
            (AffixType::LifeSteal, 2, 8),
            (AffixType::BonusStrength, 1, 5),
            (AffixType::BonusDexterity, 1, 5),
        ]
    } else {
        vec![
            (AffixType::BonusArmor, 1, 5),
            (AffixType::BonusHP, 5, 25),
            (AffixType::BonusMP, 5, 20),
            (AffixType::BonusDodge, 2, 8),
            (AffixType::FireResist, 5, 20),
            (AffixType::IceResist, 5, 20),
            (AffixType::PoisonResist, 5, 20),
            (AffixType::BonusStrength, 1, 5),
            (AffixType::BonusDexterity, 1, 5),
            (AffixType::BonusIntelligence, 1, 5),
            (AffixType::BonusVitality, 1, 5),
        ]
    };

    let (affix_type, min_val, max_val) = possible_affixes[rng.gen_range(0..possible_affixes.len())];

    // Scale the roll range based on rarity - higher rarity rolls MUCH higher values
    let (scaled_min, scaled_max) = match rarity {
        Rarity::Common => (min_val, (min_val + max_val) / 2),           // Low range (50% of max)
        Rarity::Uncommon => (min_val, max_val),                         // Full range (100% of max)
        Rarity::Rare => ((max_val * 2) / 3, max_val + max_val / 2),     // 66-150% of max
        Rarity::Epic => (max_val, max_val * 2),                          // 100-200% of max
        Rarity::Legendary => (max_val + max_val / 2, max_val * 3),       // 150-300% of max
        Rarity::Mythic => (max_val * 2, max_val * 4),                    // 200-400% of max
    };

    let value = rng.gen_range(scaled_min..=scaled_max);

    Affix { affix_type, value }
}

/// Generate a random affix (legacy, defaults to Uncommon scaling)
pub fn roll_affix(rng: &mut impl Rng, for_weapon: bool) -> Affix {
    roll_affix_with_rarity(rng, for_weapon, Rarity::Uncommon)
}

/// Generate a random weapon
pub fn generate_weapon(floor: u32, rng: &mut impl Rng) -> Item {
    let id = next_item_id();

    // Pick base weapon type
    let weapon_roll = rng.gen_range(0..6);
    let mut item = match weapon_roll {
        0 => templates::iron_sword(id),
        1 => templates::rusty_dagger(id),
        2 => templates::battle_axe(id),
        _ => templates::iron_sword(id), // Default to sword
    };

    // Roll rarity first (needed for stat bonus)
    let rarity = roll_rarity(floor, rng);
    item.rarity = rarity;

    // Scale base damage with floor AND rarity
    item.base_damage += (floor as i32 - 1) / 2;
    item.base_damage += rarity_stat_bonus(rarity, rng);

    // Add affixes based on rarity (with rarity-scaled values)
    let num_affixes = affixes_for_rarity(rarity);
    for _ in 0..num_affixes {
        item.affixes.push(roll_affix_with_rarity(rng, true, rarity));
    }

    // Generate name with affixes
    item.generate_name();

    // Scale value with rarity
    item.value = match rarity {
        Rarity::Common => item.value,
        Rarity::Uncommon => item.value * 2,
        Rarity::Rare => item.value * 4,
        Rarity::Epic => item.value * 8,
        Rarity::Legendary => item.value * 20,
        Rarity::Mythic => item.value * 50,
    };

    item
}

/// Generate a random armor piece from any equipment slot
pub fn generate_armor(floor: u32, rng: &mut impl Rng) -> Item {
    let id = next_item_id();

    // Pick random equipment slot (excluding MainHand which is weapons)
    // 0-1: Body, 2-3: Head, 4-5: Hands, 6-7: Feet, 8-9: OffHand, 10-11: Accessories
    let slot_roll = rng.gen_range(0..12);
    let mut item = match slot_roll {
        0 => templates::leather_armor(id),
        1 => templates::leather_armor(id),
        2 => templates::chain_helm(id),
        3 => templates::chain_helm(id),
        4 => templates::leather_gloves(id),
        5 => templates::leather_gloves(id),
        6 => templates::leather_boots(id),
        7 => templates::chain_boots(id),
        8 => templates::wooden_shield(id),
        9 => templates::iron_shield(id),
        10 => templates::bone_ring(id),
        _ => templates::copper_amulet(id),
    };

    // Roll rarity first (needed for stat bonus)
    let rarity = roll_rarity(floor, rng);
    item.rarity = rarity;

    // Scale base armor with floor AND rarity
    item.base_armor += (floor as i32 - 1) / 3;
    item.base_armor += rarity_stat_bonus(rarity, rng);

    // Add affixes based on rarity (with rarity-scaled values)
    let num_affixes = affixes_for_rarity(rarity);
    for _ in 0..num_affixes {
        item.affixes.push(roll_affix_with_rarity(rng, false, rarity));
    }

    item.generate_name();

    item.value = match rarity {
        Rarity::Common => item.value,
        Rarity::Uncommon => item.value * 2,
        Rarity::Rare => item.value * 4,
        Rarity::Epic => item.value * 8,
        Rarity::Legendary => item.value * 20,
        Rarity::Mythic => item.value * 50,
    };

    item
}

/// Generate a consumable
pub fn generate_consumable(rng: &mut impl Rng) -> Item {
    let id = next_item_id();

    if rng.gen_bool(0.7) {
        templates::health_potion(id)
    } else {
        templates::mana_potion(id)
    }
}

/// Generate random loot for an enemy kill
pub fn generate_enemy_loot(floor: u32, rng: &mut impl Rng) -> Vec<Item> {
    let mut loot = Vec::new();

    // Always chance for gold (handled separately)

    // 30% chance for any drop
    if !rng.gen_bool(0.30) {
        return loot;
    }

    // What type of drop?
    let roll = rng.gen_range(0..100);

    if roll < 50 {
        // 50% - Consumable
        loot.push(generate_consumable(rng));
    } else if roll < 80 {
        // 30% - Weapon
        loot.push(generate_weapon(floor, rng));
    } else {
        // 20% - Armor
        loot.push(generate_armor(floor, rng));
    }

    loot
}

/// Generate floor loot (items placed on the map)
pub fn generate_floor_loot(floor: u32, count: usize, rng: &mut impl Rng) -> Vec<Item> {
    let mut loot = Vec::with_capacity(count);

    for _ in 0..count {
        let roll = rng.gen_range(0..100);

        if roll < 40 {
            loot.push(generate_consumable(rng));
        } else if roll < 70 {
            loot.push(generate_weapon(floor, rng));
        } else {
            loot.push(generate_armor(floor, rng));
        }
    }

    loot
}

/// Generate gold drop amount based on floor
pub fn generate_gold_drop(floor: u32, rng: &mut impl Rng) -> u32 {
    let base = 5 + floor * 3;
    let variance = rng.gen_range(0..=base / 2);
    base + variance
}

/// Get minimum rarity based on floor level
/// Floor 1-4: Common drops
/// Floor 5-9: Minimum Uncommon
/// Floor 10-14: Minimum Rare
/// Floor 15-19: Minimum Epic
/// Floor 20+: Minimum Epic (with higher Legendary chance)
pub fn minimum_rarity_for_floor(floor: u32) -> Rarity {
    match floor {
        0..=4 => Rarity::Common,
        5..=9 => Rarity::Uncommon,
        10..=14 => Rarity::Rare,
        _ => Rarity::Epic,
    }
}

/// Roll rarity with a minimum threshold (for bosses)
pub fn roll_rarity_with_minimum(floor: u32, min_rarity: Rarity, rng: &mut impl Rng) -> Rarity {
    let rolled = roll_rarity(floor, rng);

    // Use sort_value() for comparison (higher = rarer)
    let rolled_value = rolled.sort_value();
    let min_value = min_rarity.sort_value();

    if rolled_value >= min_value {
        rolled
    } else {
        min_rarity
    }
}

/// Generate weapon with minimum rarity
pub fn generate_weapon_with_min_rarity(floor: u32, min_rarity: Rarity, rng: &mut impl Rng) -> Item {
    let id = next_item_id();

    let weapon_roll = rng.gen_range(0..6);
    let mut item = match weapon_roll {
        0 => templates::iron_sword(id),
        1 => templates::rusty_dagger(id),
        2 => templates::battle_axe(id),
        _ => templates::iron_sword(id),
    };

    let rarity = roll_rarity_with_minimum(floor, min_rarity, rng);
    item.rarity = rarity;

    item.base_damage += (floor as i32 - 1) / 2;
    item.base_damage += rarity_stat_bonus(rarity, rng);

    let num_affixes = affixes_for_rarity(rarity);
    for _ in 0..num_affixes {
        item.affixes.push(roll_affix_with_rarity(rng, true, rarity));
    }

    item.generate_name();

    item.value = match rarity {
        Rarity::Common => item.value,
        Rarity::Uncommon => item.value * 2,
        Rarity::Rare => item.value * 4,
        Rarity::Epic => item.value * 8,
        Rarity::Legendary => item.value * 20,
        Rarity::Mythic => item.value * 50,
    };

    item
}

/// Generate armor with minimum rarity
pub fn generate_armor_with_min_rarity(floor: u32, min_rarity: Rarity, rng: &mut impl Rng) -> Item {
    let id = next_item_id();

    // Pick random equipment slot (excluding MainHand which is weapons)
    let slot_roll = rng.gen_range(0..12);
    let mut item = match slot_roll {
        0 => templates::leather_armor(id),
        1 => templates::leather_armor(id),
        2 => templates::chain_helm(id),
        3 => templates::chain_helm(id),
        4 => templates::leather_gloves(id),
        5 => templates::leather_gloves(id),
        6 => templates::leather_boots(id),
        7 => templates::chain_boots(id),
        8 => templates::wooden_shield(id),
        9 => templates::iron_shield(id),
        10 => templates::bone_ring(id),
        _ => templates::copper_amulet(id),
    };

    let rarity = roll_rarity_with_minimum(floor, min_rarity, rng);
    item.rarity = rarity;

    item.base_armor += (floor as i32 - 1) / 3;
    item.base_armor += rarity_stat_bonus(rarity, rng);

    let num_affixes = affixes_for_rarity(rarity);
    for _ in 0..num_affixes {
        item.affixes.push(roll_affix_with_rarity(rng, false, rarity));
    }

    item.generate_name();

    item.value = match rarity {
        Rarity::Common => item.value,
        Rarity::Uncommon => item.value * 2,
        Rarity::Rare => item.value * 4,
        Rarity::Epic => item.value * 8,
        Rarity::Legendary => item.value * 20,
        Rarity::Mythic => item.value * 50,
    };

    item
}

/// Generate boss loot - guaranteed drops with minimum rarity based on floor
/// Bosses always drop:
/// - 1 weapon or armor piece at minimum rarity+1 for the floor
/// - 1 consumable
/// - Extra gold
pub fn generate_boss_loot(floor: u32, rng: &mut impl Rng) -> Vec<Item> {
    let mut loot = Vec::new();

    // Calculate minimum rarity: floor minimum + 1 tier
    let base_min = minimum_rarity_for_floor(floor);
    let boss_min = match base_min {
        Rarity::Common => Rarity::Uncommon,
        Rarity::Uncommon => Rarity::Rare,
        Rarity::Rare => Rarity::Epic,
        Rarity::Epic => Rarity::Legendary,
        Rarity::Legendary => Rarity::Legendary,
        Rarity::Mythic => Rarity::Mythic, // Mythic is the ceiling
    };

    // Always drop a weapon or armor
    if rng.gen_bool(0.5) {
        loot.push(generate_weapon_with_min_rarity(floor, boss_min, rng));
    } else {
        loot.push(generate_armor_with_min_rarity(floor, boss_min, rng));
    }

    // Always drop a consumable
    loot.push(generate_consumable(rng));

    // 50% chance for second equipment piece
    if rng.gen_bool(0.5) {
        if rng.gen_bool(0.5) {
            loot.push(generate_weapon_with_min_rarity(floor, boss_min, rng));
        } else {
            loot.push(generate_armor_with_min_rarity(floor, boss_min, rng));
        }
    }

    loot
}

/// Generate gold drop for boss (more generous)
pub fn generate_boss_gold_drop(floor: u32, rng: &mut impl Rng) -> u32 {
    let base = 50 + floor * 20;
    let variance = rng.gen_range(0..=base / 2);
    base + variance
}
