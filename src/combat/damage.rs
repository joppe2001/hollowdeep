//! Damage calculation
//!
//! Handles all combat math: damage, crits, dodges, armor.

use rand::Rng;
use crate::ecs::Stats;

/// Result of a combat attack
#[derive(Debug, Clone)]
pub struct AttackResult {
    /// Base damage before modifiers
    pub base_damage: i32,
    /// Final damage dealt
    pub final_damage: i32,
    /// Whether the attack was a critical hit
    pub is_crit: bool,
    /// Whether the attack was dodged
    pub is_dodge: bool,
    /// Whether the attack missed
    pub is_miss: bool,
    /// Descriptive message
    pub message: String,
}

impl AttackResult {
    pub fn dodged() -> Self {
        Self {
            base_damage: 0,
            final_damage: 0,
            is_crit: false,
            is_dodge: true,
            is_miss: false,
            message: "dodged".to_string(),
        }
    }

    pub fn missed() -> Self {
        Self {
            base_damage: 0,
            final_damage: 0,
            is_crit: false,
            is_dodge: false,
            is_miss: true,
            message: "missed".to_string(),
        }
    }
}

/// Calculate crit chance from DEX (percentage 0-100)
pub fn crit_chance(dex: i32) -> f32 {
    // Base 5% + 1% per DEX point above 10
    let base = 5.0;
    let bonus = ((dex - 10) as f32).max(0.0) * 1.5;
    (base + bonus).min(50.0) // Cap at 50%
}

/// Calculate dodge chance from DEX (percentage 0-100)
pub fn dodge_chance(dex: i32) -> f32 {
    // Base 3% + 1% per DEX point above 10
    let base = 3.0;
    let bonus = ((dex - 10) as f32).max(0.0) * 1.0;
    (base + bonus).min(40.0) // Cap at 40%
}

/// Calculate hit chance (base accuracy minus target dodge)
pub fn hit_chance(attacker_dex: i32, defender_dex: i32) -> f32 {
    let base_hit = 95.0; // 95% base hit chance
    let defender_dodge = dodge_chance(defender_dex);
    let attacker_accuracy = (attacker_dex as f32 - 10.0).max(0.0) * 0.5; // Small accuracy bonus
    (base_hit + attacker_accuracy - defender_dodge).clamp(20.0, 99.0)
}

/// Calculate base damage from STR
pub fn base_physical_damage(str: i32) -> i32 {
    // Base 2 + STR/2
    2 + str / 2
}

/// Calculate damage reduction percentage from armor value
/// Uses diminishing returns: reduction% = armor / (armor + K)
/// K = 20 means: at 20 armor you get 50% reduction, at 40 armor 66%, at 60 armor 75%
pub fn damage_reduction_percent(armor: i32) -> f32 {
    const K: f32 = 20.0; // Scaling constant - higher = armor is weaker
    let armor_f = armor.max(0) as f32;
    armor_f / (armor_f + K)
}

/// Calculate armor value from VIT stat
pub fn armor_from_vit(vit: i32) -> i32 {
    // VIT contributes some natural armor: VIT / 4
    vit / 4
}

/// Equipment bonuses for combat
#[derive(Debug, Clone, Default)]
pub struct EquipmentBonuses {
    /// Weapon damage bonus
    pub weapon_damage: i32,
    /// Armor bonus
    pub armor: i32,
    /// Bonus to strength
    pub str_bonus: i32,
    /// Bonus to dexterity
    pub dex_bonus: i32,
    /// Weapon crit bonus (percentage points)
    pub crit_bonus: f32,
}

/// Calculate a full attack
pub fn calculate_attack(
    attacker_stats: &Stats,
    defender_stats: &Stats,
    rng: &mut impl Rng,
) -> AttackResult {
    calculate_attack_with_equipment(
        attacker_stats,
        defender_stats,
        &EquipmentBonuses::default(),
        &EquipmentBonuses::default(),
        rng,
    )
}

/// Calculate a full attack with equipment bonuses
pub fn calculate_attack_with_equipment(
    attacker_stats: &Stats,
    defender_stats: &Stats,
    attacker_equipment: &EquipmentBonuses,
    defender_equipment: &EquipmentBonuses,
    rng: &mut impl Rng,
) -> AttackResult {
    // Effective stats with equipment bonuses
    let attacker_str = attacker_stats.strength + attacker_equipment.str_bonus;
    let attacker_dex = attacker_stats.dexterity + attacker_equipment.dex_bonus;
    let defender_dex = defender_stats.dexterity + defender_equipment.dex_bonus;
    let defender_vit = defender_stats.vitality;

    // Check for dodge first
    let hit_roll = rng.gen_range(0.0..100.0);
    let hit_pct = hit_chance(attacker_dex, defender_dex);

    if hit_roll >= hit_pct {
        // Check if it was a dodge or miss
        if dodge_chance(defender_dex) > 10.0 && rng.gen_bool(0.7) {
            return AttackResult::dodged();
        } else {
            return AttackResult::missed();
        }
    }

    // Calculate base damage (STR bonus + weapon damage)
    let base_damage = base_physical_damage(attacker_str) + attacker_equipment.weapon_damage;

    // Check for crit (base crit + weapon crit bonus)
    let crit_roll = rng.gen_range(0.0..100.0);
    let crit_pct = crit_chance(attacker_dex) + attacker_equipment.crit_bonus;
    let is_crit = crit_roll < crit_pct;

    // Apply crit multiplier (2x damage)
    let damage_after_crit = if is_crit {
        base_damage * 2
    } else {
        base_damage
    };

    // Apply armor reduction (VIT-based armor + equipment armor)
    // Uses percentage-based reduction with diminishing returns
    let total_armor = armor_from_vit(defender_vit) + defender_equipment.armor;
    let reduction_pct = damage_reduction_percent(total_armor);
    let damage_reduced = (damage_after_crit as f32 * (1.0 - reduction_pct)).round() as i32;
    let final_damage = damage_reduced.max(1); // Always at least 1 damage

    let message = if is_crit {
        format!("CRIT! {} damage", final_damage)
    } else {
        format!("{} damage", final_damage)
    };

    AttackResult {
        base_damage,
        final_damage,
        is_crit,
        is_dodge: false,
        is_miss: false,
        message,
    }
}

/// Simplified attack calculation for enemies (they have stats too)
pub fn calculate_enemy_attack(
    attacker_stats: &Stats,
    defender_stats: &Stats,
    rng: &mut impl Rng,
) -> AttackResult {
    // Same formula for enemies
    calculate_attack(attacker_stats, defender_stats, rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crit_chance() {
        assert_eq!(crit_chance(10), 5.0); // Base crit at 10 DEX
        assert!(crit_chance(20) > crit_chance(10)); // More DEX = more crit
        assert!(crit_chance(100) <= 50.0); // Capped
    }

    #[test]
    fn test_base_damage() {
        assert_eq!(base_physical_damage(10), 7); // 2 + 10/2 = 7
        assert_eq!(base_physical_damage(20), 12); // 2 + 20/2 = 12
    }
}
