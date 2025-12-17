//! Enemy templates for data-driven enemy creation
//!
//! These templates are loaded from RON files and used to spawn enemies.

use serde::{Deserialize, Serialize};
use crate::ecs::{EnemyArchetype, Stats};
use crate::world::Biome;

/// A template for creating enemies from external data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyTemplate {
    /// Unique template ID for reference
    pub id: String,
    /// Display name
    pub name: String,
    /// Display glyph
    pub glyph: char,
    /// Foreground color (RGB)
    pub fg: (u8, u8, u8),
    /// Enemy behavior archetype
    pub archetype: EnemyArchetype,
    /// Base stats
    pub stats: Stats,
    /// Base HP
    pub hp: i32,
    /// XP reward for killing
    pub xp_value: u32,
    /// Biomes where this enemy can spawn
    pub biomes: Vec<Biome>,
    /// Optional description/lore
    pub description: Option<String>,
}

/// Collection of enemy templates
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnemyTemplates {
    pub templates: Vec<EnemyTemplate>,
}

impl EnemyTemplates {
    /// Find a template by ID
    pub fn find(&self, id: &str) -> Option<&EnemyTemplate> {
        self.templates.iter().find(|t| t.id == id)
    }

    /// Get all enemies for a specific biome
    pub fn for_biome(&self, biome: Biome) -> Vec<&EnemyTemplate> {
        self.templates.iter()
            .filter(|t| t.biomes.contains(&biome))
            .collect()
    }

    /// Get all elite/boss enemies
    pub fn elites(&self) -> Vec<&EnemyTemplate> {
        self.templates.iter()
            .filter(|t| matches!(t.archetype, EnemyArchetype::Elite | EnemyArchetype::Boss | EnemyArchetype::Tank))
            .collect()
    }
}

/// Create default enemy templates (hardcoded fallback)
pub fn default_enemy_templates() -> EnemyTemplates {
    EnemyTemplates {
        templates: vec![
            // === SUNKEN CATACOMBS (Floors 1-5) ===
            EnemyTemplate {
                id: "skeleton".to_string(),
                name: "Skeleton".to_string(),
                glyph: 's',
                fg: (200, 200, 180),
                archetype: EnemyArchetype::Melee,
                stats: Stats { strength: 8, dexterity: 6, intelligence: 2, vitality: 5 },
                hp: 25,
                xp_value: 15,
                biomes: vec![Biome::SunkenCatacombs, Biome::BleedingCrypts],
                description: Some("Reanimated bones held together by dark magic.".to_string()),
            },
            EnemyTemplate {
                id: "zombie".to_string(),
                name: "Zombie".to_string(),
                glyph: 'z',
                fg: (100, 140, 80),
                archetype: EnemyArchetype::Melee,
                stats: Stats { strength: 10, dexterity: 3, intelligence: 1, vitality: 8 },
                hp: 40,
                xp_value: 20,
                biomes: vec![Biome::SunkenCatacombs],
                description: Some("A shambling corpse driven by hunger.".to_string()),
            },
            EnemyTemplate {
                id: "ghost".to_string(),
                name: "Ghost".to_string(),
                glyph: 'g',
                fg: (180, 200, 255),
                archetype: EnemyArchetype::Caster,
                stats: Stats { strength: 4, dexterity: 8, intelligence: 12, vitality: 4 },
                hp: 20,
                xp_value: 25,
                biomes: vec![Biome::SunkenCatacombs],
                description: Some("A restless spirit bound to these halls.".to_string()),
            },
            EnemyTemplate {
                id: "rat_swarm".to_string(),
                name: "Rat Swarm".to_string(),
                glyph: 'r',
                fg: (140, 100, 80),
                archetype: EnemyArchetype::Swarm,
                stats: Stats { strength: 4, dexterity: 12, intelligence: 1, vitality: 3 },
                hp: 12,
                xp_value: 8,
                biomes: vec![Biome::SunkenCatacombs],
                description: Some("Dozens of rats moving as one hungry mass.".to_string()),
            },

            // === BLEEDING CRYPTS (Floors 6-10) ===
            EnemyTemplate {
                id: "blood_cultist".to_string(),
                name: "Blood Cultist".to_string(),
                glyph: 'c',
                fg: (180, 50, 50),
                archetype: EnemyArchetype::Caster,
                stats: Stats { strength: 6, dexterity: 10, intelligence: 14, vitality: 8 },
                hp: 35,
                xp_value: 35,
                biomes: vec![Biome::BleedingCrypts, Biome::HollowCathedral],
                description: Some("A devoted follower of the crimson faith.".to_string()),
            },
            EnemyTemplate {
                id: "crimson_hound".to_string(),
                name: "Crimson Hound".to_string(),
                glyph: 'h',
                fg: (200, 60, 60),
                archetype: EnemyArchetype::Melee,
                stats: Stats { strength: 12, dexterity: 14, intelligence: 3, vitality: 7 },
                hp: 30,
                xp_value: 30,
                biomes: vec![Biome::BleedingCrypts],
                description: Some("A twisted beast bred in blood.".to_string()),
            },
            EnemyTemplate {
                id: "flesh_golem".to_string(),
                name: "Flesh Golem".to_string(),
                glyph: 'G',
                fg: (160, 100, 100),
                archetype: EnemyArchetype::Tank,
                stats: Stats { strength: 16, dexterity: 4, intelligence: 2, vitality: 18 },
                hp: 80,
                xp_value: 50,
                biomes: vec![Biome::BleedingCrypts],
                description: Some("A hulking monstrosity stitched from corpses.".to_string()),
            },

            // === HOLLOW CATHEDRAL (Floors 11-15) ===
            EnemyTemplate {
                id: "fallen_knight".to_string(),
                name: "Fallen Knight".to_string(),
                glyph: 'K',
                fg: (120, 120, 140),
                archetype: EnemyArchetype::Elite,
                stats: Stats { strength: 14, dexterity: 10, intelligence: 6, vitality: 14 },
                hp: 70,
                xp_value: 60,
                biomes: vec![Biome::HollowCathedral],
                description: Some("Once a guardian, now corrupted by darkness.".to_string()),
            },
            EnemyTemplate {
                id: "corrupted_angel".to_string(),
                name: "Corrupted Angel".to_string(),
                glyph: 'A',
                fg: (200, 180, 255),
                archetype: EnemyArchetype::Caster,
                stats: Stats { strength: 8, dexterity: 12, intelligence: 18, vitality: 10 },
                hp: 55,
                xp_value: 70,
                biomes: vec![Biome::HollowCathedral, Biome::TheAbyss],
                description: Some("Divine grace twisted into unholy wrath.".to_string()),
            },
            EnemyTemplate {
                id: "gargoyle".to_string(),
                name: "Gargoyle".to_string(),
                glyph: 'g',
                fg: (100, 100, 110),
                archetype: EnemyArchetype::Ranged,
                stats: Stats { strength: 10, dexterity: 8, intelligence: 4, vitality: 12 },
                hp: 50,
                xp_value: 45,
                biomes: vec![Biome::HollowCathedral],
                description: Some("Stone given malevolent life.".to_string()),
            },

            // === THE ABYSS (Floors 16-20) ===
            EnemyTemplate {
                id: "void_spawn".to_string(),
                name: "Void Spawn".to_string(),
                glyph: 'v',
                fg: (80, 40, 120),
                archetype: EnemyArchetype::Swarm,
                stats: Stats { strength: 8, dexterity: 16, intelligence: 8, vitality: 6 },
                hp: 25,
                xp_value: 40,
                biomes: vec![Biome::TheAbyss],
                description: Some("A fragment of the endless void.".to_string()),
            },
            EnemyTemplate {
                id: "eldritch_horror".to_string(),
                name: "Eldritch Horror".to_string(),
                glyph: 'E',
                fg: (100, 60, 160),
                archetype: EnemyArchetype::Elite,
                stats: Stats { strength: 18, dexterity: 8, intelligence: 20, vitality: 16 },
                hp: 100,
                xp_value: 100,
                biomes: vec![Biome::TheAbyss],
                description: Some("An abomination from beyond reality.".to_string()),
            },
            EnemyTemplate {
                id: "tentacle".to_string(),
                name: "Tentacle".to_string(),
                glyph: 't',
                fg: (60, 80, 100),
                archetype: EnemyArchetype::Melee,
                stats: Stats { strength: 14, dexterity: 6, intelligence: 4, vitality: 10 },
                hp: 45,
                xp_value: 35,
                biomes: vec![Biome::TheAbyss],
                description: Some("A grasping appendage of something vast.".to_string()),
            },
        ],
    }
}
