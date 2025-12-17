//! Game state machine
//!
//! Manages the overall game state and transitions between different modes.

use std::time::{Duration, Instant};
use hecs::{World, Entity};
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::world::Map;
use crate::progression::Difficulty;
use crate::ecs::{Position, Health, Mana, Stamina, Stats, Experience};
use crate::save::{PlayerProfile, load_profile, save_profile};
use crate::data::DataManager;
use crate::audio::{AudioManager, SoundId};

/// The main game struct that holds all game data
pub struct Game {
    /// Current game state
    state: GameState,
    /// ECS world containing all entities
    world: World,
    /// Current dungeon map
    map: Option<Map>,
    /// Random number generator (seeded for reproducibility)
    rng: StdRng,
    /// Current floor number
    floor: u32,
    /// Current difficulty setting
    difficulty: Difficulty,
    /// Message log
    messages: Vec<GameMessage>,
    /// Accumulated time for ambient effects
    ambient_time: f32,
    /// The player entity
    player_entity: Option<Entity>,
    /// Counter for generating unique item IDs
    item_id_counter: u64,
    /// Used shrine positions (floor, x, y) - shrines can only be used once
    used_shrines: std::collections::HashSet<(u32, i32, i32)>,
    /// Persistent player profile
    profile: PlayerProfile,
    /// Accumulated mana regeneration (fractional)
    mana_regen_accum: f32,
    /// Accumulated stamina regeneration (fractional)
    stamina_regen_accum: f32,
    /// When the current run started (for playtime tracking)
    run_start_time: Option<Instant>,
    /// External game data (items, enemies, skills, synergies)
    data: DataManager,
    /// Audio manager for sound effects
    audio: AudioManager,
}

/// All possible game states
#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    /// Main menu screen
    MainMenu,
    /// Setting up a new run
    NewRun {
        seed: Option<u64>,
        difficulty: Difficulty,
    },
    /// Actively playing
    Playing(PlayingState),
    /// Game is paused
    Paused,
    /// Selecting save slot
    SaveSlots { selected: u8 },
    /// Selecting load slot
    LoadSlots { selected: u8 },
    /// Viewing achievements and stats
    Achievements,
    /// Player died
    GameOver {
        floor_reached: u32,
        cause_of_death: String,
    },
    /// Player won
    Victory,
    /// Exit the game
    Quit,
}

/// Sub-states while playing
#[derive(Debug, Clone, PartialEq)]
pub enum PlayingState {
    /// Moving around, real-time
    Exploring,
    /// Turn-based combat
    Combat,
    /// Viewing inventory
    Inventory,
    /// Talking to NPC
    Dialogue { npc_id: u64 },
    /// At a shrine (skills/enchanting)
    Shrine { shrine_type: ShrineType },
    /// Shopping at a merchant
    Shop { npc_entity: Entity },
    /// Viewing character sheet
    Character,
    /// Viewing full map
    MapView,
    /// Help screen
    Help,
}

/// Types of shrines the player can interact with
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShrineType {
    /// Learn new skills
    Skill,
    /// Enchant/modify items
    Enchanting,
    /// Rest and restore
    Rest,
    /// Corruption shrine (risk/reward)
    Corruption,
}

/// A message to display in the game log
#[derive(Debug, Clone)]
pub struct GameMessage {
    pub text: String,
    pub timestamp: f32,
    pub category: MessageCategory,
}

/// Categories for message filtering/coloring
#[derive(Debug, Clone, PartialEq)]
pub enum MessageCategory {
    Combat,
    Item,
    System,
    Lore,
    Warning,
}

impl Game {
    /// Create a new game instance
    pub fn new() -> Self {
        let profile = load_profile();
        let data = DataManager::new();
        let audio = AudioManager::new();
        Self {
            state: GameState::MainMenu,
            world: World::new(),
            map: None,
            rng: StdRng::from_entropy(),
            floor: 0,
            difficulty: Difficulty::Normal,
            messages: Vec::new(),
            ambient_time: 0.0,
            player_entity: None,
            item_id_counter: 1000, // Start at 1000 to reserve low IDs
            used_shrines: std::collections::HashSet::new(),
            profile,
            mana_regen_accum: 0.0,
            stamina_regen_accum: 0.0,
            run_start_time: None,
            data,
            audio,
        }
    }

    /// Get access to the game data manager
    pub fn data(&self) -> &DataManager {
        &self.data
    }

    /// Get mutable access to the audio manager
    pub fn audio(&mut self) -> &mut AudioManager {
        &mut self.audio
    }

    /// Play a sound effect
    pub fn play_sound(&mut self, sound_id: SoundId) {
        self.audio.play(sound_id);
    }

    /// Get the next item ID
    pub fn next_item_id(&mut self) -> u64 {
        let id = self.item_id_counter;
        self.item_id_counter += 1;
        id
    }

    /// Get the player entity
    pub fn player(&self) -> Option<Entity> {
        self.player_entity
    }

    /// Get player position
    pub fn player_position(&self) -> Option<Position> {
        self.player_entity.and_then(|e| {
            self.world.get::<&Position>(e).ok().map(|p| *p)
        })
    }

    /// Set player position
    pub fn set_player_position(&mut self, pos: Position) {
        if let Some(entity) = self.player_entity {
            if let Ok(mut p) = self.world.get::<&mut Position>(entity) {
                *p = pos;
            }
        }
    }

    /// Get player health
    pub fn player_health(&self) -> Option<Health> {
        self.player_entity.and_then(|e| {
            self.world.get::<&Health>(e).ok().map(|h| *h)
        })
    }

    /// Get player mana
    pub fn player_mana(&self) -> Option<Mana> {
        self.player_entity.and_then(|e| {
            self.world.get::<&Mana>(e).ok().map(|m| *m)
        })
    }

    /// Get player stamina
    pub fn player_stamina(&self) -> Option<Stamina> {
        self.player_entity.and_then(|e| {
            self.world.get::<&Stamina>(e).ok().map(|s| *s)
        })
    }

    /// Get player stats
    pub fn player_stats(&self) -> Option<Stats> {
        self.player_entity.and_then(|e| {
            self.world.get::<&Stats>(e).ok().map(|s| *s)
        })
    }

    /// Get player experience
    pub fn player_experience(&self) -> Option<Experience> {
        self.player_entity.and_then(|e| {
            self.world.get::<&Experience>(e).ok().map(|x| *x)
        })
    }

    /// Deal damage to player, returns true if player died
    pub fn damage_player(&mut self, amount: i32) -> bool {
        if let Some(entity) = self.player_entity {
            if let Ok(mut health) = self.world.get::<&mut Health>(entity) {
                health.take_damage(amount);
                return health.is_dead();
            }
        }
        false
    }

    /// Heal the player (considers equipment HP bonuses)
    pub fn heal_player(&mut self, amount: i32) {
        use crate::ecs::EquipmentComponent;

        if let Some(entity) = self.player_entity {
            // Get equipment HP bonus
            let eq_hp = self.world.get::<&EquipmentComponent>(entity)
                .map(|eq| eq.equipment.hp_bonus())
                .unwrap_or(0);

            if let Ok(mut health) = self.world.get::<&mut Health>(entity) {
                // Heal up to effective max (base + equipment bonus)
                let effective_max = health.max + eq_hp;
                let actual_heal = amount.min(effective_max - health.current);
                health.current += actual_heal;
            }
        }
    }

    /// Restore player mana (considers equipment MP bonuses)
    pub fn restore_mana(&mut self, amount: i32) {
        use crate::ecs::EquipmentComponent;

        if let Some(entity) = self.player_entity {
            // Get equipment MP bonus
            let eq_mp = self.world.get::<&EquipmentComponent>(entity)
                .map(|eq| eq.equipment.mp_bonus())
                .unwrap_or(0);

            if let Ok(mut mana) = self.world.get::<&mut Mana>(entity) {
                // Restore up to effective max (base + equipment bonus)
                let effective_max = mana.max + eq_mp;
                let actual_restore = amount.min(effective_max - mana.current);
                mana.current += actual_restore;
            }
        }
    }

    /// Restore player stamina
    pub fn restore_stamina(&mut self, amount: i32) {
        if let Some(entity) = self.player_entity {
            if let Ok(mut stamina) = self.world.get::<&mut Stamina>(entity) {
                stamina.restore(amount);
            }
        }
    }

    /// Get mutable RNG
    pub fn rng(&mut self) -> &mut StdRng {
        &mut self.rng
    }

    /// Check if a position is blocked by an entity
    pub fn is_blocked_by_entity(&self, pos: Position) -> bool {
        use crate::ecs::BlocksMovement;

        for (_, (entity_pos, _)) in self.world.query::<(&Position, &BlocksMovement)>().iter() {
            if entity_pos.x == pos.x && entity_pos.y == pos.y {
                return true;
            }
        }
        false
    }

    /// Get entity at position (if any blocking entity exists)
    pub fn get_blocking_entity_at(&self, pos: Position) -> Option<hecs::Entity> {
        use crate::ecs::BlocksMovement;

        for (entity, (entity_pos, _)) in self.world.query::<(&Position, &BlocksMovement)>().iter() {
            if entity_pos.x == pos.x && entity_pos.y == pos.y {
                return Some(entity);
            }
        }
        None
    }

    /// Get the current game state
    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// Set a new game state
    pub fn set_state(&mut self, state: GameState) {
        log::debug!("State transition: {:?} -> {:?}", self.state, state);
        self.state = state;
    }

    /// Get the ECS world
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Get mutable access to the ECS world
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Get the current map
    pub fn map(&self) -> Option<&Map> {
        self.map.as_ref()
    }

    /// Get mutable access to the current map
    pub fn map_mut(&mut self) -> Option<&mut Map> {
        self.map.as_mut()
    }

    /// Get the current floor number
    pub fn floor(&self) -> u32 {
        self.floor
    }

    /// Get the current difficulty
    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    /// Get the current biome based on floor
    pub fn biome(&self) -> crate::world::Biome {
        crate::world::generation::biome_for_floor(self.floor)
    }

    /// Get all messages
    pub fn messages(&self) -> &[GameMessage] {
        &self.messages
    }

    /// Add a message to the log
    pub fn add_message(&mut self, text: impl Into<String>, category: MessageCategory) {
        self.messages.push(GameMessage {
            text: text.into(),
            timestamp: self.ambient_time,
            category,
        });

        // Keep only last 100 messages
        if self.messages.len() > 100 {
            self.messages.remove(0);
        }
    }

    /// Update game state (called every frame)
    pub fn update(&mut self, delta: Duration) {
        let delta_secs = delta.as_secs_f32();

        match &self.state {
            GameState::Playing(PlayingState::Exploring) => {
                // Update ambient time for effects
                self.ambient_time += delta_secs;

                // Passive mana regeneration while exploring
                // Base: 1 MP every 3 seconds + INT/10 bonus
                self.regenerate_resources(delta_secs);
            }
            GameState::Playing(PlayingState::Combat) => {
                // Combat is turn-based, no time updates
                // (stamina can regen between turns via rest action)
            }
            _ => {}
        }
    }

    /// Regenerate mana and stamina over time
    fn regenerate_resources(&mut self, delta_secs: f32) {
        use crate::ecs::{Stats, EquipmentComponent};

        let player = match self.player_entity {
            Some(p) => p,
            None => return,
        };

        // Get INT for mana regen scaling
        let int_bonus = self.world
            .get::<&Stats>(player)
            .map(|s| s.intelligence as f32 / 10.0)
            .unwrap_or(0.0);

        // Mana regen: base 0.33 MP/sec (1 every 3 sec) + INT scaling
        let mana_per_sec = 0.33 + int_bonus * 0.1;

        // Stamina regen: base 0.5/sec
        let stamina_per_sec = 0.5;

        // Accumulate regen (fractional amounts)
        self.mana_regen_accum += mana_per_sec * delta_secs;
        self.stamina_regen_accum += stamina_per_sec * delta_secs;

        // Apply whole mana regen
        if self.mana_regen_accum >= 1.0 {
            let regen = self.mana_regen_accum as i32;
            self.mana_regen_accum -= regen as f32;

            // Use restore_mana which considers equipment bonuses
            // (we can't call self.restore_mana here due to borrow, so inline it)
            let eq_mp = self.world.get::<&EquipmentComponent>(player)
                .map(|eq| eq.equipment.mp_bonus())
                .unwrap_or(0);
            if let Ok(mut mana) = self.world.get::<&mut Mana>(player) {
                let effective_max = mana.max + eq_mp;
                let actual_regen = regen.min(effective_max - mana.current);
                mana.current += actual_regen;
            }
        }

        // Apply whole stamina regen
        if self.stamina_regen_accum >= 1.0 {
            let regen = self.stamina_regen_accum as i32;
            self.stamina_regen_accum -= regen as f32;

            if let Ok(mut stamina) = self.world.get::<&mut Stamina>(player) {
                stamina.restore(regen);
            }
        }
    }

    /// Start a new run with the given settings
    pub fn start_new_run(&mut self, seed: Option<u64>, difficulty: Difficulty) {
        // Record run start in profile and start playtime tracking
        self.profile.record_run_start();
        self.run_start_time = Some(Instant::now());
        if let Err(e) = save_profile(&self.profile) {
            log::warn!("Failed to save profile: {}", e);
        }

        // Reset game state
        self.world = World::new();
        self.floor = 1;
        self.difficulty = difficulty;
        self.messages.clear();
        self.ambient_time = 0.0;
        self.player_entity = None;
        self.item_id_counter = 1000;
        self.used_shrines.clear();

        // Seed RNG
        self.rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };

        // Generate first floor
        self.generate_floor();

        // Spawn player at start position
        if let Some(map) = &self.map {
            let start = map.start_pos;
            let player = crate::entities::spawn_player(&mut self.world, start);
            self.player_entity = Some(player);
        }

        // Transition to playing
        self.add_message(
            "You descend into the Hollowdeep...",
            MessageCategory::System
        );
        self.set_state(GameState::Playing(PlayingState::Exploring));
    }

    /// Generate the current floor's map
    fn generate_floor(&mut self) {
        use crate::world::generation::generate_floor;
        use crate::entities::{spawn_enemies_for_floor_with_zones, BossType, spawn_boss, spawn_npcs_for_floor, spawn_chests_for_floor};

        let biome = crate::world::generation::biome_for_floor(self.floor);
        self.map = Some(generate_floor(&mut self.rng, self.floor, biome));

        // Check if this is a boss floor
        let is_boss_floor = BossType::is_boss_floor(self.floor);

        // Spawn enemies with difficulty scaling (fewer on boss floors)
        if let Some(map) = &self.map {
            let spawn_positions = map.get_spawn_positions(5); // Min 5 tiles from player

            if is_boss_floor {
                // Boss floor: spawn the boss near the exit
                if let Some(boss_type) = BossType::for_floor(self.floor) {
                    // Spawn boss at exit position (player must defeat to proceed)
                    if let Some(exit_pos) = map.exit_pos {
                        spawn_boss(&mut self.world, boss_type, exit_pos);
                        log::info!("Spawned boss {} on floor {}", boss_type.name(), self.floor);
                    }
                }
                // Spawn fewer regular enemies on boss floors
                let reduced_positions: Vec<_> = spawn_positions.iter()
                    .take(spawn_positions.len() / 2)
                    .copied()
                    .collect();
                let enemies = spawn_enemies_for_floor_with_zones(
                    &mut self.world,
                    biome,
                    self.floor,
                    &reduced_positions,
                    map,
                    &mut self.rng,
                    self.difficulty,
                );
                log::info!("Spawned {} enemies on boss floor {}", enemies.len(), self.floor);

                // Spawn fewer chests on boss floors
                let chest_positions: Vec<_> = spawn_positions.iter()
                    .skip(spawn_positions.len() / 2)
                    .take(3)
                    .copied()
                    .collect();
                let chests = spawn_chests_for_floor(
                    &mut self.world,
                    self.floor,
                    biome,
                    &chest_positions,
                    &mut self.rng,
                );
                log::info!("Spawned {} chests on boss floor {}", chests.len(), self.floor);
            } else {
                // Normal floor: use zone-aware spawning for elite rooms
                let enemies = spawn_enemies_for_floor_with_zones(
                    &mut self.world,
                    biome,
                    self.floor,
                    &spawn_positions,
                    map,
                    &mut self.rng,
                    self.difficulty,
                );
                log::info!("Spawned {} enemies on floor {} ({:?} difficulty, {} elite zones)",
                    enemies.len(), self.floor, self.difficulty, map.elite_rooms.len());

                // Spawn NPCs on non-boss floors (use NPC-specific positions to avoid corridors)
                let npc_positions = map.get_npc_spawn_positions(8); // Further from start, not in narrow passages
                let _npcs = spawn_npcs_for_floor(
                    &mut self.world,
                    biome,
                    self.floor,
                    &npc_positions,
                    &mut self.rng,
                    &mut self.item_id_counter,
                );

                // Spawn chests on normal floors
                let chest_positions = map.get_spawn_positions(6); // Slightly further from start than enemies
                let chests = spawn_chests_for_floor(
                    &mut self.world,
                    self.floor,
                    biome,
                    &chest_positions,
                    &mut self.rng,
                );
                log::info!("Spawned {} chests on floor {}", chests.len(), self.floor);
            }
        }

        log::info!("Generated floor {} ({:?})", self.floor, biome);
    }

    /// Proceed to the next floor
    pub fn descend(&mut self) {
        use crate::entities::BossType;

        self.floor += 1;

        // Track floor descent in profile
        self.profile.record_floor_descent(self.floor);
        if let Err(e) = save_profile(&self.profile) {
            log::warn!("Failed to save profile: {}", e);
        }

        self.generate_floor();

        self.add_message(
            format!("You descend to floor {}...", self.floor),
            MessageCategory::System
        );

        // Boss floor warning
        if let Some(boss_type) = BossType::for_floor(self.floor) {
            self.add_message(
                format!("⚠ {} awaits!", boss_type.name()),
                MessageCategory::Warning
            );
            self.add_message(
                boss_type.phase_description(1).to_string(),
                MessageCategory::Lore
            );
        }
    }

    /// Tick status effects on the player (called on player actions/movement)
    pub fn tick_player_status_effects(&mut self) {
        use crate::ecs::{StatusEffects, Health, Name};
        use crate::combat::apply_status_damage;

        if let Some(player) = self.player_entity {
            // Get player name
            let player_name = self.world
                .get::<&Name>(player)
                .map(|n| n.0.clone())
                .unwrap_or_else(|_| "You".to_string());

            // Tick status effects
            let tick_result = {
                if let Ok(mut effects) = self.world.get::<&mut StatusEffects>(player) {
                    effects.tick(&player_name)
                } else {
                    return;
                }
            };

            // Apply damage/healing
            if tick_result.damage_dealt != 0 {
                if let Ok(mut health) = self.world.get::<&mut Health>(player) {
                    apply_status_damage(&mut health, &tick_result);
                }
            }

            // Add messages
            for msg in tick_result.messages {
                self.add_message(msg, MessageCategory::Combat);
            }
        }
    }

    /// Tick status effects on all enemies (called during AI tick)
    pub fn tick_enemy_status_effects(&mut self) {
        use crate::ecs::{StatusEffects, Health, Name, Enemy};

        // Collect entities to tick (avoid borrow issues)
        let entities: Vec<_> = self.world
            .query::<(&Enemy, &Name)>()
            .iter()
            .map(|(e, (_, name))| (e, name.0.clone()))
            .collect();

        let mut dead_entities = Vec::new();
        let mut messages_to_add = Vec::new();

        for (entity, name) in entities {
            // Tick status effects
            let tick_result = {
                if let Ok(mut effects) = self.world.get::<&mut StatusEffects>(entity) {
                    effects.tick(&name)
                } else {
                    continue;
                }
            };

            // Apply damage/healing first (before moving messages)
            let damage = tick_result.damage_dealt;
            if damage != 0 {
                if let Ok(mut health) = self.world.get::<&mut Health>(entity) {
                    let _ = if damage > 0 {
                        health.take_damage(damage)
                    } else {
                        health.heal(-damage)
                    };

                    // Check if enemy died from DoT
                    if health.is_dead() {
                        dead_entities.push((entity, name.clone()));
                    }
                }
            }

            // Collect messages after damage processing
            for msg in tick_result.messages {
                messages_to_add.push(msg);
            }
        }

        // Add all messages after releasing borrows
        for msg in messages_to_add {
            self.add_message(msg, MessageCategory::Combat);
        }

        // Remove dead entities and add death messages
        for (entity, name) in dead_entities {
            self.add_message(
                format!("{} succumbed to their wounds!", name),
                MessageCategory::Combat,
            );
            let _ = self.world.despawn(entity);
        }
    }

    /// Run AI for all enemies (called after player action)
    pub fn run_ai_tick(&mut self) {
        use crate::ecs::{run_enemy_ai, execute_ai_actions};

        // First, tick status effects on all enemies (DoT damage applies per turn)
        self.tick_enemy_status_effects();

        // Also tick player status effects (DoT applies on their turn too)
        self.tick_player_status_effects();

        let player_pos = match self.player_position() {
            Some(pos) => pos,
            None => return,
        };

        let map = match &self.map {
            Some(m) => m,
            None => return,
        };

        // Run AI to get actions (pass rng for slow effect chance)
        let actions = run_enemy_ai(&mut self.world, map, player_pos, &mut self.rng);

        // Execute the actions (need to pass rng for combat calculations)
        let messages = execute_ai_actions(&mut self.world, actions, self.player_entity, &mut self.rng);

        // Add combat messages
        for msg in messages {
            self.add_message(msg, MessageCategory::Combat);
        }

        // Check if player died (from combat or DoT)
        if let Some(health) = self.player_health() {
            if health.is_dead() {
                self.player_died("overwhelmed by the darkness");
            }
        }
    }

    /// Handle player death
    pub fn player_died(&mut self, cause: impl Into<String>) {
        // Add playtime from this run to profile stats
        if let Some(start_time) = self.run_start_time.take() {
            let elapsed = start_time.elapsed().as_secs();
            self.profile.add_playtime(elapsed);
        }

        // Update profile stats
        self.profile.record_death(self.floor);
        if self.floor == 1 {
            self.profile.unlock_achievement("die_on_floor_1");
        }
        if let Err(e) = save_profile(&self.profile) {
            log::warn!("Failed to save profile: {}", e);
        }

        self.set_state(GameState::GameOver {
            floor_reached: self.floor,
            cause_of_death: cause.into(),
        });
    }

    /// Handle victory
    pub fn player_won(&mut self) {
        // Add playtime from this run to profile stats
        if let Some(start_time) = self.run_start_time.take() {
            let elapsed = start_time.elapsed().as_secs();
            self.profile.add_playtime(elapsed);
        }

        // Update profile stats
        self.profile.record_victory();
        if let Err(e) = save_profile(&self.profile) {
            log::warn!("Failed to save profile: {}", e);
        }

        self.set_state(GameState::Victory);
    }

    /// Request to quit the game
    pub fn quit(&mut self) {
        self.set_state(GameState::Quit);
    }

    /// Check if a shrine at the given position has been used
    pub fn is_shrine_used(&self, pos: Position) -> bool {
        self.used_shrines.contains(&(self.floor, pos.x, pos.y))
    }

    /// Mark a shrine at the given position as used
    pub fn mark_shrine_used(&mut self, pos: Position) {
        self.used_shrines.insert((self.floor, pos.x, pos.y));
    }

    /// Restore game state from save data
    pub fn restore_from_save(&mut self, save: crate::save::SaveData) -> Result<(), String> {
        use crate::ecs::{
            Renderable, Name, FactionComponent, Faction, AI, AIState,
            BlocksMovement, XpReward, Enemy, EnemyArchetype,
            InventoryComponent, EquipmentComponent, SkillsComponent, StatPoints, GroundItem,
        };
        use crate::items::{Equipment, Inventory};
        use crate::world::{Map, Tile};

        // Reset world
        self.world = World::new();
        self.floor = save.game.floor;
        self.difficulty = save.game.difficulty;
        self.messages.clear();
        self.ambient_time = 0.0;

        // Restore map
        let mut map = Map::new(
            save.map.width,
            save.map.height,
            save.map.floor_number,
            save.map.biome,
        );
        for (i, tile_data) in save.map.tiles.into_iter().enumerate() {
            if i < map.tiles.len() {
                map.tiles[i] = Tile {
                    tile_type: tile_data.tile_type,
                    explored: tile_data.explored,
                    glyph: tile_data.glyph_override,
                    ..Default::default()
                };
            }
        }
        map.start_pos = Position::new(save.map.start_pos.0, save.map.start_pos.1);
        map.exit_pos = save.map.exit_pos.map(|(x, y)| Position::new(x, y));
        for (x, y) in save.map.elite_rooms {
            map.elite_rooms.push(Position::new(x, y));
        }
        self.map = Some(map);

        // Restore player
        let player_pos = Position::new(save.player.position.0, save.player.position.1);
        let player_stats = Stats::new(
            save.player.stats.strength,
            save.player.stats.dexterity,
            save.player.stats.intelligence,
            save.player.stats.vitality,
        );
        let mut player_health = Health::new(save.player.health.1);
        player_health.current = save.player.health.0;
        let mut player_mana = Mana::new(save.player.mana.1);
        player_mana.current = save.player.mana.0;
        let mut player_stamina = Stamina::new(save.player.stamina.1);
        player_stamina.current = save.player.stamina.0;
        let player_exp = Experience {
            level: save.player.experience.level,
            current_xp: save.player.experience.current,
            xp_to_next: save.player.experience.to_next_level,
        };

        // Restore inventory with gold and items
        let mut inventory = Inventory::new();
        inventory.add_gold(save.player.gold);
        for item in save.player.inventory {
            let _ = inventory.add_item(item);
        }

        // Restore equipment
        let mut equipment = Equipment::new();
        if let Some(item) = save.player.equipment.main_hand { equipment.equip(item); }
        if let Some(item) = save.player.equipment.off_hand { equipment.equip(item); }
        if let Some(item) = save.player.equipment.head { equipment.equip(item); }
        if let Some(item) = save.player.equipment.body { equipment.equip(item); }
        if let Some(item) = save.player.equipment.hands { equipment.equip(item); }
        if let Some(item) = save.player.equipment.feet { equipment.equip(item); }
        if let Some(item) = save.player.equipment.amulet { equipment.equip(item); }
        if let Some(item) = save.player.equipment.ring1 { equipment.equip(item); }
        if let Some(item) = save.player.equipment.ring2 { equipment.equip(item); }

        // Spawn player entity
        let player = self.world.spawn((
            player_pos,
            Renderable::new('@', (255, 255, 100)).with_order(1),
            Name::new("Player"),
            player_stats,
            player_health,
            player_mana,
            player_stamina,
            player_exp,
            FactionComponent(Faction::Player),
            BlocksMovement,
            InventoryComponent { inventory },
            EquipmentComponent { equipment },
            SkillsComponent { skills: save.player.skills },
            StatPoints(save.player.stat_points),
        ));
        self.player_entity = Some(player);

        // Restore enemies
        for enemy_data in save.enemies {
            let pos = Position::new(enemy_data.position.0, enemy_data.position.1);
            let stats = Stats::new(
                enemy_data.stats.strength,
                enemy_data.stats.dexterity,
                enemy_data.stats.intelligence,
                enemy_data.stats.vitality,
            );
            let mut health = Health::new(enemy_data.health.1);
            health.current = enemy_data.health.0;

            self.world.spawn((
                Name::new(&enemy_data.name),
                pos,
                Renderable::new(enemy_data.glyph, enemy_data.color).with_order(50),
                Enemy { archetype: EnemyArchetype::Melee }, // Default archetype
                stats,
                health,
                FactionComponent(Faction::Enemy),
                AI { state: AIState::Idle, target: None, home: pos },
                BlocksMovement,
                XpReward(enemy_data.xp_reward),
            ));
        }

        // Restore items on ground
        for item_data in save.items_on_ground {
            let pos = Position::new(item_data.position.0, item_data.position.1);
            self.world.spawn((
                pos,
                GroundItem { item: item_data.item.clone() },
                Renderable::new(item_data.item.glyph, item_data.item.rarity.color()).with_order(80),
            ));
        }

        // Set game state
        self.add_message("Game loaded successfully.", MessageCategory::System);
        self.set_state(GameState::Playing(PlayingState::Exploring));

        Ok(())
    }

    // =========================================================================
    // Profile methods
    // =========================================================================

    /// Get a reference to the player profile
    pub fn profile(&self) -> &PlayerProfile {
        &self.profile
    }

    /// Get a mutable reference to the player profile
    pub fn profile_mut(&mut self) -> &mut PlayerProfile {
        &mut self.profile
    }

    /// Record an enemy kill in the profile
    pub fn record_enemy_kill(&mut self, is_boss: bool) {
        self.profile.record_enemy_kill(is_boss);
        // Save periodically (every 10 kills to reduce I/O)
        if self.profile.stats.enemies_killed % 10 == 0 {
            if let Err(e) = save_profile(&self.profile) {
                log::warn!("Failed to save profile: {}", e);
            }
        }
    }

    /// Record gold collected in the profile
    pub fn record_gold_collected(&mut self, amount: u32) {
        self.profile.record_gold(amount);
    }

    /// Record an item found in the profile
    pub fn record_item_found(&mut self, item_id: &str) {
        self.profile.record_item_found(item_id);
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
