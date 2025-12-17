//! Enemy entity creation
//!
//! Defines enemy types and spawning functions for different biomes.
//! Includes floor-based difficulty scaling.

use hecs::{World, Entity};
use crate::ecs::{
    Position, Renderable, Name, Enemy, EnemyArchetype, Stats, Health,
    FactionComponent, Faction, AI, AIState, BlocksMovement, XpReward,
    StatusEffects,
};
use crate::world::Biome;
use crate::progression::FloorScaling;

/// Enemy definition with all stats and rendering info
pub struct EnemyDef {
    pub name: &'static str,
    pub glyph: char,
    pub fg: (u8, u8, u8),
    pub archetype: EnemyArchetype,
    pub stats: Stats,
    pub hp: i32,
    pub xp_value: u32,
}

// =============================================================================
// Sunken Catacombs Enemies (Floors 1-5)
// =============================================================================

pub const SKELETON: EnemyDef = EnemyDef {
    name: "Skeleton",
    glyph: 's',
    fg: (200, 200, 180),
    archetype: EnemyArchetype::Melee,
    stats: Stats { strength: 8, dexterity: 6, intelligence: 2, vitality: 5 },
    hp: 25,
    xp_value: 15,
};

pub const ZOMBIE: EnemyDef = EnemyDef {
    name: "Zombie",
    glyph: 'z',
    fg: (100, 140, 80),
    archetype: EnemyArchetype::Melee,
    stats: Stats { strength: 10, dexterity: 3, intelligence: 1, vitality: 8 },
    hp: 40,
    xp_value: 20,
};

pub const GHOST: EnemyDef = EnemyDef {
    name: "Ghost",
    glyph: 'g',
    fg: (180, 200, 255),
    archetype: EnemyArchetype::Caster,
    stats: Stats { strength: 4, dexterity: 8, intelligence: 12, vitality: 4 },
    hp: 20,
    xp_value: 25,
};

pub const RAT_SWARM: EnemyDef = EnemyDef {
    name: "Rat Swarm",
    glyph: 'r',
    fg: (140, 100, 80),
    archetype: EnemyArchetype::Swarm,
    stats: Stats { strength: 4, dexterity: 12, intelligence: 1, vitality: 3 },
    hp: 12,
    xp_value: 8,
};

// =============================================================================
// Bleeding Crypts Enemies (Floors 6-10)
// =============================================================================

pub const BLOOD_CULTIST: EnemyDef = EnemyDef {
    name: "Blood Cultist",
    glyph: 'c',
    fg: (180, 50, 50),
    archetype: EnemyArchetype::Caster,
    stats: Stats { strength: 6, dexterity: 10, intelligence: 14, vitality: 8 },
    hp: 35,
    xp_value: 35,
};

pub const CRIMSON_HOUND: EnemyDef = EnemyDef {
    name: "Crimson Hound",
    glyph: 'h',
    fg: (200, 60, 60),
    archetype: EnemyArchetype::Melee,
    stats: Stats { strength: 12, dexterity: 14, intelligence: 3, vitality: 7 },
    hp: 30,
    xp_value: 30,
};

pub const FLESH_GOLEM: EnemyDef = EnemyDef {
    name: "Flesh Golem",
    glyph: 'G',
    fg: (160, 100, 100),
    archetype: EnemyArchetype::Tank,
    stats: Stats { strength: 16, dexterity: 4, intelligence: 2, vitality: 18 },
    hp: 80,
    xp_value: 50,
};

// =============================================================================
// Hollow Cathedral Enemies (Floors 11-15)
// =============================================================================

pub const FALLEN_KNIGHT: EnemyDef = EnemyDef {
    name: "Fallen Knight",
    glyph: 'K',
    fg: (120, 120, 140),
    archetype: EnemyArchetype::Elite,
    stats: Stats { strength: 14, dexterity: 10, intelligence: 6, vitality: 14 },
    hp: 70,
    xp_value: 60,
};

pub const CORRUPTED_ANGEL: EnemyDef = EnemyDef {
    name: "Corrupted Angel",
    glyph: 'A',
    fg: (200, 180, 255),
    archetype: EnemyArchetype::Caster,
    stats: Stats { strength: 8, dexterity: 12, intelligence: 18, vitality: 10 },
    hp: 55,
    xp_value: 70,
};

pub const GARGOYLE: EnemyDef = EnemyDef {
    name: "Gargoyle",
    glyph: 'g',
    fg: (100, 100, 110),
    archetype: EnemyArchetype::Ranged,
    stats: Stats { strength: 10, dexterity: 8, intelligence: 4, vitality: 12 },
    hp: 50,
    xp_value: 45,
};

// =============================================================================
// The Abyss Enemies (Floors 16-20)
// =============================================================================

pub const VOID_SPAWN: EnemyDef = EnemyDef {
    name: "Void Spawn",
    glyph: 'v',
    fg: (80, 40, 120),
    archetype: EnemyArchetype::Swarm,
    stats: Stats { strength: 8, dexterity: 16, intelligence: 8, vitality: 6 },
    hp: 25,
    xp_value: 40,
};

pub const ELDRITCH_HORROR: EnemyDef = EnemyDef {
    name: "Eldritch Horror",
    glyph: 'E',
    fg: (100, 60, 160),
    archetype: EnemyArchetype::Elite,
    stats: Stats { strength: 18, dexterity: 8, intelligence: 20, vitality: 16 },
    hp: 100,
    xp_value: 100,
};

pub const TENTACLE: EnemyDef = EnemyDef {
    name: "Tentacle",
    glyph: 't',
    fg: (60, 80, 100),
    archetype: EnemyArchetype::Melee,
    stats: Stats { strength: 14, dexterity: 6, intelligence: 4, vitality: 10 },
    hp: 45,
    xp_value: 35,
};

// =============================================================================
// Spawning Functions
// =============================================================================

/// Spawn an enemy from a definition at a given position (no scaling)
pub fn spawn_enemy(world: &mut World, def: &EnemyDef, pos: Position) -> Entity {
    world.spawn((
        Name::new(def.name),
        pos,
        Renderable::new(def.glyph, def.fg).with_order(50),
        Enemy { archetype: def.archetype },
        def.stats,
        Health::new(def.hp),
        FactionComponent(Faction::Enemy),
        AI {
            state: AIState::Idle,
            target: None,
            home: pos,
        },
        BlocksMovement,
        XpReward(def.xp_value),
        StatusEffects::default(),
    ))
}

/// Spawn an enemy with floor-based difficulty scaling applied
pub fn spawn_enemy_scaled(
    world: &mut World,
    def: &EnemyDef,
    pos: Position,
    scaling: &FloorScaling,
) -> Entity {
    // Scale stats
    let scaled_stats = Stats {
        strength: scaling.scale_stat(def.stats.strength),
        dexterity: scaling.scale_stat(def.stats.dexterity),
        intelligence: scaling.scale_stat(def.stats.intelligence),
        vitality: scaling.scale_stat(def.stats.vitality),
    };

    // Scale HP and XP
    let scaled_hp = scaling.scale_enemy_hp(def.hp);
    let scaled_xp = scaling.scale_xp(def.xp_value);

    world.spawn((
        Name::new(def.name),
        pos,
        Renderable::new(def.glyph, def.fg).with_order(50),
        Enemy { archetype: def.archetype },
        scaled_stats,
        Health::new(scaled_hp),
        FactionComponent(Faction::Enemy),
        AI {
            state: AIState::Idle,
            target: None,
            home: pos,
        },
        BlocksMovement,
        XpReward(scaled_xp),
        StatusEffects::default(),
    ))
}

/// Get the enemy pool for a given biome
pub fn enemies_for_biome(biome: Biome) -> Vec<&'static EnemyDef> {
    match biome {
        Biome::SunkenCatacombs => vec![&SKELETON, &ZOMBIE, &GHOST, &RAT_SWARM],
        Biome::BleedingCrypts => vec![&BLOOD_CULTIST, &CRIMSON_HOUND, &FLESH_GOLEM, &SKELETON],
        Biome::HollowCathedral => vec![&FALLEN_KNIGHT, &CORRUPTED_ANGEL, &GARGOYLE, &BLOOD_CULTIST],
        Biome::TheAbyss => vec![&VOID_SPAWN, &ELDRITCH_HORROR, &TENTACLE, &CORRUPTED_ANGEL],
    }
}

/// Get enemy count range for a floor
pub fn enemy_count_for_floor(floor: u32) -> (usize, usize) {
    match floor {
        1 => (3, 5),
        2..=3 => (4, 7),
        4..=5 => (5, 8),
        6..=10 => (6, 10),
        11..=15 => (7, 12),
        _ => (8, 15),
    }
}

/// Spawn enemies for a floor with difficulty scaling, returns list of spawned entities
pub fn spawn_enemies_for_floor(
    world: &mut World,
    biome: Biome,
    floor: u32,
    valid_positions: &[Position],
    rng: &mut impl rand::Rng,
    difficulty: crate::progression::Difficulty,
) -> Vec<Entity> {
    use rand::seq::SliceRandom;

    let scaling = FloorScaling::new(floor, difficulty);
    let enemy_pool = enemies_for_biome(biome);
    let (min_count, max_count) = enemy_count_for_floor(floor);

    // Apply difficulty scaling to enemy count
    let (count_bonus_min, count_bonus_max) = scaling.enemy_count_bonus();
    let count = rng.gen_range(min_count + count_bonus_min..=max_count + count_bonus_max);

    // Don't spawn more enemies than we have positions
    let count = count.min(valid_positions.len());

    // Choose random positions
    let mut positions = valid_positions.to_vec();
    positions.shuffle(rng);

    let mut spawned = Vec::with_capacity(count);

    // Check if we should spawn a guaranteed elite
    let spawn_elite = scaling.has_guaranteed_elite() && !enemy_pool.is_empty();
    let elite_enemies: Vec<_> = enemy_pool.iter()
        .filter(|e| e.archetype == EnemyArchetype::Elite || e.archetype == EnemyArchetype::Tank)
        .collect();

    for i in 0..count {
        // First enemy is elite if required
        let enemy_def = if i == 0 && spawn_elite && !elite_enemies.is_empty() {
            **elite_enemies.choose(rng).unwrap()
        } else {
            *enemy_pool.choose(rng).unwrap()
        };

        // Use scaled spawning
        let entity = spawn_enemy_scaled(world, enemy_def, positions[i], &scaling);
        spawned.push(entity);
    }

    spawned
}

/// Spawn enemies for a floor with elite zone support
/// Elite zones are guaranteed to have at least one enemy
pub fn spawn_enemies_for_floor_with_zones(
    world: &mut World,
    biome: Biome,
    floor: u32,
    valid_positions: &[Position],
    map: &crate::world::Map,
    rng: &mut impl rand::Rng,
    difficulty: crate::progression::Difficulty,
) -> Vec<Entity> {
    use rand::seq::SliceRandom;

    let scaling = FloorScaling::new(floor, difficulty);
    let enemy_pool = enemies_for_biome(biome);
    let (min_count, max_count) = enemy_count_for_floor(floor);

    // Apply difficulty scaling to enemy count
    let (count_bonus_min, count_bonus_max) = scaling.enemy_count_bonus();
    let count = rng.gen_range(min_count + count_bonus_min..=max_count + count_bonus_max);

    // Separate positions into elite zone and regular positions
    let mut elite_positions: Vec<Position> = valid_positions.iter()
        .filter(|pos| map.is_elite_zone(**pos))
        .copied()
        .collect();
    let mut regular_positions: Vec<Position> = valid_positions.iter()
        .filter(|pos| !map.is_elite_zone(**pos))
        .copied()
        .collect();

    elite_positions.shuffle(rng);
    regular_positions.shuffle(rng);

    let mut spawned = Vec::with_capacity(count);

    // Elite enemy pool
    let elite_enemies: Vec<_> = enemy_pool.iter()
        .filter(|e| e.archetype == EnemyArchetype::Elite || e.archetype == EnemyArchetype::Tank)
        .collect();

    // FIRST: Ensure each elite zone has at least one enemy
    // Group elite positions by their elite room center
    let elite_rooms = map.elite_rooms();
    for room_center in elite_rooms {
        // Find positions near this elite room center
        let nearby_elite_pos = elite_positions.iter()
            .find(|pos| {
                let dx = (pos.x - room_center.x).abs();
                let dy = (pos.y - room_center.y).abs();
                dx <= 5 && dy <= 5 // Within 5 tiles of room center
            })
            .copied();

        if let Some(pos) = nearby_elite_pos {
            // Remove this position from the pool
            elite_positions.retain(|p| *p != pos);

            // Spawn a strong enemy in this elite zone
            let enemy_def = if !elite_enemies.is_empty() {
                **elite_enemies.choose(rng).unwrap()
            } else {
                *enemy_pool.choose(rng).unwrap()
            };

            let elite_scaling = FloorScaling::elite_scaled(floor, difficulty);
            let entity = spawn_enemy_scaled(world, enemy_def, pos, &elite_scaling);
            spawned.push(entity);
        }
    }

    // THEN: Fill remaining count with regular spawns
    let remaining_count = count.saturating_sub(spawned.len());

    // Combine remaining positions, prioritizing elite zones slightly
    let mut all_remaining: Vec<Position> = elite_positions;
    all_remaining.extend(regular_positions);

    for i in 0..remaining_count.min(all_remaining.len()) {
        let pos = all_remaining[i];
        let is_elite_zone = map.is_elite_zone(pos);

        // In elite zones, spawn stronger enemies with better rewards
        let enemy_def = if is_elite_zone && !elite_enemies.is_empty() && rng.gen_bool(0.5) {
            **elite_enemies.choose(rng).unwrap()
        } else {
            *enemy_pool.choose(rng).unwrap()
        };

        // Create an elite scaling for elite zones (more HP/damage, more XP)
        let actual_scaling = if is_elite_zone {
            FloorScaling::elite_scaled(floor, difficulty)
        } else {
            scaling.clone()
        };

        let entity = spawn_enemy_scaled(world, enemy_def, pos, &actual_scaling);
        spawned.push(entity);
    }

    spawned
}
