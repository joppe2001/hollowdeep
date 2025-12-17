//! Item synergies and set bonuses
//!
//! When items with matching tags are equipped together, they provide bonus effects.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Tags that items can have for synergy matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SynergyTag {
    // Elemental
    Fire,
    Ice,
    Lightning,
    Poison,

    // Themed sets
    Cultist,
    Knight,
    Shadow,
    Holy,
    Corruption,

    // Build archetypes
    Berserker,  // Damage when low HP
    Arcane,     // Spell power and mana
    Assassin,   // Critical strikes
    Guardian,   // Defense and protection
    Vampire,    // Lifesteal and sustain
    Beast,      // Primal/nature themed

    // Weapon combos
    DualWield,
    TwoHanded,
}

/// Bonus type provided by a synergy
#[derive(Debug, Clone)]
pub enum SynergyBonus {
    /// Flat damage bonus
    BonusDamage(i32),
    /// Percent damage increase
    DamagePercent(f32),
    /// Flat armor bonus
    BonusArmor(i32),
    /// Flat HP bonus
    BonusHP(i32),
    /// Flat MP bonus
    BonusMP(i32),
    /// Crit chance bonus
    CritChance(f32),
    /// Lifesteal percentage
    Lifesteal(f32),
    /// Fire damage on hit
    FireDamageOnHit(i32),
    /// Poison damage on hit
    PoisonDamageOnHit(i32),
    /// Lightning damage on hit
    LightningDamageOnHit(i32),
    /// Corruption (power + penalty)
    Corruption { power: i32, penalty: i32 },
}

/// Definition of a synergy
#[derive(Debug, Clone)]
pub struct Synergy {
    /// Name of the synergy
    pub name: &'static str,
    /// Description of what it does
    pub description: &'static str,
    /// Required tag
    pub tag: SynergyTag,
    /// Number of items needed for each tier
    pub tiers: Vec<SynergyTier>,
}

/// A tier of synergy bonus
#[derive(Debug, Clone)]
pub struct SynergyTier {
    /// Items needed to activate
    pub required: u8,
    /// Bonuses granted
    pub bonuses: Vec<SynergyBonus>,
}

impl Synergy {
    /// Get the active tier based on item count
    pub fn active_tier(&self, count: u8) -> Option<&SynergyTier> {
        self.tiers.iter()
            .filter(|t| count >= t.required)
            .last()
    }
}

/// All available synergies
pub fn all_synergies() -> Vec<Synergy> {
    vec![
        // Fire synergy - burning damage
        Synergy {
            name: "Flames of Fury",
            description: "Fire items empower your attacks with burning damage.",
            tag: SynergyTag::Fire,
            tiers: vec![
                SynergyTier {
                    required: 2,
                    bonuses: vec![SynergyBonus::FireDamageOnHit(3)],
                },
                SynergyTier {
                    required: 3,
                    bonuses: vec![
                        SynergyBonus::FireDamageOnHit(6),
                        SynergyBonus::DamagePercent(0.10),
                    ],
                },
            ],
        },

        // Ice synergy - armor and slow
        Synergy {
            name: "Frozen Heart",
            description: "Ice items grant defensive bonuses and chill enemies.",
            tag: SynergyTag::Ice,
            tiers: vec![
                SynergyTier {
                    required: 2,
                    bonuses: vec![SynergyBonus::BonusArmor(5)],
                },
                SynergyTier {
                    required: 3,
                    bonuses: vec![
                        SynergyBonus::BonusArmor(10),
                        SynergyBonus::BonusHP(20),
                    ],
                },
            ],
        },

        // Poison synergy - DOT
        Synergy {
            name: "Venomous",
            description: "Poison items inflict lasting damage.",
            tag: SynergyTag::Poison,
            tiers: vec![
                SynergyTier {
                    required: 2,
                    bonuses: vec![SynergyBonus::PoisonDamageOnHit(4)],
                },
                SynergyTier {
                    required: 3,
                    bonuses: vec![
                        SynergyBonus::PoisonDamageOnHit(8),
                        SynergyBonus::CritChance(5.0),
                    ],
                },
            ],
        },

        // Cultist set - dark power
        Synergy {
            name: "Blood Rite",
            description: "Cultist items grant dark power at a cost.",
            tag: SynergyTag::Cultist,
            tiers: vec![
                SynergyTier {
                    required: 2,
                    bonuses: vec![SynergyBonus::BonusDamage(4)],
                },
                SynergyTier {
                    required: 3,
                    bonuses: vec![
                        SynergyBonus::BonusDamage(8),
                        SynergyBonus::Lifesteal(0.10),
                    ],
                },
            ],
        },

        // Knight set - defensive
        Synergy {
            name: "Knight's Valor",
            description: "Knight items provide stalwart defense.",
            tag: SynergyTag::Knight,
            tiers: vec![
                SynergyTier {
                    required: 2,
                    bonuses: vec![SynergyBonus::BonusArmor(8)],
                },
                SynergyTier {
                    required: 4,
                    bonuses: vec![
                        SynergyBonus::BonusArmor(15),
                        SynergyBonus::BonusHP(30),
                    ],
                },
            ],
        },

        // Shadow set - critical strikes
        Synergy {
            name: "Shadow's Edge",
            description: "Shadow items enhance critical strikes.",
            tag: SynergyTag::Shadow,
            tiers: vec![
                SynergyTier {
                    required: 2,
                    bonuses: vec![SynergyBonus::CritChance(10.0)],
                },
                SynergyTier {
                    required: 3,
                    bonuses: vec![
                        SynergyBonus::CritChance(20.0),
                        SynergyBonus::BonusDamage(5),
                    ],
                },
            ],
        },

        // Corruption - high risk high reward
        Synergy {
            name: "Corrupted Soul",
            description: "Corruption grants great power but weakens your body.",
            tag: SynergyTag::Corruption,
            tiers: vec![
                SynergyTier {
                    required: 1,
                    bonuses: vec![SynergyBonus::Corruption { power: 5, penalty: 10 }],
                },
                SynergyTier {
                    required: 2,
                    bonuses: vec![SynergyBonus::Corruption { power: 12, penalty: 20 }],
                },
                SynergyTier {
                    required: 3,
                    bonuses: vec![SynergyBonus::Corruption { power: 25, penalty: 35 }],
                },
            ],
        },
    ]
}

/// Active synergy with its current tier
#[derive(Debug, Clone)]
pub struct ActiveSynergy {
    pub synergy: Synergy,
    pub tier: SynergyTier,
    pub item_count: u8,
}

/// Calculate active synergies from equipped item tags
pub fn calculate_synergies(tags: &[SynergyTag]) -> Vec<ActiveSynergy> {
    // Count occurrences of each tag
    let mut tag_counts: HashMap<SynergyTag, u8> = HashMap::new();
    for tag in tags {
        *tag_counts.entry(*tag).or_insert(0) += 1;
    }

    // Find active synergies
    let mut active = Vec::new();
    for synergy in all_synergies() {
        if let Some(&count) = tag_counts.get(&synergy.tag) {
            if let Some(tier) = synergy.active_tier(count) {
                active.push(ActiveSynergy {
                    synergy: synergy.clone(),
                    tier: tier.clone(),
                    item_count: count,
                });
            }
        }
    }

    active
}

/// Aggregate bonuses from all active synergies
#[derive(Debug, Default)]
pub struct SynergyBonuses {
    pub bonus_damage: i32,
    pub damage_percent: f32,
    pub bonus_armor: i32,
    pub bonus_hp: i32,
    pub bonus_mp: i32,
    pub crit_chance: f32,
    pub lifesteal: f32,
    pub fire_damage: i32,
    pub poison_damage: i32,
    pub lightning_damage: i32,
    pub corruption_power: i32,
    pub corruption_penalty: i32,
}

impl SynergyBonuses {
    pub fn from_tags(tags: &[SynergyTag]) -> Self {
        let mut bonuses = SynergyBonuses::default();

        // Count tags
        let mut tag_counts: HashMap<SynergyTag, u8> = HashMap::new();
        for tag in tags {
            *tag_counts.entry(*tag).or_insert(0) += 1;
        }

        // Apply synergy bonuses
        for synergy in all_synergies() {
            if let Some(&count) = tag_counts.get(&synergy.tag) {
                if let Some(tier) = synergy.active_tier(count) {
                    for bonus in &tier.bonuses {
                        match bonus {
                            SynergyBonus::BonusDamage(v) => bonuses.bonus_damage += v,
                            SynergyBonus::DamagePercent(v) => bonuses.damage_percent += v,
                            SynergyBonus::BonusArmor(v) => bonuses.bonus_armor += v,
                            SynergyBonus::BonusHP(v) => bonuses.bonus_hp += v,
                            SynergyBonus::BonusMP(v) => bonuses.bonus_mp += v,
                            SynergyBonus::CritChance(v) => bonuses.crit_chance += v,
                            SynergyBonus::Lifesteal(v) => bonuses.lifesteal += v,
                            SynergyBonus::FireDamageOnHit(v) => bonuses.fire_damage += v,
                            SynergyBonus::PoisonDamageOnHit(v) => bonuses.poison_damage += v,
                            SynergyBonus::LightningDamageOnHit(v) => bonuses.lightning_damage += v,
                            SynergyBonus::Corruption { power, penalty } => {
                                bonuses.corruption_power += power;
                                bonuses.corruption_penalty += penalty;
                            }
                        }
                    }
                }
            }
        }

        bonuses
    }

    /// Check if any synergy is active
    pub fn has_active_synergy(&self) -> bool {
        self.bonus_damage != 0 ||
        self.damage_percent != 0.0 ||
        self.bonus_armor != 0 ||
        self.bonus_hp != 0 ||
        self.bonus_mp != 0 ||
        self.crit_chance != 0.0 ||
        self.lifesteal != 0.0 ||
        self.fire_damage != 0 ||
        self.poison_damage != 0 ||
        self.lightning_damage != 0 ||
        self.corruption_power != 0
    }
}
