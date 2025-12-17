//! Boss entity creation and management
//!
//! Bosses are powerful multi-phase enemies that appear at the end of each biome.

use hecs::{World, Entity};
use crate::ecs::{
    Position, Renderable, Name, Enemy, EnemyArchetype, Stats, Health,
    FactionComponent, Faction, AI, AIState, BlocksMovement, XpReward,
};
use crate::world::Biome;

/// Boss-specific component tracking phase and abilities
#[derive(Debug, Clone)]
pub struct BossComponent {
    /// Which boss this is
    pub boss_type: BossType,
    /// Current phase (1, 2, or 3)
    pub phase: u8,
    /// Turns until next special attack
    pub special_cooldown: u8,
    /// Whether the boss has been defeated
    pub defeated: bool,
}

/// Types of bosses, one per biome
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BossType {
    /// Floor 5 - Sunken Catacombs boss
    CryptLord,
    /// Floor 10 - Bleeding Crypts boss
    BloodMother,
    /// Floor 15 - Hollow Cathedral boss
    FallenSeraph,
    /// Floor 20 - The Abyss boss
    VoidHarbinger,
}

impl BossType {
    /// Get boss for a given floor (only boss floors)
    pub fn for_floor(floor: u32) -> Option<Self> {
        match floor {
            5 => Some(BossType::CryptLord),
            10 => Some(BossType::BloodMother),
            15 => Some(BossType::FallenSeraph),
            20 => Some(BossType::VoidHarbinger),
            _ => None,
        }
    }

    /// Check if a floor is a boss floor
    pub fn is_boss_floor(floor: u32) -> bool {
        matches!(floor, 5 | 10 | 15 | 20)
    }

    /// Get the boss name
    pub fn name(&self) -> &'static str {
        match self {
            BossType::CryptLord => "The Crypt Lord",
            BossType::BloodMother => "The Blood Mother",
            BossType::FallenSeraph => "Fallen Seraph",
            BossType::VoidHarbinger => "Void Harbinger",
        }
    }

    /// Get the boss glyph
    pub fn glyph(&self) -> char {
        match self {
            BossType::CryptLord => 'L',
            BossType::BloodMother => 'M',
            BossType::FallenSeraph => 'S',
            BossType::VoidHarbinger => 'V',
        }
    }

    /// Get the boss color
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            BossType::CryptLord => (200, 180, 150),
            BossType::BloodMother => (200, 50, 50),
            BossType::FallenSeraph => (220, 200, 255),
            BossType::VoidHarbinger => (100, 50, 200),
        }
    }

    /// Get base stats for this boss
    pub fn base_stats(&self) -> Stats {
        match self {
            BossType::CryptLord => Stats {
                strength: 18,
                dexterity: 10,
                intelligence: 8,
                vitality: 20,
            },
            BossType::BloodMother => Stats {
                strength: 14,
                dexterity: 12,
                intelligence: 18,
                vitality: 16,
            },
            BossType::FallenSeraph => Stats {
                strength: 16,
                dexterity: 16,
                intelligence: 20,
                vitality: 18,
            },
            BossType::VoidHarbinger => Stats {
                strength: 22,
                dexterity: 14,
                intelligence: 24,
                vitality: 25,
            },
        }
    }

    /// Get base HP for this boss
    pub fn base_hp(&self) -> i32 {
        match self {
            BossType::CryptLord => 200,
            BossType::BloodMother => 250,
            BossType::FallenSeraph => 300,
            BossType::VoidHarbinger => 400,
        }
    }

    /// Get XP reward for killing this boss
    pub fn xp_reward(&self) -> u32 {
        match self {
            BossType::CryptLord => 200,
            BossType::BloodMother => 400,
            BossType::FallenSeraph => 600,
            BossType::VoidHarbinger => 1000,
        }
    }

    /// Get the health thresholds for phase transitions (as percentages)
    pub fn phase_thresholds(&self) -> (f32, f32) {
        // Phase 2 at 66%, Phase 3 at 33%
        (0.66, 0.33)
    }

    /// Get special attack cooldown for this boss
    pub fn special_cooldown(&self) -> u8 {
        match self {
            BossType::CryptLord => 4,
            BossType::BloodMother => 3,
            BossType::FallenSeraph => 3,
            BossType::VoidHarbinger => 2,
        }
    }

    /// Get description for current phase
    pub fn phase_description(&self, phase: u8) -> &'static str {
        match (self, phase) {
            (BossType::CryptLord, 1) => "The ancient lord awakens, bones rattling with fury.",
            (BossType::CryptLord, 2) => "Dark energy surrounds the Crypt Lord. Skeletons rise to aid him!",
            (BossType::CryptLord, 3) => "Desperate and enraged, the Crypt Lord's attacks become frenzied!",

            (BossType::BloodMother, 1) => "The Blood Mother emerges from her crimson pool.",
            (BossType::BloodMother, 2) => "Blood tendrils lash out as she begins her dark ritual!",
            (BossType::BloodMother, 3) => "The Blood Mother sacrifices her minions to heal herself!",

            (BossType::FallenSeraph, 1) => "Once holy, now twisted. The Seraph descends.",
            (BossType::FallenSeraph, 2) => "Corrupted light radiates from the Seraph's broken wings!",
            (BossType::FallenSeraph, 3) => "In divine fury, the Seraph unleashes forbidden powers!",

            (BossType::VoidHarbinger, 1) => "Reality tears as the Harbinger manifests.",
            (BossType::VoidHarbinger, 2) => "The void spreads. Your vision warps and twists!",
            (BossType::VoidHarbinger, 3) => "The Harbinger prepares to unmake everything!",

            _ => "The boss grows more dangerous!",
        }
    }
}

/// Spawn a boss entity
pub fn spawn_boss(world: &mut World, boss_type: BossType, pos: Position) -> Entity {
    let color = boss_type.color();

    world.spawn((
        Name::new(boss_type.name()),
        pos,
        Renderable::new(boss_type.glyph(), color).with_order(100), // Bosses render on top
        Enemy { archetype: EnemyArchetype::Elite },
        boss_type.base_stats(),
        Health::new(boss_type.base_hp()),
        FactionComponent(Faction::Enemy),
        AI {
            state: AIState::Idle,
            target: None,
            home: pos,
        },
        BlocksMovement,
        XpReward(boss_type.xp_reward()),
        BossComponent {
            boss_type,
            phase: 1,
            special_cooldown: boss_type.special_cooldown(),
            defeated: false,
        },
    ))
}

/// Check and update boss phase based on current health
pub fn update_boss_phase(health: &Health, boss: &mut BossComponent) -> Option<u8> {
    let hp_percent = health.percentage();
    let (phase2_threshold, phase3_threshold) = boss.boss_type.phase_thresholds();

    let new_phase = if hp_percent <= phase3_threshold {
        3
    } else if hp_percent <= phase2_threshold {
        2
    } else {
        1
    };

    if new_phase > boss.phase {
        boss.phase = new_phase;
        Some(new_phase)
    } else {
        None
    }
}

/// Get boss for the given biome (used for spawning)
pub fn boss_for_biome(biome: Biome) -> BossType {
    match biome {
        Biome::SunkenCatacombs => BossType::CryptLord,
        Biome::BleedingCrypts => BossType::BloodMother,
        Biome::HollowCathedral => BossType::FallenSeraph,
        Biome::TheAbyss => BossType::VoidHarbinger,
    }
}
