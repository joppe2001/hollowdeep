//! Synergy definitions for data-driven set bonuses
//!
//! These templates are loaded from RON files and define item synergies.

use serde::{Deserialize, Serialize};
use crate::items::synergies::{SynergyTag, SynergyBonus, Synergy, SynergyTier};

/// A template for synergy definitions from external data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynergyDef {
    /// Unique synergy ID
    pub id: String,
    /// Display name of the synergy
    pub name: String,
    /// Description of the synergy
    pub description: String,
    /// Required tag to activate
    pub tag: SynergyTag,
    /// Tier definitions
    pub tiers: Vec<SynergyTierDef>,
}

/// A tier definition for synergy bonuses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynergyTierDef {
    /// Items needed to activate
    pub required: u8,
    /// Bonuses granted
    pub bonuses: Vec<SynergyBonusDef>,
}

/// A bonus definition for synergies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SynergyBonusDef {
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

impl SynergyBonusDef {
    /// Convert to runtime SynergyBonus
    pub fn to_runtime(&self) -> SynergyBonus {
        match self {
            SynergyBonusDef::BonusDamage(v) => SynergyBonus::BonusDamage(*v),
            SynergyBonusDef::DamagePercent(v) => SynergyBonus::DamagePercent(*v),
            SynergyBonusDef::BonusArmor(v) => SynergyBonus::BonusArmor(*v),
            SynergyBonusDef::BonusHP(v) => SynergyBonus::BonusHP(*v),
            SynergyBonusDef::BonusMP(v) => SynergyBonus::BonusMP(*v),
            SynergyBonusDef::CritChance(v) => SynergyBonus::CritChance(*v),
            SynergyBonusDef::Lifesteal(v) => SynergyBonus::Lifesteal(*v),
            SynergyBonusDef::FireDamageOnHit(v) => SynergyBonus::FireDamageOnHit(*v),
            SynergyBonusDef::PoisonDamageOnHit(v) => SynergyBonus::PoisonDamageOnHit(*v),
            SynergyBonusDef::LightningDamageOnHit(v) => SynergyBonus::LightningDamageOnHit(*v),
            SynergyBonusDef::Corruption { power, penalty } => SynergyBonus::Corruption {
                power: *power,
                penalty: *penalty,
            },
        }
    }
}

impl SynergyDef {
    /// Convert to runtime Synergy
    pub fn to_runtime(&self) -> Synergy {
        Synergy {
            name: Box::leak(self.name.clone().into_boxed_str()),
            description: Box::leak(self.description.clone().into_boxed_str()),
            tag: self.tag,
            tiers: self.tiers.iter().map(|t| SynergyTier {
                required: t.required,
                bonuses: t.bonuses.iter().map(|b| b.to_runtime()).collect(),
            }).collect(),
        }
    }
}

/// Collection of synergy definitions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SynergyDefs {
    pub synergies: Vec<SynergyDef>,
}

impl SynergyDefs {
    /// Find a synergy by ID
    pub fn find(&self, id: &str) -> Option<&SynergyDef> {
        self.synergies.iter().find(|s| s.id == id)
    }

    /// Find a synergy by tag
    pub fn for_tag(&self, tag: SynergyTag) -> Option<&SynergyDef> {
        self.synergies.iter().find(|s| s.tag == tag)
    }

    /// Convert all to runtime synergies
    pub fn to_runtime(&self) -> Vec<Synergy> {
        self.synergies.iter().map(|s| s.to_runtime()).collect()
    }
}

/// Create default synergy definitions (hardcoded fallback)
pub fn default_synergy_defs() -> SynergyDefs {
    SynergyDefs {
        synergies: vec![
            // Fire synergy - burning damage
            SynergyDef {
                id: "flames_of_fury".to_string(),
                name: "Flames of Fury".to_string(),
                description: "Fire items empower your attacks with burning damage.".to_string(),
                tag: SynergyTag::Fire,
                tiers: vec![
                    SynergyTierDef {
                        required: 2,
                        bonuses: vec![SynergyBonusDef::FireDamageOnHit(3)],
                    },
                    SynergyTierDef {
                        required: 3,
                        bonuses: vec![
                            SynergyBonusDef::FireDamageOnHit(6),
                            SynergyBonusDef::DamagePercent(0.10),
                        ],
                    },
                ],
            },
            // Ice synergy - armor and defense
            SynergyDef {
                id: "frozen_heart".to_string(),
                name: "Frozen Heart".to_string(),
                description: "Ice items grant defensive bonuses and chill enemies.".to_string(),
                tag: SynergyTag::Ice,
                tiers: vec![
                    SynergyTierDef {
                        required: 2,
                        bonuses: vec![SynergyBonusDef::BonusArmor(5)],
                    },
                    SynergyTierDef {
                        required: 3,
                        bonuses: vec![
                            SynergyBonusDef::BonusArmor(10),
                            SynergyBonusDef::BonusHP(20),
                        ],
                    },
                ],
            },
            // Poison synergy - DOT
            SynergyDef {
                id: "venomous".to_string(),
                name: "Venomous".to_string(),
                description: "Poison items inflict lasting damage.".to_string(),
                tag: SynergyTag::Poison,
                tiers: vec![
                    SynergyTierDef {
                        required: 2,
                        bonuses: vec![SynergyBonusDef::PoisonDamageOnHit(4)],
                    },
                    SynergyTierDef {
                        required: 3,
                        bonuses: vec![
                            SynergyBonusDef::PoisonDamageOnHit(8),
                            SynergyBonusDef::CritChance(5.0),
                        ],
                    },
                ],
            },
            // Cultist set - dark power
            SynergyDef {
                id: "blood_rite".to_string(),
                name: "Blood Rite".to_string(),
                description: "Cultist items grant dark power at a cost.".to_string(),
                tag: SynergyTag::Cultist,
                tiers: vec![
                    SynergyTierDef {
                        required: 2,
                        bonuses: vec![SynergyBonusDef::BonusDamage(4)],
                    },
                    SynergyTierDef {
                        required: 3,
                        bonuses: vec![
                            SynergyBonusDef::BonusDamage(8),
                            SynergyBonusDef::Lifesteal(0.10),
                        ],
                    },
                ],
            },
            // Knight set - defensive
            SynergyDef {
                id: "knights_valor".to_string(),
                name: "Knight's Valor".to_string(),
                description: "Knight items provide stalwart defense.".to_string(),
                tag: SynergyTag::Knight,
                tiers: vec![
                    SynergyTierDef {
                        required: 2,
                        bonuses: vec![SynergyBonusDef::BonusArmor(8)],
                    },
                    SynergyTierDef {
                        required: 4,
                        bonuses: vec![
                            SynergyBonusDef::BonusArmor(15),
                            SynergyBonusDef::BonusHP(30),
                        ],
                    },
                ],
            },
            // Shadow set - critical strikes
            SynergyDef {
                id: "shadows_edge".to_string(),
                name: "Shadow's Edge".to_string(),
                description: "Shadow items enhance critical strikes.".to_string(),
                tag: SynergyTag::Shadow,
                tiers: vec![
                    SynergyTierDef {
                        required: 2,
                        bonuses: vec![SynergyBonusDef::CritChance(10.0)],
                    },
                    SynergyTierDef {
                        required: 3,
                        bonuses: vec![
                            SynergyBonusDef::CritChance(20.0),
                            SynergyBonusDef::BonusDamage(5),
                        ],
                    },
                ],
            },
            // Corruption - high risk high reward
            SynergyDef {
                id: "corrupted_soul".to_string(),
                name: "Corrupted Soul".to_string(),
                description: "Corruption grants great power but weakens your body.".to_string(),
                tag: SynergyTag::Corruption,
                tiers: vec![
                    SynergyTierDef {
                        required: 1,
                        bonuses: vec![SynergyBonusDef::Corruption { power: 5, penalty: 10 }],
                    },
                    SynergyTierDef {
                        required: 2,
                        bonuses: vec![SynergyBonusDef::Corruption { power: 12, penalty: 20 }],
                    },
                    SynergyTierDef {
                        required: 3,
                        bonuses: vec![SynergyBonusDef::Corruption { power: 25, penalty: 35 }],
                    },
                ],
            },
            // Berserker - power at low HP
            SynergyDef {
                id: "berserker_rage".to_string(),
                name: "Berserker Rage".to_string(),
                description: "Berserker items grant more power as your health drops.".to_string(),
                tag: SynergyTag::Berserker,
                tiers: vec![
                    SynergyTierDef {
                        required: 2,
                        bonuses: vec![
                            SynergyBonusDef::BonusDamage(5),
                            SynergyBonusDef::CritChance(5.0),
                        ],
                    },
                    SynergyTierDef {
                        required: 3,
                        bonuses: vec![
                            SynergyBonusDef::BonusDamage(10),
                            SynergyBonusDef::DamagePercent(0.15),
                            SynergyBonusDef::CritChance(10.0),
                        ],
                    },
                ],
            },
            // Arcane - spell power
            SynergyDef {
                id: "arcane_mastery".to_string(),
                name: "Arcane Mastery".to_string(),
                description: "Arcane items enhance your magical abilities.".to_string(),
                tag: SynergyTag::Arcane,
                tiers: vec![
                    SynergyTierDef {
                        required: 2,
                        bonuses: vec![SynergyBonusDef::BonusMP(20)],
                    },
                    SynergyTierDef {
                        required: 3,
                        bonuses: vec![
                            SynergyBonusDef::BonusMP(40),
                            SynergyBonusDef::DamagePercent(0.12),
                        ],
                    },
                ],
            },
            // Assassin - critical strikes
            SynergyDef {
                id: "assassins_mark".to_string(),
                name: "Assassin's Mark".to_string(),
                description: "Assassin items make your critical strikes devastating.".to_string(),
                tag: SynergyTag::Assassin,
                tiers: vec![
                    SynergyTierDef {
                        required: 2,
                        bonuses: vec![SynergyBonusDef::CritChance(15.0)],
                    },
                    SynergyTierDef {
                        required: 3,
                        bonuses: vec![
                            SynergyBonusDef::CritChance(25.0),
                            SynergyBonusDef::BonusDamage(8),
                        ],
                    },
                ],
            },
            // Guardian - protection
            SynergyDef {
                id: "stalwart_guardian".to_string(),
                name: "Stalwart Guardian".to_string(),
                description: "Guardian items provide unmatched protection.".to_string(),
                tag: SynergyTag::Guardian,
                tiers: vec![
                    SynergyTierDef {
                        required: 2,
                        bonuses: vec![
                            SynergyBonusDef::BonusArmor(8),
                            SynergyBonusDef::BonusHP(15),
                        ],
                    },
                    SynergyTierDef {
                        required: 4,
                        bonuses: vec![
                            SynergyBonusDef::BonusArmor(18),
                            SynergyBonusDef::BonusHP(40),
                        ],
                    },
                ],
            },
            // Vampire - lifesteal
            SynergyDef {
                id: "blood_hunger".to_string(),
                name: "Blood Hunger".to_string(),
                description: "Vampire items drain the life from your enemies.".to_string(),
                tag: SynergyTag::Vampire,
                tiers: vec![
                    SynergyTierDef {
                        required: 2,
                        bonuses: vec![SynergyBonusDef::Lifesteal(0.08)],
                    },
                    SynergyTierDef {
                        required: 3,
                        bonuses: vec![
                            SynergyBonusDef::Lifesteal(0.15),
                            SynergyBonusDef::BonusHP(20),
                        ],
                    },
                ],
            },
            // Beast - primal power
            SynergyDef {
                id: "primal_fury".to_string(),
                name: "Primal Fury".to_string(),
                description: "Beast items awaken your primal instincts.".to_string(),
                tag: SynergyTag::Beast,
                tiers: vec![
                    SynergyTierDef {
                        required: 2,
                        bonuses: vec![
                            SynergyBonusDef::BonusDamage(4),
                            SynergyBonusDef::BonusHP(10),
                        ],
                    },
                    SynergyTierDef {
                        required: 3,
                        bonuses: vec![
                            SynergyBonusDef::BonusDamage(8),
                            SynergyBonusDef::BonusHP(25),
                            SynergyBonusDef::CritChance(8.0),
                        ],
                    },
                ],
            },
        ],
    }
}
