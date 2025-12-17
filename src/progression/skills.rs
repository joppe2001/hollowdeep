//! Skill system
//!
//! Player abilities that can be used in combat.

use serde::{Deserialize, Serialize};
use rand::Rng;

/// Unique skill ID
pub type SkillId = u32;

/// Skill rarity determines power level and availability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl SkillRarity {
    pub fn name(&self) -> &'static str {
        match self {
            SkillRarity::Common => "Common",
            SkillRarity::Uncommon => "Uncommon",
            SkillRarity::Rare => "Rare",
            SkillRarity::Epic => "Epic",
            SkillRarity::Legendary => "Legendary",
        }
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            SkillRarity::Common => (180, 180, 180),
            SkillRarity::Uncommon => (30, 255, 30),
            SkillRarity::Rare => (30, 144, 255),
            SkillRarity::Epic => (163, 53, 238),
            SkillRarity::Legendary => (255, 165, 0),
        }
    }
}

/// Cost type for using a skill
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillCost {
    /// Costs mana points
    Mana(i32),
    /// Costs stamina points
    Stamina(i32),
    /// No cost, but has cooldown
    Cooldown,
    /// Limited uses per floor/rest
    Charge(u8),
}

/// Targeting type for skills
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetType {
    /// Hits a single enemy
    SingleEnemy,
    /// Hits self
    Self_,
    /// Hits all adjacent enemies
    AllAdjacent,
    /// Hits all enemies in range
    AllInRange(i32),
    /// Ground targeted (AoE)
    Ground { range: i32, radius: i32 },
}

/// Effect type of the skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillEffect {
    /// Deal damage to target
    Damage { base: i32, scaling_stat: ScalingStat },
    /// Heal self
    Heal { base: i32, scaling_stat: ScalingStat },
    /// Apply status effect
    ApplyStatus { status: StatusType, duration: u32, chance: f32 },
    /// Buff self
    BuffSelf { buff: BuffType, duration: u32 },
    /// Move/teleport
    Movement { range: i32 },
    /// Combined effects
    Multi(Vec<SkillEffect>),
}

/// Stat that the skill scales with
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScalingStat {
    Strength,
    Dexterity,
    Intelligence,
    None,
}

/// Status effects that skills can apply
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusType {
    Poison,
    Burn,
    Bleed,
    Slow,
    Stun,
    Weakness,
}

/// Buff types for self-buffs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuffType {
    Strength(i32),
    Dexterity(i32),
    Intelligence(i32),
    Vitality(i32),
    Armor(i32),
    Regeneration(i32),
    Haste,
    Shield(i32),
}

/// A skill definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: SkillId,
    pub name: String,
    pub description: String,
    pub icon: char,
    pub rarity: SkillRarity,
    pub cost: SkillCost,
    pub cooldown_turns: u8,
    pub target: TargetType,
    pub effect: SkillEffect,
}

impl Skill {
    /// Get the mana cost if any
    pub fn mana_cost(&self) -> i32 {
        match self.cost {
            SkillCost::Mana(n) => n,
            _ => 0,
        }
    }

    /// Get the stamina cost if any
    pub fn stamina_cost(&self) -> i32 {
        match self.cost {
            SkillCost::Stamina(n) => n,
            _ => 0,
        }
    }
}

/// Player's equipped skills (up to 5 slots)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EquippedSkills {
    /// Skills in slots 1-5 (None if slot is empty)
    pub slots: [Option<Skill>; 5],
    /// Remaining cooldowns for each slot
    pub cooldowns: [u8; 5],
    /// Remaining charges for charge-based skills
    pub charges: [u8; 5],
    /// All learned skills (including those not currently equipped)
    pub learned: Vec<Skill>,
}

impl EquippedSkills {
    pub fn new() -> Self {
        Self::default()
    }

    /// Learn a new skill (add to learned list if not already known)
    pub fn learn(&mut self, skill: Skill) {
        // Check if already learned (by ID)
        if !self.learned.iter().any(|s| s.id == skill.id) {
            self.learned.push(skill);
        }
    }

    /// Check if a skill is learned (by ID)
    pub fn has_learned(&self, skill_id: SkillId) -> bool {
        self.learned.iter().any(|s| s.id == skill_id)
    }

    /// Get all learned skills that are not currently equipped
    pub fn unequipped_skills(&self) -> Vec<&Skill> {
        self.learned.iter()
            .filter(|skill| !self.slots.iter().any(|s| s.as_ref().map(|eq| eq.id == skill.id).unwrap_or(false)))
            .collect()
    }

    /// Equip a skill to a slot (0-4)
    pub fn equip(&mut self, slot: usize, skill: Skill) {
        if slot < 5 {
            // Set initial charges if it's a charge skill
            let initial_charges = match skill.cost {
                SkillCost::Charge(n) => n,
                _ => 0,
            };
            // Make sure skill is learned
            self.learn(skill.clone());
            self.slots[slot] = Some(skill);
            self.cooldowns[slot] = 0;
            self.charges[slot] = initial_charges;
        }
    }

    /// Equip a skill from learned by index in learned list
    pub fn equip_from_learned(&mut self, slot: usize, learned_index: usize) -> bool {
        if slot >= 5 || learned_index >= self.learned.len() {
            return false;
        }
        let skill = self.learned[learned_index].clone();
        // Don't allow equipping if already in another slot
        if self.slots.iter().any(|s| s.as_ref().map(|eq| eq.id == skill.id).unwrap_or(false)) {
            return false;
        }
        let initial_charges = match skill.cost {
            SkillCost::Charge(n) => n,
            _ => 0,
        };
        self.slots[slot] = Some(skill);
        self.cooldowns[slot] = 0;
        self.charges[slot] = initial_charges;
        true
    }

    /// Remove a skill from a slot (keeps it in learned)
    pub fn unequip(&mut self, slot: usize) -> Option<Skill> {
        if slot < 5 {
            self.cooldowns[slot] = 0;
            self.charges[slot] = 0;
            self.slots[slot].take()
        } else {
            None
        }
    }

    /// Check if a skill can be used (has resources, not on cooldown)
    pub fn can_use(&self, slot: usize, current_mana: i32, current_stamina: i32) -> bool {
        if slot >= 5 {
            return false;
        }
        let skill = match &self.slots[slot] {
            Some(s) => s,
            None => return false,
        };

        // Check cooldown
        if self.cooldowns[slot] > 0 {
            return false;
        }

        // Check cost
        match skill.cost {
            SkillCost::Mana(n) => current_mana >= n,
            SkillCost::Stamina(n) => current_stamina >= n,
            SkillCost::Cooldown => true, // Cooldown already checked
            SkillCost::Charge(_) => self.charges[slot] > 0,
        }
    }

    /// Use a skill (deduct cost, start cooldown)
    pub fn use_skill(&mut self, slot: usize) -> Option<&Skill> {
        if slot >= 5 {
            return None;
        }
        let skill = self.slots[slot].as_ref()?;

        // Start cooldown
        self.cooldowns[slot] = skill.cooldown_turns;

        // Deduct charges if applicable
        if matches!(skill.cost, SkillCost::Charge(_)) {
            if self.charges[slot] > 0 {
                self.charges[slot] -= 1;
            }
        }

        self.slots[slot].as_ref()
    }

    /// Advance cooldowns by one turn
    pub fn tick_cooldowns(&mut self) {
        for cd in &mut self.cooldowns {
            if *cd > 0 {
                *cd -= 1;
            }
        }
    }

    /// Restore all charges (e.g., at a shrine)
    pub fn restore_charges(&mut self) {
        for (i, skill) in self.slots.iter().enumerate() {
            if let Some(s) = skill {
                if let SkillCost::Charge(n) = s.cost {
                    self.charges[i] = n;
                }
            }
        }
    }
}

// =============================================================================
// Starting Skills
// =============================================================================

/// Create the basic starting skill: Power Strike
pub fn skill_power_strike() -> Skill {
    Skill {
        id: 1,
        name: "Power Strike".to_string(),
        description: "A powerful attack dealing 150% weapon damage.".to_string(),
        icon: '⚔',
        rarity: SkillRarity::Common,
        cost: SkillCost::Stamina(15),
        cooldown_turns: 2,
        target: TargetType::SingleEnemy,
        effect: SkillEffect::Damage {
            base: 5,
            scaling_stat: ScalingStat::Strength,
        },
    }
}

/// Create a healing skill
pub fn skill_first_aid() -> Skill {
    Skill {
        id: 2,
        name: "First Aid".to_string(),
        description: "Heal yourself for 20 + VIT HP.".to_string(),
        icon: '❤',
        rarity: SkillRarity::Common,
        cost: SkillCost::Mana(20),
        cooldown_turns: 4,
        target: TargetType::Self_,
        effect: SkillEffect::Heal {
            base: 20,
            scaling_stat: ScalingStat::None,
        },
    }
}

// =============================================================================
// Common Skills (easy to find)
// =============================================================================

pub fn skill_quick_strike() -> Skill {
    Skill {
        id: 10,
        name: "Quick Strike".to_string(),
        description: "A fast attack with DEX scaling.".to_string(),
        icon: '⚡',
        rarity: SkillRarity::Common,
        cost: SkillCost::Stamina(8),
        cooldown_turns: 1,
        target: TargetType::SingleEnemy,
        effect: SkillEffect::Damage {
            base: 3,
            scaling_stat: ScalingStat::Dexterity,
        },
    }
}

pub fn skill_bandage() -> Skill {
    Skill {
        id: 11,
        name: "Bandage".to_string(),
        description: "Heal 15 HP. No cooldown but costs more.".to_string(),
        icon: '🩹',
        rarity: SkillRarity::Common,
        cost: SkillCost::Mana(25),
        cooldown_turns: 0,
        target: TargetType::Self_,
        effect: SkillEffect::Heal {
            base: 15,
            scaling_stat: ScalingStat::None,
        },
    }
}

pub fn skill_bash() -> Skill {
    Skill {
        id: 12,
        name: "Bash".to_string(),
        description: "Heavy blow. 30% chance to stun for 2 turns.".to_string(),
        icon: '💥',
        rarity: SkillRarity::Common,
        cost: SkillCost::Stamina(12),
        cooldown_turns: 3,
        target: TargetType::SingleEnemy,
        effect: SkillEffect::Multi(vec![
            SkillEffect::Damage {
                base: 4,
                scaling_stat: ScalingStat::Strength,
            },
            SkillEffect::ApplyStatus {
                status: StatusType::Stun,
                duration: 2,
                chance: 0.3,
            },
        ]),
    }
}

// =============================================================================
// Uncommon Skills
// =============================================================================

pub fn skill_envenom() -> Skill {
    Skill {
        id: 3,
        name: "Envenom".to_string(),
        description: "Strike with a poisoned blade. 60% chance to poison.".to_string(),
        icon: '☠',
        rarity: SkillRarity::Uncommon,
        cost: SkillCost::Stamina(10),
        cooldown_turns: 3,
        target: TargetType::SingleEnemy,
        effect: SkillEffect::Multi(vec![
            SkillEffect::Damage {
                base: 3,
                scaling_stat: ScalingStat::Dexterity,
            },
            SkillEffect::ApplyStatus {
                status: StatusType::Poison,
                duration: 5,
                chance: 0.6,
            },
        ]),
    }
}

pub fn skill_iron_skin() -> Skill {
    Skill {
        id: 4,
        name: "Iron Skin".to_string(),
        description: "Gain +5 armor for 5 turns.".to_string(),
        icon: '🛡',
        rarity: SkillRarity::Uncommon,
        cost: SkillCost::Mana(15),
        cooldown_turns: 6,
        target: TargetType::Self_,
        effect: SkillEffect::BuffSelf {
            buff: BuffType::Armor(5),
            duration: 5,
        },
    }
}

pub fn skill_burning_strike() -> Skill {
    Skill {
        id: 20,
        name: "Burning Strike".to_string(),
        description: "Fire-infused attack. 50% chance to burn.".to_string(),
        icon: '🔥',
        rarity: SkillRarity::Uncommon,
        cost: SkillCost::Mana(12),
        cooldown_turns: 2,
        target: TargetType::SingleEnemy,
        effect: SkillEffect::Multi(vec![
            SkillEffect::Damage {
                base: 5,
                scaling_stat: ScalingStat::Intelligence,
            },
            SkillEffect::ApplyStatus {
                status: StatusType::Burn,
                duration: 4,
                chance: 0.5,
            },
        ]),
    }
}

pub fn skill_battle_cry() -> Skill {
    Skill {
        id: 21,
        name: "Battle Cry".to_string(),
        description: "Boost STR by 3 for 4 turns.".to_string(),
        icon: '📢',
        rarity: SkillRarity::Uncommon,
        cost: SkillCost::Stamina(15),
        cooldown_turns: 5,
        target: TargetType::Self_,
        effect: SkillEffect::BuffSelf {
            buff: BuffType::Strength(3),
            duration: 4,
        },
    }
}

pub fn skill_recuperate() -> Skill {
    Skill {
        id: 22,
        name: "Recuperate".to_string(),
        description: "Heal over time. +3 HP/turn for 5 turns.".to_string(),
        icon: '💚',
        rarity: SkillRarity::Uncommon,
        cost: SkillCost::Mana(18),
        cooldown_turns: 6,
        target: TargetType::Self_,
        effect: SkillEffect::BuffSelf {
            buff: BuffType::Regeneration(3),
            duration: 5,
        },
    }
}

// =============================================================================
// Rare Skills
// =============================================================================

pub fn skill_whirlwind() -> Skill {
    Skill {
        id: 5,
        name: "Whirlwind".to_string(),
        description: "Attack all adjacent enemies.".to_string(),
        icon: '🌀',
        rarity: SkillRarity::Rare,
        cost: SkillCost::Stamina(25),
        cooldown_turns: 4,
        target: TargetType::AllAdjacent,
        effect: SkillEffect::Damage {
            base: 6,
            scaling_stat: ScalingStat::Strength,
        },
    }
}

pub fn skill_shadow_step() -> Skill {
    Skill {
        id: 30,
        name: "Shadow Step".to_string(),
        description: "Teleport up to 4 tiles away.".to_string(),
        icon: '👤',
        rarity: SkillRarity::Rare,
        cost: SkillCost::Stamina(20),
        cooldown_turns: 5,
        target: TargetType::Self_,
        effect: SkillEffect::Movement { range: 4 },
    }
}

pub fn skill_frost_nova() -> Skill {
    Skill {
        id: 31,
        name: "Frost Nova".to_string(),
        description: "Freeze all adjacent enemies. 70% slow for 3 turns.".to_string(),
        icon: '❄',
        rarity: SkillRarity::Rare,
        cost: SkillCost::Mana(22),
        cooldown_turns: 5,
        target: TargetType::AllAdjacent,
        effect: SkillEffect::Multi(vec![
            SkillEffect::Damage {
                base: 4,
                scaling_stat: ScalingStat::Intelligence,
            },
            SkillEffect::ApplyStatus {
                status: StatusType::Slow,
                duration: 3,
                chance: 0.7,
            },
        ]),
    }
}

pub fn skill_life_drain() -> Skill {
    Skill {
        id: 32,
        name: "Life Drain".to_string(),
        description: "Steal life from an enemy. Deals damage and heals you.".to_string(),
        icon: '🩸',
        rarity: SkillRarity::Rare,
        cost: SkillCost::Mana(20),
        cooldown_turns: 4,
        target: TargetType::SingleEnemy,
        effect: SkillEffect::Multi(vec![
            SkillEffect::Damage {
                base: 8,
                scaling_stat: ScalingStat::Intelligence,
            },
            SkillEffect::Heal {
                base: 8,
                scaling_stat: ScalingStat::None,
            },
        ]),
    }
}

pub fn skill_executioner() -> Skill {
    Skill {
        id: 33,
        name: "Executioner".to_string(),
        description: "Massive damage to a single target.".to_string(),
        icon: '⚰',
        rarity: SkillRarity::Rare,
        cost: SkillCost::Stamina(30),
        cooldown_turns: 5,
        target: TargetType::SingleEnemy,
        effect: SkillEffect::Damage {
            base: 15,
            scaling_stat: ScalingStat::Strength,
        },
    }
}

// =============================================================================
// Epic Skills
// =============================================================================

pub fn skill_berserker_rage() -> Skill {
    Skill {
        id: 40,
        name: "Berserker Rage".to_string(),
        description: "Go berserk! +5 STR, +3 DEX for 6 turns.".to_string(),
        icon: '😡',
        rarity: SkillRarity::Epic,
        cost: SkillCost::Charge(2),
        cooldown_turns: 0,
        target: TargetType::Self_,
        effect: SkillEffect::Multi(vec![
            SkillEffect::BuffSelf {
                buff: BuffType::Strength(5),
                duration: 6,
            },
            SkillEffect::BuffSelf {
                buff: BuffType::Dexterity(3),
                duration: 6,
            },
        ]),
    }
}

pub fn skill_chain_lightning() -> Skill {
    Skill {
        id: 41,
        name: "Chain Lightning".to_string(),
        description: "Lightning bounces to all enemies in range 3.".to_string(),
        icon: '⚡',
        rarity: SkillRarity::Epic,
        cost: SkillCost::Mana(35),
        cooldown_turns: 6,
        target: TargetType::AllInRange(3),
        effect: SkillEffect::Damage {
            base: 10,
            scaling_stat: ScalingStat::Intelligence,
        },
    }
}

pub fn skill_shield_wall() -> Skill {
    Skill {
        id: 42,
        name: "Shield Wall".to_string(),
        description: "Absorb 30 damage before taking HP loss.".to_string(),
        icon: '🏰',
        rarity: SkillRarity::Epic,
        cost: SkillCost::Mana(25),
        cooldown_turns: 8,
        target: TargetType::Self_,
        effect: SkillEffect::BuffSelf {
            buff: BuffType::Shield(30),
            duration: 10,
        },
    }
}

pub fn skill_assassinate() -> Skill {
    Skill {
        id: 43,
        name: "Assassinate".to_string(),
        description: "Critical strike with 100% bleed chance.".to_string(),
        icon: '🗡',
        rarity: SkillRarity::Epic,
        cost: SkillCost::Stamina(35),
        cooldown_turns: 6,
        target: TargetType::SingleEnemy,
        effect: SkillEffect::Multi(vec![
            SkillEffect::Damage {
                base: 12,
                scaling_stat: ScalingStat::Dexterity,
            },
            SkillEffect::ApplyStatus {
                status: StatusType::Bleed,
                duration: 5,
                chance: 1.0,
            },
        ]),
    }
}

// =============================================================================
// Legendary Skills
// =============================================================================

pub fn skill_meteor_strike() -> Skill {
    Skill {
        id: 50,
        name: "Meteor Strike".to_string(),
        description: "Call down a meteor! Massive AoE damage.".to_string(),
        icon: '☄',
        rarity: SkillRarity::Legendary,
        cost: SkillCost::Charge(1),
        cooldown_turns: 0,
        target: TargetType::Ground { range: 5, radius: 2 },
        effect: SkillEffect::Damage {
            base: 25,
            scaling_stat: ScalingStat::Intelligence,
        },
    }
}

pub fn skill_divine_intervention() -> Skill {
    Skill {
        id: 51,
        name: "Divine Intervention".to_string(),
        description: "Full heal and clear all debuffs.".to_string(),
        icon: '✨',
        rarity: SkillRarity::Legendary,
        cost: SkillCost::Charge(1),
        cooldown_turns: 0,
        target: TargetType::Self_,
        effect: SkillEffect::Heal {
            base: 100,
            scaling_stat: ScalingStat::None,
        },
    }
}

pub fn skill_deaths_embrace() -> Skill {
    Skill {
        id: 52,
        name: "Death's Embrace".to_string(),
        description: "Mark of death. Huge damage + poison + bleed.".to_string(),
        icon: '💀',
        rarity: SkillRarity::Legendary,
        cost: SkillCost::Mana(50),
        cooldown_turns: 8,
        target: TargetType::SingleEnemy,
        effect: SkillEffect::Multi(vec![
            SkillEffect::Damage {
                base: 20,
                scaling_stat: ScalingStat::Dexterity,
            },
            SkillEffect::ApplyStatus {
                status: StatusType::Poison,
                duration: 6,
                chance: 1.0,
            },
            SkillEffect::ApplyStatus {
                status: StatusType::Bleed,
                duration: 6,
                chance: 1.0,
            },
        ]),
    }
}

// =============================================================================
// Skill Collections
// =============================================================================

/// Get all available starting skills
pub fn starting_skills() -> Vec<Skill> {
    vec![
        skill_power_strike(),
        skill_first_aid(),
    ]
}

/// Get all skills by rarity
pub fn all_skills_by_rarity(rarity: SkillRarity) -> Vec<Skill> {
    match rarity {
        SkillRarity::Common => vec![
            skill_quick_strike(),
            skill_bandage(),
            skill_bash(),
        ],
        SkillRarity::Uncommon => vec![
            skill_envenom(),
            skill_iron_skin(),
            skill_burning_strike(),
            skill_battle_cry(),
            skill_recuperate(),
        ],
        SkillRarity::Rare => vec![
            skill_whirlwind(),
            skill_shadow_step(),
            skill_frost_nova(),
            skill_life_drain(),
            skill_executioner(),
        ],
        SkillRarity::Epic => vec![
            skill_berserker_rage(),
            skill_chain_lightning(),
            skill_shield_wall(),
            skill_assassinate(),
        ],
        SkillRarity::Legendary => vec![
            skill_meteor_strike(),
            skill_divine_intervention(),
            skill_deaths_embrace(),
        ],
    }
}

/// Roll a skill rarity based on floor
pub fn roll_skill_rarity(floor: u32, rng: &mut impl Rng) -> SkillRarity {
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

    // Legendary skills: 0% early, very rare even late
    // Epic skills: 1% early, up to 8% late
    // Rare skills: 10% early, up to 25% late
    // Uncommon: 30% early, up to 40% late
    // Common: remainder

    let legendary_threshold = match floor_tier {
        0 => 1000,  // 0% - no legendary skills floors 1-5
        1 => 997,   // 0.3% - floors 6-10
        2 => 992,   // 0.8% - floors 11-15
        3 => 985,   // 1.5% - floors 16-20
        _ => 975,   // 2.5% - floors 21+
    };

    let epic_threshold = match floor_tier {
        0 => 990,   // 1% - early floors
        1 => 975,   // 2.5% - floors 6-10
        2 => 955,   // 4.5% - floors 11-15
        3 => 930,   // 7% - floors 16-20
        _ => 900,   // 10% - floors 21+
    };

    let rare_threshold = match floor_tier {
        0 => 900,   // 10% - early floors
        1 => 865,   // 13.5% - floors 6-10
        2 => 820,   // 18% - floors 11-15
        3 => 770,   // 23% - floors 16-20
        _ => 720,   // 28% - floors 21+
    };

    let uncommon_threshold = match floor_tier {
        0 => 700,   // 30% - early floors
        1 => 650,   // 35% - floors 6-10
        2 => 600,   // 40% - floors 11-15
        3 => 550,   // 45% - floors 16-20
        _ => 500,   // 50% - floors 21+
    };

    if roll >= legendary_threshold {
        SkillRarity::Legendary
    } else if roll >= epic_threshold {
        SkillRarity::Epic
    } else if roll >= rare_threshold {
        SkillRarity::Rare
    } else if roll >= uncommon_threshold {
        SkillRarity::Uncommon
    } else {
        SkillRarity::Common
    }
}

/// Generate random skills for a shrine based on floor
pub fn generate_shrine_skills(floor: u32, count: usize, rng: &mut impl Rng) -> Vec<Skill> {
    let mut skills = Vec::new();

    for _ in 0..count {
        let rarity = roll_skill_rarity(floor, rng);
        let available = all_skills_by_rarity(rarity);

        if !available.is_empty() {
            let idx = rng.gen_range(0..available.len());
            skills.push(available[idx].clone());
        }
    }

    skills
}

/// Get all learnable skills (legacy - returns all non-starting skills)
pub fn learnable_skills() -> Vec<Skill> {
    let mut all = Vec::new();
    all.extend(all_skills_by_rarity(SkillRarity::Common));
    all.extend(all_skills_by_rarity(SkillRarity::Uncommon));
    all.extend(all_skills_by_rarity(SkillRarity::Rare));
    all.extend(all_skills_by_rarity(SkillRarity::Epic));
    all.extend(all_skills_by_rarity(SkillRarity::Legendary));
    all
}
