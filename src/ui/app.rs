//! Main UI Application
//!
//! Coordinates rendering and input handling across all screens.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::Rng;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Clear},
};

use crate::game::{Game, GameState, PlayingState, MessageCategory, ShrineType};
use crate::ecs::Position;
use crate::render::{RenderMode, TileRenderer, detect_render_mode};
use crate::world::TileType;
use crate::audio::SoundId;

/// Truncate a string to fit within max_len characters, adding "…" if truncated
fn truncate_name(name: &str, max_len: usize) -> String {
    if name.chars().count() <= max_len {
        name.to_string()
    } else if max_len <= 1 {
        "…".to_string()
    } else {
        let truncated: String = name.chars().take(max_len - 1).collect();
        format!("{}…", truncated)
    }
}

/// Main UI application
pub struct App {
    /// Current camera position for map rendering
    camera: Position,
    /// Message verbosity level (0=minimal, 1=detailed, 2=narrative) - reserved for Phase 7
    #[allow(dead_code)]
    message_verbosity: u8,
    /// Current render mode (ASCII, Unicode, Kitty)
    render_mode: RenderMode,
    /// Tile renderer instance
    tile_renderer: TileRenderer,
    /// Current inventory cursor position
    inventory_cursor: usize,
    /// Inventory tab (0=items, 1=equipment)
    inventory_tab: u8,
    /// Current inventory sort mode
    inventory_sort_mode: crate::items::SortMode,
    /// Character sheet selected slot (0-7 for equipment slots, 8-12 for skill slots)
    character_slot: usize,
    /// Skill selection mode in character screen (selecting from learned skills)
    skill_selection_mode: bool,
    /// Cursor for skill selection (index into learned skills)
    skill_selection_cursor: usize,
    /// Currently selected skill slot for swapping (0-4)
    skill_slot_to_swap: usize,
    /// Shop selection cursor
    shop_selection: usize,
    /// Shop mode: 0=Buy, 1=Sell
    shop_mode: u8,
    /// Sell selection cursor (index in player inventory)
    sell_selection: usize,
    /// Whether we're in equip selection mode (selecting item from inventory to equip)
    equip_selection_mode: bool,
    /// Cursor for equip selection (index into filtered inventory)
    equip_selection_cursor: usize,
    /// Enchanting shrine: selected enchantment option (0-5, plus rare 6 for +1 max slots)
    enchant_affix_cursor: usize,
    /// Enchanting shrine: swap mode (true when item is at max enchantments)
    enchant_swap_mode: bool,
    /// Enchanting shrine: which existing enchantment to swap (when in swap mode)
    enchant_swap_cursor: usize,
    /// Enchanting shrine: whether the rare +1 max slot option is available (5% chance)
    enchant_upgrade_available: bool,
    /// Enchanting shrine: which equipment slot is selected (None = choosing equipment, Some = choosing enchant)
    enchant_selected_slot: Option<crate::items::EquipSlot>,
    /// Enchanting shrine: cursor for selecting equipment
    enchant_equipment_cursor: usize,
    /// Skill shrine: randomly generated skills for this shrine visit
    shrine_skills: Vec<crate::progression::Skill>,
    /// Skill shrine: cursor for skill selection
    shrine_skill_cursor: usize,
    /// Skill shrine: swap mode (when all slots are full)
    shrine_skill_swap_mode: bool,
    /// Skill shrine: cursor for selecting which equipped skill to replace
    shrine_skill_swap_cursor: usize,
    /// Skill shrine: the skill pending to be learned (stored when entering swap mode)
    shrine_pending_skill: Option<crate::progression::Skill>,
    /// Help screen scroll position
    help_scroll: u16,
    /// Pending movement skill (e.g., Shadow Step) - stores the range when awaiting direction
    pending_movement_skill: Option<i32>,
    /// Whether we're showing the difficulty selection popup
    difficulty_selection_mode: bool,
    /// Currently highlighted difficulty option (0=Easy, 1=Normal, 2=Hard, 3=Nightmare)
    difficulty_selection_cursor: usize,
}

impl App {
    pub fn new() -> Self {
        let render_mode = detect_render_mode();
        log::info!("Using render mode: {:?}", render_mode);

        Self {
            camera: Position::new(0, 0),
            message_verbosity: 1,
            render_mode,
            tile_renderer: TileRenderer::new(render_mode),
            inventory_cursor: 0,
            inventory_tab: 0,
            inventory_sort_mode: crate::items::SortMode::Category,
            character_slot: 0,
            skill_selection_mode: false,
            skill_selection_cursor: 0,
            skill_slot_to_swap: 0,
            shop_selection: 0,
            shop_mode: 0,
            sell_selection: 0,
            equip_selection_mode: false,
            equip_selection_cursor: 0,
            enchant_affix_cursor: 0,
            enchant_swap_mode: false,
            enchant_swap_cursor: 0,
            enchant_upgrade_available: false,
            enchant_selected_slot: None,
            enchant_equipment_cursor: 0,
            shrine_skills: Vec::new(),
            shrine_skill_cursor: 0,
            shrine_skill_swap_mode: false,
            shrine_skill_swap_cursor: 0,
            shrine_pending_skill: None,
            help_scroll: 0,
            pending_movement_skill: None,
            difficulty_selection_mode: false,
            difficulty_selection_cursor: 1, // Default to Normal
        }
    }

    /// Get the current render mode
    pub fn render_mode(&self) -> RenderMode {
        self.render_mode
    }

    /// Cycle through render modes (for testing/user preference)
    pub fn cycle_render_mode(&mut self) {
        self.render_mode = match self.render_mode {
            RenderMode::Ascii => RenderMode::Unicode,
            RenderMode::Unicode => RenderMode::NerdFont,
            RenderMode::NerdFont => RenderMode::Kitty,
            RenderMode::Kitty => RenderMode::Ascii,
        };
        self.tile_renderer = TileRenderer::new(self.render_mode);
        log::info!("Switched to render mode: {:?}", self.render_mode);
    }

    /// Handle keyboard input, returns true if should quit
    pub fn handle_input(&mut self, key: KeyEvent, game: &mut Game) -> Result<bool> {
        // Global quit shortcut
        if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(true);
        }

        match game.state().clone() {
            GameState::MainMenu => self.handle_main_menu_input(key, game),
            GameState::Playing(playing_state) => {
                self.handle_playing_input(key, game, playing_state)
            }
            GameState::Paused => self.handle_pause_input(key, game),
            GameState::SaveSlots { selected } => self.handle_save_slots_input(key, game, selected),
            GameState::LoadSlots { selected } => self.handle_load_slots_input(key, game, selected),
            GameState::Achievements => self.handle_achievements_input(key, game),
            GameState::GameOver { .. } => self.handle_game_over_input(key, game),
            GameState::Victory => self.handle_victory_input(key, game),
            GameState::NewRun { .. } => self.handle_new_run_input(key, game),
            GameState::Quit => Ok(true),
        }
    }

    fn handle_main_menu_input(&mut self, key: KeyEvent, game: &mut Game) -> Result<bool> {
        // Check if we're in difficulty selection mode
        if self.difficulty_selection_mode {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.difficulty_selection_cursor > 0 {
                        game.play_sound(SoundId::MenuMove);
                        self.difficulty_selection_cursor -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.difficulty_selection_cursor < 3 {
                        game.play_sound(SoundId::MenuMove);
                        self.difficulty_selection_cursor += 1;
                    }
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    game.play_sound(SoundId::MenuSelect);
                    // Start new game with selected difficulty
                    let difficulty = match self.difficulty_selection_cursor {
                        0 => crate::progression::Difficulty::Easy,
                        1 => crate::progression::Difficulty::Normal,
                        2 => crate::progression::Difficulty::Hard,
                        3 => crate::progression::Difficulty::Nightmare,
                        _ => crate::progression::Difficulty::Normal,
                    };
                    self.difficulty_selection_mode = false;
                    game.start_new_run(None, difficulty);
                    // Sync camera to player position
                    if let Some(pos) = game.player_position() {
                        self.camera = pos;
                    }
                }
                KeyCode::Esc => {
                    game.play_sound(SoundId::MenuBack);
                    // Cancel difficulty selection
                    self.difficulty_selection_mode = false;
                }
                _ => {}
            }
            return Ok(false);
        }

        match key.code {
            KeyCode::Enter | KeyCode::Char('n') => {
                game.play_sound(SoundId::MenuSelect);
                // Show difficulty selection popup
                self.difficulty_selection_mode = true;
                self.difficulty_selection_cursor = 1; // Default to Normal
            }
            KeyCode::Char('l') => {
                // Open load game slot selection
                game.set_state(GameState::LoadSlots { selected: 0 });
            }
            KeyCode::Char('a') => {
                // View achievements and stats
                game.set_state(GameState::Achievements);
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                game.quit();
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_playing_input(
        &mut self,
        key: KeyEvent,
        game: &mut Game,
        state: PlayingState,
    ) -> Result<bool> {
        match state {
            PlayingState::Exploring => self.handle_exploring_input(key, game),
            PlayingState::Inventory => self.handle_inventory_input(key, game),
            PlayingState::Character => self.handle_character_input(key, game),
            PlayingState::MapView => self.handle_mapview_input(key, game),
            PlayingState::Help => self.handle_help_input(key, game),
            PlayingState::Shrine { shrine_type } => self.handle_shrine_input(key, game, shrine_type),
            PlayingState::Shop { npc_entity } => self.handle_shop_input(key, game, npc_entity),
            _ => Ok(false),
        }
    }

    fn handle_exploring_input(&mut self, key: KeyEvent, game: &mut Game) -> Result<bool> {
        // Check for pending movement skill (Shadow Step, etc.)
        if let Some(range) = self.pending_movement_skill {
            let direction: Option<(i32, i32)> = match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some((0, -1)),
                KeyCode::Down | KeyCode::Char('j') => Some((0, 1)),
                KeyCode::Left | KeyCode::Char('h') => Some((-1, 0)),
                KeyCode::Right | KeyCode::Char('l') => Some((1, 0)),
                KeyCode::Char('y') => Some((-1, -1)),
                KeyCode::Char('u') => Some((1, -1)),
                KeyCode::Char('b') => Some((-1, 1)),
                KeyCode::Char('n') => Some((1, 1)),
                KeyCode::Esc => {
                    // Cancel the movement skill
                    self.pending_movement_skill = None;
                    game.add_message("Movement cancelled.".to_string(), MessageCategory::System);
                    return Ok(false);
                }
                _ => None,
            };

            if let Some((dx, dy)) = direction {
                self.execute_movement_skill(game, dx, dy, range);
                self.pending_movement_skill = None;
                return Ok(false);
            }
            // If not a valid direction key, ignore
            return Ok(false);
        }

        match key.code {
            // Movement
            KeyCode::Up | KeyCode::Char('k') => self.try_move(game, 0, -1),
            KeyCode::Down | KeyCode::Char('j') => self.try_move(game, 0, 1),
            KeyCode::Left | KeyCode::Char('h') => self.try_move(game, -1, 0),
            KeyCode::Right | KeyCode::Char('l') => self.try_move(game, 1, 0),

            // Diagonal movement
            KeyCode::Char('y') => self.try_move(game, -1, -1),
            KeyCode::Char('u') => self.try_move(game, 1, -1),
            KeyCode::Char('b') => self.try_move(game, -1, 1),
            KeyCode::Char('n') => self.try_move(game, 1, 1),

            // Wait/Rest - skip turn, small HP and stamina regen
            KeyCode::Char('.') | KeyCode::Char(' ') => {
                // Small HP regen when resting (1 HP per rest)
                game.heal_player(1);
                // Stamina regenerates faster when resting (5 SP per rest)
                game.restore_stamina(5);
                // Mana also gets a small boost when resting (2 MP per rest)
                game.restore_mana(2);
                // Enemies still get their turn
                game.run_ai_tick();
            }

            // Interact with stairs
            KeyCode::Char('>') => {
                if let Some(map) = game.map() {
                    if Some(self.camera) == map.exit_pos {
                        game.play_sound(SoundId::Descend);
                        game.add_message("You descend deeper into the darkness...".to_string(), MessageCategory::System);
                        game.descend();
                        game.play_sound(SoundId::NewFloor);
                        if let Some(new_map) = game.map() {
                            self.camera = new_map.start_pos;
                        }
                    } else {
                        game.add_message("There are no stairs here.".to_string(), MessageCategory::System);
                    }
                }
            }

            // UI toggles
            KeyCode::Char('i') => {
                game.set_state(GameState::Playing(PlayingState::Inventory));
            }
            KeyCode::Char('c') => {
                game.set_state(GameState::Playing(PlayingState::Character));
            }
            KeyCode::Char('m') => {
                game.set_state(GameState::Playing(PlayingState::MapView));
            }
            KeyCode::Char('?') => {
                game.set_state(GameState::Playing(PlayingState::Help));
            }
            KeyCode::Esc => {
                game.set_state(GameState::Paused);
            }
            // Toggle render mode
            KeyCode::Char('r') => {
                self.cycle_render_mode();
            }
            // Pickup items
            KeyCode::Char('g') => {
                self.pickup_items(game);
            }
            // Interact with tile (shrines, etc.)
            KeyCode::Char('e') | KeyCode::Enter => {
                self.interact_with_tile(game);
            }
            // Use skills (1-5)
            KeyCode::Char('1') => self.use_skill(game, 0),
            KeyCode::Char('2') => self.use_skill(game, 1),
            KeyCode::Char('3') => self.use_skill(game, 2),
            KeyCode::Char('4') => self.use_skill(game, 3),
            KeyCode::Char('5') => self.use_skill(game, 4),
            _ => {}
        }
        Ok(false)
    }

    fn pickup_items(&mut self, game: &mut Game) {
        use crate::ecs::{GroundItem, InventoryComponent};

        let player_pos = match game.player_position() {
            Some(pos) => pos,
            None => return,
        };

        // Find all items within pickup range (on tile or adjacent - Chebyshev distance <= 1)
        let items_in_range: Vec<(hecs::Entity, crate::items::Item, i32)> = game.world()
            .query::<(&Position, &GroundItem)>()
            .iter()
            .filter_map(|(e, (pos, gi))| {
                let dist = player_pos.chebyshev_distance(pos);
                if dist <= 1 {
                    Some((e, gi.item.clone(), dist))
                } else {
                    None
                }
            })
            .collect();

        if items_in_range.is_empty() {
            game.add_message("Nothing to pick up nearby.".to_string(), MessageCategory::System);
            return;
        }

        // Get player entity
        let player = match game.player() {
            Some(p) => p,
            None => return,
        };

        // Sort by distance (pick up items on same tile first)
        let mut items_sorted = items_in_range;
        items_sorted.sort_by_key(|(_, _, dist)| *dist);

        // Try to add each item to inventory
        for (entity, item, _) in items_sorted {
            let item_name = item.name.clone();
            let item_base_name = item.base_name.clone();
            let item_rarity = item.rarity.name();
            let added = {
                if let Ok(mut inv) = game.world_mut().get::<&mut InventoryComponent>(player) {
                    inv.inventory.add_item(item)
                } else {
                    false
                }
            };

            if added {
                game.play_sound(SoundId::ItemPickup);
                game.add_message(
                    format!("Picked up: {} [{}]", item_name, item_rarity),
                    MessageCategory::Item
                );
                let _ = game.world_mut().despawn(entity);
                game.record_item_found(&item_base_name);
            } else {
                game.play_sound(SoundId::InventoryFull);
                game.add_message(
                    format!("Inventory full! Cannot pick up {}", item_name),
                    MessageCategory::Warning
                );
                break;
            }
        }

        // Also try to open any nearby chests
        self.open_nearby_chests(game);
    }

    fn open_nearby_chests(&mut self, game: &mut Game) {
        use crate::ecs::{Chest, InventoryComponent, GroundItem, Renderable};
        use crate::entities::{mark_chest_opened, generate_chest_loot};

        let player_pos = match game.player_position() {
            Some(pos) => pos,
            None => return,
        };

        // Find all chests within range (on tile or adjacent)
        let chests_in_range: Vec<(hecs::Entity, crate::ecs::ChestRarity, Position)> = game.world()
            .query::<(&Position, &Chest)>()
            .iter()
            .filter_map(|(e, (pos, chest))| {
                let dist = player_pos.chebyshev_distance(pos);
                if dist <= 1 && !chest.opened {
                    Some((e, chest.rarity, *pos))
                } else {
                    None
                }
            })
            .collect();

        let player = match game.player() {
            Some(p) => p,
            None => return,
        };

        for (entity, rarity, chest_pos) in chests_in_range {
            // Play chest open sound
            game.play_sound(SoundId::ChestOpen);

            // Generate loot based on chest rarity
            let floor = game.floor();
            let (items, gold) = {
                let rng = game.rng();
                generate_chest_loot(rarity, floor, rng)
            };

            // Add gold
            if gold > 0 {
                game.play_sound(SoundId::GoldPickup);
                if let Ok(mut inv) = game.world_mut().get::<&mut InventoryComponent>(player) {
                    inv.inventory.add_gold(gold);
                }
                game.add_message(
                    format!("Found {} gold in {:?} chest!", gold, rarity),
                    MessageCategory::Item
                );
                game.record_gold_collected(gold);
            }

            // Spawn items on the ground near the chest
            for item in items {
                let item_name = item.name.clone();
                let item_rarity = item.rarity;
                game.world_mut().spawn((
                    chest_pos,
                    GroundItem { item: item.clone() },
                    Renderable::new(item.glyph, item_rarity.color()).with_order(80),
                ));
                game.add_message(
                    format!("Found: {} [{}]", item_name, item_rarity.name()),
                    MessageCategory::Item
                );
            }

            // Mark chest as opened
            mark_chest_opened(game.world_mut(), entity);

            game.add_message(
                format!("Opened a {:?} chest!", rarity),
                MessageCategory::System
            );
        }
    }

    /// Open a specific chest (called when walking into it)
    fn open_chest(&mut self, game: &mut Game, chest_entity: hecs::Entity, rarity: crate::ecs::ChestRarity, chest_pos: Position) {
        use crate::ecs::{InventoryComponent, GroundItem, Renderable};
        use crate::entities::{mark_chest_opened, generate_chest_loot};
        use crate::game::MessageCategory;

        // Play chest open sound
        game.play_sound(SoundId::ChestOpen);

        let player = match game.player() {
            Some(p) => p,
            None => return,
        };

        // Generate loot based on chest rarity
        let floor = game.floor();
        let (items, gold) = {
            let rng = game.rng();
            generate_chest_loot(rarity, floor, rng)
        };

        // Add gold
        if gold > 0 {
            game.play_sound(SoundId::GoldPickup);
            if let Ok(mut inv) = game.world_mut().get::<&mut InventoryComponent>(player) {
                inv.inventory.add_gold(gold);
            }
            game.add_message(
                format!("Found {} gold in {:?} chest!", gold, rarity),
                MessageCategory::Item
            );
            game.record_gold_collected(gold);
        }

        // Spawn items on the ground at the chest position
        for item in items {
            let item_name = item.name.clone();
            let item_rarity = item.rarity;
            game.world_mut().spawn((
                chest_pos,
                GroundItem { item: item.clone() },
                Renderable::new(item.glyph, item_rarity.color()).with_order(80),
            ));
            game.add_message(
                format!("Found: {} [{}]", item_name, item_rarity.name()),
                MessageCategory::Item
            );
        }

        // Mark chest as opened
        mark_chest_opened(game.world_mut(), chest_entity);

        game.add_message(
            format!("Opened a {:?} chest!", rarity),
            MessageCategory::System
        );
    }

    fn use_skill(&mut self, game: &mut Game, slot: usize) {
        use crate::ecs::{SkillsComponent, Health, Mana, Stamina, Enemy, Stats, EquipmentComponent, StatusEffects, StatusEffect, StatusEffectType};
        use crate::progression::skills::{SkillCost, TargetType, SkillEffect, ScalingStat, StatusType};

        let player = match game.player() {
            Some(p) => p,
            None => return,
        };

        // Get current mana/stamina
        let current_mana = game.world()
            .get::<&Mana>(player)
            .map(|m| m.current)
            .unwrap_or(0);
        let current_stamina = game.world()
            .get::<&Stamina>(player)
            .map(|s| s.current)
            .unwrap_or(0);

        // Check if skill can be used
        let can_use = game.world()
            .get::<&SkillsComponent>(player)
            .map(|sc| sc.skills.can_use(slot, current_mana, current_stamina))
            .unwrap_or(false);

        if !can_use {
            // Check if slot is empty
            let has_skill = game.world()
                .get::<&SkillsComponent>(player)
                .map(|sc| sc.skills.slots[slot].is_some())
                .unwrap_or(false);

            if !has_skill {
                game.add_message(format!("No skill in slot {}", slot + 1), MessageCategory::Warning);
            } else {
                game.add_message("Cannot use skill (on cooldown or not enough resources)".to_string(), MessageCategory::Warning);
            }
            return;
        }

        // Get skill info before using (to avoid borrow issues)
        let skill_info = game.world()
            .get::<&SkillsComponent>(player)
            .ok()
            .and_then(|sc| sc.skills.slots[slot].clone());

        let skill = match skill_info {
            Some(s) => s,
            None => return,
        };

        let skill_name = skill.name.clone();
        let skill_effect = skill.effect.clone();
        let skill_cost = skill.cost;
        let skill_target = skill.target;

        // Get player stats for damage scaling
        let player_stats = game.world()
            .get::<&Stats>(player)
            .map(|s| *s)
            .unwrap_or_default();

        // Deduct cost
        match skill_cost {
            SkillCost::Mana(n) => {
                if let Ok(mut mana) = game.world_mut().get::<&mut Mana>(player) {
                    mana.current = (mana.current - n).max(0);
                }
            }
            SkillCost::Stamina(n) => {
                if let Ok(mut stam) = game.world_mut().get::<&mut Stamina>(player) {
                    stam.current = (stam.current - n).max(0);
                }
            }
            _ => {}
        }

        // Mark skill as used (start cooldown, deduct charges)
        if let Ok(mut sc) = game.world_mut().get::<&mut SkillsComponent>(player) {
            sc.skills.use_skill(slot);
        }

        // Helper to convert skill StatusType to ECS StatusEffectType
        fn convert_status(status: StatusType) -> StatusEffectType {
            match status {
                StatusType::Poison => StatusEffectType::Poison,
                StatusType::Burn => StatusEffectType::Burn,
                StatusType::Bleed => StatusEffectType::Bleed,
                StatusType::Slow => StatusEffectType::Slow,
                StatusType::Stun => StatusEffectType::Slow, // Map stun to slow for now
                StatusType::Weakness => StatusEffectType::Weakness,
            }
        }

        // Get player position for targeting
        let player_pos = match game.player_position() {
            Some(pos) => pos,
            None => return,
        };

        // Collect targets based on targeting type
        let targets: Vec<hecs::Entity> = match skill_target {
            TargetType::AllAdjacent => {
                game.world()
                    .query::<(&Position, &Enemy, &Health)>()
                    .iter()
                    .filter(|(_, (pos, _, _))| pos.chebyshev_distance(&player_pos) <= 1)
                    .map(|(e, _)| e)
                    .collect()
            }
            TargetType::AllInRange(range) => {
                game.world()
                    .query::<(&Position, &Enemy, &Health)>()
                    .iter()
                    .filter(|(_, (pos, _, _))| pos.chebyshev_distance(&player_pos) <= range)
                    .map(|(e, _)| e)
                    .collect()
            }
            TargetType::SingleEnemy => {
                game.world()
                    .query::<(&Position, &Enemy, &Health)>()
                    .iter()
                    .filter(|(_, (pos, _, _))| pos.chebyshev_distance(&player_pos) <= 3)
                    .min_by_key(|(_, (pos, _, _))| pos.chebyshev_distance(&player_pos))
                    .map(|(e, _)| e)
                    .into_iter()
                    .collect()
            }
            _ => Vec::new(),
        };

        // Flatten Multi effects into a list of effects to process
        let effects_to_process: Vec<SkillEffect> = match &skill_effect {
            SkillEffect::Multi(effects) => effects.clone(),
            other => vec![other.clone()],
        };

        let mut total_damage = 0i32;
        let mut total_heal = 0i32;
        let mut hit_count = 0;
        let mut statuses_applied: Vec<String> = Vec::new();
        let mut killed: Vec<hecs::Entity> = Vec::new();
        let mut is_movement_skill = false;

        // Process each effect
        for effect in effects_to_process {
            match effect {
                SkillEffect::Heal { base, scaling_stat } => {
                    let bonus = match scaling_stat {
                        ScalingStat::Intelligence => player_stats.intelligence / 2,
                        _ => 0,
                    };
                    let heal_amount = base + bonus;
                    // Get equipment HP bonus for effective max
                    let eq_hp = game.world()
                        .get::<&EquipmentComponent>(player)
                        .map(|eq| eq.equipment.hp_bonus())
                        .unwrap_or(0);
                    if let Ok(mut hp) = game.world_mut().get::<&mut Health>(player) {
                        let effective_max = hp.max + eq_hp;
                        let actual = heal_amount.min(effective_max - hp.current);
                        hp.current += actual;
                        total_heal += actual;
                    }
                }
                SkillEffect::Damage { base, scaling_stat } => {
                    let bonus = match scaling_stat {
                        ScalingStat::Strength => player_stats.strength / 2,
                        ScalingStat::Dexterity => player_stats.dexterity / 2,
                        ScalingStat::Intelligence => player_stats.intelligence / 2,
                        ScalingStat::None => 0,
                    };
                    let damage = base + bonus;
                    total_damage += damage;

                    // Apply damage to all targets
                    for target in &targets {
                        if let Ok(mut hp) = game.world_mut().get::<&mut Health>(*target) {
                            hp.current -= damage;
                            hit_count += 1;
                            if hp.current <= 0 && !killed.contains(target) {
                                killed.push(*target);
                            }
                        }
                    }
                }
                SkillEffect::ApplyStatus { status, duration, chance } => {
                    let status_name = format!("{:?}", status);
                    let effect_type = convert_status(status);

                    // Apply status to all targets
                    for target in &targets {
                        // Check if status applies (based on chance)
                        let roll: f32 = rand::random();
                        if roll < chance {
                            // Check if entity has StatusEffects component
                            let has_component = game.world().get::<&StatusEffects>(*target).is_ok();

                            if has_component {
                                // Update existing component
                                if let Ok(mut effects) = game.world_mut().get::<&mut StatusEffects>(*target) {
                                    effects.effects.retain(|e| e.effect_type != effect_type);
                                    effects.effects.push(StatusEffect {
                                        effect_type,
                                        duration: duration as f32,
                                        intensity: 3,
                                    });
                                }
                            } else {
                                // Add StatusEffects component
                                let _ = game.world_mut().insert_one(*target, StatusEffects {
                                    effects: vec![StatusEffect {
                                        effect_type,
                                        duration: duration as f32,
                                        intensity: 3,
                                    }],
                                });
                            }
                            if !statuses_applied.contains(&status_name) {
                                statuses_applied.push(status_name.clone());
                            }
                        }
                    }
                }
                SkillEffect::BuffSelf { buff: _, duration } => {
                    game.add_message(format!("{} grants you a buff for {} turns!", skill_name, duration), MessageCategory::Combat);
                }
                SkillEffect::Movement { range } => {
                    // Set pending movement - player must choose direction
                    self.pending_movement_skill = Some(range);
                    game.add_message(format!("{} - choose direction to teleport (arrow keys)", skill_name), MessageCategory::System);
                    is_movement_skill = true;
                }
                SkillEffect::Multi(_) => {
                    // Nested Multi shouldn't happen, but ignore if it does
                }
            }
        }

        // Handle deaths
        let mut total_xp = 0u32;
        for dead in &killed {
            // Get XP reward
            let xp = game.world()
                .get::<&crate::ecs::XpReward>(*dead)
                .map(|x| x.0)
                .unwrap_or(15);
            total_xp += xp;

            // Despawn the dead enemy
            let _ = game.world_mut().despawn(*dead);
        }

        // Grant XP if any kills
        if total_xp > 0 {
            let level_up_info = if let Some(player_entity) = game.player() {
                if let Ok(mut xp) = game.world_mut().get::<&mut crate::ecs::Experience>(player_entity) {
                    let leveled = xp.add_xp(total_xp);
                    if leveled { Some(xp.level) } else { None }
                } else { None }
            } else { None };

            if let Some(new_level) = level_up_info {
                game.add_message(format!("Level up! You are now level {}!", new_level), MessageCategory::System);
            }
            game.add_message(format!("+{} XP", total_xp), MessageCategory::System);
        }

        // Build result message
        if is_movement_skill {
            // Don't run AI tick yet - wait for direction input
            return;
        }

        let mut msg_parts: Vec<String> = Vec::new();
        if total_damage > 0 && hit_count > 0 {
            msg_parts.push(format!("{} damage to {} target(s)", total_damage, hit_count));
        }
        if total_heal > 0 {
            msg_parts.push(format!("{} HP healed", total_heal));
        }
        if !statuses_applied.is_empty() {
            msg_parts.push(format!("applied {}", statuses_applied.join(", ")));
        }
        if !killed.is_empty() {
            msg_parts.push(format!("{} killed", killed.len()));
        }

        if msg_parts.is_empty() {
            if targets.is_empty() && matches!(skill_target, TargetType::SingleEnemy | TargetType::AllAdjacent | TargetType::AllInRange(_)) {
                game.add_message(format!("{} hits nothing (no enemies in range)", skill_name), MessageCategory::Combat);
            } else {
                game.add_message(format!("Used {}!", skill_name), MessageCategory::Combat);
            }
        } else {
            game.add_message(format!("{}: {}", skill_name, msg_parts.join(", ")), MessageCategory::Combat);
        }

        // Tick cooldowns for self-targeted/non-attack skills
        // (Attack skills tick in attack_enemy instead)
        let is_self_target = matches!(skill_target, TargetType::Self_);
        if is_self_target {
            if let Ok(mut sc) = game.world_mut().get::<&mut SkillsComponent>(player) {
                sc.skills.tick_cooldowns();
            }
        }

        // Enemies take their turn after skill use
        game.run_ai_tick();
    }

    fn interact_with_tile(&mut self, game: &mut Game) {
        use crate::world::TileType;

        let player_pos = match game.player_position() {
            Some(pos) => pos,
            None => return,
        };

        let tile_type = match game.map() {
            Some(map) => map.get_tile(player_pos.x, player_pos.y).map(|t| t.tile_type),
            None => return,
        };

        match tile_type {
            Some(TileType::ShrineSkill) => {
                if game.is_shrine_used(player_pos) {
                    game.add_message("This shrine's power has already been used.".to_string(), MessageCategory::Warning);
                } else {
                    game.play_sound(SoundId::ShrineApproach);
                    game.add_message("You approach the Skill Shrine. It pulses with arcane energy.".to_string(), MessageCategory::System);
                    // Generate random skills based on floor
                    let floor = game.floor();
                    self.shrine_skills = crate::progression::generate_shrine_skills(floor, 3, game.rng());
                    self.shrine_skill_cursor = 0;
                    game.set_state(GameState::Playing(PlayingState::Shrine { shrine_type: ShrineType::Skill }));
                }
            }
            Some(TileType::ShrineEnchant) => {
                if game.is_shrine_used(player_pos) {
                    game.add_message("This shrine's power has already been used.".to_string(), MessageCategory::Warning);
                } else {
                    game.play_sound(SoundId::ShrineApproach);
                    game.add_message("You approach the Enchanting Shrine. Select equipment to enchant.".to_string(), MessageCategory::System);
                    // Very rare chance (5%) for +1 max enchantment slot option
                    self.enchant_upgrade_available = game.rng().gen_bool(0.05);
                    self.enchant_affix_cursor = 0;
                    self.enchant_swap_mode = false;
                    self.enchant_swap_cursor = 0;
                    self.enchant_selected_slot = None;  // Start in equipment selection mode
                    self.enchant_equipment_cursor = 0;
                    game.set_state(GameState::Playing(PlayingState::Shrine { shrine_type: ShrineType::Enchanting }));
                }
            }
            Some(TileType::ShrineRest) => {
                if game.is_shrine_used(player_pos) {
                    game.add_message("This shrine's power has already been used.".to_string(), MessageCategory::Warning);
                } else {
                    // Rest shrine - heal fully and restore charges
                    game.play_sound(SoundId::ShrineUse);
                    game.add_message("You rest at the shrine. Your wounds heal and your abilities are restored.".to_string(), MessageCategory::System);
                    if let Some(player) = game.player() {
                        // Get equipment bonuses for effective max
                        let (eq_hp, eq_mp) = game.world()
                            .get::<&crate::ecs::EquipmentComponent>(player)
                            .map(|eq| (eq.equipment.hp_bonus(), eq.equipment.mp_bonus()))
                            .unwrap_or((0, 0));
                        // Heal to full (including equipment bonus)
                        if let Ok(mut hp) = game.world_mut().get::<&mut crate::ecs::Health>(player) {
                            hp.current = hp.max + eq_hp;
                        }
                        // Restore mana (including equipment bonus)
                        if let Ok(mut mp) = game.world_mut().get::<&mut crate::ecs::Mana>(player) {
                            mp.current = mp.max + eq_mp;
                        }
                        // Restore stamina
                        if let Ok(mut sp) = game.world_mut().get::<&mut crate::ecs::Stamina>(player) {
                            sp.current = sp.max;
                        }
                        // Restore skill charges
                        if let Ok(mut skills) = game.world_mut().get::<&mut crate::ecs::SkillsComponent>(player) {
                            skills.skills.restore_charges();
                        }
                    }
                    // Mark shrine as used
                    game.mark_shrine_used(player_pos);
                }
            }
            Some(TileType::ShrineCorruption) => {
                // Corruption shrine can be used multiple times (risk/reward)
                game.play_sound(SoundId::ShrineApproach);
                game.add_message("You approach the Corruption Shrine. Dark power calls to you...".to_string(), MessageCategory::Combat);
                game.set_state(GameState::Playing(PlayingState::Shrine { shrine_type: ShrineType::Corruption }));
            }
            Some(TileType::StairsDown) => {
                game.play_sound(SoundId::Descend);
                game.add_message("You descend deeper into the darkness...".to_string(), MessageCategory::System);
                game.descend();
                game.play_sound(SoundId::NewFloor);
                if let Some(new_map) = game.map() {
                    self.camera = new_map.start_pos;
                }
            }
            _ => {
                game.add_message("Nothing to interact with here.".to_string(), MessageCategory::System);
            }
        }
    }

    fn try_move(&mut self, game: &mut Game, dx: i32, dy: i32) {
        use crate::entities::{NpcMarker, NpcComponent, NpcType};
        use crate::ecs::Chest;

        let new_x = self.camera.x + dx;
        let new_y = self.camera.y + dy;

        // Check walkability first (immutable borrow)
        let can_walk = game.map().map(|m| m.is_walkable(new_x, new_y)).unwrap_or(false);

        if !can_walk {
            return;
        }

        let new_pos = Position::new(new_x, new_y);

        // Check for NPC interaction (before enemy check)
        let npc_at_pos = {
            let mut found = None;
            for (entity, (pos, _, npc)) in game.world().query::<(&Position, &NpcMarker, &NpcComponent)>().iter() {
                if pos.x == new_pos.x && pos.y == new_pos.y {
                    found = Some((entity, npc.npc_type));
                    break;
                }
            }
            found
        };

        if let Some((npc_entity, npc_type)) = npc_at_pos {
            // Interact with NPC
            match npc_type {
                NpcType::Merchant => {
                    // Open shop
                    game.add_message(
                        format!("{}: \"{}\"", npc_type.name(), npc_type.greeting()),
                        crate::game::MessageCategory::System,
                    );
                    game.set_state(GameState::Playing(PlayingState::Shop { npc_entity }));
                }
                NpcType::Healer => {
                    // Heal the player
                    game.heal_player(50);
                    game.add_message(
                        format!("{}: \"{}\" (Healed 50 HP)", npc_type.name(), npc_type.greeting()),
                        crate::game::MessageCategory::System,
                    );
                }
                _ => {
                    // Generic greeting
                    game.add_message(
                        format!("{}: \"{}\"", npc_type.name(), npc_type.greeting()),
                        crate::game::MessageCategory::Lore,
                    );
                }
            }
            return;
        }

        // Check for chest interaction (walk into chest to open it)
        let chest_at_pos = {
            let mut found = None;
            for (entity, (pos, chest)) in game.world().query::<(&Position, &Chest)>().iter() {
                if pos.x == new_pos.x && pos.y == new_pos.y && !chest.opened {
                    found = Some((entity, chest.rarity));
                    break;
                }
            }
            found
        };

        if let Some((chest_entity, rarity)) = chest_at_pos {
            // Open the chest
            self.open_chest(game, chest_entity, rarity, new_pos);
            // Move onto the chest tile after opening
            self.camera = new_pos;
            game.set_player_position(new_pos);
            if let Some(map) = game.map_mut() {
                crate::world::compute_fov(map, self.camera, 8);
            }
            game.run_ai_tick();
            return;
        }

        // Check for blocking entity (enemy collision = attack!)
        if let Some(target_entity) = game.get_blocking_entity_at(new_pos) {
            self.attack_enemy(game, target_entity);
            // Run enemy AI after player action (even attacks count as actions)
            game.run_ai_tick();
            return;
        }

        // Move player
        self.camera = new_pos;
        game.set_player_position(new_pos);

        // Update FOV (separate mutable borrow)
        if let Some(map) = game.map_mut() {
            crate::world::compute_fov(map, self.camera, 8);
        }

        // Run enemy AI after player action
        game.run_ai_tick();
    }

    /// Execute a movement skill (teleport) in the given direction
    fn execute_movement_skill(&mut self, game: &mut Game, dx: i32, dy: i32, range: i32) {
        let player_pos = match game.player_position() {
            Some(pos) => pos,
            None => return,
        };

        // Find the farthest valid tile in the direction, up to range
        let mut final_x = player_pos.x;
        let mut final_y = player_pos.y;

        for step in 1..=range {
            let test_x = player_pos.x + (dx * step);
            let test_y = player_pos.y + (dy * step);

            // Check if the tile is walkable
            let walkable = game.map()
                .map(|m| m.is_walkable(test_x, test_y))
                .unwrap_or(false);

            if walkable {
                // Check for blocking entity (don't teleport into enemies)
                let blocked_by_entity = game.get_blocking_entity_at(Position::new(test_x, test_y)).is_some();
                if !blocked_by_entity {
                    final_x = test_x;
                    final_y = test_y;
                }
            } else {
                // Hit a wall, stop here
                break;
            }
        }

        let final_pos = Position::new(final_x, final_y);

        // Check if we actually moved
        if final_pos.x == player_pos.x && final_pos.y == player_pos.y {
            game.add_message("Cannot teleport - path is blocked!".to_string(), MessageCategory::Warning);
            return;
        }

        let distance = ((final_pos.x - player_pos.x).abs() + (final_pos.y - player_pos.y).abs()) as f32;
        let diagonal_dist = if dx != 0 && dy != 0 {
            ((final_pos.x - player_pos.x).abs().max((final_pos.y - player_pos.y).abs())) as f32
        } else {
            distance
        };

        // Teleport the player
        self.camera = final_pos;
        game.set_player_position(final_pos);

        // Update FOV
        if let Some(map) = game.map_mut() {
            crate::world::compute_fov(map, self.camera, 8);
        }

        game.add_message(format!("Shadow Step! Teleported {} tiles.", diagonal_dist as i32), MessageCategory::Combat);

        // Run enemy AI after teleport
        game.run_ai_tick();
    }

    fn attack_enemy(&mut self, game: &mut Game, target: hecs::Entity) {
        use crate::ecs::{Name, Health, Stats, GroundItem, EquipmentComponent};
        use crate::game::MessageCategory;
        use crate::combat::{calculate_attack_with_equipment, EquipmentBonuses};
        use crate::items::{generate_enemy_loot, generate_gold_drop, generate_boss_loot, generate_boss_gold_drop};

        // Get player and target stats
        let player_stats = game.player_stats().unwrap_or(Stats::player_base());
        let target_stats = game.world()
            .get::<&Stats>(target)
            .map(|s| *s)
            .unwrap_or(Stats::new(5, 5, 5, 5));

        // Get player equipment bonuses
        let player_equipment = if let Some(player) = game.player() {
            game.world()
                .get::<&EquipmentComponent>(player)
                .map(|eq| EquipmentBonuses {
                    weapon_damage: eq.equipment.weapon_damage(),
                    armor: eq.equipment.total_armor(),
                    str_bonus: eq.equipment.strength_bonus(),
                    dex_bonus: eq.equipment.dexterity_bonus(),
                    crit_bonus: eq.equipment.weapon_crit_bonus(),
                })
                .unwrap_or_default()
        } else {
            EquipmentBonuses::default()
        };

        // Get target info and position (need position before despawn for loot)
        let target_name = game.world()
            .get::<&Name>(target)
            .map(|n| n.0.clone())
            .unwrap_or_else(|_| "something".to_string());

        let target_pos = game.world()
            .get::<&Position>(target)
            .map(|p| *p)
            .unwrap_or(self.camera);

        // Calculate attack with crits, dodges, equipment bonuses
        let result = calculate_attack_with_equipment(
            &player_stats,
            &target_stats,
            &player_equipment,
            &EquipmentBonuses::default(), // Enemies don't have equipment (yet)
            game.rng(),
        );

        // Handle dodge/miss
        if result.is_dodge {
            game.play_sound(SoundId::Dodge);
            game.add_message(
                format!("The {} dodges your attack!", target_name),
                MessageCategory::Combat
            );
            return;
        }
        if result.is_miss {
            game.play_sound(SoundId::Miss);
            game.add_message(
                format!("You miss the {}!", target_name),
                MessageCategory::Combat
            );
            return;
        }

        // Apply damage
        let (target_died, current_health) = {
            if let Ok(mut health) = game.world_mut().get::<&mut Health>(target) {
                health.take_damage(result.final_damage);
                (health.is_dead(), Some(*health))
            } else {
                (false, None)
            }
        };

        // Check for boss phase transition (separate borrow)
        let phase_changed = if let Some(health) = current_health {
            if let Ok(mut boss) = game.world_mut().get::<&mut crate::entities::BossComponent>(target) {
                crate::entities::update_boss_phase(&health, &mut boss)
                    .map(|new_phase| (boss.boss_type, new_phase))
            } else {
                None
            }
        } else {
            None
        };

        // Handle boss phase transition message
        if let Some((boss_type, new_phase)) = phase_changed {
            game.add_message(
                format!("⚠ {} enters phase {}!", boss_type.name(), new_phase),
                MessageCategory::Warning
            );
            game.add_message(
                boss_type.phase_description(new_phase).to_string(),
                MessageCategory::Lore
            );
        }

        if target_died {
            // Play critical or normal hit sound, followed by death
            if result.is_crit {
                game.play_sound(SoundId::Critical);
            } else {
                game.play_sound(SoundId::Hit);
            }
            game.play_sound(SoundId::EnemyDeath);

            let msg = if result.is_crit {
                format!("CRITICAL HIT! You destroy the {} for {} damage!", target_name, result.final_damage)
            } else {
                format!("You strike the {} for {} damage! It dies!", target_name, result.final_damage)
            };
            game.add_message(msg, MessageCategory::Combat);

            // Check if this was a boss
            let is_boss = game.world()
                .get::<&crate::entities::BossComponent>(target)
                .is_ok();

            // Generate and drop loot (bosses get better loot)
            let floor = game.floor();
            let loot = if is_boss {
                game.add_message(
                    "★ The boss drops powerful loot! ★".to_string(),
                    MessageCategory::Item
                );
                generate_boss_loot(floor, game.rng())
            } else {
                generate_enemy_loot(floor, game.rng())
            };

            for item in loot {
                // Include rarity in the drop message
                let rarity_name = item.rarity.name();
                game.add_message(
                    format!("The {} dropped: {} [{}]", target_name, item.name, rarity_name),
                    MessageCategory::Item
                );
                // Spawn item entity on ground
                game.world_mut().spawn((
                    target_pos,
                    crate::ecs::Renderable::new(item.glyph, item.rarity.color()).with_order(10),
                    GroundItem { item },
                ));
            }

            // Drop gold (bosses drop more)
            let gold = if is_boss {
                generate_boss_gold_drop(floor, game.rng())
            } else {
                generate_gold_drop(floor, game.rng())
            };
            if gold > 0 {
                // Add gold directly to player inventory
                let added_gold = if let Some(player) = game.player() {
                    if let Ok(mut inv) = game.world_mut().get::<&mut crate::ecs::InventoryComponent>(player) {
                        inv.inventory.add_gold(gold);
                        true
                    } else { false }
                } else { false };

                if added_gold {
                    game.add_message(format!("You found {} gold!", gold), MessageCategory::Item);
                    game.record_gold_collected(gold);
                }
            }

            // Get XP reward before despawning
            let xp_reward = game.world()
                .get::<&crate::ecs::XpReward>(target)
                .map(|xp| xp.0)
                .unwrap_or(15); // Default 15 XP if no XpReward component

            // Remove the dead entity
            let _ = game.world_mut().despawn(target);

            // Record enemy kill in profile stats
            game.record_enemy_kill(is_boss);

            // Grant XP
            game.add_message(format!("+{} XP", xp_reward), MessageCategory::System);

            let leveled_up = if let Some(player) = game.player() {
                if let Ok(mut xp) = game.world_mut().get::<&mut crate::ecs::Experience>(player) {
                    let did_level = xp.add_xp(xp_reward);
                    if did_level { Some(xp.level) } else { None }
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(new_level) = leveled_up {
                game.play_sound(SoundId::LevelUp);
                // Grant stat point on level up
                if let Some(player) = game.player() {
                    if let Ok(mut sp) = game.world_mut().get::<&mut crate::ecs::StatPoints>(player) {
                        sp.0 += 1;
                    }
                }
                game.add_message(
                    format!("LEVEL UP! You are now level {}! (+1 stat point)", new_level),
                    MessageCategory::System
                );
            }
        } else {
            // Target didn't die - play hit/crit sound
            if result.is_crit {
                game.play_sound(SoundId::Critical);
            } else {
                game.play_sound(SoundId::Hit);
            }
            let msg = if result.is_crit {
                format!("CRITICAL HIT! You strike the {} for {} damage!", target_name, result.final_damage)
            } else {
                format!("You strike the {} for {} damage.", target_name, result.final_damage)
            };
            game.add_message(msg, MessageCategory::Combat);
        }

        // Apply lifesteal (vampiric) if player has it and did damage
        // Each point of LifeSteal = 5% of damage converted to health
        if result.final_damage > 0 {
            let lifesteal_flat = if let Some(player) = game.player() {
                game.world()
                    .get::<&EquipmentComponent>(player)
                    .map(|eq| eq.equipment.stat_bonus(crate::items::item::AffixType::LifeSteal))
                    .unwrap_or(0)
            } else {
                0
            };

            if lifesteal_flat > 0 {
                // Each point of LifeSteal = 5% of damage, so +3 LifeSteal = 15% of damage healed
                let lifesteal_percent = lifesteal_flat * 5;
                let heal_amount = (result.final_damage * lifesteal_percent / 100).max(1);
                let actual_heal = if let Some(player) = game.player() {
                    // Get equipment HP bonus for effective max
                    let eq_hp = game.world()
                        .get::<&EquipmentComponent>(player)
                        .map(|eq| eq.equipment.hp_bonus())
                        .unwrap_or(0);
                    if let Ok(mut hp) = game.world_mut().get::<&mut Health>(player) {
                        let effective_max = hp.max + eq_hp;
                        let actual = heal_amount.min(effective_max - hp.current);
                        hp.current += actual;
                        actual
                    } else {
                        0
                    }
                } else {
                    0
                };
                if actual_heal > 0 {
                    game.add_message(
                        format!("💉 Vampiric ({}%): +{} HP", lifesteal_percent, actual_heal),
                        MessageCategory::System
                    );
                }
            }
        }

        // Tick cooldowns after player action
        if let Some(player) = game.player() {
            if let Ok(mut skills) = game.world_mut().get::<&mut crate::ecs::SkillsComponent>(player) {
                skills.skills.tick_cooldowns();
            }
        }
    }

    fn handle_inventory_input(&mut self, key: KeyEvent, game: &mut Game) -> Result<bool> {
        use crate::ecs::{InventoryComponent, EquipmentComponent, Health, Mana};
        use crate::items::ConsumableEffect;

        let player = match game.player() {
            Some(p) => p,
            None => return Ok(false),
        };

        // Get inventory length for bounds checking
        let inv_len = game.world()
            .get::<&InventoryComponent>(player)
            .map(|inv| inv.inventory.count())
            .unwrap_or(0);

        // Get equipment count for tab 1
        let equipment_count = game.world()
            .get::<&InventoryComponent>(player)
            .map(|inv| inv.inventory.items().into_iter().filter(|i| i.category.is_equipment()).count())
            .unwrap_or(0);

        // Get max cursor based on current tab (tab 0 = all items, tab 1 = equipment only)
        let max_cursor = if self.inventory_tab == 0 {
            inv_len
        } else {
            equipment_count
        };

        match key.code {
            KeyCode::Esc | KeyCode::Char('i') => {
                // Mark all items as seen when closing inventory
                if let Ok(mut inv) = game.world_mut().get::<&mut InventoryComponent>(player) {
                    inv.inventory.mark_all_seen();
                }
                game.set_state(GameState::Playing(PlayingState::Exploring));
            }
            // Navigation
            KeyCode::Up | KeyCode::Char('k') => {
                if self.inventory_cursor > 0 {
                    self.inventory_cursor -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.inventory_cursor + 1 < max_cursor {
                    self.inventory_cursor += 1;
                }
            }
            // Switch tabs
            KeyCode::Tab => {
                self.inventory_tab = (self.inventory_tab + 1) % 2;
                self.inventory_cursor = 0;
            }
            // Use consumable
            KeyCode::Char('u') | KeyCode::Enter => {
                if self.inventory_tab == 0 && inv_len > 0 {
                    // Get item info before consuming
                    let item_info = game.world()
                        .get::<&InventoryComponent>(player)
                        .ok()
                        .and_then(|inv| inv.inventory.get(self.inventory_cursor).cloned());

                    if let Some(item) = item_info {
                        if item.is_consumable() {
                            // Apply effect
                            let effect_msg = match item.consumable_effect {
                                Some(ConsumableEffect::HealHP(amount)) => {
                                    // Get equipment HP bonus for effective max
                                    let eq_hp = game.world()
                                        .get::<&EquipmentComponent>(player)
                                        .map(|eq| eq.equipment.hp_bonus())
                                        .unwrap_or(0);
                                    if let Ok(mut hp) = game.world_mut().get::<&mut Health>(player) {
                                        let effective_max = hp.max + eq_hp;
                                        let actual_heal = amount.min(effective_max - hp.current);
                                        hp.current += actual_heal;
                                        Some(format!("Healed {} HP!", actual_heal))
                                    } else { None }
                                }
                                Some(ConsumableEffect::RestoreMP(amount)) => {
                                    // Get equipment MP bonus
                                    let eq_mp = game.world()
                                        .get::<&EquipmentComponent>(player)
                                        .map(|eq| eq.equipment.mp_bonus())
                                        .unwrap_or(0);

                                    if let Ok(mut mp) = game.world_mut().get::<&mut Mana>(player) {
                                        let effective_max = mp.max + eq_mp;
                                        let actual_restore = amount.min(effective_max - mp.current);
                                        mp.current += actual_restore;
                                        Some(format!("Restored {} MP!", actual_restore))
                                    } else { None }
                                }
                                _ => None,
                            };

                            // Consume the item
                            if let Ok(mut inv) = game.world_mut().get::<&mut InventoryComponent>(player) {
                                inv.inventory.consume_at(self.inventory_cursor);
                            }

                            if let Some(msg) = effect_msg {
                                game.add_message(msg, MessageCategory::Item);
                            }

                            // Using a consumable takes a turn - enemies act
                            game.run_ai_tick();

                            // Adjust cursor if needed
                            let new_len = game.world()
                                .get::<&InventoryComponent>(player)
                                .map(|inv| inv.inventory.count())
                                .unwrap_or(0);
                            if self.inventory_cursor >= new_len && new_len > 0 {
                                self.inventory_cursor = new_len - 1;
                            }
                        } else if item.is_equippable() {
                            // Equip the item
                            let item_name = item.name.clone();

                            // Remove from inventory and equip
                            let removed = {
                                if let Ok(mut inv) = game.world_mut().get::<&mut InventoryComponent>(player) {
                                    inv.inventory.remove_at(self.inventory_cursor)
                                } else { None }
                            };

                            if let Some(to_equip) = removed {
                                let old_item = {
                                    if let Ok(mut eq) = game.world_mut().get::<&mut EquipmentComponent>(player) {
                                        eq.equipment.equip(to_equip)
                                    } else { None }
                                };

                                // Put old item back in inventory
                                if let Some(old) = old_item {
                                    let old_name = old.name.clone();
                                    if let Ok(mut inv) = game.world_mut().get::<&mut InventoryComponent>(player) {
                                        inv.inventory.add_item(old);
                                    }
                                    game.add_message(
                                        format!("Unequipped {} and equipped {}", old_name, item_name),
                                        MessageCategory::Item
                                    );
                                } else {
                                    game.add_message(
                                        format!("Equipped {}", item_name),
                                        MessageCategory::Item
                                    );
                                }
                            }

                            // Adjust cursor
                            let new_len = game.world()
                                .get::<&InventoryComponent>(player)
                                .map(|inv| inv.inventory.count())
                                .unwrap_or(0);
                            if self.inventory_cursor >= new_len && new_len > 0 {
                                self.inventory_cursor = new_len - 1;
                            }
                        }
                    }
                } else if self.inventory_tab == 1 && equipment_count > 0 {
                    // Equip from equipment tab (filtered view of inventory equipment)
                    // Get the equipment items
                    let equipment_items: Vec<crate::items::ItemId> = game.world()
                        .get::<&InventoryComponent>(player)
                        .map(|inv| inv.inventory.items()
                            .into_iter()
                            .filter(|i| i.category.is_equipment())
                            .map(|i| i.id)
                            .collect())
                        .unwrap_or_default();

                    if self.inventory_cursor < equipment_items.len() {
                        let item_id = equipment_items[self.inventory_cursor];

                        // Get item info and remove from inventory
                        let removed = {
                            if let Ok(mut inv) = game.world_mut().get::<&mut InventoryComponent>(player) {
                                inv.inventory.remove_by_id(item_id)
                            } else { None }
                        };

                        if let Some(to_equip) = removed {
                            let item_name = to_equip.name.clone();
                            let old_item = {
                                if let Ok(mut eq) = game.world_mut().get::<&mut EquipmentComponent>(player) {
                                    eq.equipment.equip(to_equip)
                                } else { None }
                            };

                            // Put old item back in inventory
                            if let Some(old) = old_item {
                                let old_name = old.name.clone();
                                if let Ok(mut inv) = game.world_mut().get::<&mut InventoryComponent>(player) {
                                    inv.inventory.add_item(old);
                                }
                                game.add_message(
                                    format!("Swapped {} for {}", old_name, item_name),
                                    MessageCategory::Item
                                );
                            } else {
                                game.add_message(
                                    format!("Equipped {}", item_name),
                                    MessageCategory::Item
                                );
                            }

                            // Adjust cursor
                            let new_count = game.world()
                                .get::<&InventoryComponent>(player)
                                .map(|inv| inv.inventory.items().into_iter().filter(|i| i.category.is_equipment()).count())
                                .unwrap_or(0);
                            if self.inventory_cursor >= new_count && new_count > 0 {
                                self.inventory_cursor = new_count - 1;
                            }
                        }
                    }
                }
            }
            // Destroy item (permanently delete)
            KeyCode::Char('d') => {
                if self.inventory_tab == 0 && inv_len > 0 {
                    // Destroy from all items tab
                    let removed = {
                        if let Ok(mut inv) = game.world_mut().get::<&mut InventoryComponent>(player) {
                            inv.inventory.remove_at(self.inventory_cursor)
                        } else { None }
                    };

                    if let Some(item) = removed {
                        let item_name = item.name.clone();
                        // Item is simply dropped (destroyed) - not spawned on ground
                        game.add_message(format!("Destroyed {}", item_name), MessageCategory::Item);

                        let new_len = game.world()
                            .get::<&InventoryComponent>(player)
                            .map(|inv| inv.inventory.count())
                            .unwrap_or(0);
                        if self.inventory_cursor >= new_len && new_len > 0 {
                            self.inventory_cursor = new_len - 1;
                        }
                    }
                } else if self.inventory_tab == 1 && equipment_count > 0 {
                    // Destroy from equipment tab (filtered view)
                    let equipment_items: Vec<crate::items::ItemId> = game.world()
                        .get::<&InventoryComponent>(player)
                        .map(|inv| inv.inventory.items()
                            .into_iter()
                            .filter(|i| i.category.is_equipment())
                            .map(|i| i.id)
                            .collect())
                        .unwrap_or_default();

                    if self.inventory_cursor < equipment_items.len() {
                        let item_id = equipment_items[self.inventory_cursor];
                        let removed = {
                            if let Ok(mut inv) = game.world_mut().get::<&mut InventoryComponent>(player) {
                                inv.inventory.remove_by_id(item_id)
                            } else { None }
                        };

                        if let Some(item) = removed {
                            let item_name = item.name.clone();
                            // Item is simply dropped (destroyed) - not spawned on ground
                            game.add_message(format!("Destroyed {}", item_name), MessageCategory::Item);

                            let new_count = game.world()
                                .get::<&InventoryComponent>(player)
                                .map(|inv| inv.inventory.items().into_iter().filter(|i| i.category.is_equipment()).count())
                                .unwrap_or(0);
                            if self.inventory_cursor >= new_count && new_count > 0 {
                                self.inventory_cursor = new_count - 1;
                            }
                        }
                    }
                }
            }
            // Sort inventory
            KeyCode::Char('s') => {
                if self.inventory_tab == 0 {
                    use crate::items::SortMode;
                    // Cycle through sort modes
                    self.inventory_sort_mode = match self.inventory_sort_mode {
                        SortMode::Size => SortMode::Rarity,
                        SortMode::Rarity => SortMode::Category,
                        SortMode::Category => SortMode::Name,
                        SortMode::Name => SortMode::New,
                        SortMode::New => SortMode::Size,
                    };

                    // Apply the sort
                    if let Ok(mut inv) = game.world_mut().get::<&mut InventoryComponent>(player) {
                        inv.inventory.sort_by(self.inventory_sort_mode);
                    }

                    let mode_name = match self.inventory_sort_mode {
                        SortMode::Size => "Size",
                        SortMode::Rarity => "Rarity",
                        SortMode::Category => "Category",
                        SortMode::Name => "Name",
                        SortMode::New => "New Items First",
                    };
                    game.add_message(format!("Sorted by: {}", mode_name), MessageCategory::System);
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_character_input(&mut self, key: KeyEvent, game: &mut Game) -> Result<bool> {
        use crate::ecs::{EquipmentComponent, InventoryComponent, StatPoints, Stats, Health, Mana, SkillsComponent};
        use crate::items::EquipSlot;

        // 9 equipment slots + 5 skill slots = 14 total slots
        const NUM_EQUIP_SLOTS: usize = 9;
        const NUM_SKILL_SLOTS: usize = 5;
        const NUM_SLOTS: usize = NUM_EQUIP_SLOTS + NUM_SKILL_SLOTS;
        let slots = [
            EquipSlot::Head,
            EquipSlot::MainHand,
            EquipSlot::Body,
            EquipSlot::OffHand,
            EquipSlot::Hands,
            EquipSlot::Feet,
            EquipSlot::Amulet,
            EquipSlot::Ring1,
            EquipSlot::Ring2,
        ];

        // Handle skill selection mode (selecting from learned skills to equip)
        if self.skill_selection_mode {
            let player = match game.player() {
                Some(p) => p,
                None => return Ok(false),
            };

            // Get learned skills that are not equipped
            let unequipped_skills: Vec<(usize, String, char)> = game.world()
                .get::<&SkillsComponent>(player)
                .map(|sk| {
                    sk.skills.learned.iter().enumerate()
                        .filter(|(_, skill)| {
                            !sk.skills.slots.iter().any(|s| s.as_ref().map(|eq| eq.id == skill.id).unwrap_or(false))
                        })
                        .map(|(i, skill)| (i, skill.name.clone(), skill.icon))
                        .collect()
                })
                .unwrap_or_default();

            match key.code {
                KeyCode::Esc | KeyCode::Left => {
                    self.skill_selection_mode = false;
                    self.skill_selection_cursor = 0;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.skill_selection_cursor > 0 {
                        self.skill_selection_cursor -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.skill_selection_cursor + 1 < unequipped_skills.len() {
                        self.skill_selection_cursor += 1;
                    }
                }
                KeyCode::Enter | KeyCode::Right => {
                    if self.skill_selection_cursor < unequipped_skills.len() {
                        let (learned_idx, skill_name, _) = unequipped_skills[self.skill_selection_cursor].clone();

                        // Equip the skill to the selected slot
                        let equipped = if let Ok(mut sk) = game.world_mut().get::<&mut SkillsComponent>(player) {
                            sk.skills.equip_from_learned(self.skill_slot_to_swap, learned_idx)
                        } else {
                            false
                        };

                        if equipped {
                            game.add_message(format!("Equipped {}", skill_name), MessageCategory::System);
                        }

                        self.skill_selection_mode = false;
                        self.skill_selection_cursor = 0;
                    }
                }
                _ => {}
            }
            return Ok(false);
        }

        // Handle equip selection mode (selecting item from inventory)
        if self.equip_selection_mode {
            let current_slot = slots[self.character_slot];

            // Get filtered inventory items that match this slot
            let player = match game.player() {
                Some(p) => p,
                None => return Ok(false),
            };

            let matching_items: Vec<(usize, String)> = game.world()
                .get::<&InventoryComponent>(player)
                .map(|inv| {
                    inv.inventory.items().iter().enumerate()
                        .filter(|(_, item)| item.equip_slot == Some(current_slot))
                        .map(|(i, item)| (i, item.name.clone()))
                        .collect()
                })
                .unwrap_or_default();

            match key.code {
                KeyCode::Esc | KeyCode::Left => {
                    // Exit equip selection mode
                    self.equip_selection_mode = false;
                    self.equip_selection_cursor = 0;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.equip_selection_cursor > 0 {
                        self.equip_selection_cursor -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.equip_selection_cursor + 1 < matching_items.len() {
                        self.equip_selection_cursor += 1;
                    }
                }
                KeyCode::Enter | KeyCode::Right => {
                    // Equip the selected item
                    if self.equip_selection_cursor < matching_items.len() {
                        let (inv_index, item_name) = matching_items[self.equip_selection_cursor].clone();

                        // Remove from inventory
                        let item = {
                            if let Ok(mut inv) = game.world_mut().get::<&mut InventoryComponent>(player) {
                                inv.inventory.remove_at(inv_index)
                            } else {
                                None
                            }
                        };

                        if let Some(item) = item {
                            // Unequip current item if any
                            let old_item = {
                                if let Ok(mut eq) = game.world_mut().get::<&mut EquipmentComponent>(player) {
                                    eq.equipment.unequip(current_slot)
                                } else {
                                    None
                                }
                            };

                            // Put old item back in inventory
                            if let Some(old) = old_item {
                                if let Ok(mut inv) = game.world_mut().get::<&mut InventoryComponent>(player) {
                                    inv.inventory.add_item(old);
                                }
                            }

                            // Equip new item
                            if let Ok(mut eq) = game.world_mut().get::<&mut EquipmentComponent>(player) {
                                eq.equipment.equip(item);
                            }

                            game.add_message(format!("Equipped {}", item_name), MessageCategory::Item);
                        }

                        self.equip_selection_mode = false;
                        self.equip_selection_cursor = 0;
                    }
                }
                _ => {}
            }
            return Ok(false);
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('c') => {
                game.set_state(GameState::Playing(PlayingState::Exploring));
            }
            // Stat point allocation (1=STR, 2=DEX, 3=INT, 4=VIT)
            KeyCode::Char('1') | KeyCode::Char('2') | KeyCode::Char('3') | KeyCode::Char('4') => {
                let player = match game.player() {
                    Some(p) => p,
                    None => return Ok(false),
                };

                // Check if player has stat points
                let has_points = game.world()
                    .get::<&StatPoints>(player)
                    .map(|sp| sp.0 > 0)
                    .unwrap_or(false);

                if has_points {
                    // Spend the stat point
                    let spent = {
                        if let Ok(mut sp) = game.world_mut().get::<&mut StatPoints>(player) {
                            if sp.0 > 0 {
                                sp.0 -= 1;
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    };

                    if spent {
                        // Increase the appropriate stat
                        let (stat_name, extra_msg) = {
                            let stat_name = if let Ok(mut stats) = game.world_mut().get::<&mut Stats>(player) {
                                match key.code {
                                    KeyCode::Char('1') => {
                                        stats.strength += 1;
                                        "Strength"
                                    }
                                    KeyCode::Char('2') => {
                                        stats.dexterity += 1;
                                        "Dexterity"
                                    }
                                    KeyCode::Char('3') => {
                                        stats.intelligence += 1;
                                        "Intelligence"
                                    }
                                    KeyCode::Char('4') => {
                                        stats.vitality += 1;
                                        "Vitality"
                                    }
                                    _ => "Unknown",
                                }
                            } else {
                                "Unknown"
                            };

                            // Handle derived stat increases
                            let extra = match key.code {
                                KeyCode::Char('4') => {
                                    // Vitality: +5 max HP per point
                                    if let Ok(mut hp) = game.world_mut().get::<&mut Health>(player) {
                                        hp.max += 5;
                                        hp.current += 5; // Also heal for the bonus
                                        Some("+5 Max HP")
                                    } else {
                                        None
                                    }
                                }
                                KeyCode::Char('3') => {
                                    // Intelligence: +3 max MP per point
                                    if let Ok(mut mp) = game.world_mut().get::<&mut Mana>(player) {
                                        mp.max += 3;
                                        mp.current += 3; // Also restore the bonus
                                        Some("+3 Max MP")
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            };
                            (stat_name, extra)
                        };

                        let msg = if let Some(extra) = extra_msg {
                            format!("{} increased! ({})", stat_name, extra)
                        } else {
                            format!("{} increased!", stat_name)
                        };
                        game.add_message(msg, MessageCategory::System);
                    }
                }
            }
            // Navigation
            KeyCode::Up | KeyCode::Char('k') => {
                if self.character_slot > 0 {
                    self.character_slot -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.character_slot < NUM_SLOTS - 1 {
                    self.character_slot += 1;
                }
            }
            // Enter selection mode (equip item or skill)
            KeyCode::Right | KeyCode::Char('l') => {
                let player = match game.player() {
                    Some(p) => p,
                    None => return Ok(false),
                };

                if self.character_slot < NUM_EQUIP_SLOTS {
                    // Equipment slot - enter equip selection mode
                    let current_slot = slots[self.character_slot];
                    let has_matching = game.world()
                        .get::<&InventoryComponent>(player)
                        .map(|inv| {
                            inv.inventory.items().iter().any(|item| item.equip_slot == Some(current_slot))
                        })
                        .unwrap_or(false);

                    if has_matching {
                        self.equip_selection_mode = true;
                        self.equip_selection_cursor = 0;
                    } else {
                        game.add_message(
                            format!("No {} items in inventory", current_slot.name()),
                            MessageCategory::Warning
                        );
                    }
                } else {
                    // Skill slot - enter skill selection mode
                    let skill_slot = self.character_slot - NUM_EQUIP_SLOTS;
                    let has_unequipped_skills = game.world()
                        .get::<&SkillsComponent>(player)
                        .map(|sk| {
                            sk.skills.learned.iter().any(|skill| {
                                !sk.skills.slots.iter().any(|s| s.as_ref().map(|eq| eq.id == skill.id).unwrap_or(false))
                            })
                        })
                        .unwrap_or(false);

                    if has_unequipped_skills {
                        self.skill_selection_mode = true;
                        self.skill_selection_cursor = 0;
                        self.skill_slot_to_swap = skill_slot;
                    } else {
                        game.add_message(
                            "No unequipped skills available".to_string(),
                            MessageCategory::Warning
                        );
                    }
                }
            }
            // Unequip selected item or skill
            KeyCode::Char('u') | KeyCode::Enter => {
                let player = match game.player() {
                    Some(p) => p,
                    None => return Ok(false),
                };

                if self.character_slot < NUM_EQUIP_SLOTS {
                    // Unequip equipment
                    let slot = slots[self.character_slot];

                    let unequipped = {
                        if let Ok(mut eq) = game.world_mut().get::<&mut EquipmentComponent>(player) {
                            eq.equipment.unequip(slot)
                        } else {
                            None
                        }
                    };

                    if let Some(item) = unequipped {
                        let item_name = item.name.clone();
                        let added = {
                            if let Ok(mut inv) = game.world_mut().get::<&mut InventoryComponent>(player) {
                                inv.inventory.add_item(item)
                            } else {
                                false
                            }
                        };

                        if added {
                            game.add_message(
                                format!("Unequipped {}", item_name),
                                MessageCategory::Item,
                            );
                        } else {
                            game.add_message(
                                "Inventory full! Cannot unequip.".to_string(),
                                MessageCategory::Warning,
                            );
                        }
                    }
                } else {
                    // Unequip skill
                    let skill_slot = self.character_slot - NUM_EQUIP_SLOTS;
                    let skill_name = {
                        if let Ok(mut sk) = game.world_mut().get::<&mut SkillsComponent>(player) {
                            if let Some(skill) = sk.skills.unequip(skill_slot) {
                                Some(skill.name)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };

                    if let Some(name) = skill_name {
                        game.add_message(format!("Unequipped {}", name), MessageCategory::System);
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_mapview_input(&mut self, key: KeyEvent, game: &mut Game) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('m') => {
                game.set_state(GameState::Playing(PlayingState::Exploring));
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_help_input(&mut self, key: KeyEvent, game: &mut Game) -> Result<bool> {
        const HELP_LINES: u16 = 90; // Approximate number of lines in help

        match key.code {
            KeyCode::Esc | KeyCode::Char('?') => {
                self.help_scroll = 0; // Reset scroll on close
                game.set_state(GameState::Playing(PlayingState::Exploring));
            }
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                self.help_scroll = self.help_scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                if self.help_scroll < HELP_LINES {
                    self.help_scroll += 1;
                }
            }
            KeyCode::PageUp => {
                self.help_scroll = self.help_scroll.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.help_scroll = (self.help_scroll + 10).min(HELP_LINES);
            }
            KeyCode::Home => {
                self.help_scroll = 0;
            }
            KeyCode::End => {
                self.help_scroll = HELP_LINES;
            }
            _ => {}
        }
        Ok(false)
    }

    /// Get the number of enchantments on an equipped item
    fn get_equipped_item_enchant_count(&self, game: &Game, slot: crate::items::EquipSlot) -> usize {
        use crate::ecs::EquipmentComponent;

        if let Some(player) = game.player() {
            if let Ok(eq) = game.world().get::<&EquipmentComponent>(player) {
                if let Some(item) = eq.equipment.get(slot) {
                    return item.affixes.len();
                }
            }
        }
        0
    }

    /// Get (current enchantments, max enchantments) for an equipped item
    fn get_equipped_item_enchant_slots(&self, game: &Game, slot: crate::items::EquipSlot) -> (usize, usize) {
        use crate::ecs::EquipmentComponent;

        if let Some(player) = game.player() {
            if let Ok(eq) = game.world().get::<&EquipmentComponent>(player) {
                if let Some(item) = eq.equipment.get(slot) {
                    return (item.affixes.len(), item.max_enchantments);
                }
            }
        }
        (0, 0)
    }

    /// Get list of equipped items (slot, name, enchant count, max enchants)
    fn get_equipped_items_for_enchant(&self, game: &Game) -> Vec<(crate::items::EquipSlot, String, usize, usize)> {
        use crate::ecs::EquipmentComponent;
        use crate::items::EquipSlot;

        let mut items = Vec::new();
        let slots = [
            EquipSlot::MainHand,
            EquipSlot::OffHand,
            EquipSlot::Head,
            EquipSlot::Body,
            EquipSlot::Hands,
            EquipSlot::Feet,
            EquipSlot::Amulet,
            EquipSlot::Ring1,
            EquipSlot::Ring2,
        ];

        if let Some(player) = game.player() {
            if let Ok(eq) = game.world().get::<&EquipmentComponent>(player) {
                for slot in slots {
                    if let Some(item) = eq.equipment.get(slot) {
                        items.push((slot, item.name.clone(), item.affixes.len(), item.max_enchantments));
                    }
                }
            }
        }
        items
    }

    /// Check if an equip slot is a weapon slot
    fn is_weapon_slot(slot: crate::items::EquipSlot) -> bool {
        use crate::items::EquipSlot;
        matches!(slot, EquipSlot::MainHand | EquipSlot::OffHand)
    }

    fn handle_shrine_input(&mut self, key: KeyEvent, game: &mut Game, shrine_type: ShrineType) -> Result<bool> {
        use crate::ecs::{SkillsComponent, StatusEffects, StatusEffect, StatusEffectType};

        match key.code {
            KeyCode::Esc => {
                // Check if we're in skill swap mode - cancel swap and go back to skill selection
                if shrine_type == ShrineType::Skill && self.shrine_skill_swap_mode {
                    self.shrine_skill_swap_mode = false;
                    self.shrine_pending_skill = None;
                    game.add_message("Cancelled skill replacement.".to_string(), MessageCategory::System);
                    return Ok(false);
                }
                // Check if we're in enchantment selection mode - go back to equipment selection
                if shrine_type == ShrineType::Enchanting && self.enchant_selected_slot.is_some() {
                    self.enchant_selected_slot = None;
                    self.enchant_affix_cursor = 0;
                    self.enchant_swap_mode = false;
                    self.enchant_swap_cursor = 0;
                    return Ok(false);
                }
                // Reset enchanting cursors when leaving
                self.enchant_affix_cursor = 0;
                self.enchant_swap_mode = false;
                self.enchant_swap_cursor = 0;
                self.enchant_selected_slot = None;
                self.enchant_equipment_cursor = 0;
                // Reset skill shrine state when leaving
                self.shrine_skill_swap_mode = false;
                self.shrine_skill_swap_cursor = 0;
                self.shrine_pending_skill = None;
                self.shrine_skills.clear();
                game.set_state(GameState::Playing(PlayingState::Exploring));
            }
            // Corruption shrine pacts (1-3 to select)
            KeyCode::Char('1') | KeyCode::Char('2') | KeyCode::Char('3') if shrine_type == ShrineType::Corruption => {
                let pact_idx = match key.code {
                    KeyCode::Char('1') => 0,
                    KeyCode::Char('2') => 1,
                    KeyCode::Char('3') => 2,
                    _ => return Ok(false),
                };

                // Define corruption pacts: (curse_type, curse_intensity, blessing_type, blessing_intensity, name)
                let pacts = [
                    (StatusEffectType::Weakness, 20, StatusEffectType::Strength, 30, "Pact of Power"),
                    (StatusEffectType::Poison, 5, StatusEffectType::Regeneration, 8, "Pact of Vitality"),
                    (StatusEffectType::Slow, 25, StatusEffectType::Haste, 40, "Pact of Swiftness"),
                ];

                if pact_idx < pacts.len() {
                    let (curse, curse_str, blessing, blessing_str, name) = pacts[pact_idx];

                    if let Some(player) = game.player() {
                        // Get or create status effects component
                        let result = game.world_mut().get::<&mut StatusEffects>(player);

                        if let Ok(mut effects) = result {
                            // Add curse (very long duration - essentially permanent for the floor)
                            effects.effects.push(StatusEffect {
                                effect_type: curse,
                                duration: 9999.0,
                                intensity: curse_str,
                            });
                            // Add blessing
                            effects.effects.push(StatusEffect {
                                effect_type: blessing,
                                duration: 9999.0,
                                intensity: blessing_str,
                            });
                        }
                    }

                    game.add_message(
                        format!("You accept the {}. Dark power courses through you!", name),
                        MessageCategory::Combat
                    );
                    game.set_state(GameState::Playing(PlayingState::Exploring));
                }
            }
            // Navigate skill shrine options
            KeyCode::Up | KeyCode::Char('k') if shrine_type == ShrineType::Skill => {
                if self.shrine_skill_swap_mode {
                    // In swap mode: navigate equipped skills
                    if self.shrine_skill_swap_cursor > 0 {
                        self.shrine_skill_swap_cursor -= 1;
                    }
                } else {
                    // Normal mode: navigate shrine skill options
                    if self.shrine_skill_cursor > 0 {
                        self.shrine_skill_cursor -= 1;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') if shrine_type == ShrineType::Skill => {
                if self.shrine_skill_swap_mode {
                    // In swap mode: navigate equipped skills (5 slots)
                    if self.shrine_skill_swap_cursor < 4 {
                        self.shrine_skill_swap_cursor += 1;
                    }
                } else {
                    // Normal mode: navigate shrine skill options
                    if self.shrine_skill_cursor + 1 < self.shrine_skills.len() {
                        self.shrine_skill_cursor += 1;
                    }
                }
            }
            // Learn skill at skill shrine (Enter to select)
            KeyCode::Enter | KeyCode::Char(' ') if shrine_type == ShrineType::Skill => {
                if self.shrine_skill_swap_mode {
                    // In swap mode: replace the selected equipped skill
                    if let Some(new_skill) = self.shrine_pending_skill.take() {
                        let skill_name = new_skill.name.clone();
                        let skill_rarity = new_skill.rarity.name();
                        let slot = self.shrine_skill_swap_cursor;

                        // Get the name of the skill being replaced
                        let replaced_name = if let Some(player) = game.player() {
                            game.world()
                                .get::<&SkillsComponent>(player)
                                .ok()
                                .and_then(|sc| sc.skills.slots[slot].as_ref().map(|s| s.name.clone()))
                        } else {
                            None
                        };

                        // Do the swap
                        if let Some(player) = game.player() {
                            if let Ok(mut skills) = game.world_mut().get::<&mut SkillsComponent>(player) {
                                skills.skills.equip(slot, new_skill);
                            }
                        }

                        let replaced_msg = replaced_name.map(|n| format!(" (replaced {})", n)).unwrap_or_default();
                        game.add_message(
                            format!("You learned {} [{}]! (Slot {}){}", skill_name, skill_rarity, slot + 1, replaced_msg),
                            MessageCategory::System
                        );

                        // Mark shrine as used
                        if let Some(pos) = game.player_position() {
                            game.mark_shrine_used(pos);
                        }
                        self.shrine_skills.clear();
                        self.shrine_skill_swap_mode = false;
                        game.set_state(GameState::Playing(PlayingState::Exploring));
                    }
                } else if self.shrine_skill_cursor < self.shrine_skills.len() {
                    // Normal mode: try to learn the skill
                    let skill = self.shrine_skills[self.shrine_skill_cursor].clone();
                    let skill_name = skill.name.clone();
                    let skill_rarity = skill.rarity.name();

                    // Do the skill equipping in a separate scope to release the borrow
                    let result = if let Some(player) = game.player() {
                        if let Ok(mut skills) = game.world_mut().get::<&mut SkillsComponent>(player) {
                            // Find first empty slot
                            if let Some(slot) = skills.skills.slots.iter().position(|s| s.is_none()) {
                                skills.skills.equip(slot, skill);
                                Some((true, slot))
                            } else {
                                Some((false, 0))
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Now handle messages and state changes after the borrow is released
                    match result {
                        Some((true, slot)) => {
                            game.add_message(
                                format!("You learned {} [{}]! (Slot {})", skill_name, skill_rarity, slot + 1),
                                MessageCategory::System
                            );
                            // Mark shrine as used
                            if let Some(pos) = game.player_position() {
                                game.mark_shrine_used(pos);
                            }
                            self.shrine_skills.clear();
                            game.set_state(GameState::Playing(PlayingState::Exploring));
                        }
                        Some((false, _)) => {
                            // All slots full - enter swap mode
                            self.shrine_skill_swap_mode = true;
                            self.shrine_skill_swap_cursor = 0;
                            self.shrine_pending_skill = Some(self.shrine_skills[self.shrine_skill_cursor].clone());
                            game.add_message(
                                "All skill slots full! Select a skill to replace:".to_string(),
                                MessageCategory::System
                            );
                        }
                        None => {}
                    }
                }
            }
            // Enchanting shrine - two phases: equipment selection, then enchantment selection
            KeyCode::Up | KeyCode::Char('k') if shrine_type == ShrineType::Enchanting => {
                if self.enchant_selected_slot.is_none() {
                    // Phase 1: Navigate equipment list
                    if self.enchant_equipment_cursor > 0 {
                        self.enchant_equipment_cursor -= 1;
                    }
                } else if self.enchant_swap_mode {
                    // Phase 2, swap mode: navigate existing enchantments
                    if self.enchant_swap_cursor > 0 {
                        self.enchant_swap_cursor -= 1;
                    }
                } else {
                    // Phase 2: Navigate enchantment options
                    if self.enchant_affix_cursor > 0 {
                        self.enchant_affix_cursor -= 1;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') if shrine_type == ShrineType::Enchanting => {
                if self.enchant_selected_slot.is_none() {
                    // Phase 1: Navigate equipment list
                    let items = self.get_equipped_items_for_enchant(game);
                    if self.enchant_equipment_cursor + 1 < items.len() {
                        self.enchant_equipment_cursor += 1;
                    }
                } else if self.enchant_swap_mode {
                    // Phase 2, swap mode: get selected item's enchantment count
                    if let Some(slot) = self.enchant_selected_slot {
                        let enchant_count = self.get_equipped_item_enchant_count(game, slot);
                        if self.enchant_swap_cursor + 1 < enchant_count {
                            self.enchant_swap_cursor += 1;
                        }
                    }
                } else {
                    // Phase 2: Navigate enchantment options
                    // 6 base enchantments (0-5) + optional +1 slot (6) + 4 endgame upgrades
                    let max_option = if self.enchant_upgrade_available { 10 } else { 9 };
                    if self.enchant_affix_cursor < max_option {
                        self.enchant_affix_cursor += 1;
                    }
                }
            }
            KeyCode::Tab if shrine_type == ShrineType::Enchanting => {
                // Toggle swap mode (only if item is at max enchantments)
                if let Some(slot) = self.enchant_selected_slot {
                    let (current, max) = self.get_equipped_item_enchant_slots(game, slot);
                    if current >= max && max > 0 {
                        self.enchant_swap_mode = !self.enchant_swap_mode;
                        self.enchant_swap_cursor = 0;
                    }
                }
            }
            // Select equipment or apply enchantment with Enter
            KeyCode::Enter | KeyCode::Char(' ') if shrine_type == ShrineType::Enchanting => {
                use crate::ecs::EquipmentComponent;
                use crate::items::item::{Affix, AffixType};

                // Phase 1: Select equipment to enchant
                if self.enchant_selected_slot.is_none() {
                    let items = self.get_equipped_items_for_enchant(game);
                    if self.enchant_equipment_cursor < items.len() {
                        let (slot, name, current, max) = &items[self.enchant_equipment_cursor];
                        self.enchant_selected_slot = Some(*slot);
                        self.enchant_affix_cursor = 0;
                        self.enchant_swap_mode = false;
                        self.enchant_swap_cursor = 0;
                        game.add_message(
                            format!("Selected {} ({}/{} enchants). Choose an enchantment.", name, current, max),
                            MessageCategory::System
                        );
                    }
                    return Ok(false);
                }

                // Phase 2: Apply enchantment to selected item
                let target_slot = self.enchant_selected_slot.unwrap();
                let is_weapon = Self::is_weapon_slot(target_slot);

                // Different enchantments for weapons vs armor
                let weapon_enchants: [(AffixType, u32, i32); 6] = [
                    (AffixType::BonusDamage, 50, 3),
                    (AffixType::FireDamage, 75, 4),
                    (AffixType::IceDamage, 75, 4),
                    (AffixType::BonusCritChance, 60, 5),
                    (AffixType::LifeSteal, 100, 3),
                    (AffixType::LightningDamage, 80, 4),
                ];
                let armor_enchants: [(AffixType, u32, i32); 6] = [
                    (AffixType::BonusArmor, 50, 3),
                    (AffixType::BonusHP, 60, 10),
                    (AffixType::BonusMP, 50, 8),
                    (AffixType::FireResist, 70, 10),
                    (AffixType::IceResist, 70, 10),
                    (AffixType::PoisonResist, 70, 10),
                ];

                let enchantments = if is_weapon { weapon_enchants } else { armor_enchants };

                // Special case: +1 max enchantment slot (only if rare upgrade available)
                if self.enchant_affix_cursor == 6 && self.enchant_upgrade_available {
                    let cost = 200u32;
                    let gold = game.player()
                        .and_then(|p| game.world().get::<&crate::ecs::InventoryComponent>(p).ok())
                        .map(|inv| inv.inventory.gold())
                        .unwrap_or(0);

                    let result = if gold < cost {
                        Some(Err("no_gold"))
                    } else if let Some(player) = game.player() {
                        if let Ok(mut equip) = game.world_mut().get::<&mut EquipmentComponent>(player) {
                            if let Some(item) = equip.equipment.get_mut(target_slot) {
                                item.max_enchantments += 1;
                                Some(Ok((item.name.clone(), item.max_enchantments, cost)))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    match result {
                        Some(Ok((name, new_max, cost))) => {
                            if let Some(player) = game.player() {
                                if let Ok(mut inv) = game.world_mut().get::<&mut crate::ecs::InventoryComponent>(player) {
                                    inv.inventory.spend_gold(cost);
                                }
                            }
                            game.add_message(format!("✦ {} can now hold {} enchantments!", name, new_max), MessageCategory::Item);
                            if let Some(pos) = game.player_position() {
                                game.mark_shrine_used(pos);
                            }
                            self.enchant_selected_slot = None;
                            self.enchant_affix_cursor = 0;
                            self.enchant_swap_mode = false;
                            game.set_state(GameState::Playing(PlayingState::Exploring));
                        }
                        Some(Err("no_gold")) => {
                            game.add_message("Not enough gold! (200g required)".to_string(), MessageCategory::Warning);
                        }
                        _ => {}
                    }
                    return Ok(false);
                }

                // Handle endgame upgrade options
                let base_option = if self.enchant_upgrade_available { 7 } else { 6 };
                let is_endgame_option = self.enchant_affix_cursor >= base_option;

                if is_endgame_option && !self.enchant_swap_mode {
                    let option_index = self.enchant_affix_cursor - base_option;
                    let gold = game.player()
                        .and_then(|p| game.world().get::<&crate::ecs::InventoryComponent>(p).ok())
                        .map(|inv| inv.inventory.gold())
                        .unwrap_or(0);

                    match option_index {
                        0 => {
                            // Enchant+ (increase enchantment level)
                            let result = if let Some(player) = game.player() {
                                if let Ok(mut equip) = game.world_mut().get::<&mut EquipmentComponent>(player) {
                                    if let Some(item) = equip.equipment.get_mut(target_slot) {
                                        let cost = item.enchant_cost();
                                        if gold < cost {
                                            Some(Err("no_gold"))
                                        } else if item.enchantment_level >= 15 {
                                            Some(Err("max_level"))
                                        } else {
                                            item.enchant();
                                            Some(Ok((item.display_name(), cost, item.enchantment_level)))
                                        }
                                    } else { None }
                                } else { None }
                            } else { None };

                            match result {
                                Some(Ok((name, cost, level))) => {
                                    if let Some(player) = game.player() {
                                        if let Ok(mut inv) = game.world_mut().get::<&mut crate::ecs::InventoryComponent>(player) {
                                            inv.inventory.spend_gold(cost);
                                        }
                                    }
                                    game.add_message(format!("⚔ {} is now +{}! (+10% stats)", name, level), MessageCategory::Item);
                                }
                                Some(Err("no_gold")) => {
                                    game.add_message("Not enough gold for enchantment!".to_string(), MessageCategory::Warning);
                                }
                                Some(Err("max_level")) => {
                                    game.add_message("Item is at max enchantment level (+15)!".to_string(), MessageCategory::Warning);
                                }
                                _ => {}
                            }
                        }
                        1 => {
                            // Awaken (infinite tier)
                            let result = if let Some(player) = game.player() {
                                if let Ok(mut equip) = game.world_mut().get::<&mut EquipmentComponent>(player) {
                                    if let Some(item) = equip.equipment.get_mut(target_slot) {
                                        let cost = item.awakening_cost();
                                        if gold < cost {
                                            Some(Err("no_gold"))
                                        } else {
                                            item.awaken();
                                            Some(Ok((item.display_name(), cost, item.awakening_level)))
                                        }
                                    } else { None }
                                } else { None }
                            } else { None };

                            match result {
                                Some(Ok((name, cost, level))) => {
                                    if let Some(player) = game.player() {
                                        if let Ok(mut inv) = game.world_mut().get::<&mut crate::ecs::InventoryComponent>(player) {
                                            inv.inventory.spend_gold(cost);
                                        }
                                    }
                                    game.add_message(format!("✦ {} awakened to tier {}! (+10% all stats)", name, level), MessageCategory::Item);
                                }
                                Some(Err("no_gold")) => {
                                    game.add_message("Not enough gold for awakening!".to_string(), MessageCategory::Warning);
                                }
                                _ => {}
                            }
                        }
                        2 => {
                            // Add Socket
                            let result = if let Some(player) = game.player() {
                                if let Ok(mut equip) = game.world_mut().get::<&mut EquipmentComponent>(player) {
                                    if let Some(item) = equip.equipment.get_mut(target_slot) {
                                        let socket_cost = 300 + (item.sockets.len() as u32 * 200);
                                        if gold < socket_cost {
                                            Some(Err("no_gold"))
                                        } else if !item.add_socket() {
                                            Some(Err("max_sockets"))
                                        } else {
                                            Some(Ok((item.display_name(), socket_cost, item.sockets.len())))
                                        }
                                    } else { None }
                                } else { None }
                            } else { None };

                            match result {
                                Some(Ok((name, cost, sockets))) => {
                                    if let Some(player) = game.player() {
                                        if let Ok(mut inv) = game.world_mut().get::<&mut crate::ecs::InventoryComponent>(player) {
                                            inv.inventory.spend_gold(cost);
                                        }
                                    }
                                    game.add_message(format!("◇ Added socket to {}! ({} sockets total)", name, sockets), MessageCategory::Item);
                                }
                                Some(Err("no_gold")) => {
                                    game.add_message("Not enough gold for socket!".to_string(), MessageCategory::Warning);
                                }
                                Some(Err("max_sockets")) => {
                                    game.add_message("Item is at max sockets!".to_string(), MessageCategory::Warning);
                                }
                                _ => {}
                            }
                        }
                        3 => {
                            // Corrupt (risk/reward)
                            let result = if let Some(player) = game.player() {
                                if let Ok(mut equip) = game.world_mut().get::<&mut EquipmentComponent>(player) {
                                    if let Some(item) = equip.equipment.get_mut(target_slot) {
                                        let cost = item.corruption_cost();
                                        if gold < cost {
                                            Some(Err("no_gold"))
                                        } else if item.corruption_level >= 10 {
                                            Some(Err("max_corruption"))
                                        } else {
                                            item.corrupt();
                                            Some(Ok((item.display_name(), cost, item.corruption_level)))
                                        }
                                    } else { None }
                                } else { None }
                            } else { None };

                            match result {
                                Some(Ok((name, cost, level))) => {
                                    if let Some(player) = game.player() {
                                        if let Ok(mut inv) = game.world_mut().get::<&mut crate::ecs::InventoryComponent>(player) {
                                            inv.inventory.spend_gold(cost);
                                        }
                                    }
                                    game.add_message(format!("☠ {} corrupted to {{C{}}}! (+15% dmg, -5% HP)", name, level), MessageCategory::Item);
                                }
                                Some(Err("no_gold")) => {
                                    game.add_message("Not enough gold for corruption!".to_string(), MessageCategory::Warning);
                                }
                                Some(Err("max_corruption")) => {
                                    game.add_message("Item is at max corruption level!".to_string(), MessageCategory::Warning);
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                    return Ok(false);
                }

                let selected_enchant = enchantments.get(self.enchant_affix_cursor).cloned();

                if let Some((affix_type, cost, value)) = selected_enchant {
                    let result = if let Some(player) = game.player() {
                        let gold = game.world().get::<&crate::ecs::InventoryComponent>(player)
                            .map(|inv| inv.inventory.gold())
                            .unwrap_or(0);

                        if gold < cost {
                            Some(Err("no_gold"))
                        } else if let Ok(mut equip) = game.world_mut().get::<&mut EquipmentComponent>(player) {
                            if let Some(item) = equip.equipment.get_mut(target_slot) {
                                // Check if item already has this affix type
                                let existing_idx = item.affixes.iter().position(|a| a.affix_type == affix_type);
                                let current_count = item.affixes.len();
                                let max_count = item.max_enchantments;

                                if let Some(idx) = existing_idx {
                                    // Item has this affix type - allow upgrading/swapping
                                    let old_value = item.affixes[idx].value;
                                    if old_value == value {
                                        Some(Err("same_value"))
                                    } else {
                                        // Swap the existing affix with the new value
                                        item.affixes[idx] = Affix { affix_type, value };
                                        item.generate_name();
                                        Some(Ok(("upgrade", item.name.clone(), cost, old_value)))
                                    }
                                } else if current_count >= max_count && !self.enchant_swap_mode {
                                    Some(Err("max_affixes"))
                                } else if self.enchant_swap_mode {
                                    if self.enchant_swap_cursor < item.affixes.len() {
                                        item.affixes[self.enchant_swap_cursor] = Affix { affix_type, value };
                                        item.generate_name();
                                        Some(Ok(("swap", item.name.clone(), cost, 0)))
                                    } else {
                                        None
                                    }
                                } else {
                                    item.affixes.push(Affix { affix_type, value });
                                    item.generate_name();
                                    Some(Ok(("add", item.name.clone(), cost, 0)))
                                }
                            } else {
                                Some(Err("no_item"))
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    match result {
                        Some(Ok((mode, item_name, spent, old_value))) => {
                            if let Some(player) = game.player() {
                                if let Ok(mut inv) = game.world_mut().get::<&mut crate::ecs::InventoryComponent>(player) {
                                    inv.inventory.spend_gold(spent);
                                }
                            }
                            let msg = match mode {
                                "upgrade" => format!("✦ Upgraded {} on {}! ({} → {})", affix_type.name(), item_name, old_value, value),
                                "swap" => format!("✦ Swapped enchantment on {}! (+{} {})", item_name, affix_type.name(), value),
                                _ => format!("✦ Enchanted {}! (+{} {})", item_name, affix_type.name(), value),
                            };
                            game.add_message(msg, MessageCategory::Item);
                            if let Some(pos) = game.player_position() {
                                game.mark_shrine_used(pos);
                            }
                            self.enchant_selected_slot = None;
                            self.enchant_affix_cursor = 0;
                            self.enchant_swap_mode = false;
                            game.set_state(GameState::Playing(PlayingState::Exploring));
                        }
                        Some(Err("no_gold")) => {
                            game.add_message("Not enough gold!".to_string(), MessageCategory::Warning);
                        }
                        Some(Err("no_item")) => {
                            game.add_message("No item in this slot!".to_string(), MessageCategory::Warning);
                        }
                        Some(Err("same_value")) => {
                            game.add_message("Item already has this exact enchantment!".to_string(), MessageCategory::Warning);
                        }
                        Some(Err("max_affixes")) => {
                            game.add_message("Item is at max enchantments! Use Tab to swap.".to_string(), MessageCategory::Warning);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_shop_input(&mut self, key: KeyEvent, game: &mut Game, npc_entity: hecs::Entity) -> Result<bool> {
        use crate::entities::NpcComponent;
        use crate::ecs::InventoryComponent;
        use crate::entities::npcs::ShopItem;

        // Get shop item count (for buy mode)
        let shop_item_count = game.world()
            .get::<&NpcComponent>(npc_entity)
            .map(|npc| npc.shop_items.len())
            .unwrap_or(0);

        // Get player inventory item count (for sell mode)
        let player_item_count = game.player()
            .and_then(|p| game.world().get::<&InventoryComponent>(p).ok())
            .map(|inv| inv.inventory.items().len())
            .unwrap_or(0);

        match key.code {
            KeyCode::Esc => {
                self.shop_selection = 0;
                self.sell_selection = 0;
                self.shop_mode = 0;
                game.set_state(GameState::Playing(PlayingState::Exploring));
            }
            KeyCode::Tab => {
                // Switch between Buy (0) and Sell (1)
                self.shop_mode = if self.shop_mode == 0 { 1 } else { 0 };
                // Reset cursors when switching
                self.shop_selection = 0;
                self.sell_selection = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.shop_mode == 0 {
                    // Buy mode
                    if self.shop_selection > 0 {
                        self.shop_selection -= 1;
                    }
                } else {
                    // Sell mode
                    if self.sell_selection > 0 {
                        self.sell_selection -= 1;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.shop_mode == 0 {
                    // Buy mode
                    if self.shop_selection + 1 < shop_item_count {
                        self.shop_selection += 1;
                    }
                } else {
                    // Sell mode
                    if self.sell_selection + 1 < player_item_count {
                        self.sell_selection += 1;
                    }
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if self.shop_mode == 0 {
                    // BUY MODE
                    let result = {
                        let npc = game.world().get::<&NpcComponent>(npc_entity);
                        let player = game.player();

                        if let (Ok(npc), Some(player)) = (npc, player) {
                            if let Some(shop_item) = npc.shop_items.get(self.shop_selection) {
                                let price = shop_item.buy_price;
                                let item_name = shop_item.item.name.clone();
                                let item = shop_item.item.clone();

                                // Check player gold
                                let gold = game.world()
                                    .get::<&InventoryComponent>(player)
                                    .map(|inv| inv.inventory.gold())
                                    .unwrap_or(0);

                                if gold >= price {
                                    Some((item, price, item_name, player, self.shop_selection))
                                } else {
                                    None // Not enough gold
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };

                    if let Some((item, price, item_name, player, bought_idx)) = result {
                        // Deduct gold and add item
                        let purchase_result = {
                            if let Ok(mut inv) = game.world_mut().get::<&mut InventoryComponent>(player) {
                                if inv.inventory.spend_gold(price) {
                                    if inv.inventory.add_item(item) {
                                        Some(true) // Success
                                    } else {
                                        // Refund gold if inventory full
                                        inv.inventory.add_gold(price);
                                        Some(false) // Inventory full
                                    }
                                } else {
                                    None // Couldn't spend gold
                                }
                            } else {
                                None
                            }
                        };

                        match purchase_result {
                            Some(true) => {
                                // Remove item from merchant inventory
                                if let Ok(mut npc) = game.world_mut().get::<&mut NpcComponent>(npc_entity) {
                                    if bought_idx < npc.shop_items.len() {
                                        npc.shop_items.remove(bought_idx);
                                    }
                                }
                                // Adjust cursor if needed
                                if self.shop_selection > 0 && self.shop_selection >= shop_item_count.saturating_sub(1) {
                                    self.shop_selection = self.shop_selection.saturating_sub(1);
                                }
                                game.add_message(
                                    format!("Bought {} for {} gold.", item_name, price),
                                    MessageCategory::Item
                                );
                            }
                            Some(false) => {
                                game.add_message(
                                    "Inventory full!".to_string(),
                                    MessageCategory::Warning
                                );
                            }
                            None => {}
                        }
                    } else if shop_item_count > 0 {
                        game.add_message(
                            "Not enough gold!".to_string(),
                            MessageCategory::Warning
                        );
                    }
                } else {
                    // SELL MODE
                    let sell_result = {
                        let player = game.player();
                        if let Some(player) = player {
                            if let Ok(inv) = game.world().get::<&InventoryComponent>(player) {
                                if let Some(&item) = inv.inventory.items().get(self.sell_selection) {
                                    // Calculate sell price (40% of item value)
                                    let sell_price = (item.value as f32 * 0.4).max(1.0) as u32;
                                    Some((item.clone(), sell_price, item.name.clone(), player, self.sell_selection))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };

                    if let Some((item, sell_price, item_name, player, sell_idx)) = sell_result {
                        // Remove item from player inventory and add gold
                        let removed = {
                            if let Ok(mut inv) = game.world_mut().get::<&mut InventoryComponent>(player) {
                                if inv.inventory.remove_at(sell_idx).is_some() {
                                    inv.inventory.add_gold(sell_price);
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        };

                        if removed {
                            // Give item to merchant
                            if let Ok(mut npc) = game.world_mut().get::<&mut NpcComponent>(npc_entity) {
                                npc.shop_items.push(ShopItem::new(item));
                                npc.gold = npc.gold.saturating_sub(sell_price);
                            }
                            // Adjust cursor if needed
                            let new_count = player_item_count.saturating_sub(1);
                            if self.sell_selection > 0 && self.sell_selection >= new_count {
                                self.sell_selection = self.sell_selection.saturating_sub(1);
                            }
                            game.add_message(
                                format!("Sold {} for {} gold.", item_name, sell_price),
                                MessageCategory::Item
                            );
                            game.record_gold_collected(sell_price);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_pause_input(&mut self, key: KeyEvent, game: &mut Game) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('p') => {
                game.set_state(GameState::Playing(PlayingState::Exploring));
            }
            KeyCode::Char('s') => {
                // Open save game slot selection
                game.set_state(GameState::SaveSlots { selected: 0 });
            }
            KeyCode::Char('q') => {
                game.set_state(GameState::MainMenu);
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_save_slots_input(&mut self, key: KeyEvent, game: &mut Game, selected: u8) -> Result<bool> {
        use crate::save::{save_game, save_exists};

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let new_selected = if selected > 0 { selected - 1 } else { 2 };
                game.set_state(GameState::SaveSlots { selected: new_selected });
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let new_selected = if selected < 2 { selected + 1 } else { 0 };
                game.set_state(GameState::SaveSlots { selected: new_selected });
            }
            KeyCode::Enter => {
                // Save to selected slot
                match save_game(game, selected) {
                    Ok(()) => {
                        game.add_message("Game saved successfully!", crate::game::MessageCategory::System);
                        game.set_state(GameState::Playing(PlayingState::Exploring));
                    }
                    Err(e) => {
                        game.add_message(&format!("Failed to save: {}", e), crate::game::MessageCategory::System);
                        game.set_state(GameState::Paused);
                    }
                }
            }
            KeyCode::Char('d') => {
                // Delete save in selected slot
                if save_exists(selected) {
                    if let Err(e) = crate::save::delete_save(selected) {
                        game.add_message(&format!("Failed to delete: {}", e), crate::game::MessageCategory::System);
                    }
                }
            }
            KeyCode::Esc => {
                game.set_state(GameState::Paused);
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_load_slots_input(&mut self, key: KeyEvent, game: &mut Game, selected: u8) -> Result<bool> {
        use crate::save::{load_game, save_exists};

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let new_selected = if selected > 0 { selected - 1 } else { 2 };
                game.set_state(GameState::LoadSlots { selected: new_selected });
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let new_selected = if selected < 2 { selected + 1 } else { 0 };
                game.set_state(GameState::LoadSlots { selected: new_selected });
            }
            KeyCode::Enter => {
                // Load from selected slot (only if save exists)
                if save_exists(selected) {
                    match load_game(selected) {
                        Ok(save_data) => {
                            if let Err(e) = game.restore_from_save(save_data) {
                                game.add_message(&format!("Failed to restore: {}", e), crate::game::MessageCategory::System);
                                game.set_state(GameState::MainMenu);
                            } else {
                                // Sync camera to player
                                if let Some(pos) = game.player_position() {
                                    self.camera = pos;
                                }
                            }
                        }
                        Err(e) => {
                            game.add_message(&format!("Failed to load: {}", e), crate::game::MessageCategory::System);
                            game.set_state(GameState::MainMenu);
                        }
                    }
                }
            }
            KeyCode::Char('d') => {
                // Delete save in selected slot
                if save_exists(selected) {
                    if let Err(e) = crate::save::delete_save(selected) {
                        game.add_message(&format!("Failed to delete: {}", e), crate::game::MessageCategory::System);
                    }
                }
            }
            KeyCode::Esc => {
                game.set_state(GameState::MainMenu);
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_achievements_input(&mut self, key: KeyEvent, game: &mut Game) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('a') => {
                game.set_state(GameState::MainMenu);
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_game_over_input(&mut self, key: KeyEvent, game: &mut Game) -> Result<bool> {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => {
                game.set_state(GameState::MainMenu);
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_victory_input(&mut self, key: KeyEvent, game: &mut Game) -> Result<bool> {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => {
                game.set_state(GameState::MainMenu);
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_new_run_input(&mut self, key: KeyEvent, game: &mut Game) -> Result<bool> {
        // For now, just start with defaults
        match key.code {
            KeyCode::Enter => {
                game.start_new_run(None, crate::progression::Difficulty::Normal);
            }
            KeyCode::Esc => {
                game.set_state(GameState::MainMenu);
            }
            _ => {}
        }
        Ok(false)
    }

    /// Render the current game state
    pub fn render(&self, frame: &mut Frame, game: &Game) {
        // Clear the entire screen first to prevent artifacts
        frame.render_widget(Clear, frame.area());

        match game.state() {
            GameState::MainMenu => self.render_main_menu(frame),
            GameState::Playing(state) => self.render_playing(frame, game, state),
            GameState::Paused => self.render_pause(frame, game),
            GameState::SaveSlots { selected } => self.render_save_slots(frame, game, *selected),
            GameState::LoadSlots { selected } => self.render_load_slots(frame, *selected),
            GameState::Achievements => self.render_achievements(frame, game),
            GameState::GameOver { floor_reached, cause_of_death } => {
                self.render_game_over(frame, *floor_reached, cause_of_death);
            }
            GameState::Victory => self.render_victory(frame),
            GameState::NewRun { .. } => self.render_new_run(frame),
            GameState::Quit => {}
        }
    }

    fn render_main_menu(&self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ])
            .split(area);

        // Title
        let title = vec![
            Line::from(Span::styled(
                r"  _   _       _ _               _                  ",
                Style::default().fg(Color::Rgb(180, 50, 50)),
            )),
            Line::from(Span::styled(
                r" | | | | ___ | | | _____      _| |_ ___ _ __  _ __ ",
                Style::default().fg(Color::Rgb(180, 50, 50)),
            )),
            Line::from(Span::styled(
                r" | |_| |/ _ \| | |/ _ \ \ /\ / / __/ _ \ '_ \| '_ \",
                Style::default().fg(Color::Rgb(150, 40, 40)),
            )),
            Line::from(Span::styled(
                r" |  _  | (_) | | | (_) \ V  V /| ||  __/ |_) | |_) |",
                Style::default().fg(Color::Rgb(120, 30, 30)),
            )),
            Line::from(Span::styled(
                r" |_| |_|\___/|_|_|\___/ \_/\_/  \__\___| .__/| .__/",
                Style::default().fg(Color::Rgb(100, 25, 25)),
            )),
            Line::from(Span::styled(
                r"                                       |_|   |_|   ",
                Style::default().fg(Color::Rgb(80, 20, 20)),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "           Descend into darkness...",
                Style::default().fg(Color::Rgb(100, 100, 100)),
            )),
        ];

        let title_para = Paragraph::new(title)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(title_para, chunks[0]);

        // Menu options
        let menu = vec![
            Line::from(""),
            Line::from(Span::styled(
                "[N] New Game",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "[L] Load Game",
                Style::default().fg(Color::White),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "[A] Achievements",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "[O] Options",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "[Q] Quit",
                Style::default().fg(Color::Gray),
            )),
        ];

        let menu_para = Paragraph::new(menu)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(menu_para, chunks[1]);

        // Version
        let version = Paragraph::new(format!("v{}", env!("CARGO_PKG_VERSION")))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(version, chunks[2]);

        // Difficulty selection popup
        if self.difficulty_selection_mode {
            self.render_difficulty_popup(frame);
        }
    }

    fn render_difficulty_popup(&self, frame: &mut Frame) {
        use crate::progression::Difficulty;

        let popup_area = centered_rect(50, 50, frame.area());
        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" ⚔ Choose Your Fate ⚔ ")
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let difficulties = [
            (Difficulty::Easy, "Relaxed experience", "Enemy damage -30%, Enemy HP -20%, XP -20%"),
            (Difficulty::Normal, "Balanced challenge", "Standard difficulty"),
            (Difficulty::Hard, "Dangerous depths", "Enemy damage +30%, Enemy HP +25%, XP +20%"),
            (Difficulty::Nightmare, "True suffering", "Enemy damage +60%, Enemy HP +50%, XP +50%, More enemies"),
        ];

        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "What difficulty would you like to pursue?",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(""));

        for (i, (diff, name, desc)) in difficulties.iter().enumerate() {
            let is_selected = i == self.difficulty_selection_cursor;
            let prefix = if is_selected { "► " } else { "  " };

            let color = match diff {
                Difficulty::Easy => Color::Green,
                Difficulty::Normal => Color::White,
                Difficulty::Hard => Color::Yellow,
                Difficulty::Nightmare => Color::Rgb(200, 50, 50),
            };

            let name_style = if is_selected {
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color)
            };

            lines.push(Line::from(vec![
                Span::styled(prefix, if is_selected { Style::default().fg(Color::Yellow) } else { Style::default() }),
                Span::styled(format!("{:<12}", diff.name()), name_style),
                Span::styled(format!(" - {}", name), Style::default().fg(Color::Gray)),
            ]));

            if is_selected {
                lines.push(Line::from(Span::styled(
                    format!("    {}", desc),
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                )));
            }
            lines.push(Line::from(""));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "[↑↓] Select  [Enter] Start  [Esc] Cancel",
            Style::default().fg(Color::DarkGray),
        )));

        let para = Paragraph::new(lines)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(para, inner);
    }

    fn render_playing(&self, frame: &mut Frame, game: &Game, state: &PlayingState) {
        let area = frame.area();

        // Main layout: sidebar on right
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(40), Constraint::Length(25)])
            .split(area);

        // Map area with message log at bottom
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(20), Constraint::Length(7)])
            .split(chunks[0]);

        // Render map
        self.render_map(frame, game, left_chunks[0]);

        // Render message log
        self.render_messages(frame, game, left_chunks[1]);

        // Render sidebar
        self.render_sidebar(frame, game, chunks[1]);

        // Render overlay for special states
        match state {
            PlayingState::Inventory => self.render_inventory_overlay(frame, game),
            PlayingState::Character => self.render_character_overlay(frame, game),
            PlayingState::MapView => self.render_fullmap_overlay(frame, game),
            PlayingState::Help => self.render_help_overlay(frame),
            PlayingState::Shrine { shrine_type } => self.render_shrine_overlay(frame, game, *shrine_type),
            PlayingState::Shop { npc_entity } => self.render_shop_overlay(frame, game, *npc_entity),
            _ => {}
        }
    }

    fn render_map(&self, frame: &mut Frame, game: &Game, area: Rect) {
        let map = match game.map() {
            Some(m) => m,
            None => return,
        };

        // Get biome config for ambient colors and glyph variations
        let biome_config = map.biome.config();
        let ambient = biome_config.ambient_color;

        // Show render mode in title
        let mode_indicator = match self.render_mode {
            RenderMode::Ascii => "[ASCII]",
            RenderMode::Unicode => "[Unicode]",
            RenderMode::NerdFont => "[Nerd]",
            RenderMode::Kitty => "[Kitty]",
        };

        // Color the border based on biome
        let border_color = Color::Rgb(
            (ambient.0 as f32 * 0.8) as u8,
            (ambient.1 as f32 * 0.8) as u8,
            (ambient.2 as f32 * 0.8) as u8,
        );

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} - Floor {} {} ", map.biome.name(), map.floor_number, mode_indicator))
            .border_style(Style::default().fg(border_color));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Calculate viewport
        let view_width = inner.width as i32;
        let view_height = inner.height as i32;

        let cam_x = self.camera.x - view_width / 2;
        let cam_y = self.camera.y - view_height / 2;

        // Render tiles using the tile renderer with biome colors
        for screen_y in 0..view_height {
            for screen_x in 0..view_width {
                let map_x = cam_x + screen_x;
                let map_y = cam_y + screen_y;

                let cell_x = inner.x + screen_x as u16;
                let cell_y = inner.y + screen_y as u16;

                if cell_x >= inner.x + inner.width || cell_y >= inner.y + inner.height {
                    continue;
                }

                let buf = frame.buffer_mut();

                if let Some(tile) = map.get_tile(map_x, map_y) {
                    if tile.explored {
                        // Use biome-specific glyph variation based on position
                        let ch = self.get_biome_glyph(tile.tile_type, &biome_config, map_x, map_y);

                        // Use biome-aware colors for ambient lighting
                        let fg = self.tile_renderer.tile_fg_color_biome(tile.tile_type, tile.visible, ambient);
                        let bg = self.tile_renderer.tile_bg_color_biome(tile.tile_type, tile.visible, ambient);

                        buf[(cell_x, cell_y)].set_char(ch);
                        buf[(cell_x, cell_y)].set_fg(fg);
                        buf[(cell_x, cell_y)].set_bg(bg);
                    } else {
                        // Unexplored - tint with biome ambient
                        buf[(cell_x, cell_y)].set_char(' ');
                        buf[(cell_x, cell_y)].set_bg(Color::Rgb(
                            (ambient.0 as f32 * 0.1) as u8,
                            (ambient.1 as f32 * 0.1) as u8,
                            (ambient.2 as f32 * 0.1) as u8,
                        ));
                    }
                } else {
                    // Out of bounds - dark with subtle biome tint
                    buf[(cell_x, cell_y)].set_char(' ');
                    buf[(cell_x, cell_y)].set_bg(Color::Rgb(
                        (ambient.0 as f32 * 0.05) as u8,
                        (ambient.1 as f32 * 0.05) as u8,
                        (ambient.2 as f32 * 0.05) as u8,
                    ));
                }

            }
        }

        // Render all entities with Position and Renderable
        // Query for enemies with health to color by HP
        use crate::ecs::{Position, Renderable, Health, Enemy};
        for (_, (pos, renderable, maybe_health, maybe_enemy)) in game.world()
            .query::<(&Position, &Renderable, Option<&Health>, Option<&Enemy>)>()
            .iter()
        {
            // Check if entity is in view
            let screen_x = pos.x - cam_x;
            let screen_y = pos.y - cam_y;

            if screen_x >= 0 && screen_x < view_width && screen_y >= 0 && screen_y < view_height {
                let cell_x = inner.x + screen_x as u16;
                let cell_y = inner.y + screen_y as u16;

                // Check if tile is visible
                if let Some(tile) = map.get_tile(pos.x, pos.y) {
                    if tile.visible {
                        let buf = frame.buffer_mut();
                        buf[(cell_x, cell_y)].set_char(renderable.glyph);

                        // Color enemies by health percentage
                        let fg_color = if maybe_enemy.is_some() {
                            if let Some(hp) = maybe_health {
                                let pct = hp.percentage();
                                if pct > 0.6 {
                                    // Healthy - use normal color
                                    Color::Rgb(renderable.fg.0, renderable.fg.1, renderable.fg.2)
                                } else if pct > 0.3 {
                                    // Wounded - yellow tint
                                    Color::Rgb(255, 200, 100)
                                } else {
                                    // Critical - red
                                    Color::Rgb(255, 80, 80)
                                }
                            } else {
                                Color::Rgb(renderable.fg.0, renderable.fg.1, renderable.fg.2)
                            }
                        } else {
                            Color::Rgb(renderable.fg.0, renderable.fg.1, renderable.fg.2)
                        };

                        buf[(cell_x, cell_y)].set_fg(fg_color);
                    }
                }
            }
        }

        // Draw player on top (highest render order)
        let player_screen_x = self.camera.x - cam_x;
        let player_screen_y = self.camera.y - cam_y;
        if player_screen_x >= 0 && player_screen_x < view_width
            && player_screen_y >= 0 && player_screen_y < view_height
        {
            let cell_x = inner.x + player_screen_x as u16;
            let cell_y = inner.y + player_screen_y as u16;
            let player_char = match self.render_mode {
                RenderMode::Ascii => '@',
                RenderMode::Unicode => '☺',
                RenderMode::NerdFont => '󰀄',
                RenderMode::Kitty => '☺', // Will be sprite later
            };
            let buf = frame.buffer_mut();
            buf[(cell_x, cell_y)].set_char(player_char);
            buf[(cell_x, cell_y)].set_fg(Color::Rgb(255, 255, 200));
        }

        // Render minimap overlay in top-right corner
        self.render_minimap(frame, game, inner);
    }

    /// Render a minimap in the corner of the map area
    fn render_minimap(&self, frame: &mut Frame, game: &Game, map_area: Rect) {
        let map = match game.map() {
            Some(m) => m,
            None => return,
        };

        // Minimap dimensions (scaled down)
        let minimap_width: u16 = 20;
        let minimap_height: u16 = 10;

        // Position in top-right corner with a small margin
        if map_area.width < minimap_width + 4 || map_area.height < minimap_height + 2 {
            return; // Not enough space for minimap
        }

        let minimap_x = map_area.x + map_area.width - minimap_width - 1;
        let minimap_y = map_area.y + 1;

        let minimap_area = Rect {
            x: minimap_x,
            y: minimap_y,
            width: minimap_width,
            height: minimap_height,
        };

        // Draw semi-transparent background
        let buf = frame.buffer_mut();
        for y in minimap_area.y..minimap_area.y + minimap_area.height {
            for x in minimap_area.x..minimap_area.x + minimap_area.width {
                buf[(x, y)].set_bg(Color::Rgb(20, 20, 30));
            }
        }

        // Draw border
        let border_color = Color::Rgb(60, 60, 80);
        // Top border
        for x in minimap_area.x..minimap_area.x + minimap_area.width {
            buf[(x, minimap_area.y)].set_char('─');
            buf[(x, minimap_area.y)].set_fg(border_color);
        }
        // Bottom border
        for x in minimap_area.x..minimap_area.x + minimap_area.width {
            buf[(x, minimap_area.y + minimap_area.height - 1)].set_char('─');
            buf[(x, minimap_area.y + minimap_area.height - 1)].set_fg(border_color);
        }
        // Left border
        for y in minimap_area.y..minimap_area.y + minimap_area.height {
            buf[(minimap_area.x, y)].set_char('│');
            buf[(minimap_area.x, y)].set_fg(border_color);
        }
        // Right border
        for y in minimap_area.y..minimap_area.y + minimap_area.height {
            buf[(minimap_area.x + minimap_area.width - 1, y)].set_char('│');
            buf[(minimap_area.x + minimap_area.width - 1, y)].set_fg(border_color);
        }
        // Corners
        buf[(minimap_area.x, minimap_area.y)].set_char('┌');
        buf[(minimap_area.x + minimap_area.width - 1, minimap_area.y)].set_char('┐');
        buf[(minimap_area.x, minimap_area.y + minimap_area.height - 1)].set_char('└');
        buf[(minimap_area.x + minimap_area.width - 1, minimap_area.y + minimap_area.height - 1)].set_char('┘');

        // Inner area for the actual map
        let inner_x = minimap_area.x + 1;
        let inner_y = minimap_area.y + 1;
        let inner_w = minimap_area.width - 2;
        let inner_h = minimap_area.height - 2;

        // Scale factor - how many map tiles per minimap cell
        let scale_x = (map.width as f32 / inner_w as f32).ceil() as i32;
        let scale_y = (map.height as f32 / inner_h as f32).ceil() as i32;

        // Get player position
        let player_pos = game.player_position().unwrap_or(Position::new(0, 0));

        // Get enemy positions
        let mut enemy_positions: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
        for (_, (pos, _)) in game.world().query::<(&Position, &crate::ecs::Enemy)>().iter() {
            enemy_positions.insert((pos.x, pos.y));
        }

        // Draw minimap tiles
        for screen_y in 0..inner_h as i32 {
            for screen_x in 0..inner_w as i32 {
                let cell_x = inner_x + screen_x as u16;
                let cell_y = inner_y + screen_y as u16;

                // Sample the center of this minimap cell's region
                let map_x = screen_x * scale_x + scale_x / 2;
                let map_y = screen_y * scale_y + scale_y / 2;

                // Check if player is in this cell's region
                let player_in_region = player_pos.x >= screen_x * scale_x
                    && player_pos.x < (screen_x + 1) * scale_x
                    && player_pos.y >= screen_y * scale_y
                    && player_pos.y < (screen_y + 1) * scale_y;

                // Check if any enemy is in this region
                let enemy_in_region = (screen_x * scale_x..(screen_x + 1) * scale_x)
                    .any(|mx| (screen_y * scale_y..(screen_y + 1) * scale_y)
                        .any(|my| enemy_positions.contains(&(mx, my))));

                if player_in_region {
                    // Player marker - bright yellow
                    buf[(cell_x, cell_y)].set_char('@');
                    buf[(cell_x, cell_y)].set_fg(Color::Rgb(255, 255, 100));
                } else if enemy_in_region {
                    // Enemy marker - red dot
                    buf[(cell_x, cell_y)].set_char('•');
                    buf[(cell_x, cell_y)].set_fg(Color::Red);
                } else if let Some(tile) = map.get_tile(map_x, map_y) {
                    if tile.explored {
                        let (ch, fg) = match tile.tile_type {
                            TileType::Wall => ('█', Color::Rgb(60, 50, 50)),
                            TileType::Floor | TileType::Corridor => {
                                if tile.visible {
                                    ('·', Color::Rgb(80, 80, 100))
                                } else {
                                    ('·', Color::Rgb(40, 40, 50))
                                }
                            }
                            TileType::StairsDown => ('>', Color::Rgb(100, 200, 100)),
                            TileType::StairsUp => ('<', Color::Rgb(100, 100, 200)),
                            TileType::DoorClosed | TileType::DoorOpen => ('+', Color::Rgb(139, 90, 43)),
                            TileType::ShrineSkill | TileType::ShrineEnchant | TileType::ShrineRest | TileType::ShrineCorruption => ('☼', Color::Rgb(150, 100, 200)),
                            TileType::Lava => ('~', Color::Rgb(200, 60, 20)),
                            TileType::Pit => ('○', Color::Rgb(30, 30, 30)),
                            TileType::Torch | TileType::Brazier => ('*', Color::Rgb(200, 150, 50)),
                            _ => (' ', Color::Rgb(30, 30, 40)),
                        };
                        buf[(cell_x, cell_y)].set_char(ch);
                        buf[(cell_x, cell_y)].set_fg(fg);
                    } else {
                        buf[(cell_x, cell_y)].set_char(' ');
                    }
                }
            }
        }
    }

    /// Get a biome-specific glyph variation for visual variety
    fn get_biome_glyph(&self, tile_type: TileType, config: &crate::world::generation::BiomeConfig, x: i32, y: i32) -> char {
        // Use position to create pseudo-random but consistent variation
        let hash = ((x.wrapping_mul(7) ^ y.wrapping_mul(13)).abs() as usize) % 97;

        match tile_type {
            TileType::Wall => {
                // Vary wall glyphs based on biome config
                if !config.wall_glyphs.is_empty() {
                    let idx = hash % config.wall_glyphs.len();
                    config.wall_glyphs[idx]
                } else {
                    self.tile_renderer.tile_char(tile_type)
                }
            }
            TileType::Floor => {
                // Vary floor glyphs based on biome config
                if !config.floor_glyphs.is_empty() {
                    let idx = hash % config.floor_glyphs.len();
                    config.floor_glyphs[idx]
                } else {
                    self.tile_renderer.tile_char(tile_type)
                }
            }
            TileType::Corridor => {
                // Corridors can also use floor glyph variations
                if !config.floor_glyphs.is_empty() && hash % 3 == 0 {
                    let idx = hash % config.floor_glyphs.len();
                    config.floor_glyphs[idx]
                } else {
                    self.tile_renderer.tile_char(tile_type)
                }
            }
            _ => self.tile_renderer.tile_char(tile_type),
        }
    }

    fn render_messages(&self, frame: &mut Frame, game: &Game, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Messages ")
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);

        let messages: Vec<Line> = game
            .messages()
            .iter()
            .rev()
            .take(inner.height as usize)
            .rev()
            .map(|msg| {
                let color = match msg.category {
                    MessageCategory::Combat => Color::Red,
                    MessageCategory::Item => Color::Yellow,
                    MessageCategory::System => Color::Cyan,
                    MessageCategory::Lore => Color::Magenta,
                    MessageCategory::Warning => Color::LightRed,
                };
                Line::from(Span::styled(&msg.text, Style::default().fg(color)))
            })
            .collect();

        let para = Paragraph::new(messages).block(block);
        frame.render_widget(para, area);
    }

    fn render_sidebar(&self, frame: &mut Frame, game: &Game, area: Rect) {
        use crate::ecs::{EquipmentComponent, StatusEffects, StatusEffectType};

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Status ")
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Get real player stats
        let health = game.player_health().unwrap_or(crate::ecs::Health::new(100));
        let mana = game.player_mana().unwrap_or(crate::ecs::Mana::new(50));
        let stamina = game.player_stamina().unwrap_or(crate::ecs::Stamina::new(50));
        let stats = game.player_stats().unwrap_or(crate::ecs::Stats::player_base());
        let xp = game.player_experience().unwrap_or(crate::ecs::Experience::new());

        // Get equipment bonuses for HP/MP
        let (eq_hp, eq_mp) = if let Some(player) = game.player() {
            game.world().get::<&EquipmentComponent>(player)
                .map(|eq| (eq.equipment.hp_bonus(), eq.equipment.mp_bonus()))
                .unwrap_or((0, 0))
        } else {
            (0, 0)
        };

        // Calculate effective max values
        let effective_max_hp = health.max + eq_hp;
        let effective_max_mp = mana.max + eq_mp;

        // HP color based on percentage (using effective max)
        let hp_pct = health.current as f32 / effective_max_hp as f32;
        let hp_color = if hp_pct > 0.6 {
            Color::Green
        } else if hp_pct > 0.3 {
            Color::Yellow
        } else {
            Color::Red
        };

        // Format HP/MP with bonus indicators
        let hp_str = if eq_hp > 0 {
            format!("{}/{} (+{})", health.current, effective_max_hp, eq_hp)
        } else {
            format!("{}/{}", health.current, effective_max_hp)
        };
        let mp_str = if eq_mp > 0 {
            format!("{}/{} (+{})", mana.current, effective_max_mp, eq_mp)
        } else {
            format!("{}/{}", mana.current, effective_max_mp)
        };

        let mut lines = vec![
            Line::from(Span::styled("Hero", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::raw("HP: "),
                Span::styled(hp_str, Style::default().fg(hp_color)),
            ]),
            Line::from(vec![
                Span::raw("MP: "),
                Span::styled(mp_str, Style::default().fg(Color::Blue)),
            ]),
            Line::from(vec![
                Span::raw("SP: "),
                Span::styled(format!("{}/{}", stamina.current, stamina.max), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from(Span::styled(format!("Level {}", xp.level), Style::default().fg(Color::Cyan))),
            Line::from(vec![
                Span::raw("XP: "),
                Span::raw(format!("{}/{}", xp.current_xp, xp.xp_to_next)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Floor ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{}", game.floor()), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(Span::styled(
                game.biome().name(),
                Style::default().fg(Color::Rgb(
                    game.biome().config().ambient_color.0,
                    game.biome().config().ambient_color.1,
                    game.biome().config().ambient_color.2,
                )).add_modifier(Modifier::ITALIC),
            )),
            Line::from(vec![
                Span::styled(format!("[{}]", game.difficulty().name()), Style::default().fg(match game.difficulty() {
                    crate::progression::Difficulty::Easy => Color::Green,
                    crate::progression::Difficulty::Normal => Color::White,
                    crate::progression::Difficulty::Hard => Color::Yellow,
                    crate::progression::Difficulty::Nightmare => Color::Red,
                })),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("STR", Style::default().fg(Color::DarkGray)),
                Span::raw(format!(":{} ", stats.strength)),
                Span::styled("DEX", Style::default().fg(Color::DarkGray)),
                Span::raw(format!(":{}", stats.dexterity)),
            ]),
            Line::from(vec![
                Span::styled("INT", Style::default().fg(Color::DarkGray)),
                Span::raw(format!(":{} ", stats.intelligence)),
                Span::styled("VIT", Style::default().fg(Color::DarkGray)),
                Span::raw(format!(":{}", stats.vitality)),
            ]),
        ];

        // Add status effects section
        if let Some(player) = game.player() {
            if let Ok(status) = game.world().get::<&StatusEffects>(player) {
                if !status.effects.is_empty() {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled("Effects", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));

                    // Group effects into buffs and debuffs
                    let mut buff_spans: Vec<Span> = Vec::new();
                    let mut debuff_spans: Vec<Span> = Vec::new();

                    for effect in &status.effects {
                        let (icon, color, is_buff) = match effect.effect_type {
                            // Debuffs
                            StatusEffectType::Poison => ("☠", Color::Green, false),
                            StatusEffectType::Burn => ("🔥", Color::Red, false),
                            StatusEffectType::Bleed => ("💉", Color::Red, false),
                            StatusEffectType::Slow => ("🐌", Color::Blue, false),
                            StatusEffectType::Weakness => ("↓", Color::Magenta, false),
                            StatusEffectType::Curse => ("☽", Color::Rgb(100, 50, 100), false),
                            // Buffs
                            StatusEffectType::Regeneration => ("❤", Color::Green, true),
                            StatusEffectType::Haste => ("⚡", Color::Yellow, true),
                            StatusEffectType::Shield => ("🛡", Color::Cyan, true),
                            StatusEffectType::Strength => ("↑", Color::Red, true),
                        };

                        let duration_text = if effect.duration > 0.0 {
                            format!("{}{:.0}", icon, effect.duration)
                        } else {
                            icon.to_string()
                        };

                        let span = Span::styled(format!("{} ", duration_text), Style::default().fg(color));

                        if is_buff {
                            buff_spans.push(span);
                        } else {
                            debuff_spans.push(span);
                        }
                    }

                    // Display buffs on one line
                    if !buff_spans.is_empty() {
                        let mut line_spans = vec![Span::styled("+ ", Style::default().fg(Color::Green))];
                        line_spans.extend(buff_spans);
                        lines.push(Line::from(line_spans));
                    }

                    // Display debuffs on another line
                    if !debuff_spans.is_empty() {
                        let mut line_spans = vec![Span::styled("- ", Style::default().fg(Color::Red))];
                        line_spans.extend(debuff_spans);
                        lines.push(Line::from(line_spans));
                    }
                }
            }
        }

        // Add nearby enemies section
        let player_pos = game.player_position().unwrap_or(Position::new(0, 0));
        let mut nearby_enemies: Vec<_> = game.world()
            .query::<(&Position, &crate::ecs::Name, &crate::ecs::Health, &crate::ecs::Enemy)>()
            .iter()
            .filter(|(_, (pos, _, _, _))| pos.chebyshev_distance(&player_pos) <= 8)
            .map(|(_, (pos, name, hp, _))| (pos.chebyshev_distance(&player_pos), name.0.clone(), *hp))
            .collect();

        nearby_enemies.sort_by_key(|(dist, _, _)| *dist);

        if !nearby_enemies.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Nearby", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))));

            for (dist, name, hp) in nearby_enemies.iter().take(5) {
                let hp_pct = hp.percentage();
                let hp_color = if hp_pct > 0.6 { Color::Green }
                              else if hp_pct > 0.3 { Color::Yellow }
                              else { Color::Red };

                // Create a mini health bar
                let bar_width = 8;
                let filled = (bar_width as f32 * hp_pct).round() as usize;
                let bar = format!("{}{}", "█".repeat(filled), "░".repeat(bar_width - filled));

                lines.push(Line::from(vec![
                    Span::styled(format!("{} ", name), Style::default().fg(Color::White)),
                    Span::styled(format!("({})", dist), Style::default().fg(Color::DarkGray)),
                ]));
                lines.push(Line::from(Span::styled(bar, Style::default().fg(hp_color))));
            }
        }

        // Skills hotbar
        if let Some(player) = game.player() {
            if let Ok(skills) = game.world().get::<&crate::ecs::SkillsComponent>(player) {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled("Skills", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))));

                for i in 0..5 {
                    if let Some(skill) = &skills.skills.slots[i] {
                        let cd = skills.skills.cooldowns[i];
                        let can_use = skills.skills.can_use(i, mana.current, stamina.current);

                        let (key_style, skill_style) = if cd > 0 {
                            (Style::default().fg(Color::Red), Style::default().fg(Color::DarkGray))
                        } else if can_use {
                            (Style::default().fg(Color::Yellow), Style::default().fg(Color::White))
                        } else {
                            (Style::default().fg(Color::DarkGray), Style::default().fg(Color::DarkGray))
                        };

                        let cd_text = if cd > 0 { format!("({})", cd) } else { String::new() };

                        lines.push(Line::from(vec![
                            Span::styled(format!("[{}]", i + 1), key_style),
                            Span::styled(format!("{}", skill.icon), skill_style),
                            Span::styled(cd_text, Style::default().fg(Color::Red)),
                        ]));
                    }
                }
            }
        }

        // Controls section
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Controls", Style::default().fg(Color::DarkGray))));
        lines.push(Line::from(Span::styled("[I]nventory", Style::default().fg(Color::DarkGray))));
        lines.push(Line::from(Span::styled("[C]haracter", Style::default().fg(Color::DarkGray))));
        lines.push(Line::from(Span::styled("[M]ap", Style::default().fg(Color::DarkGray))));
        lines.push(Line::from(Span::styled("[G]rab item", Style::default().fg(Color::DarkGray))));
        lines.push(Line::from(Span::styled("[>] Descend", Style::default().fg(Color::DarkGray))));

        let para = Paragraph::new(lines);
        frame.render_widget(para, inner);
    }

    fn render_inventory_overlay(&self, frame: &mut Frame, game: &Game) {
        // Use near-fullscreen overlay for better space utilization
        let area = fullscreen_overlay(frame.area());
        frame.render_widget(Clear, area);

        // Main block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Inventory ")
            .border_style(Style::default().fg(Color::Yellow));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split into tabs header + content + help
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Tab bar
                Constraint::Min(10),   // Content
                Constraint::Length(3), // Help
            ])
            .split(inner);

        // Tab bar
        let tab_items = if self.inventory_tab == 0 {
            vec![
                Span::styled(" [All Items] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" Equipment ", Style::default().fg(Color::DarkGray)),
            ]
        } else {
            vec![
                Span::styled(" All Items ", Style::default().fg(Color::DarkGray)),
                Span::styled(" [Equipment] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]
        };
        let tab_line = Line::from(tab_items);
        frame.render_widget(Paragraph::new(vec![tab_line, Line::from("")]), layout[0]);

        // Get player data
        let player = game.player();

        if self.inventory_tab == 0 {
            // Items tab
            self.render_items_tab(frame, game, player, layout[1]);
        } else {
            // Equipment tab
            self.render_equipment_tab(frame, game, player, layout[1]);
        }

        // Help bar
        let help = if self.inventory_tab == 0 {
            "[Tab] Switch | [↑↓] Navigate | [Enter] Use/Equip | [D]estroy | [S]ort | [Esc] Close"
        } else {
            "[Tab] Switch | [↑↓] Navigate | [Enter] Unequip | [Esc] Close"
        };
        let help_para = Paragraph::new(help)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(help_para, layout[2]);
    }

    fn render_items_tab(&self, frame: &mut Frame, game: &Game, player: Option<hecs::Entity>, area: Rect) {
        use crate::ecs::InventoryComponent;

        let player = match player {
            Some(p) => p,
            None => return,
        };

        let inv = match game.world().get::<&InventoryComponent>(player) {
            Ok(i) => i,
            Err(_) => return,
        };

        // Split: item list on left, details on right
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Item list
        let items = inv.inventory.items();
        let mut lines: Vec<Line> = Vec::new();

        // Gold and sort mode display
        let sort_mode_name = match self.inventory_sort_mode {
            crate::items::SortMode::Size => "Size",
            crate::items::SortMode::Rarity => "Rarity",
            crate::items::SortMode::Category => "Category",
            crate::items::SortMode::Name => "Name",
            crate::items::SortMode::New => "New First",
        };
        let new_count = inv.inventory.count_new();
        let new_indicator = if new_count > 0 {
            format!(" ({} new)", new_count)
        } else {
            String::new()
        };

        lines.push(Line::from(vec![
            Span::styled("Gold: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", inv.inventory.gold()), Style::default().fg(Color::Yellow)),
            Span::styled("  Sort: ", Style::default().fg(Color::DarkGray)),
            Span::styled(sort_mode_name, Style::default().fg(Color::Cyan)),
            Span::styled(new_indicator, Style::default().fg(Color::Green)),
        ]));
        lines.push(Line::from(""));

        if items.is_empty() {
            lines.push(Line::from(Span::styled(
                "  (empty)",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            for (i, item) in items.iter().enumerate() {
                let is_selected = i == self.inventory_cursor;
                let prefix = if is_selected { "> " } else { "  " };

                let rarity_color = item.rarity.color();
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Rgb(rarity_color.0, rarity_color.1, rarity_color.2))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::Rgb(rarity_color.0, rarity_color.1, rarity_color.2))
                };

                let stack_str = if item.stack_count > 1 {
                    format!(" (x{})", item.stack_count)
                } else {
                    String::new()
                };

                // "NEW" indicator for recently picked up items
                let new_str = if item.is_new { " NEW" } else { "" };

                // Truncate item name to fit (max 18 chars for name + stack + new info)
                let display_name = truncate_name(&item.name, 18);

                lines.push(Line::from(vec![
                    Span::raw(prefix),
                    Span::styled(format!("{}{}", display_name, stack_str), style),
                    Span::styled(new_str, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ]));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("{}/{} slots", items.len(), inv.inventory.capacity()),
            Style::default().fg(Color::DarkGray),
        )));

        let list_block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(Color::DarkGray));
        let list_para = Paragraph::new(lines).block(list_block);
        frame.render_widget(list_para, layout[0]);

        // Item details on right (with comparison for equipment)
        if let Some(item) = items.get(self.inventory_cursor) {
            use crate::ecs::EquipmentComponent;

            let rarity_color = item.rarity.color();
            let mut detail_lines: Vec<Line> = Vec::new();

            // Check if this is an equippable item for comparison
            let equipped_item = if let Some(slot) = item.equip_slot {
                game.world()
                    .get::<&EquipmentComponent>(player)
                    .ok()
                    .and_then(|eq| eq.equipment.get(slot).cloned())
            } else {
                None
            };

            if item.is_equippable() && equipped_item.is_some() {
                let equipped = equipped_item.as_ref().unwrap();
                let eq_color = equipped.rarity.color();

                // ══════════════════════════════════════
                // SECTION 1: SELECTED ITEM (what you're hovering)
                // ══════════════════════════════════════
                detail_lines.push(Line::from(Span::styled(
                    "▼ SELECTED ITEM",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )));
                detail_lines.push(Line::from("─".repeat(28)));

                // Item name and rarity
                let display_name = truncate_name(&item.name, 26);
                detail_lines.push(Line::from(Span::styled(
                    display_name,
                    Style::default()
                        .fg(Color::Rgb(rarity_color.0, rarity_color.1, rarity_color.2))
                        .add_modifier(Modifier::BOLD),
                )));
                detail_lines.push(Line::from(Span::styled(
                    item.rarity.name(),
                    Style::default().fg(Color::Rgb(rarity_color.0, rarity_color.1, rarity_color.2)),
                )));

                // Stats with comparison indicators
                if item.base_damage > 0 {
                    let new_dmg = item.total_damage();
                    let old_dmg = equipped.total_damage();
                    let diff = new_dmg - old_dmg;
                    let (diff_indicator, diff_color) = if diff > 0 {
                        (format!(" ▲+{}", diff), Color::Green)
                    } else if diff < 0 {
                        (format!(" ▼{}", diff), Color::Red)
                    } else {
                        (" ─".to_string(), Color::Gray)
                    };

                    detail_lines.push(Line::from(vec![
                        Span::styled("  ⚔ Damage: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("{}", new_dmg), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                        Span::styled(diff_indicator, Style::default().fg(diff_color).add_modifier(Modifier::BOLD)),
                    ]));
                }

                if item.base_armor > 0 {
                    let new_arm = item.total_armor();
                    let old_arm = equipped.total_armor();
                    let diff = new_arm - old_arm;
                    let (diff_indicator, diff_color) = if diff > 0 {
                        (format!(" ▲+{}", diff), Color::Green)
                    } else if diff < 0 {
                        (format!(" ▼{}", diff), Color::Red)
                    } else {
                        (" ─".to_string(), Color::Gray)
                    };

                    detail_lines.push(Line::from(vec![
                        Span::styled("  🛡 Armor: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("{}", new_arm), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                        Span::styled(diff_indicator, Style::default().fg(diff_color).add_modifier(Modifier::BOLD)),
                    ]));
                }

                // Affixes
                if !item.affixes.is_empty() {
                    detail_lines.push(Line::from(Span::styled("  Enchantments:", Style::default().fg(Color::DarkGray))));
                    for affix in &item.affixes {
                        detail_lines.push(Line::from(vec![
                            Span::styled(format!("    ✦ +{} ", affix.value), Style::default().fg(Color::Green)),
                            Span::styled(affix.affix_type.name(), Style::default().fg(Color::Green)),
                        ]));
                    }
                }

                // ══════════════════════════════════════
                // SECTION 2: CURRENTLY EQUIPPED (what you'll replace)
                // ══════════════════════════════════════
                detail_lines.push(Line::from(""));
                detail_lines.push(Line::from(Span::styled(
                    "▶ CURRENTLY EQUIPPED",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )));
                detail_lines.push(Line::from("─".repeat(28)));

                // Equipped item name and rarity
                let eq_display_name = truncate_name(&equipped.name, 26);
                detail_lines.push(Line::from(Span::styled(
                    eq_display_name,
                    Style::default()
                        .fg(Color::Rgb(eq_color.0, eq_color.1, eq_color.2))
                        .add_modifier(Modifier::BOLD),
                )));
                detail_lines.push(Line::from(Span::styled(
                    equipped.rarity.name(),
                    Style::default().fg(Color::Rgb(eq_color.0, eq_color.1, eq_color.2)),
                )));

                // Stats
                if equipped.base_damage > 0 {
                    detail_lines.push(Line::from(vec![
                        Span::styled("  ⚔ Damage: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("{}", equipped.total_damage()), Style::default().fg(Color::Red)),
                    ]));
                }

                if equipped.base_armor > 0 {
                    detail_lines.push(Line::from(vec![
                        Span::styled("  🛡 Armor: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("{}", equipped.total_armor()), Style::default().fg(Color::Blue)),
                    ]));
                }

                // Equipped affixes
                if !equipped.affixes.is_empty() {
                    detail_lines.push(Line::from(Span::styled("  Enchantments:", Style::default().fg(Color::DarkGray))));
                    for affix in &equipped.affixes {
                        detail_lines.push(Line::from(vec![
                            Span::styled(format!("    ✦ +{} ", affix.value), Style::default().fg(Color::Cyan)),
                            Span::styled(affix.affix_type.name(), Style::default().fg(Color::Cyan)),
                        ]));
                    }
                }

                // Hint
                detail_lines.push(Line::from(""));
                detail_lines.push(Line::from(Span::styled(
                    "[E] to equip and replace",
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                )));
            } else {
                // Regular item view (no comparison)
                let display_name = truncate_name(&item.name, 28);
                detail_lines.push(Line::from(Span::styled(
                    display_name,
                    Style::default()
                        .fg(Color::Rgb(rarity_color.0, rarity_color.1, rarity_color.2))
                        .add_modifier(Modifier::BOLD),
                )));
                detail_lines.push(Line::from(Span::styled(
                    item.rarity.name(),
                    Style::default().fg(Color::Rgb(rarity_color.0, rarity_color.1, rarity_color.2)),
                )));
                detail_lines.push(Line::from(""));

                if !item.description.is_empty() {
                    detail_lines.push(Line::from(Span::styled(
                        &item.description,
                        Style::default().fg(Color::Gray),
                    )));
                    detail_lines.push(Line::from(""));
                }

                // Stats
                if item.base_damage > 0 {
                    detail_lines.push(Line::from(vec![
                        Span::styled("Damage: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("{}", item.total_damage()), Style::default().fg(Color::Red)),
                    ]));
                }
                if item.base_armor > 0 {
                    detail_lines.push(Line::from(vec![
                        Span::styled("Armor: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("{}", item.total_armor()), Style::default().fg(Color::Blue)),
                    ]));
                }

                // Affixes with descriptions
                if !item.affixes.is_empty() {
                    detail_lines.push(Line::from(Span::styled("Enchantments:", Style::default().fg(Color::DarkGray))));
                    for affix in &item.affixes {
                        detail_lines.push(Line::from(vec![
                            Span::styled(format!("  +{} ", affix.value), Style::default().fg(Color::Green)),
                            Span::styled(affix.affix_type.name(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                        ]));
                        detail_lines.push(Line::from(Span::styled(
                            format!("    {}", affix.affix_type.description()),
                            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                        )));
                    }
                }

                // Consumable effect
                if let Some(effect) = &item.consumable_effect {
                    use crate::items::ConsumableEffect;
                    let effect_str = match effect {
                        ConsumableEffect::HealHP(n) => format!("Heals {} HP", n),
                        ConsumableEffect::RestoreMP(n) => format!("Restores {} MP", n),
                        ConsumableEffect::RestoreSP(n) => format!("Restores {} SP", n),
                        _ => "Special effect".to_string(),
                    };
                    detail_lines.push(Line::from(""));
                    detail_lines.push(Line::from(Span::styled(
                        effect_str,
                        Style::default().fg(Color::Cyan),
                    )));
                }

                // Show slot for equippable items with no comparison
                if let Some(slot) = item.equip_slot {
                    detail_lines.push(Line::from(""));
                    detail_lines.push(Line::from(vec![
                        Span::styled("Slot: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(slot.name(), Style::default().fg(Color::White)),
                        Span::styled(" (empty)", Style::default().fg(Color::DarkGray)),
                    ]));
                }
            }

            // Value (always show)
            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(vec![
                Span::styled("Value: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} gold", item.value), Style::default().fg(Color::Yellow)),
            ]));

            let detail_para = Paragraph::new(detail_lines);
            frame.render_widget(detail_para, layout[1]);
        }
    }

    fn render_equipment_tab(&self, frame: &mut Frame, game: &Game, player: Option<hecs::Entity>, area: Rect) {
        use crate::ecs::{InventoryComponent, EquipmentComponent};
        use crate::items::ItemCategory;

        let player = match player {
            Some(p) => p,
            None => return,
        };

        let inv = match game.world().get::<&InventoryComponent>(player) {
            Ok(i) => i,
            Err(_) => return,
        };

        // Filter to only equipment items (weapons, armor, accessories)
        let equipment_items: Vec<&crate::items::Item> = inv.inventory.items()
            .into_iter()
            .filter(|item| item.category.is_equipment())
            .collect();

        // Split: item list on left, details on right
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let mut lines: Vec<Line> = Vec::new();

        // Header showing this is equipment from inventory
        lines.push(Line::from(Span::styled(
            "Unequipped Gear",
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(""));

        if equipment_items.is_empty() {
            lines.push(Line::from(Span::styled(
                "  (no equipment found)",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            for (i, item) in equipment_items.iter().enumerate() {
                let is_selected = i == self.inventory_cursor;
                let prefix = if is_selected { "> " } else { "  " };

                let rarity_color = item.rarity.color();
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Rgb(rarity_color.0, rarity_color.1, rarity_color.2))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::Rgb(rarity_color.0, rarity_color.1, rarity_color.2))
                };

                // Show slot type indicator
                let slot_indicator = match item.category {
                    ItemCategory::Weapon => "⚔",
                    ItemCategory::Armor => "🛡",
                    ItemCategory::Accessory => "💍",
                    _ => " ",
                };

                // "NEW" indicator
                let new_str = if item.is_new { " NEW" } else { "" };

                let display_name = truncate_name(&item.name, 16);
                lines.push(Line::from(vec![
                    Span::raw(prefix),
                    Span::styled(format!("{} ", slot_indicator), Style::default().fg(Color::DarkGray)),
                    Span::styled(display_name, style),
                    Span::styled(new_str, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ]));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("{} equipment items", equipment_items.len()),
            Style::default().fg(Color::DarkGray),
        )));

        let list_block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(Color::DarkGray));
        let list_para = Paragraph::new(lines).block(list_block);
        frame.render_widget(list_para, layout[0]);

        // Selected item details (with comparison to equipped item)
        if self.inventory_cursor < equipment_items.len() {
            let item = equipment_items[self.inventory_cursor];
            let rarity_color = item.rarity.color();

            // Get currently equipped item for comparison
            let equipped_item = if let Some(slot) = item.equip_slot {
                game.world()
                    .get::<&EquipmentComponent>(player)
                    .ok()
                    .and_then(|eq| eq.equipment.get(slot).cloned())
            } else {
                None
            };

            let mut detail_lines: Vec<Line> = Vec::new();

            if let Some(equipped) = &equipped_item {
                // Show comparison view
                let eq_color = equipped.rarity.color();

                detail_lines.push(Line::from(vec![
                    Span::styled("NEW: ", Style::default().fg(Color::Green)),
                    Span::styled(
                        truncate_name(&item.name, 12),
                        Style::default().fg(Color::Rgb(rarity_color.0, rarity_color.1, rarity_color.2)),
                    ),
                ]));
                detail_lines.push(Line::from(vec![
                    Span::styled("OLD: ", Style::default().fg(Color::Red)),
                    Span::styled(
                        truncate_name(&equipped.name, 12),
                        Style::default().fg(Color::Rgb(eq_color.0, eq_color.1, eq_color.2)),
                    ),
                ]));
                detail_lines.push(Line::from(""));

                // Compare stats
                if item.base_damage > 0 || equipped.base_damage > 0 {
                    let diff = item.total_damage() - equipped.total_damage();
                    let diff_str = if diff > 0 { format!("+{}", diff) } else { format!("{}", diff) };
                    let diff_color = if diff > 0 { Color::Green } else if diff < 0 { Color::Red } else { Color::DarkGray };
                    detail_lines.push(Line::from(vec![
                        Span::styled("Damage: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("{}", item.total_damage()), Style::default().fg(Color::White)),
                        Span::styled(" vs ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("{}", equipped.total_damage()), Style::default().fg(Color::DarkGray)),
                        Span::styled(format!(" ({})", diff_str), Style::default().fg(diff_color)),
                    ]));
                }

                if item.base_armor > 0 || equipped.base_armor > 0 {
                    let diff = item.total_armor() - equipped.total_armor();
                    let diff_str = if diff > 0 { format!("+{}", diff) } else { format!("{}", diff) };
                    let diff_color = if diff > 0 { Color::Green } else if diff < 0 { Color::Red } else { Color::DarkGray };
                    detail_lines.push(Line::from(vec![
                        Span::styled("Armor: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("{}", item.total_armor()), Style::default().fg(Color::White)),
                        Span::styled(" vs ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("{}", equipped.total_armor()), Style::default().fg(Color::DarkGray)),
                        Span::styled(format!(" ({})", diff_str), Style::default().fg(diff_color)),
                    ]));
                }
            } else {
                // No equipped item - just show this item's stats
                let display_name = truncate_name(&item.name, 24);
                detail_lines.push(Line::from(Span::styled(
                    display_name,
                    Style::default()
                        .fg(Color::Rgb(rarity_color.0, rarity_color.1, rarity_color.2))
                        .add_modifier(Modifier::BOLD),
                )));
                detail_lines.push(Line::from(Span::styled(
                    item.rarity.name(),
                    Style::default().fg(Color::Rgb(rarity_color.0, rarity_color.1, rarity_color.2)),
                )));

                if let Some(slot) = item.equip_slot {
                    detail_lines.push(Line::from(Span::styled(
                        format!("Slot: {} (empty)", slot.name()),
                        Style::default().fg(Color::DarkGray),
                    )));
                }

                detail_lines.push(Line::from(""));

                if item.base_damage > 0 {
                    detail_lines.push(Line::from(vec![
                        Span::styled("Damage: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("{}", item.total_damage()), Style::default().fg(Color::Red)),
                    ]));
                }

                if item.base_armor > 0 {
                    detail_lines.push(Line::from(vec![
                        Span::styled("Armor: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("{}", item.total_armor()), Style::default().fg(Color::Blue)),
                    ]));
                }
            }

            // Show affixes
            detail_lines.push(Line::from(""));
            if !item.affixes.is_empty() {
                detail_lines.push(Line::from(Span::styled("Affixes:", Style::default().fg(Color::Magenta))));
                for affix in &item.affixes {
                    detail_lines.push(Line::from(Span::styled(
                        format!("  +{} {}", affix.value, affix.affix_type.name()),
                        Style::default().fg(Color::Cyan),
                    )));
                }
            }

            let detail_para = Paragraph::new(detail_lines);
            frame.render_widget(detail_para, layout[1]);
        }
    }

    fn render_character_overlay(&self, frame: &mut Frame, game: &Game) {
        use crate::ecs::{EquipmentComponent, Health, Mana, Stamina, Stats, Experience, StatPoints, SkillsComponent};
        use crate::items::{EquipSlot, AffixType};
        use crate::combat::{crit_chance, dodge_chance};
        use crate::progression::SkillCost;

        let area = fullscreen_overlay(frame.area());
        frame.render_widget(Clear, area);

        // Main container
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Double)
            .title(Span::styled(" CHARACTER ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
            .title_alignment(ratatui::layout::Alignment::Center)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let player = match game.player() {
            Some(p) => p,
            None => return,
        };

        // Equipment slot order
        let slot_order = [
            EquipSlot::Head, EquipSlot::MainHand, EquipSlot::Body, EquipSlot::OffHand,
            EquipSlot::Hands, EquipSlot::Feet, EquipSlot::Amulet, EquipSlot::Ring1, EquipSlot::Ring2,
        ];
        let slot_names = ["Head", "Weapon", "Body", "Off Hand", "Hands", "Feet", "Amulet", "Ring 1", "Ring 2"];

        // Get all player data
        let health = game.world().get::<&Health>(player).map(|h| *h).unwrap_or(Health::new(100));
        let mana = game.world().get::<&Mana>(player).map(|m| *m).unwrap_or(Mana::new(50));
        let stamina = game.world().get::<&Stamina>(player).map(|s| *s).unwrap_or(Stamina::new(50));
        let base_stats = game.world().get::<&Stats>(player).map(|s| *s).unwrap_or(Stats::player_base());
        let xp = game.world().get::<&Experience>(player).map(|x| *x).unwrap_or(Experience::new());
        let stat_points = game.world().get::<&StatPoints>(player).map(|s| s.0).unwrap_or(0);
        let skills = game.world().get::<&SkillsComponent>(player).ok();
        let equipment = game.world().get::<&EquipmentComponent>(player).ok();

        // Equipment bonuses
        let (eq_str, eq_dex, eq_int, eq_vit, weapon_dmg, total_armor, crit_bonus, eq_hp, eq_mp) =
            if let Some(eq) = &equipment {
                (eq.equipment.strength_bonus(), eq.equipment.dexterity_bonus(),
                 eq.equipment.intelligence_bonus(), eq.equipment.vitality_bonus(),
                 eq.equipment.weapon_damage(), eq.equipment.total_armor(),
                 eq.equipment.weapon_crit_bonus(), eq.equipment.hp_bonus(), eq.equipment.mp_bonus())
            } else { (0, 0, 0, 0, 2, 0, 0.0, 0, 0) };

        let effective_max_hp = health.max + eq_hp;
        let effective_max_mp = mana.max + eq_mp;
        let eff_str = base_stats.strength + eq_str;
        let eff_dex = base_stats.dexterity + eq_dex;
        let _eff_int = base_stats.intelligence + eq_int;
        let eff_vit = base_stats.vitality + eq_vit;
        let total_crit = crit_chance(eff_dex) + crit_bonus;
        let total_dodge = dodge_chance(eff_dex);
        let phys_damage = 2 + eff_str / 2 + weapon_dmg;
        let effective_armor = eff_vit / 4 + total_armor;
        let damage_reduction = effective_armor as f32 / (effective_armor as f32 + 20.0) * 100.0;

        // Get all combat bonuses from equipment
        let fire_dmg = equipment.as_ref().map(|e| e.equipment.stat_bonus(AffixType::FireDamage)).unwrap_or(0);
        let ice_dmg = equipment.as_ref().map(|e| e.equipment.stat_bonus(AffixType::IceDamage)).unwrap_or(0);
        let lightning_dmg = equipment.as_ref().map(|e| e.equipment.stat_bonus(AffixType::LightningDamage)).unwrap_or(0);
        let poison_dmg = equipment.as_ref().map(|e| e.equipment.stat_bonus(AffixType::PoisonDamage)).unwrap_or(0);
        let lifesteal = equipment.as_ref().map(|e| e.equipment.stat_bonus(AffixType::LifeSteal)).unwrap_or(0);
        let bonus_crit_dmg = equipment.as_ref().map(|e| e.equipment.stat_bonus(AffixType::BonusCritDamage)).unwrap_or(0);
        let fire_res = equipment.as_ref().map(|e| e.equipment.stat_bonus(AffixType::FireResist)).unwrap_or(0);
        let ice_res = equipment.as_ref().map(|e| e.equipment.stat_bonus(AffixType::IceResist)).unwrap_or(0);
        let poison_res = equipment.as_ref().map(|e| e.equipment.stat_bonus(AffixType::PoisonResist)).unwrap_or(0);
        let bonus_xp = equipment.as_ref().map(|e| e.equipment.stat_bonus(AffixType::BonusXP)).unwrap_or(0);
        let gold_find = equipment.as_ref().map(|e| e.equipment.stat_bonus(AffixType::GoldFind)).unwrap_or(0);
        let magic_find = equipment.as_ref().map(|e| e.equipment.stat_bonus(AffixType::MagicFind)).unwrap_or(0);

        // === THREE ROW LAYOUT ===
        // Row 1: Hero stats | Row 2: Combat Stats | Row 3: Equipment/Skills + Details
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),   // Help bar
                Constraint::Length(9),   // Top row (hero stats)
                Constraint::Length(6),   // Combat stats row
                Constraint::Min(10),     // Bottom row (equipment/skills + details)
            ])
            .split(inner);

        // Help bar
        let help_text = Line::from(vec![
            Span::styled("[↑↓]", Style::default().fg(Color::Yellow)),
            Span::styled(" Select ", Style::default().fg(Color::DarkGray)),
            Span::styled("[→]", Style::default().fg(Color::Yellow)),
            Span::styled(" Equip ", Style::default().fg(Color::DarkGray)),
            Span::styled("[U]", Style::default().fg(Color::Yellow)),
            Span::styled(" Unequip ", Style::default().fg(Color::DarkGray)),
            Span::styled("[1-4]", Style::default().fg(Color::Yellow)),
            Span::styled(" +Stats ", Style::default().fg(Color::DarkGray)),
            Span::styled("[C/Esc]", Style::default().fg(Color::Yellow)),
            Span::styled(" Close", Style::default().fg(Color::DarkGray)),
        ]);
        frame.render_widget(Paragraph::new(help_text).alignment(ratatui::layout::Alignment::Center), rows[0]);

        // === TOP ROW: Level | Vitals | Attributes ===
        let top_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(16),  // Level/XP
                Constraint::Length(20),  // Vitals
                Constraint::Min(22),     // Attributes
            ])
            .split(rows[1]);

        // --- LEVEL/XP COLUMN --- (14 chars wide)
        let mut level_lines: Vec<Line> = Vec::new();
        level_lines.push(Line::from(Span::styled("┌─ HERO ─────┐", Style::default().fg(Color::Cyan))));
        level_lines.push(Line::from(vec![
            Span::styled("│", Style::default().fg(Color::Cyan)),
            Span::styled(" ★ ", Style::default().fg(Color::Yellow)),
            Span::styled(format!("Lv.{:<3}", xp.level), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("   │", Style::default().fg(Color::Cyan)),
        ]));
        level_lines.push(Line::from(Span::styled("├────────────┤", Style::default().fg(Color::Cyan))));

        let xp_pct = xp.current_xp as f32 / xp.xp_to_next as f32;
        let xp_filled = (10.0 * xp_pct).round() as usize;
        level_lines.push(Line::from(vec![
            Span::styled("│", Style::default().fg(Color::Cyan)),
            Span::styled("█".repeat(xp_filled), Style::default().fg(Color::Cyan)),
            Span::styled("░".repeat(10 - xp_filled), Style::default().fg(Color::DarkGray)),
            Span::styled(" │", Style::default().fg(Color::Cyan)),
        ]));
        level_lines.push(Line::from(vec![
            Span::styled("│", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{:>5}/{:<5}", xp.current_xp, xp.xp_to_next), Style::default().fg(Color::DarkGray)),
            Span::styled("│", Style::default().fg(Color::Cyan)),
        ]));
        level_lines.push(Line::from(Span::styled("└────────────┘", Style::default().fg(Color::Cyan))));

        if stat_points > 0 {
            level_lines.push(Line::from(vec![
                Span::styled("  ✦ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} points", stat_points), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
        }

        frame.render_widget(Paragraph::new(level_lines), top_cols[0]);

        // --- VITALS COLUMN --- (18 chars wide)
        let mut vitals_lines: Vec<Line> = Vec::new();
        vitals_lines.push(Line::from(Span::styled("┌─ VITALS ───────┐", Style::default().fg(Color::Red))));

        // HP bar
        let hp_pct = health.current as f32 / effective_max_hp as f32;
        let hp_color = if hp_pct > 0.6 { Color::Green } else if hp_pct > 0.3 { Color::Yellow } else { Color::Red };
        let hp_filled = (10.0 * hp_pct).round() as usize;
        vitals_lines.push(Line::from(vec![
            Span::styled("│", Style::default().fg(Color::Red)),
            Span::styled("HP ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("█".repeat(hp_filled), Style::default().fg(hp_color)),
            Span::styled("░".repeat(10 - hp_filled), Style::default().fg(Color::DarkGray)),
            Span::styled("   │", Style::default().fg(Color::Red)),
        ]));
        // HP numbers
        let hp_str = format!("{:>3}/{:<3}", health.current, effective_max_hp);
        let hp_bonus = if eq_hp > 0 { format!("+{:<2}", eq_hp) } else { "   ".to_string() };
        vitals_lines.push(Line::from(vec![
            Span::styled("│   ", Style::default().fg(Color::Red)),
            Span::styled(hp_str, Style::default().fg(Color::White)),
            Span::styled(hp_bonus, Style::default().fg(Color::Green)),
            Span::styled("  │", Style::default().fg(Color::Red)),
        ]));

        // MP bar
        let mp_pct = mana.current as f32 / effective_max_mp as f32;
        let mp_filled = (10.0 * mp_pct).round() as usize;
        vitals_lines.push(Line::from(vec![
            Span::styled("│", Style::default().fg(Color::Red)),
            Span::styled("MP ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            Span::styled("█".repeat(mp_filled), Style::default().fg(Color::Blue)),
            Span::styled("░".repeat(10 - mp_filled), Style::default().fg(Color::DarkGray)),
            Span::styled("   │", Style::default().fg(Color::Red)),
        ]));
        // MP numbers
        let mp_str = format!("{:>3}/{:<3}", mana.current, effective_max_mp);
        let mp_bonus = if eq_mp > 0 { format!("+{:<2}", eq_mp) } else { "   ".to_string() };
        vitals_lines.push(Line::from(vec![
            Span::styled("│   ", Style::default().fg(Color::Red)),
            Span::styled(mp_str, Style::default().fg(Color::White)),
            Span::styled(mp_bonus, Style::default().fg(Color::Green)),
            Span::styled("  │", Style::default().fg(Color::Red)),
        ]));

        // SP bar
        let sp_pct = stamina.current as f32 / stamina.max as f32;
        let sp_filled = (10.0 * sp_pct).round() as usize;
        vitals_lines.push(Line::from(vec![
            Span::styled("│", Style::default().fg(Color::Red)),
            Span::styled("SP ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("█".repeat(sp_filled), Style::default().fg(Color::Yellow)),
            Span::styled("░".repeat(10 - sp_filled), Style::default().fg(Color::DarkGray)),
            Span::styled("   │", Style::default().fg(Color::Red)),
        ]));
        // SP numbers
        vitals_lines.push(Line::from(vec![
            Span::styled("│   ", Style::default().fg(Color::Red)),
            Span::styled(format!("{:>3}/{:<3}", stamina.current, stamina.max), Style::default().fg(Color::White)),
            Span::styled("     │", Style::default().fg(Color::Red)),
        ]));
        vitals_lines.push(Line::from(Span::styled("└────────────────┘", Style::default().fg(Color::Red))));

        frame.render_widget(Paragraph::new(vitals_lines), top_cols[1]);

        // --- ATTRIBUTES COLUMN --- (21 chars wide)
        let mut attr_lines: Vec<Line> = Vec::new();
        let attr_border = if stat_points > 0 { Color::Yellow } else { Color::DarkGray };
        attr_lines.push(Line::from(Span::styled("┌─ ATTRIBUTES ──────┐", Style::default().fg(attr_border))));

        let stat_row = |key: &str, name: &str, base: i32, bonus: i32, color: Color, border: Color, has_pts: bool| -> Line<'static> {
            let key_span = if has_pts {
                Span::styled(format!("[{}]", key), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("   ", Style::default())
            };
            let total = base + bonus;
            let bonus_str = if bonus > 0 {
                format!("(+{:<2})", bonus)
            } else {
                "     ".to_string()
            };
            Line::from(vec![
                Span::styled("│", Style::default().fg(border)),
                key_span,
                Span::styled(format!(" {} ", name), Style::default().fg(Color::Gray)),
                Span::styled(format!("{:>2}", total), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::styled(" ", Style::default()),
                Span::styled(bonus_str, Style::default().fg(Color::Green)),
                Span::styled("  │", Style::default().fg(border)),
            ])
        };

        let has_pts = stat_points > 0;
        attr_lines.push(stat_row("1", "STR", base_stats.strength, eq_str, Color::Red, attr_border, has_pts));
        attr_lines.push(stat_row("2", "DEX", base_stats.dexterity, eq_dex, Color::Green, attr_border, has_pts));
        attr_lines.push(stat_row("3", "INT", base_stats.intelligence, eq_int, Color::Blue, attr_border, has_pts));
        attr_lines.push(stat_row("4", "VIT", base_stats.vitality, eq_vit, Color::Yellow, attr_border, has_pts));
        attr_lines.push(Line::from(Span::styled("└───────────────────┘", Style::default().fg(attr_border))));

        frame.render_widget(Paragraph::new(attr_lines), top_cols[2]);

        // === COMBAT STATS ROW (HORIZONTAL LAYOUT) ===
        let mut combat_lines: Vec<Line> = Vec::new();

        // Row 1: Header with main combat stats
        combat_lines.push(Line::from(vec![
            Span::styled("─── COMBAT ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Phys ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", phys_damage), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Armor ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", total_armor), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" ({:.0}%)", damage_reduction), Style::default().fg(Color::DarkGray)),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Crit ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.1}%", total_crit), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            if bonus_crit_dmg > 0 { Span::styled(format!(" +{}%dmg", bonus_crit_dmg), Style::default().fg(Color::Yellow)) } else { Span::raw("") },
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Dodge ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.1}%", total_dodge), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            if lifesteal > 0 { Span::styled(format!(" │ Steal {}%", lifesteal * 5), Style::default().fg(Color::Magenta)) } else { Span::raw("") },
        ]));

        // Row 2: Elemental damage and resistances
        combat_lines.push(Line::from(vec![
            Span::styled("─── ELEMENT ", Style::default().fg(Color::DarkGray)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("🔥 ", Style::default().fg(Color::Red)),
            Span::styled(format!("{:<2}", fire_dmg), Style::default().fg(if fire_dmg > 0 { Color::Red } else { Color::DarkGray })),
            Span::styled(" ❄ ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{:<2}", ice_dmg), Style::default().fg(if ice_dmg > 0 { Color::Cyan } else { Color::DarkGray })),
            Span::styled(" ⚡ ", Style::default().fg(Color::Yellow)),
            Span::styled(format!("{:<2}", lightning_dmg), Style::default().fg(if lightning_dmg > 0 { Color::Yellow } else { Color::DarkGray })),
            Span::styled(" ☠ ", Style::default().fg(Color::Green)),
            Span::styled(format!("{:<2}", poison_dmg), Style::default().fg(if poison_dmg > 0 { Color::Green } else { Color::DarkGray })),
            Span::styled(" ║ ", Style::default().fg(Color::DarkGray)),
            Span::styled("RESIST ", Style::default().fg(Color::DarkGray)),
            Span::styled("🔥 ", Style::default().fg(Color::Red)),
            Span::styled(format!("{}%", fire_res), Style::default().fg(if fire_res > 0 { Color::Red } else { Color::DarkGray })),
            Span::styled("  ❄ ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{}%", ice_res), Style::default().fg(if ice_res > 0 { Color::Cyan } else { Color::DarkGray })),
            Span::styled("  ☠ ", Style::default().fg(Color::Green)),
            Span::styled(format!("{}%", poison_res), Style::default().fg(if poison_res > 0 { Color::Green } else { Color::DarkGray })),
        ]));

        // Row 3: Bonuses
        combat_lines.push(Line::from(vec![
            Span::styled("─── BONUSES ", Style::default().fg(Color::DarkGray)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("XP ", Style::default().fg(Color::Gray)),
            Span::styled(format!("+{}%", bonus_xp), Style::default().fg(if bonus_xp > 0 { Color::Cyan } else { Color::DarkGray })),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Gold ", Style::default().fg(Color::Gray)),
            Span::styled(format!("+{}%", gold_find), Style::default().fg(if gold_find > 0 { Color::Yellow } else { Color::DarkGray })),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled("MagicFind ", Style::default().fg(Color::Gray)),
            Span::styled(format!("+{}%", magic_find), Style::default().fg(if magic_find > 0 { Color::Magenta } else { Color::DarkGray })),
        ]));

        frame.render_widget(Paragraph::new(combat_lines), rows[2]);

        // === BOTTOM ROW: Equipment+Skills (left) | Item Details (right) ===
        let bottom_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),  // Equipment + Skills
                Constraint::Percentage(60),  // Item Details
            ])
            .split(rows[3]);

        // Split left column into Equipment (top) and Skills (bottom)
        let left_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(12),  // Equipment (9 slots + header/footer)
                Constraint::Min(6),      // Skills
            ])
            .split(bottom_cols[0]);

        // --- EQUIPMENT COLUMN ---
        let mut equip_lines: Vec<Line> = Vec::new();
        equip_lines.push(Line::from(Span::styled("╔═══ EQUIPMENT ═══════════════╗", Style::default().fg(Color::Yellow))));

        const NUM_EQUIP_SLOTS: usize = 9;
        for (i, slot) in slot_order.iter().enumerate() {
            let is_selected = i == self.character_slot && self.character_slot < NUM_EQUIP_SLOTS;
            let item = equipment.as_ref().and_then(|e| e.equipment.get(*slot));

            let prefix = if is_selected { "▶" } else { " " };
            let prefix_style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let line = if let Some(item) = item {
                let color = item.rarity.color();
                let name_style = if is_selected {
                    Style::default().fg(Color::Rgb(color.0, color.1, color.2)).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Rgb(color.0, color.1, color.2))
                };
                let display_name = truncate_name(&item.name, 18);
                Line::from(vec![
                    Span::styled("║ ", Style::default().fg(Color::Yellow)),
                    Span::styled(prefix, prefix_style),
                    Span::styled(format!("{:<9}", slot_names[i]), Style::default().fg(Color::Gray)),
                    Span::styled(display_name, name_style),
                ])
            } else {
                Line::from(vec![
                    Span::styled("║ ", Style::default().fg(Color::Yellow)),
                    Span::styled(prefix, prefix_style),
                    Span::styled(format!("{:<9}", slot_names[i]), Style::default().fg(Color::DarkGray)),
                    Span::styled("- empty -", Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)),
                ])
            };
            equip_lines.push(line);
        }
        equip_lines.push(Line::from(Span::styled("╚══════════════════════════════╝", Style::default().fg(Color::Yellow))));

        frame.render_widget(Paragraph::new(equip_lines), left_rows[0]);

        // --- ITEM DETAILS COLUMN ---
        let mut detail_lines: Vec<Line> = Vec::new();

        // Check if in equip selection mode
        if self.equip_selection_mode && self.character_slot < NUM_EQUIP_SLOTS {
            use crate::ecs::InventoryComponent;
            let current_slot = slot_order[self.character_slot];

            detail_lines.push(Line::from(Span::styled(
                format!("╔═══ SELECT {} ════════════════════╗", slot_names[self.character_slot].to_uppercase()),
                Style::default().fg(Color::Yellow),
            )));

            let matching_items: Vec<(usize, crate::items::Item)> = game.world()
                .get::<&InventoryComponent>(player)
                .map(|inv| inv.inventory.items().into_iter().enumerate()
                    .filter(|(_, item)| item.equip_slot == Some(current_slot))
                    .map(|(i, item)| (i, item.clone())).collect())
                .unwrap_or_default();

            if matching_items.is_empty() {
                detail_lines.push(Line::from(Span::styled("║ No matching items", Style::default().fg(Color::DarkGray))));
            } else {
                for (idx, (_, item)) in matching_items.iter().enumerate() {
                    let is_sel = idx == self.equip_selection_cursor;
                    let prefix = if is_sel { "▶ " } else { "  " };
                    let color = item.rarity.color();
                    let style = if is_sel {
                        Style::default().fg(Color::Rgb(color.0, color.1, color.2)).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Rgb(color.0, color.1, color.2))
                    };
                    detail_lines.push(Line::from(vec![
                        Span::styled("║ ", Style::default().fg(Color::Yellow)),
                        Span::styled(prefix, if is_sel { Style::default().fg(Color::Yellow) } else { Style::default() }),
                        Span::styled(truncate_name(&item.name, 28), style),
                    ]));
                }
            }
            detail_lines.push(Line::from(Span::styled("║ [Enter] Equip  [Esc] Cancel", Style::default().fg(Color::DarkGray))));
            detail_lines.push(Line::from(Span::styled("╚═══════════════════════════════════════╝", Style::default().fg(Color::Yellow))));
        } else if self.character_slot < NUM_EQUIP_SLOTS {
            // Show selected equipment item details
            let selected_slot = slot_order.get(self.character_slot);
            let selected_item = selected_slot.and_then(|slot| equipment.as_ref().and_then(|e| e.equipment.get(*slot)));

            detail_lines.push(Line::from(Span::styled("╔═══ ITEM DETAILS ══════════════════════╗", Style::default().fg(Color::Yellow))));

            if let Some(item) = selected_item {
                let color = item.rarity.color();
                detail_lines.push(Line::from(vec![
                    Span::styled("║ ", Style::default().fg(Color::Yellow)),
                    Span::styled(&item.name, Style::default().fg(Color::Rgb(color.0, color.1, color.2)).add_modifier(Modifier::BOLD)),
                ]));
                detail_lines.push(Line::from(vec![
                    Span::styled("║ ", Style::default().fg(Color::Yellow)),
                    Span::styled(item.rarity.name(), Style::default().fg(Color::Rgb(color.0, color.1, color.2))),
                ]));

                if item.base_damage > 0 {
                    detail_lines.push(Line::from(vec![
                        Span::styled("║ ", Style::default().fg(Color::Yellow)),
                        Span::styled("Damage: ", Style::default().fg(Color::Gray)),
                        Span::styled(format!("{}", item.total_damage()), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    ]));
                }
                if item.base_armor > 0 {
                    detail_lines.push(Line::from(vec![
                        Span::styled("║ ", Style::default().fg(Color::Yellow)),
                        Span::styled("Armor: ", Style::default().fg(Color::Gray)),
                        Span::styled(format!("{}", item.total_armor()), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                    ]));
                }

                // Affixes
                if !item.affixes.is_empty() {
                    detail_lines.push(Line::from(Span::styled("╟─ Enchantments ─────────────────────────", Style::default().fg(Color::DarkGray))));
                    for affix in &item.affixes {
                        let affix_color = if affix.affix_type.is_mythic_only() { Color::Cyan } else { Color::Green };
                        detail_lines.push(Line::from(vec![
                            Span::styled("║  ", Style::default().fg(Color::Yellow)),
                            Span::styled(format!("✦ +{} {}", affix.value, affix.affix_type.name()), Style::default().fg(affix_color)),
                        ]));
                    }
                }

                // Enhancements
                if item.enchantment_level > 0 || item.awakening_level > 0 || item.corruption_level > 0 {
                    detail_lines.push(Line::from(Span::styled("╟─ Enhancements ──────────────────────────", Style::default().fg(Color::DarkGray))));
                    if item.enchantment_level > 0 {
                        detail_lines.push(Line::from(vec![
                            Span::styled("║  ", Style::default().fg(Color::Yellow)),
                            Span::styled(format!("⚡ +{} Enchant (+{}%)", item.enchantment_level, item.enchantment_level * 10), Style::default().fg(Color::Cyan)),
                        ]));
                    }
                    if item.awakening_level > 0 {
                        detail_lines.push(Line::from(vec![
                            Span::styled("║  ", Style::default().fg(Color::Yellow)),
                            Span::styled(format!("✨ Awakened Lv.{}", item.awakening_level), Style::default().fg(Color::Yellow)),
                        ]));
                    }
                    if item.corruption_level > 0 {
                        detail_lines.push(Line::from(vec![
                            Span::styled("║  ", Style::default().fg(Color::Yellow)),
                            Span::styled(format!("💀 Corrupted Lv.{}", item.corruption_level), Style::default().fg(Color::Red)),
                        ]));
                    }
                }

                // Sockets
                if !item.sockets.is_empty() {
                    detail_lines.push(Line::from(Span::styled("╟─ Sockets ───────────────────────────────", Style::default().fg(Color::DarkGray))));
                    for (i, socket) in item.sockets.iter().enumerate() {
                        if let Some(gem) = socket {
                            let gc = gem.gem_type.color();
                            detail_lines.push(Line::from(vec![
                                Span::styled("║  ", Style::default().fg(Color::Yellow)),
                                Span::styled(format!("[{}] {} T{}", i+1, gem.gem_type.name(), gem.tier), Style::default().fg(Color::Rgb(gc.0, gc.1, gc.2))),
                            ]));
                        } else {
                            detail_lines.push(Line::from(vec![
                                Span::styled("║  ", Style::default().fg(Color::Yellow)),
                                Span::styled(format!("[{}] empty", i+1), Style::default().fg(Color::DarkGray)),
                            ]));
                        }
                    }
                }

                detail_lines.push(Line::from(vec![
                    Span::styled("║ ", Style::default().fg(Color::Yellow)),
                    Span::styled(format!("Value: {} gold", item.value), Style::default().fg(Color::Yellow)),
                ]));
            } else {
                detail_lines.push(Line::from(vec![
                    Span::styled("║ ", Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{} - empty", slot_names[self.character_slot]), Style::default().fg(Color::DarkGray)),
                ]));
                detail_lines.push(Line::from(vec![
                    Span::styled("║ ", Style::default().fg(Color::Yellow)),
                    Span::styled("Press [→] to equip", Style::default().fg(Color::DarkGray)),
                ]));
            }
            detail_lines.push(Line::from(Span::styled("╚═══════════════════════════════════════╝", Style::default().fg(Color::Yellow))));
        } else {
            // Skill selected - show skill details
            let skill_idx = self.character_slot - NUM_EQUIP_SLOTS;
            let selected_skill = skills.as_ref().and_then(|sk| sk.skills.slots[skill_idx].as_ref());

            detail_lines.push(Line::from(Span::styled("╔═══ SKILL DETAILS ═════════════════════╗", Style::default().fg(Color::Magenta))));

            if let Some(skill) = selected_skill {
                let color = skill.rarity.color();
                detail_lines.push(Line::from(vec![
                    Span::styled("║ ", Style::default().fg(Color::Magenta)),
                    Span::styled(format!("{} ", skill.icon), Style::default().fg(Color::Rgb(color.0, color.1, color.2))),
                    Span::styled(&skill.name, Style::default().fg(Color::Rgb(color.0, color.1, color.2)).add_modifier(Modifier::BOLD)),
                ]));
                detail_lines.push(Line::from(vec![
                    Span::styled("║ ", Style::default().fg(Color::Magenta)),
                    Span::styled(skill.rarity.name(), Style::default().fg(Color::Rgb(color.0, color.1, color.2))),
                ]));
                detail_lines.push(Line::from(vec![
                    Span::styled("║ ", Style::default().fg(Color::Magenta)),
                    Span::styled(&skill.description, Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
                ]));

                let cost_str = match skill.cost {
                    SkillCost::Mana(n) => format!("{} Mana", n),
                    SkillCost::Stamina(n) => format!("{} Stamina", n),
                    SkillCost::Cooldown => "No cost".to_string(),
                    SkillCost::Charge(n) => format!("{} Charges", n),
                };
                detail_lines.push(Line::from(vec![
                    Span::styled("║ ", Style::default().fg(Color::Magenta)),
                    Span::styled("Cost: ", Style::default().fg(Color::Gray)),
                    Span::styled(cost_str, Style::default().fg(Color::Cyan)),
                ]));
                if skill.cooldown_turns > 0 {
                    detail_lines.push(Line::from(vec![
                        Span::styled("║ ", Style::default().fg(Color::Magenta)),
                        Span::styled(format!("Cooldown: {} turns", skill.cooldown_turns), Style::default().fg(Color::Red)),
                    ]));
                }
            } else {
                detail_lines.push(Line::from(vec![
                    Span::styled("║ ", Style::default().fg(Color::Magenta)),
                    Span::styled(format!("Skill Slot {} - empty", skill_idx + 1), Style::default().fg(Color::DarkGray)),
                ]));
                detail_lines.push(Line::from(vec![
                    Span::styled("║ ", Style::default().fg(Color::Magenta)),
                    Span::styled("Visit Skill Shrines to learn", Style::default().fg(Color::DarkGray)),
                ]));
            }
            detail_lines.push(Line::from(Span::styled("╚═══════════════════════════════════════╝", Style::default().fg(Color::Magenta))));
        }

        frame.render_widget(Paragraph::new(detail_lines), bottom_cols[1]);

        // --- SKILLS COLUMN ---
        let mut skill_lines: Vec<Line> = Vec::new();
        skill_lines.push(Line::from(Span::styled("╔═══ SKILLS ═══════════════╗", Style::default().fg(Color::Magenta))));

        if let Some(sk) = &skills {
            for i in 0..5 {
                let is_selected = self.character_slot == NUM_EQUIP_SLOTS + i;
                let prefix = if is_selected { "▶" } else { " " };
                let prefix_style = if is_selected {
                    Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Magenta)
                };

                if let Some(skill) = &sk.skills.slots[i] {
                    let cd = sk.skills.cooldowns[i];
                    let skill_style = if cd > 0 {
                        Style::default().fg(Color::DarkGray)
                    } else if is_selected {
                        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let cd_text = if cd > 0 { format!("[{}]", cd) } else { String::new() };
                    let cost_str = match skill.cost {
                        SkillCost::Mana(n) => format!("{}mp", n),
                        SkillCost::Stamina(n) => format!("{}sp", n),
                        SkillCost::Cooldown => "cd".to_string(),
                        SkillCost::Charge(n) => format!("{}ch", n),
                    };

                    skill_lines.push(Line::from(vec![
                        Span::styled("║ ", Style::default().fg(Color::Magenta)),
                        Span::styled(prefix, prefix_style),
                        Span::styled(format!("[{}] ", i+1), Style::default().fg(Color::Yellow)),
                        Span::styled(format!("{} ", skill.icon), skill_style),
                        Span::styled(truncate_name(&skill.name, 10), skill_style),
                        Span::styled(cd_text, Style::default().fg(Color::Red)),
                        Span::styled(format!(" {}", cost_str), Style::default().fg(Color::DarkGray)),
                    ]));
                } else {
                    skill_lines.push(Line::from(vec![
                        Span::styled("║ ", Style::default().fg(Color::Magenta)),
                        Span::styled(prefix, prefix_style),
                        Span::styled(format!("[{}] ", i+1), Style::default().fg(Color::DarkGray)),
                        Span::styled("- empty -", Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)),
                    ]));
                }
            }

            let total = sk.skills.learned.len();
            let equipped = sk.skills.slots.iter().filter(|s| s.is_some()).count();
            skill_lines.push(Line::from(Span::styled("╟───────────────────────────╢", Style::default().fg(Color::Magenta))));
            skill_lines.push(Line::from(vec![
                Span::styled("║ ", Style::default().fg(Color::Magenta)),
                Span::styled(format!("{} learned, {} free", total, total.saturating_sub(equipped)), Style::default().fg(Color::DarkGray)),
            ]));
        } else {
            for i in 1..=5 {
                skill_lines.push(Line::from(vec![
                    Span::styled("║  ", Style::default().fg(Color::Magenta)),
                    Span::styled(format!("[{}] - empty -", i), Style::default().fg(Color::DarkGray)),
                ]));
            }
        }
        skill_lines.push(Line::from(Span::styled("╚═══════════════════════════╝", Style::default().fg(Color::Magenta))));

        frame.render_widget(Paragraph::new(skill_lines), left_rows[1]);
    }

    fn render_fullmap_overlay(&self, frame: &mut Frame, game: &Game) {
        // Use near-fullscreen overlay for the map
        let area = fullscreen_overlay(frame.area());
        frame.render_widget(Clear, area);

        let Some(map) = game.map() else {
            // No map available
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Map ")
                .border_style(Style::default().fg(Color::Green));
            frame.render_widget(block, area);
            return;
        };
        let floor = game.floor();

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Map - Floor {} ", floor))
            .border_style(Style::default().fg(Color::Green));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Get player position for highlighting
        let player_pos = game.player_position();

        // Get enemy positions from world
        let mut enemy_positions: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
        for (_entity, (pos, _enemy)) in game.world().query::<(&crate::ecs::Position, &crate::ecs::Enemy)>().iter() {
            enemy_positions.insert((pos.x, pos.y));
        }

        // Get item positions from world
        let mut item_positions: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
        for (_entity, (pos, _item)) in game.world().query::<(&crate::ecs::Position, &crate::items::Item)>().iter() {
            item_positions.insert((pos.x, pos.y));
        }

        // Get chest positions
        let mut chest_positions: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
        for (_entity, (pos, _chest)) in game.world().query::<(&crate::ecs::Position, &crate::ecs::Chest)>().iter() {
            chest_positions.insert((pos.x, pos.y));
        }

        // Calculate scale - we want to fit the 80x50 map into the available area
        let map_width = map.width as u16;
        let map_height = map.height as u16;
        let available_width = inner.width;
        let available_height = inner.height.saturating_sub(4); // Leave room for legend

        // Each map tile gets 1 character, but we may need to scale
        let scale_x = if map_width > available_width { 2 } else { 1 };
        let scale_y = if map_height > available_height { 2 } else { 1 };

        // Build map lines
        let mut map_lines: Vec<Line> = Vec::new();

        for y in 0..map_height {
            // Skip line if scaling Y
            if scale_y == 2 && y % 2 == 1 { continue; }

            let mut spans: Vec<Span> = Vec::new();
            for x in 0..map_width {
                // Skip every other tile if we need to scale down
                if scale_x == 2 && x % 2 == 1 { continue; }

                let tile = map.get_tile(x as i32, y as i32);
                let is_player = player_pos.map_or(false, |p| p.x == x as i32 && p.y == y as i32);
                let is_enemy = enemy_positions.contains(&(x as i32, y as i32));
                let is_item = item_positions.contains(&(x as i32, y as i32));
                let is_chest = chest_positions.contains(&(x as i32, y as i32));
                let is_exit = map.exit_pos == Some(crate::ecs::Position::new(x as i32, y as i32));
                let is_start = map.start_pos.x == x as i32 && map.start_pos.y == y as i32;

                let (ch, style) = if let Some(tile) = tile {
                    if !tile.explored {
                        // Unexplored - dark
                        (' ', Style::default().bg(Color::Rgb(20, 20, 20)))
                    } else if is_player {
                        // Player - bright white on blue
                        ('@', Style::default().fg(Color::White).bg(Color::Blue).add_modifier(Modifier::BOLD))
                    } else if is_enemy && tile.visible {
                        // Enemy - red
                        ('!', Style::default().fg(Color::Red).bg(Color::Rgb(40, 20, 20)))
                    } else if is_chest {
                        // Chest - yellow
                        ('$', Style::default().fg(Color::Yellow))
                    } else if is_item && tile.visible {
                        // Item - cyan
                        ('*', Style::default().fg(Color::Cyan))
                    } else if is_exit {
                        // Exit - green
                        ('>', Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                    } else if is_start {
                        // Start - light blue
                        ('<', Style::default().fg(Color::LightBlue))
                    } else {
                        // Normal tile based on type
                        use crate::world::TileType;
                        match tile.tile_type {
                            TileType::Wall => ('#', Style::default().fg(Color::Rgb(80, 80, 100))),
                            TileType::Floor => ('.', Style::default().fg(Color::Rgb(60, 60, 60))),
                            TileType::Corridor => ('.', Style::default().fg(Color::Rgb(50, 50, 50))),
                            TileType::Lava => ('~', Style::default().fg(Color::Rgb(255, 100, 0))),
                            TileType::Pit => (' ', Style::default().bg(Color::Rgb(10, 10, 10))),
                            TileType::DoorClosed => ('+', Style::default().fg(Color::Rgb(139, 90, 43))),
                            TileType::DoorOpen => ('/', Style::default().fg(Color::Rgb(139, 90, 43))),
                            TileType::StairsDown => ('>', Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                            TileType::StairsUp => ('<', Style::default().fg(Color::LightBlue)),
                            TileType::Torch => ('≈', Style::default().fg(Color::Yellow)),
                            TileType::Brazier => ('Ω', Style::default().fg(Color::Rgb(255, 150, 50))),
                            TileType::ShrineRest => ('♥', Style::default().fg(Color::LightRed)),
                            TileType::ShrineSkill => ('★', Style::default().fg(Color::Magenta)),
                            TileType::ShrineEnchant => ('◆', Style::default().fg(Color::Cyan)),
                            TileType::ShrineCorruption => ('✧', Style::default().fg(Color::Rgb(128, 0, 128))),
                            TileType::Bones => (',', Style::default().fg(Color::Rgb(200, 200, 180))),
                            TileType::BloodStain => (',', Style::default().fg(Color::Rgb(100, 30, 30))),
                            TileType::Rubble => (';', Style::default().fg(Color::Rgb(100, 100, 100))),
                            TileType::Cobweb => (':', Style::default().fg(Color::Rgb(180, 180, 180))),
                            TileType::Cracks => ('_', Style::default().fg(Color::Rgb(80, 80, 80))),
                            TileType::Moss => ('"', Style::default().fg(Color::Rgb(50, 100, 50))),
                            TileType::Ashes => ('`', Style::default().fg(Color::Rgb(100, 100, 100))),
                            TileType::Grime => ('~', Style::default().fg(Color::Rgb(60, 70, 50))),
                        }
                    }
                } else {
                    (' ', Style::default())
                };

                spans.push(Span::styled(ch.to_string(), style));
            }
            map_lines.push(Line::from(spans));
        }

        // Add legend at bottom
        map_lines.push(Line::from(""));
        map_lines.push(Line::from(vec![
            Span::styled("@ ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("You  ", Style::default().fg(Color::Gray)),
            Span::styled("> ", Style::default().fg(Color::Green)),
            Span::styled("Exit  ", Style::default().fg(Color::Gray)),
            Span::styled("! ", Style::default().fg(Color::Red)),
            Span::styled("Enemy  ", Style::default().fg(Color::Gray)),
            Span::styled("$ ", Style::default().fg(Color::Yellow)),
            Span::styled("Chest  ", Style::default().fg(Color::Gray)),
            Span::styled("* ", Style::default().fg(Color::Cyan)),
            Span::styled("Item  ", Style::default().fg(Color::Gray)),
            Span::styled("♥★◆ ", Style::default().fg(Color::Magenta)),
            Span::styled("Shrines", Style::default().fg(Color::Gray)),
        ]));
        map_lines.push(Line::from(Span::styled(
            "Press [M] or [Esc] to close",
            Style::default().fg(Color::DarkGray),
        )));

        let map_para = Paragraph::new(map_lines);
        frame.render_widget(map_para, inner);
    }

    fn render_help_overlay(&self, frame: &mut Frame) {
        let area = centered_rect(75, 85, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" ? Help - Hollowdeep ? ")
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines: Vec<Line> = Vec::new();

        // Controls section
        lines.push(Line::from(Span::styled(
            "═══ CONTROLS ═══",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  Arrow Keys / WASD ", Style::default().fg(Color::White)),
            Span::styled("Move", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Space / .         ", Style::default().fg(Color::White)),
            Span::styled("Wait one turn", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  E                 ", Style::default().fg(Color::White)),
            Span::styled("Interact (shrines, stairs, NPCs)", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  I                 ", Style::default().fg(Color::White)),
            Span::styled("Inventory", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  C                 ", Style::default().fg(Color::White)),
            Span::styled("Character sheet", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  1-5               ", Style::default().fg(Color::White)),
            Span::styled("Use skills", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  G                 ", Style::default().fg(Color::White)),
            Span::styled("Pick up item", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  R                 ", Style::default().fg(Color::White)),
            Span::styled("Cycle render mode (ASCII/Unicode/Nerd)", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Esc               ", Style::default().fg(Color::White)),
            Span::styled("Pause / Close menu", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(""));

        // Tiles section
        lines.push(Line::from(Span::styled(
            "═══ MAP SYMBOLS ═══",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  @  ", Style::default().fg(Color::Rgb(255, 255, 100))),
            Span::styled("You (the player)", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  .  ", Style::default().fg(Color::Rgb(80, 80, 80))),
            Span::styled("Floor / Corridor", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  #  ", Style::default().fg(Color::Rgb(130, 110, 90))),
            Span::styled("Wall", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  >  ", Style::default().fg(Color::Rgb(200, 200, 200))),
            Span::styled("Stairs down (descend with E)", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  !  ", Style::default().fg(Color::Rgb(255, 200, 50))),
            Span::styled("Torch (light source)", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  %  ", Style::default().fg(Color::Rgb(200, 200, 180))),
            Span::styled("Bones (decoration)", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ✧  ", Style::default().fg(Color::Rgb(255, 200, 100))),
            Span::styled("Elite zone marker (dangerous!)", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(""));

        // Shrines
        lines.push(Line::from(Span::styled(
            "═══ SHRINES ═══",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  ⚝  ", Style::default().fg(Color::Rgb(200, 100, 255))),
            Span::styled("Skill Shrine - Learn new abilities", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ✦  ", Style::default().fg(Color::Rgb(100, 200, 255))),
            Span::styled("Enchant Shrine - Upgrade items", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ☥  ", Style::default().fg(Color::Rgb(100, 255, 100))),
            Span::styled("Rest Shrine - Full heal & restore", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ☠  ", Style::default().fg(Color::Rgb(200, 50, 100))),
            Span::styled("Corruption Shrine - Curse for power", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(""));

        // Entities
        lines.push(Line::from(Span::styled(
            "═══ ENTITIES ═══",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  s z g r ", Style::default().fg(Color::Red)),
            Span::styled("Enemies (bump to attack)", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  $  ", Style::default().fg(Color::Rgb(255, 215, 0))),
            Span::styled("Merchant (bump to trade)", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  &  ", Style::default().fg(Color::Rgb(180, 100, 60))),
            Span::styled("Blacksmith", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  +  ", Style::default().fg(Color::Rgb(100, 255, 100))),
            Span::styled("Healer", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(""));

        // Stats
        lines.push(Line::from(Span::styled(
            "═══ STATS ═══",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  STR ", Style::default().fg(Color::Rgb(255, 100, 100))),
            Span::styled("Physical damage, carry weight", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  DEX ", Style::default().fg(Color::Rgb(100, 255, 100))),
            Span::styled("Attack speed, dodge, crit chance", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  INT ", Style::default().fg(Color::Rgb(100, 100, 255))),
            Span::styled("Magic damage, mana pool", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  VIT ", Style::default().fg(Color::Rgb(255, 200, 100))),
            Span::styled("Max HP, HP regen, poison resist", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(""));

        // Item Rarity
        lines.push(Line::from(Span::styled(
            "═══ ITEM RARITY ═══",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  Common    ", Style::default().fg(Color::Rgb(200, 200, 200))),
            Span::styled("Basic stats, 0-1 enchantments", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Uncommon  ", Style::default().fg(Color::Rgb(100, 255, 100))),
            Span::styled("+1-2 base, 1-2 enchantments", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Rare      ", Style::default().fg(Color::Rgb(100, 150, 255))),
            Span::styled("+2-4 base, 2-3 enchantments", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Epic      ", Style::default().fg(Color::Rgb(200, 100, 255))),
            Span::styled("+4-6 base, 3-4 enchantments", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Legendary ", Style::default().fg(Color::Rgb(255, 180, 50))),
            Span::styled("+6-10 base, 3-5 enchantments", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(""));

        // Enchantments/Affixes
        lines.push(Line::from(Span::styled(
            "═══ ENCHANTMENTS ═══",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  Sharp     ", Style::default().fg(Color::White)),
            Span::styled("+Physical damage", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Fortified ", Style::default().fg(Color::White)),
            Span::styled("+Armor rating", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Flaming   ", Style::default().fg(Color::Rgb(255, 100, 50))),
            Span::styled("+Fire damage", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Frozen    ", Style::default().fg(Color::Rgb(100, 200, 255))),
            Span::styled("+Ice damage, slows enemies", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Shocking  ", Style::default().fg(Color::Rgb(255, 255, 100))),
            Span::styled("+Lightning damage, may chain", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Venomous  ", Style::default().fg(Color::Rgb(100, 255, 100))),
            Span::styled("+Poison damage over time", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Vampiric  ", Style::default().fg(Color::Rgb(200, 50, 100))),
            Span::styled("Heal on hit (life steal)", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Precise   ", Style::default().fg(Color::White)),
            Span::styled("+Critical hit chance", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Deadly    ", Style::default().fg(Color::Rgb(255, 50, 50))),
            Span::styled("+Critical hit damage", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  of Might/Agility/Wisdom/Vitality ", Style::default().fg(Color::Cyan)),
            Span::styled("+STR/DEX/INT/VIT", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(""));

        // Tips
        lines.push(Line::from(Span::styled(
            "═══ TIPS ═══",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  • Elite zones (✧) have stronger enemies but better XP",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "  • Collect item sets for powerful synergy bonuses",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "  • Rest shrines fully restore HP, MP, and skill charges",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "  • Boss floors (5, 10, 15, 20) have powerful guardians",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));

        lines.push(Line::from(Span::styled(
            "[↑↓/PgUp/PgDn] Scroll  [?/Esc] Close",
            Style::default().fg(Color::Cyan),
        )));

        let text = Paragraph::new(lines)
            .scroll((self.help_scroll, 0));
        frame.render_widget(text, inner);
    }

    fn render_shrine_overlay(&self, frame: &mut Frame, game: &Game, shrine_type: ShrineType) {
        use crate::ecs::SkillsComponent;
        use crate::progression::SkillCost;

        let (title, color) = match shrine_type {
            ShrineType::Skill => (" ⚝ Skill Shrine ⚝ ", Color::Rgb(200, 100, 255)),
            ShrineType::Enchanting => (" ✦ Enchanting Shrine ✦ ", Color::Rgb(100, 200, 255)),
            ShrineType::Rest => (" ☥ Rest Shrine ☥ ", Color::Rgb(100, 255, 100)),
            ShrineType::Corruption => (" ⛧ Corruption Shrine ⛧ ", Color::Rgb(200, 50, 50)),
        };

        let area = centered_rect(60, 60, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(color));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines: Vec<Line> = Vec::new();

        match shrine_type {
            ShrineType::Skill => {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "The shrine pulses with ancient power.",
                    Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
                )));

                if self.shrine_skill_swap_mode {
                    // SWAP MODE: Show equipped skills to replace
                    lines.push(Line::from(Span::styled(
                        "All skill slots are full! Choose a skill to replace:",
                        Style::default().fg(Color::Yellow),
                    )));
                    lines.push(Line::from(""));

                    // Show the skill being learned
                    if let Some(ref new_skill) = self.shrine_pending_skill {
                        let rarity_color = new_skill.rarity.color();
                        lines.push(Line::from(vec![
                            Span::styled("Learning: ", Style::default().fg(Color::White)),
                            Span::styled(format!("{} ", new_skill.icon), Style::default().fg(Color::Magenta)),
                            Span::styled(new_skill.name.clone(), Style::default().fg(Color::Rgb(rarity_color.0, rarity_color.1, rarity_color.2)).add_modifier(Modifier::BOLD)),
                            Span::styled(format!(" [{}]", new_skill.rarity.name()), Style::default().fg(Color::Rgb(rarity_color.0, rarity_color.1, rarity_color.2))),
                        ]));
                        lines.push(Line::from(""));
                    }

                    lines.push(Line::from(Span::styled(
                        "Current Skills:",
                        Style::default().fg(Color::White),
                    )));
                    lines.push(Line::from(""));

                    // Get equipped skills
                    let equipped_skills: Vec<Option<crate::progression::Skill>> = game.player().map(|player| {
                        game.world().get::<&SkillsComponent>(player)
                            .map(|skills| skills.skills.slots.clone().into_iter().collect())
                            .unwrap_or_else(|_| vec![None; 5])
                    }).unwrap_or_else(|| vec![None; 5]);

                    for (i, skill_opt) in equipped_skills.iter().enumerate() {
                        let is_selected = i == self.shrine_skill_swap_cursor;
                        let prefix = if is_selected { "> " } else { "  " };

                        let select_style = if is_selected {
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Gray)
                        };

                        if let Some(skill) = skill_opt {
                            let rarity_color = skill.rarity.color();
                            let rarity_style = Style::default().fg(Color::Rgb(rarity_color.0, rarity_color.1, rarity_color.2));
                            let name_style = if is_selected {
                                rarity_style.add_modifier(Modifier::BOLD)
                            } else {
                                rarity_style
                            };

                            lines.push(Line::from(vec![
                                Span::styled(prefix, select_style),
                                Span::styled(format!("Slot {}: ", i + 1), Style::default().fg(Color::DarkGray)),
                                Span::styled(format!("{} ", skill.icon), Style::default().fg(Color::Magenta)),
                                Span::styled(skill.name.clone(), name_style),
                                Span::styled(format!(" [{}]", skill.rarity.name()), rarity_style),
                            ]));
                        } else {
                            lines.push(Line::from(vec![
                                Span::styled(prefix, select_style),
                                Span::styled(format!("Slot {}: ", i + 1), Style::default().fg(Color::DarkGray)),
                                Span::styled("(empty)", Style::default().fg(Color::DarkGray)),
                            ]));
                        }
                    }

                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        "[↑↓] Select   [Enter] Replace   [Esc] Cancel",
                        Style::default().fg(Color::DarkGray),
                    )));
                } else {
                    // NORMAL MODE: Choose skill to learn
                    lines.push(Line::from(Span::styled(
                        "Choose a skill to learn:",
                        Style::default().fg(Color::White),
                    )));
                    lines.push(Line::from(""));

                    // Get current skills to check which are already learned
                    let known_skills: Vec<u32> = game.player().map(|player| {
                        game.world().get::<&SkillsComponent>(player)
                            .map(|skills| {
                                skills.skills.slots.iter()
                                    .filter_map(|s| s.as_ref().map(|sk| sk.id))
                                    .collect()
                            })
                            .unwrap_or_default()
                    }).unwrap_or_default();

                    // Use generated shrine skills
                    for (i, skill) in self.shrine_skills.iter().enumerate() {
                        let is_selected = i == self.shrine_skill_cursor;
                        let already_known = known_skills.contains(&skill.id);
                        let prefix = if is_selected { "> " } else { "  " };

                        // Get rarity color
                        let rarity_color = skill.rarity.color();
                        let rarity_style = Style::default().fg(Color::Rgb(rarity_color.0, rarity_color.1, rarity_color.2));

                        // Format cost
                        let cost_str = match skill.cost {
                            SkillCost::Mana(n) => format!("{} MP", n),
                            SkillCost::Stamina(n) => format!("{} SP", n),
                            SkillCost::Cooldown => format!("{} turn CD", skill.cooldown_turns),
                            SkillCost::Charge(n) => format!("{} charges", n),
                        };

                        // Cooldown info
                        let cd_str = if skill.cooldown_turns > 0 {
                            format!(" ({}t CD)", skill.cooldown_turns)
                        } else {
                            String::new()
                        };

                        if already_known {
                            lines.push(Line::from(vec![
                                Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                                Span::styled(skill.name.clone(), Style::default().fg(Color::DarkGray)),
                                Span::styled(" (known)", Style::default().fg(Color::DarkGray)),
                            ]));
                        } else {
                            let select_style = if is_selected {
                                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::Gray)
                            };

                            let name_style = if is_selected {
                                rarity_style.add_modifier(Modifier::BOLD)
                            } else {
                                rarity_style
                            };

                            lines.push(Line::from(vec![
                                Span::styled(prefix, select_style),
                                Span::styled(format!("{} ", skill.icon), Style::default().fg(Color::Magenta)),
                                Span::styled(skill.name.clone(), name_style),
                                Span::styled(format!(" [{}]", skill.rarity.name()), rarity_style),
                            ]));

                            // Cost and cooldown
                            lines.push(Line::from(vec![
                                Span::styled("    ", Style::default()),
                                Span::styled(cost_str, Style::default().fg(Color::Cyan)),
                                Span::styled(cd_str, Style::default().fg(Color::DarkGray)),
                            ]));
                        }

                        // Description on next line
                        lines.push(Line::from(vec![
                            Span::styled("    ", Style::default()),
                            Span::styled(skill.description.clone(), Style::default().fg(Color::Gray)),
                        ]));
                        lines.push(Line::from(""));
                    }

                    if self.shrine_skills.is_empty() {
                        lines.push(Line::from(Span::styled(
                            "  (No skills available)",
                            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                        )));
                    }

                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        "[↑↓] Select   [Enter] Learn   [Esc] Leave",
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
            ShrineType::Enchanting => {
                use crate::ecs::{InventoryComponent, EquipmentComponent};
                use crate::items::item::AffixType;

                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "The runes glow with mystical energy.",
                    Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
                )));
                lines.push(Line::from(""));

                // Get player gold
                let gold = game.player()
                    .and_then(|p| game.world().get::<&InventoryComponent>(p).ok())
                    .map(|inv| inv.inventory.gold())
                    .unwrap_or(0);

                // Show gold
                lines.push(Line::from(vec![
                    Span::styled("Gold: ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{}", gold), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(""));

                // Phase 1: Equipment selection
                if self.enchant_selected_slot.is_none() {
                    let items = self.get_equipped_items_for_enchant(game);

                    if items.is_empty() {
                        lines.push(Line::from(Span::styled(
                            "No equipment to enchant!",
                            Style::default().fg(Color::Red),
                        )));
                        lines.push(Line::from(""));
                        lines.push(Line::from(Span::styled(
                            "Equip items first (press 'c' for character sheet).",
                            Style::default().fg(Color::Gray),
                        )));
                    } else {
                        lines.push(Line::from(Span::styled(
                            "Select Equipment to Enchant (↑↓ to select):",
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                        )));
                        lines.push(Line::from(""));

                        for (i, (slot, name, current, max)) in items.iter().enumerate() {
                            let is_selected = i == self.enchant_equipment_cursor;
                            let prefix = if is_selected { "► " } else { "  " };
                            let slots_color = if *current >= *max { Color::Red } else { Color::Green };

                            let slot_name = match slot {
                                crate::items::EquipSlot::MainHand => "Weapon",
                                crate::items::EquipSlot::OffHand => "Off-hand",
                                crate::items::EquipSlot::Head => "Head",
                                crate::items::EquipSlot::Body => "Body",
                                crate::items::EquipSlot::Hands => "Hands",
                                crate::items::EquipSlot::Feet => "Feet",
                                crate::items::EquipSlot::Amulet => "Amulet",
                                crate::items::EquipSlot::Ring1 => "Ring 1",
                                crate::items::EquipSlot::Ring2 => "Ring 2",
                            };

                            let name_style = if is_selected {
                                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::White)
                            };

                            lines.push(Line::from(vec![
                                Span::styled(prefix, name_style),
                                Span::styled(format!("{}: ", slot_name), Style::default().fg(Color::DarkGray)),
                                Span::styled(truncate_name(name, 20), name_style),
                                Span::styled(format!(" [{}/{}]", current, max), Style::default().fg(slots_color)),
                            ]));
                        }
                    }

                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        "[↑↓] Select  [Enter] Choose  [Esc] Leave",
                        Style::default().fg(Color::DarkGray),
                    )));
                } else {
                    // Phase 2: Enchantment selection for the chosen item
                    let target_slot = self.enchant_selected_slot.unwrap();
                    let is_weapon = Self::is_weapon_slot(target_slot);

                    // Get selected item info
                    let item_info: Option<(String, Vec<(String, i32)>, usize, usize)> = {
                        let mut result = None;
                        if let Some(player) = game.player() {
                            if let Ok(eq) = game.world().get::<&EquipmentComponent>(player) {
                                if let Some(item) = eq.equipment.get(target_slot) {
                                    let affixes: Vec<(String, i32)> = item.affixes.iter()
                                        .map(|a| (a.affix_type.name().to_string(), a.value))
                                        .collect();
                                    result = Some((item.name.clone(), affixes, item.affixes.len(), item.max_enchantments));
                                }
                            }
                        }
                        result
                    };

                    if let Some((item_name, current_affixes, affix_count, max_affixes)) = item_info {
                        // Show selected item
                        lines.push(Line::from(Span::styled(
                            if is_weapon { "Enchanting Weapon:" } else { "Enchanting Armor:" },
                            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                        )));

                        let slots_color = if affix_count >= max_affixes { Color::Red } else { Color::Green };
                        lines.push(Line::from(vec![
                            Span::styled("  ", Style::default()),
                            Span::styled(truncate_name(&item_name, 24), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                            Span::styled(format!(" [{}/{}]", affix_count, max_affixes), Style::default().fg(slots_color)),
                        ]));

                        // Show current enchantments
                        if !current_affixes.is_empty() {
                            lines.push(Line::from(Span::styled(
                                "  Current enchantments:",
                                Style::default().fg(Color::DarkGray),
                            )));
                            for (i, (name, value)) in current_affixes.iter().enumerate() {
                                let is_swap_selected = self.enchant_swap_mode && i == self.enchant_swap_cursor;
                                let prefix = if is_swap_selected { "  ✗ " } else { "    " };
                                let style = if is_swap_selected {
                                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                                } else {
                                    Style::default().fg(Color::Green)
                                };
                                lines.push(Line::from(Span::styled(
                                    format!("{}✦ +{} {}", prefix, value, name),
                                    style,
                                )));
                            }
                        }

                        lines.push(Line::from(""));

                        // Show swap mode notice if at max
                        if affix_count >= max_affixes {
                            if self.enchant_swap_mode {
                                lines.push(Line::from(Span::styled(
                                    "SWAP MODE: Select enchantment to replace (↑↓)",
                                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                                )));
                            } else {
                                lines.push(Line::from(Span::styled(
                                    "Item at max! Press Tab to swap an enchantment.",
                                    Style::default().fg(Color::Yellow),
                                )));
                            }
                            lines.push(Line::from(""));
                        }

                        // Enchantment options (different for weapon vs armor)
                        let header = if is_weapon { "Weapon Enchantments:" } else { "Armor Enchantments:" };
                        lines.push(Line::from(Span::styled(
                            header,
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                        )));

                        let weapon_enchants: [(AffixType, u32, i32, &str); 6] = [
                            (AffixType::BonusDamage, 50, 3, "Physical damage"),
                            (AffixType::FireDamage, 75, 4, "Fire damage"),
                            (AffixType::IceDamage, 75, 4, "Ice damage"),
                            (AffixType::BonusCritChance, 60, 5, "Crit chance"),
                            (AffixType::LifeSteal, 100, 3, "15% lifesteal"),
                            (AffixType::LightningDamage, 80, 4, "Lightning dmg"),
                        ];
                        let armor_enchants: [(AffixType, u32, i32, &str); 6] = [
                            (AffixType::BonusArmor, 50, 3, "+3 armor"),
                            (AffixType::BonusHP, 60, 10, "+10 max HP"),
                            (AffixType::BonusMP, 50, 8, "+8 max mana"),
                            (AffixType::FireResist, 70, 10, "10% fire resist"),
                            (AffixType::IceResist, 70, 10, "10% ice resist"),
                            (AffixType::PoisonResist, 70, 10, "10% poison resist"),
                        ];

                        let enchantments = if is_weapon { weapon_enchants } else { armor_enchants };

                        for (i, (affix_type, cost, value, desc)) in enchantments.iter().enumerate() {
                            let is_selected = !self.enchant_swap_mode && i == self.enchant_affix_cursor;
                            let can_afford = gold >= *cost;
                            // Check if item has this affix and what value
                            let existing_affix = current_affixes.iter()
                                .find(|(n, _)| n == affix_type.name())
                                .map(|(_, v)| *v);
                            let is_same_value = existing_affix == Some(*value);
                            let is_upgrade = existing_affix.is_some() && !is_same_value;

                            let prefix = if is_selected { "► " } else { "  " };
                            let name_style = if is_selected {
                                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                            } else if is_same_value {
                                // Same exact value - can't apply
                                Style::default().fg(Color::DarkGray).add_modifier(Modifier::CROSSED_OUT)
                            } else if is_upgrade {
                                // Can upgrade existing affix
                                Style::default().fg(Color::Cyan)
                            } else if can_afford {
                                Style::default().fg(Color::White)
                            } else {
                                Style::default().fg(Color::DarkGray)
                            };

                            let price_style = if can_afford {
                                Style::default().fg(Color::Yellow)
                            } else {
                                Style::default().fg(Color::Red)
                            };

                            // Build the line with optional upgrade indicator
                            let mut spans = vec![
                                Span::styled(prefix, name_style),
                                Span::styled(format!("+{} ", affix_type.name()), name_style),
                            ];

                            if let Some(old_val) = existing_affix {
                                if is_same_value {
                                    spans.push(Span::styled(format!("(={}) ", value), Style::default().fg(Color::DarkGray)));
                                } else {
                                    // Show upgrade: old → new
                                    spans.push(Span::styled(format!("({} → {}) ", old_val, value), Style::default().fg(Color::Cyan)));
                                }
                            } else {
                                spans.push(Span::styled(format!("(+{}) ", value), Style::default().fg(Color::Green)));
                            }

                            spans.push(Span::styled(format!("{}g ", cost), price_style));
                            spans.push(Span::styled(format!("({})", desc), Style::default().fg(Color::DarkGray)));

                            lines.push(Line::from(spans));
                        }

                        // Special option: +1 max enchantment slot (only if rare upgrade available)
                        if self.enchant_upgrade_available {
                            let plus_one_selected = !self.enchant_swap_mode && self.enchant_affix_cursor == 6;
                            let plus_one_cost = 200u32;
                            let can_afford_plus = gold >= plus_one_cost;
                            let plus_style = if plus_one_selected {
                                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
                            } else if can_afford_plus {
                                Style::default().fg(Color::Magenta)
                            } else {
                                Style::default().fg(Color::DarkGray)
                            };
                            let prefix = if plus_one_selected { "► " } else { "  " };
                            lines.push(Line::from(""));
                            lines.push(Line::from(vec![
                                Span::styled(prefix, plus_style),
                                Span::styled("★ +1 Enchant Slot ", plus_style),
                                Span::styled(format!("{}g ", plus_one_cost), if can_afford_plus { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::Red) }),
                                Span::styled("(✨ RARE!)", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                            ]));
                        }

                        // ===== ENDGAME UPGRADE OPTIONS =====
                        // Get current item enhancement levels for display
                        let (enchant_lvl, awaken_lvl, socket_count, max_sockets, corrupt_lvl, enchant_cost, awaken_cost, corrupt_cost) = {
                            if let Some(player) = game.player() {
                                if let Ok(eq) = game.world().get::<&EquipmentComponent>(player) {
                                    if let Some(item) = eq.equipment.get(target_slot) {
                                        let max_sockets = match item.rarity {
                                            crate::items::Rarity::Common => 1,
                                            crate::items::Rarity::Uncommon => 2,
                                            crate::items::Rarity::Rare => 3,
                                            crate::items::Rarity::Epic => 4,
                                            crate::items::Rarity::Legendary => 5,
                                            crate::items::Rarity::Mythic => 6,
                                        };
                                        (item.enchantment_level, item.awakening_level, item.sockets.len(), max_sockets, item.corruption_level,
                                         item.enchant_cost(), item.awakening_cost(), item.corruption_cost())
                                    } else {
                                        (0, 0, 0, 1, 0, 100, 500, 200)
                                    }
                                } else {
                                    (0, 0, 0, 1, 0, 100, 500, 200)
                                }
                            } else {
                                (0, 0, 0, 1, 0, 100, 500, 200)
                            }
                        };

                        let base_option = if self.enchant_upgrade_available { 7 } else { 6 };

                        lines.push(Line::from(""));
                        lines.push(Line::from(Span::styled("─── Item Enhancement ───", Style::default().fg(Color::DarkGray))));

                        // Option: Enchant+ (increase enchantment level)
                        let enchant_selected = !self.enchant_swap_mode && self.enchant_affix_cursor == base_option;
                        let can_enchant = enchant_lvl < 15 && gold >= enchant_cost;
                        let enchant_style = if enchant_selected {
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                        } else if can_enchant {
                            Style::default().fg(Color::Cyan)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        };
                        let e_prefix = if enchant_selected { "► " } else { "  " };
                        lines.push(Line::from(vec![
                            Span::styled(e_prefix, enchant_style),
                            Span::styled(format!("⚔ Enchant +{} → +{} ", enchant_lvl, enchant_lvl + 1), enchant_style),
                            Span::styled(format!("{}g", enchant_cost), if gold >= enchant_cost { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::Red) }),
                            Span::styled(" (+10% stats)", Style::default().fg(Color::DarkGray)),
                        ]));

                        // Option: Awaken (increase awakening level)
                        let awaken_selected = !self.enchant_swap_mode && self.enchant_affix_cursor == base_option + 1;
                        let can_awaken = gold >= awaken_cost;
                        let awaken_style = if awaken_selected {
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                        } else if can_awaken {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        };
                        let a_prefix = if awaken_selected { "► " } else { "  " };
                        lines.push(Line::from(vec![
                            Span::styled(a_prefix, awaken_style),
                            Span::styled(format!("✦ Awaken [A{}] → [A{}] ", awaken_lvl, awaken_lvl + 1), awaken_style),
                            Span::styled(format!("{}g", awaken_cost), if gold >= awaken_cost { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::Red) }),
                            Span::styled(" (+10% all, ∞)", Style::default().fg(Color::DarkGray)),
                        ]));

                        // Option: Add Socket
                        let socket_selected = !self.enchant_swap_mode && self.enchant_affix_cursor == base_option + 2;
                        let socket_cost = 300 + (socket_count as u32 * 200);
                        let can_socket = socket_count < max_sockets && gold >= socket_cost;
                        let socket_style = if socket_selected {
                            Style::default().fg(Color::Rgb(200, 200, 255)).add_modifier(Modifier::BOLD)
                        } else if can_socket {
                            Style::default().fg(Color::Rgb(200, 200, 255))
                        } else {
                            Style::default().fg(Color::DarkGray)
                        };
                        let s_prefix = if socket_selected { "► " } else { "  " };
                        let socket_text = if socket_count >= max_sockets {
                            format!("◇ Sockets [{}/{}] MAX", socket_count, max_sockets)
                        } else {
                            format!("◇ Add Socket [{}/{}]", socket_count, max_sockets)
                        };
                        lines.push(Line::from(vec![
                            Span::styled(s_prefix, socket_style),
                            Span::styled(socket_text, socket_style),
                            Span::styled(if can_socket { format!(" {}g", socket_cost) } else { "".to_string() }, if gold >= socket_cost { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::Red) }),
                        ]));

                        // Option: Corrupt (risk/reward)
                        let corrupt_selected = !self.enchant_swap_mode && self.enchant_affix_cursor == base_option + 3;
                        let can_corrupt = corrupt_lvl < 10 && gold >= corrupt_cost;
                        let corrupt_style = if corrupt_selected {
                            Style::default().fg(Color::Rgb(150, 50, 150)).add_modifier(Modifier::BOLD)
                        } else if can_corrupt {
                            Style::default().fg(Color::Rgb(150, 50, 150))
                        } else {
                            Style::default().fg(Color::DarkGray)
                        };
                        let c_prefix = if corrupt_selected { "► " } else { "  " };
                        lines.push(Line::from(vec![
                            Span::styled(c_prefix, corrupt_style),
                            Span::styled(format!("☠ Corrupt {{C{}}} → {{C{}}} ", corrupt_lvl, corrupt_lvl + 1), corrupt_style),
                            Span::styled(format!("{}g", corrupt_cost), if gold >= corrupt_cost { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::Red) }),
                            Span::styled(" (+15%dmg/-5%HP)", Style::default().fg(Color::DarkGray)),
                        ]));
                    }

                    lines.push(Line::from(""));
                    let controls = if self.enchant_swap_mode {
                        "[↑↓] Select to replace  [Enter] Swap  [Tab] Cancel  [Esc] Back"
                    } else {
                        "[↑↓] Select  [Enter] Apply  [Tab] Swap mode  [Esc] Back"
                    };
                    lines.push(Line::from(Span::styled(
                        controls,
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
            ShrineType::Rest => {
                // Rest shrine heals immediately, so this shouldn't normally show
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "You feel your wounds heal...",
                    Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "[Esc] Leave shrine",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            ShrineType::Corruption => {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Dark energy swirls around this altar...",
                    Style::default().fg(Color::Rgb(200, 50, 50)).add_modifier(Modifier::ITALIC),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Accept a curse for great power?",
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));

                // Pact 1: Power
                lines.push(Line::from(vec![
                    Span::styled("[1] ", Style::default().fg(Color::Yellow)),
                    Span::styled("Pact of Power", Style::default().fg(Color::Rgb(255, 100, 100)).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("    Curse: ", Style::default().fg(Color::Rgb(150, 50, 50))),
                    Span::styled("-20% damage dealt", Style::default().fg(Color::Red)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("    Blessing: ", Style::default().fg(Color::Rgb(50, 150, 50))),
                    Span::styled("+30% strength", Style::default().fg(Color::Green)),
                ]));
                lines.push(Line::from(""));

                // Pact 2: Vitality
                lines.push(Line::from(vec![
                    Span::styled("[2] ", Style::default().fg(Color::Yellow)),
                    Span::styled("Pact of Vitality", Style::default().fg(Color::Rgb(100, 255, 100)).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("    Curse: ", Style::default().fg(Color::Rgb(150, 50, 50))),
                    Span::styled("Poison (5 dmg/turn)", Style::default().fg(Color::Red)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("    Blessing: ", Style::default().fg(Color::Rgb(50, 150, 50))),
                    Span::styled("Regenerate 8 HP/turn", Style::default().fg(Color::Green)),
                ]));
                lines.push(Line::from(""));

                // Pact 3: Swiftness
                lines.push(Line::from(vec![
                    Span::styled("[3] ", Style::default().fg(Color::Yellow)),
                    Span::styled("Pact of Swiftness", Style::default().fg(Color::Rgb(100, 200, 255)).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("    Curse: ", Style::default().fg(Color::Rgb(150, 50, 50))),
                    Span::styled("-25% movement speed", Style::default().fg(Color::Red)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("    Blessing: ", Style::default().fg(Color::Rgb(50, 150, 50))),
                    Span::styled("+40% attack speed", Style::default().fg(Color::Green)),
                ]));
                lines.push(Line::from(""));

                lines.push(Line::from(Span::styled(
                    "[1-3] Accept pact   [Esc] Refuse the power",
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }

        let text = Paragraph::new(lines);
        frame.render_widget(text, inner);
    }

    fn render_shop_overlay(&self, frame: &mut Frame, game: &Game, npc_entity: hecs::Entity) {
        use crate::entities::NpcComponent;
        use crate::ecs::InventoryComponent;

        let area = centered_rect(60, 70, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" $ Merchant $ ")
            .border_style(Style::default().fg(Color::Rgb(255, 215, 0)));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines: Vec<Line> = Vec::new();

        // Get player gold
        let player_gold = game.player().map(|player| {
            game.world().get::<&InventoryComponent>(player)
                .map(|inv| inv.inventory.gold())
                .unwrap_or(0)
        }).unwrap_or(0);

        // Tab bar
        let buy_style = if self.shop_mode == 0 {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let sell_style = if self.shop_mode == 1 {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        lines.push(Line::from(vec![
            Span::styled("[", Style::default().fg(Color::Gray)),
            Span::styled("Buy", buy_style),
            Span::styled("]  [", Style::default().fg(Color::Gray)),
            Span::styled("Sell", sell_style),
            Span::styled("]", Style::default().fg(Color::Gray)),
            Span::styled("     Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("Tab", Style::default().fg(Color::White)),
            Span::styled(" to switch", Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(""));

        // Gold display
        lines.push(Line::from(vec![
            Span::styled("Your Gold: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", player_gold), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));

        if self.shop_mode == 0 {
            // BUY MODE
            lines.push(Line::from(Span::styled(
                "Items for Sale:",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            // Get shop items
            let shop_items: Vec<_> = game.world()
                .get::<&NpcComponent>(npc_entity)
                .map(|npc| npc.shop_items.clone())
                .unwrap_or_default();

            if shop_items.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  (No items available)",
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                )));
            } else {
                for (i, shop_item) in shop_items.iter().enumerate() {
                    let is_selected = i == self.shop_selection;
                    let can_afford = player_gold >= shop_item.buy_price;

                    let rarity_color = Color::Rgb(
                        shop_item.item.rarity.color().0,
                        shop_item.item.rarity.color().1,
                        shop_item.item.rarity.color().2,
                    );

                    let prefix = if is_selected { "> " } else { "  " };
                    let selector_style = if is_selected {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    let name_style = if can_afford {
                        Style::default().fg(rarity_color)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    let price_style = if can_afford {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Red)
                    };

                    let display_name = truncate_name(&shop_item.item.name, 20);

                    // Build stats string
                    let mut stats_spans = Vec::new();
                    if shop_item.item.base_damage > 0 {
                        stats_spans.push(Span::styled(
                            format!(" ⚔{}", shop_item.item.total_damage()),
                            Style::default().fg(Color::Red)
                        ));
                    }
                    if shop_item.item.base_armor > 0 {
                        stats_spans.push(Span::styled(
                            format!(" 🛡{}", shop_item.item.total_armor()),
                            Style::default().fg(Color::Blue)
                        ));
                    }

                    let mut line_spans = vec![
                        Span::styled(prefix, selector_style),
                        Span::styled(format!("{} ", shop_item.item.glyph), name_style),
                        Span::styled(display_name, name_style),
                    ];
                    line_spans.extend(stats_spans);
                    line_spans.push(Span::styled(format!(" - {} gold", shop_item.buy_price), price_style));
                    lines.push(Line::from(line_spans));

                    // Show item description and affixes for selected item
                    if is_selected {
                        // Description
                        if !shop_item.item.description.is_empty() {
                            lines.push(Line::from(vec![
                                Span::styled("     ", Style::default()),
                                Span::styled(shop_item.item.description.clone(), Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
                            ]));
                        }
                        // Show affixes if any
                        for affix in &shop_item.item.affixes {
                            lines.push(Line::from(vec![
                                Span::styled("     ", Style::default()),
                                Span::styled(format!("✦ +{} {}", affix.value, affix.affix_type.name()), Style::default().fg(Color::Green)),
                            ]));
                        }

                        // Show comparison with equipped item
                        if let Some(slot) = shop_item.item.equip_slot {
                            use crate::ecs::EquipmentComponent;
                            let equipped_opt = game.player()
                                .and_then(|p| game.world().get::<&EquipmentComponent>(p).ok())
                                .and_then(|eq| eq.equipment.get(slot).cloned());

                            if let Some(equipped) = equipped_opt {
                                lines.push(Line::from(""));
                                let eq_color = equipped.rarity.color();
                                lines.push(Line::from(vec![
                                    Span::styled("     ▶ Equipped: ", Style::default().fg(Color::Cyan)),
                                    Span::styled(
                                        truncate_name(&equipped.name, 16),
                                        Style::default().fg(Color::Rgb(eq_color.0, eq_color.1, eq_color.2))
                                    ),
                                ]));

                                // Damage comparison
                                if shop_item.item.base_damage > 0 || equipped.base_damage > 0 {
                                    let new_dmg = shop_item.item.total_damage();
                                    let old_dmg = equipped.total_damage();
                                    let diff = new_dmg - old_dmg;
                                    let (indicator, diff_color) = if diff > 0 {
                                        (format!("▲+{}", diff), Color::Green)
                                    } else if diff < 0 {
                                        (format!("▼{}", diff), Color::Red)
                                    } else {
                                        ("─".to_string(), Color::Gray)
                                    };

                                    lines.push(Line::from(vec![
                                        Span::styled("       ⚔ ", Style::default().fg(Color::DarkGray)),
                                        Span::styled(format!("{}", new_dmg), Style::default().fg(Color::Red)),
                                        Span::styled(" vs ", Style::default().fg(Color::DarkGray)),
                                        Span::styled(format!("{}", old_dmg), Style::default().fg(Color::Red)),
                                        Span::styled(format!(" {}", indicator), Style::default().fg(diff_color).add_modifier(Modifier::BOLD)),
                                    ]));
                                }

                                // Armor comparison
                                if shop_item.item.base_armor > 0 || equipped.base_armor > 0 {
                                    let new_arm = shop_item.item.total_armor();
                                    let old_arm = equipped.total_armor();
                                    let diff = new_arm - old_arm;
                                    let (indicator, diff_color) = if diff > 0 {
                                        (format!("▲+{}", diff), Color::Green)
                                    } else if diff < 0 {
                                        (format!("▼{}", diff), Color::Red)
                                    } else {
                                        ("─".to_string(), Color::Gray)
                                    };

                                    lines.push(Line::from(vec![
                                        Span::styled("       🛡 ", Style::default().fg(Color::DarkGray)),
                                        Span::styled(format!("{}", new_arm), Style::default().fg(Color::Blue)),
                                        Span::styled(" vs ", Style::default().fg(Color::DarkGray)),
                                        Span::styled(format!("{}", old_arm), Style::default().fg(Color::Blue)),
                                        Span::styled(format!(" {}", indicator), Style::default().fg(diff_color).add_modifier(Modifier::BOLD)),
                                    ]));
                                }
                            } else {
                                lines.push(Line::from(vec![
                                    Span::styled("     ", Style::default()),
                                    Span::styled(format!("Slot: {} (empty)", slot.name()), Style::default().fg(Color::DarkGray)),
                                ]));
                            }
                        }
                    }
                }
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "[↑↓] Select  [Enter] Buy  [Tab] Sell  [Esc] Leave",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            // SELL MODE
            lines.push(Line::from(Span::styled(
                "Your Items:",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            // Get player inventory
            let player_items: Vec<_> = game.player()
                .and_then(|p| game.world().get::<&InventoryComponent>(p).ok())
                .map(|inv| inv.inventory.items_owned())
                .unwrap_or_default();

            if player_items.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  (No items to sell)",
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                )));
            } else {
                for (i, item) in player_items.iter().enumerate() {
                    let is_selected = i == self.sell_selection;
                    // Calculate sell price (40% of value)
                    let sell_price = (item.value as f32 * 0.4).max(1.0) as u32;

                    let rarity_color = Color::Rgb(
                        item.rarity.color().0,
                        item.rarity.color().1,
                        item.rarity.color().2,
                    );

                    let prefix = if is_selected { "> " } else { "  " };
                    let selector_style = if is_selected {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    let name_style = Style::default().fg(rarity_color);
                    let price_style = Style::default().fg(Color::Green);

                    let display_name = truncate_name(&item.name, 20);

                    // Build stats string
                    let mut stats_spans = Vec::new();
                    if item.base_damage > 0 {
                        stats_spans.push(Span::styled(
                            format!(" ⚔{}", item.total_damage()),
                            Style::default().fg(Color::Red)
                        ));
                    }
                    if item.base_armor > 0 {
                        stats_spans.push(Span::styled(
                            format!(" 🛡{}", item.total_armor()),
                            Style::default().fg(Color::Blue)
                        ));
                    }

                    let mut line_spans = vec![
                        Span::styled(prefix, selector_style),
                        Span::styled(format!("{} ", item.glyph), name_style),
                        Span::styled(display_name, name_style),
                    ];
                    line_spans.extend(stats_spans);
                    line_spans.push(Span::styled(format!(" - {} gold", sell_price), price_style));
                    lines.push(Line::from(line_spans));

                    // Show item description and affixes for selected item
                    if is_selected {
                        lines.push(Line::from(vec![
                            Span::styled("     ", Style::default()),
                            Span::styled(item.description.clone(), Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
                        ]));
                        // Show affixes if any
                        for affix in &item.affixes {
                            lines.push(Line::from(vec![
                                Span::styled("     ", Style::default()),
                                Span::styled(format!("✦ +{} {}", affix.value, affix.affix_type.name()), Style::default().fg(Color::Green)),
                            ]));
                        }
                    }
                }
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "[↑↓] Select  [Enter] Sell  [Tab] Buy  [Esc] Leave",
                Style::default().fg(Color::DarkGray),
            )));
        }

        let text = Paragraph::new(lines);
        frame.render_widget(text, inner);
    }

    fn render_pause(&self, frame: &mut Frame, game: &Game) {
        // Render game in background
        self.render_playing(frame, game, &PlayingState::Exploring);

        // Overlay pause menu
        let area = centered_rect(30, 30, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" PAUSED ")
            .border_style(Style::default().fg(Color::White));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let menu = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled("[Esc] Resume", Style::default().fg(Color::White))),
            Line::from(""),
            Line::from(Span::styled("[S] Save Game", Style::default().fg(Color::White))),
            Line::from(""),
            Line::from(Span::styled("[Q] Quit to Menu", Style::default().fg(Color::Gray))),
        ])
        .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(menu, inner);
    }

    fn render_save_slots(&self, frame: &mut Frame, game: &Game, selected: u8) {
        use crate::save::list_saves;

        // Render game in background
        self.render_playing(frame, game, &PlayingState::Exploring);

        // Overlay save slots menu
        let area = centered_rect(50, 50, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" SAVE GAME ")
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let saves = list_saves();
        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Select a slot to save:",
                Style::default().fg(Color::Gray),
            )),
            Line::from(""),
        ];

        for (slot, summary) in saves {
            let is_selected = slot == selected;
            let prefix = if is_selected { "> " } else { "  " };
            let style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let slot_text = match summary {
                Some(s) => format!("{}Slot {}: Floor {} - Level {} ({:?})", prefix, slot + 1, s.floor, s.level, s.difficulty),
                None => format!("{}Slot {}: Empty", prefix, slot + 1),
            };

            lines.push(Line::from(Span::styled(slot_text, style)));
            lines.push(Line::from(""));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "[Enter] Save  [D] Delete  [Esc] Cancel",
            Style::default().fg(Color::DarkGray),
        )));

        let menu = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(menu, inner);
    }

    fn render_load_slots(&self, frame: &mut Frame, selected: u8) {
        use crate::save::list_saves;

        let area = frame.area();

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" LOAD GAME ")
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let saves = list_saves();
        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Select a save to load:",
                Style::default().fg(Color::Gray),
            )),
            Line::from(""),
        ];

        for (slot, summary) in saves {
            let is_selected = slot == selected;
            let prefix = if is_selected { "> " } else { "  " };

            let (slot_text, style) = match summary {
                Some(s) => {
                    let text = format!("{}Slot {}: Floor {} - Level {} ({:?})", prefix, slot + 1, s.floor, s.level, s.difficulty);
                    let style = if is_selected {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    (text, style)
                }
                None => {
                    let text = format!("{}Slot {}: Empty", prefix, slot + 1);
                    let style = Style::default().fg(Color::DarkGray);
                    (text, style)
                }
            };

            lines.push(Line::from(Span::styled(slot_text, style)));
            lines.push(Line::from(""));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "[Enter] Load  [D] Delete  [Esc] Back",
            Style::default().fg(Color::DarkGray),
        )));

        let menu = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(menu, inner);
    }

    fn render_achievements(&self, frame: &mut Frame, game: &Game) {
        use crate::save::all_achievements;

        let area = frame.area();

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" ACHIEVEMENTS & STATS ")
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let profile = game.profile();
        let achievements = all_achievements();

        // Layout: left side for stats, right side for achievements
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(inner);

        // Stats panel
        let stats_block = Block::default()
            .borders(Borders::ALL)
            .title(" Statistics ")
            .border_style(Style::default().fg(Color::Cyan));
        let stats_inner = stats_block.inner(chunks[0]);
        frame.render_widget(stats_block, chunks[0]);

        let playtime_hours = profile.stats.playtime_seconds / 3600;
        let playtime_mins = (profile.stats.playtime_seconds % 3600) / 60;

        let stats_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Total Runs: ", Style::default().fg(Color::Gray)),
                Span::styled(profile.stats.total_runs.to_string(), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Deaths: ", Style::default().fg(Color::Gray)),
                Span::styled(profile.stats.total_deaths.to_string(), Style::default().fg(Color::Red)),
            ]),
            Line::from(vec![
                Span::styled("Victories: ", Style::default().fg(Color::Gray)),
                Span::styled(profile.victories.to_string(), Style::default().fg(Color::Green)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Highest Floor: ", Style::default().fg(Color::Gray)),
                Span::styled(profile.highest_floor.to_string(), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("Floors Descended: ", Style::default().fg(Color::Gray)),
                Span::styled(profile.stats.floors_descended.to_string(), Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Enemies Killed: ", Style::default().fg(Color::Gray)),
                Span::styled(profile.stats.enemies_killed.to_string(), Style::default().fg(Color::Red)),
            ]),
            Line::from(vec![
                Span::styled("Bosses Defeated: ", Style::default().fg(Color::Gray)),
                Span::styled(profile.stats.bosses_defeated.to_string(), Style::default().fg(Color::Magenta)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Gold Collected: ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{}", profile.stats.gold_collected), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("Items Found: ", Style::default().fg(Color::Gray)),
                Span::styled(profile.stats.items_found.to_string(), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Playtime: ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{}h {}m", playtime_hours, playtime_mins), Style::default().fg(Color::White)),
            ]),
        ];

        let stats_para = Paragraph::new(stats_lines);
        frame.render_widget(stats_para, stats_inner);

        // Achievements panel
        let achieved_count = profile.achievements.len();
        let total_count = achievements.len();
        let achievements_block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Achievements ({}/{}) ", achieved_count, total_count))
            .border_style(Style::default().fg(Color::Yellow));
        let achievements_inner = achievements_block.inner(chunks[1]);
        frame.render_widget(achievements_block, chunks[1]);

        let mut achievement_lines: Vec<Line> = vec![Line::from("")];

        for achievement in achievements {
            let unlocked = profile.has_achievement(achievement.id);

            if achievement.hidden && !unlocked {
                // Show hidden achievements as "???"
                achievement_lines.push(Line::from(vec![
                    Span::styled("[ ] ", Style::default().fg(Color::DarkGray)),
                    Span::styled("???", Style::default().fg(Color::DarkGray)),
                ]));
            } else {
                let (check, name_style, desc_style) = if unlocked {
                    (
                        Span::styled("[X] ", Style::default().fg(Color::Green)),
                        Style::default().fg(Color::Yellow),
                        Style::default().fg(Color::Gray),
                    )
                } else {
                    (
                        Span::styled("[ ] ", Style::default().fg(Color::DarkGray)),
                        Style::default().fg(Color::White),
                        Style::default().fg(Color::DarkGray),
                    )
                };

                achievement_lines.push(Line::from(vec![
                    check,
                    Span::styled(achievement.name, name_style),
                ]));
                achievement_lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(achievement.description, desc_style),
                ]));
            }
            achievement_lines.push(Line::from(""));
        }

        achievement_lines.push(Line::from(""));
        achievement_lines.push(Line::from(Span::styled(
            "[Esc] Back to Menu",
            Style::default().fg(Color::DarkGray),
        )));

        let achievements_para = Paragraph::new(achievement_lines);
        frame.render_widget(achievements_para, achievements_inner);
    }

    fn render_game_over(&self, frame: &mut Frame, floor: u32, cause: &str) {
        let area = frame.area();

        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "YOU HAVE FALLEN",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!("Reached Floor: {}", floor)),
            Line::from(""),
            Line::from(Span::styled(cause, Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "Press [Enter] to continue",
                Style::default().fg(Color::Gray),
            )),
        ];

        let para = Paragraph::new(text)
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(para, area);
    }

    fn render_victory(&self, frame: &mut Frame) {
        let area = frame.area();

        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "VICTORY",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("You have conquered the Hollowdeep!"),
            Line::from(""),
            Line::from(Span::styled(
                "Press [Enter] to continue",
                Style::default().fg(Color::Gray),
            )),
        ];

        let para = Paragraph::new(text)
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(para, area);
    }

    fn render_new_run(&self, frame: &mut Frame) {
        let area = frame.area();

        let text = Paragraph::new("New Run Setup - Coming Soon\n\nPress [Enter] to start with defaults")
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" New Run "));

        frame.render_widget(text, area);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Create a near-fullscreen overlay with small margins that adapts to terminal size
/// Uses most of the available space while keeping small margins (1-2 cells on each side)
fn fullscreen_overlay(r: Rect) -> Rect {
    // Use a 2-cell margin on all sides for large terminals, 1-cell for smaller
    let margin = if r.width > 100 && r.height > 40 { 2 } else { 1 };
    Rect {
        x: r.x + margin,
        y: r.y + margin,
        width: r.width.saturating_sub(margin * 2),
        height: r.height.saturating_sub(margin * 2),
    }
}
