//! Main graphical application
//!
//! Entry point and main loop for the graphical frontend.

use macroquad::prelude::*;
use crate::game::{Game, GameState};
use crate::progression::Difficulty;
use super::renderer::{self, Camera, TILE_SIZE};
use super::input::{self, InputAction};
use super::colors;

/// UI screen state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    MainMenu,
    DifficultySelect,
    Playing,
    Inventory,
    Character,
    Paused,
    GameOver,
    Victory,
}

/// Main graphical application state
struct GraphicalApp {
    screen: Screen,
    camera: Camera,
    menu_cursor: usize,
    difficulty_cursor: usize,
    show_minimap: bool,
}

impl GraphicalApp {
    fn new() -> Self {
        Self {
            screen: Screen::MainMenu,
            camera: Camera::new(),
            menu_cursor: 0,
            difficulty_cursor: 1, // Default to Normal
            show_minimap: true,
        }
    }

    fn handle_input(&mut self, game: &mut Game) -> bool {
        match self.screen {
            Screen::MainMenu => self.handle_main_menu_input(game),
            Screen::DifficultySelect => self.handle_difficulty_input(game),
            Screen::Playing => self.handle_playing_input(game),
            Screen::Inventory => self.handle_inventory_input(game),
            Screen::Character => self.handle_character_input(game),
            Screen::Paused => self.handle_paused_input(game),
            Screen::GameOver | Screen::Victory => self.handle_end_screen_input(game),
        }
    }

    fn handle_main_menu_input(&mut self, _game: &mut Game) -> bool {
        if let Some(action) = input::get_menu_input() {
            match action {
                InputAction::ScrollUp => {
                    if self.menu_cursor > 0 {
                        self.menu_cursor -= 1;
                    }
                }
                InputAction::ScrollDown => {
                    if self.menu_cursor < 3 {
                        self.menu_cursor += 1;
                    }
                }
                InputAction::Confirm => {
                    match self.menu_cursor {
                        0 => self.screen = Screen::DifficultySelect, // New Game
                        1 => {
                            // Continue (TODO: implement save loading)
                            self.screen = Screen::DifficultySelect;
                        }
                        2 => {} // Options (TODO)
                        3 => return true, // Quit
                        _ => {}
                    }
                }
                InputAction::Cancel => return true,
                _ => {}
            }
        }
        false
    }

    fn handle_difficulty_input(&mut self, game: &mut Game) -> bool {
        if let Some(action) = input::get_menu_input() {
            match action {
                InputAction::ScrollUp => {
                    if self.difficulty_cursor > 0 {
                        self.difficulty_cursor -= 1;
                    }
                }
                InputAction::ScrollDown => {
                    if self.difficulty_cursor < 3 {
                        self.difficulty_cursor += 1;
                    }
                }
                InputAction::Confirm => {
                    let difficulty = match self.difficulty_cursor {
                        0 => Difficulty::Easy,
                        1 => Difficulty::Normal,
                        2 => Difficulty::Hard,
                        _ => Difficulty::Nightmare,
                    };
                    game.set_difficulty(difficulty);
                    game.start_new_run();
                    self.screen = Screen::Playing;
                }
                InputAction::Cancel => {
                    self.screen = Screen::MainMenu;
                }
                _ => {}
            }
        }
        false
    }

    fn handle_playing_input(&mut self, game: &mut Game) -> bool {
        if let Some(action) = input::get_input_action() {
            match action {
                // Movement
                InputAction::MoveUp => { game.try_move_player(0, -1); }
                InputAction::MoveDown => { game.try_move_player(0, 1); }
                InputAction::MoveLeft => { game.try_move_player(-1, 0); }
                InputAction::MoveRight => { game.try_move_player(1, 0); }
                InputAction::MoveUpLeft => { game.try_move_player(-1, -1); }
                InputAction::MoveUpRight => { game.try_move_player(1, -1); }
                InputAction::MoveDownLeft => { game.try_move_player(-1, 1); }
                InputAction::MoveDownRight => { game.try_move_player(1, 1); }
                InputAction::Wait => { game.wait_turn(); }

                // Interact
                InputAction::Confirm => {
                    game.interact();
                }

                // Pick up items
                InputAction::PickUp => {
                    game.pickup_item();
                }

                // Menus
                InputAction::Inventory => {
                    self.screen = Screen::Inventory;
                }
                InputAction::Character => {
                    self.screen = Screen::Character;
                }
                InputAction::Pause | InputAction::Cancel => {
                    self.screen = Screen::Paused;
                }
                InputAction::Map => {
                    self.show_minimap = !self.show_minimap;
                }

                // Skills
                InputAction::Skill1 => { game.use_skill(0); }
                InputAction::Skill2 => { game.use_skill(1); }
                InputAction::Skill3 => { game.use_skill(2); }
                InputAction::Skill4 => { game.use_skill(3); }
                InputAction::Skill5 => { game.use_skill(4); }

                InputAction::Quit => return true,
                _ => {}
            }
        }

        // Check game state
        match game.state() {
            GameState::GameOver => self.screen = Screen::GameOver,
            GameState::Victory => self.screen = Screen::Victory,
            GameState::Quit => return true,
            _ => {}
        }

        false
    }

    fn handle_inventory_input(&mut self, _game: &mut Game) -> bool {
        if let Some(action) = input::get_menu_input() {
            match action {
                InputAction::Cancel => {
                    self.screen = Screen::Playing;
                }
                _ => {}
            }
        }
        false
    }

    fn handle_character_input(&mut self, game: &mut Game) -> bool {
        if let Some(action) = input::get_menu_input() {
            match action {
                InputAction::Cancel => {
                    self.screen = Screen::Playing;
                }
                // Stat allocation
                InputAction::StatStr => { game.allocate_stat_point(crate::ecs::StatType::Strength); }
                InputAction::StatDex => { game.allocate_stat_point(crate::ecs::StatType::Dexterity); }
                InputAction::StatInt => { game.allocate_stat_point(crate::ecs::StatType::Intelligence); }
                InputAction::StatVit => { game.allocate_stat_point(crate::ecs::StatType::Vitality); }
                _ => {}
            }
        }
        false
    }

    fn handle_paused_input(&mut self, _game: &mut Game) -> bool {
        if let Some(action) = input::get_menu_input() {
            match action {
                InputAction::Confirm | InputAction::Cancel => {
                    self.screen = Screen::Playing;
                }
                _ => {}
            }
        }
        false
    }

    fn handle_end_screen_input(&mut self, game: &mut Game) -> bool {
        if let Some(action) = input::get_menu_input() {
            if action == InputAction::Confirm {
                game.reset();
                self.screen = Screen::MainMenu;
                self.menu_cursor = 0;
            }
        }
        false
    }

    fn render(&mut self, game: &Game) {
        clear_background(colors::BACKGROUND);

        match self.screen {
            Screen::MainMenu => {
                renderer::render_main_menu(self.menu_cursor, false, 0);
            }
            Screen::DifficultySelect => {
                renderer::render_main_menu(0, true, self.difficulty_cursor);
            }
            Screen::Playing => {
                self.render_game_screen(game);
            }
            Screen::Inventory => {
                self.render_game_screen(game);
                self.render_inventory_overlay(game);
            }
            Screen::Character => {
                self.render_game_screen(game);
                self.render_character_overlay(game);
            }
            Screen::Paused => {
                self.render_game_screen(game);
                renderer::render_overlay("PAUSED", "Game is paused");
            }
            Screen::GameOver => {
                self.render_game_screen(game);
                renderer::render_overlay("GAME OVER", "You have fallen in the depths...");
            }
            Screen::Victory => {
                self.render_game_screen(game);
                renderer::render_overlay("VICTORY", "You have conquered Hollowdeep!");
            }
        }
    }

    fn render_game_screen(&mut self, game: &Game) {
        let screen_w = screen_width();
        let screen_h = screen_height();

        // Layout
        let sidebar_width = 180.0;
        let message_height = 120.0;
        let map_area = Rect::new(0.0, 0.0, screen_w - sidebar_width, screen_h - message_height);
        let sidebar_area = Rect::new(screen_w - sidebar_width, 0.0, sidebar_width, screen_h - message_height);
        let message_area = Rect::new(0.0, screen_h - message_height, screen_w, message_height);

        // Center camera on player
        if let Some(pos) = game.player_position() {
            self.camera.center_on(pos, map_area.w, map_area.h);
        }

        // Render map
        renderer::render_map(game, &self.camera, map_area);

        // Render entities
        renderer::render_entities(game, &self.camera, map_area);

        // Render UI panels
        renderer::render_status_panel(game, sidebar_area);
        renderer::render_messages(game, message_area);

        // Render minimap (top-right of map area)
        if self.show_minimap {
            let minimap_size = 120.0;
            let minimap_area = Rect::new(
                map_area.x + map_area.w - minimap_size - 10.0,
                map_area.y + 10.0,
                minimap_size,
                minimap_size,
            );
            renderer::render_minimap(game, minimap_area);
        }

        // Render hover tooltip
        self.render_hover_tooltip(game, &map_area);

        // Controls hint
        draw_text(
            "[I]nventory  [C]haracter  [M]inimap  [Esc]Pause",
            10.0,
            screen_h - message_height - 10.0,
            12.0,
            colors::TEXT_MUTED,
        );
    }

    fn render_hover_tooltip(&self, game: &Game, map_area: &Rect) {
        let (mx, my) = mouse_position();

        // Check if mouse is over map area
        if mx < map_area.x || mx > map_area.x + map_area.w
            || my < map_area.y || my > map_area.y + map_area.h
        {
            return;
        }

        // Convert screen position to tile position
        let tile_x = ((mx - map_area.x + self.camera.x) / TILE_SIZE).floor() as i32;
        let tile_y = ((my - map_area.y + self.camera.y) / TILE_SIZE).floor() as i32;

        let pos = crate::ecs::Position::new(tile_x, tile_y);
        let map = game.map();

        if !map.is_visible(pos) {
            return;
        }

        // Check for entities at this position
        let world = game.world();
        for (entity, (epos, _)) in world.query::<(&crate::ecs::Position, &crate::ecs::Renderable)>().iter() {
            if *epos == pos {
                if let Ok(name) = world.get::<&crate::ecs::Name>(entity) {
                    let mut tooltip = name.0.clone();

                    // Add health info for enemies
                    if let Ok(health) = world.get::<&crate::ecs::Health>(entity) {
                        tooltip = format!("{} ({}/{})", tooltip, health.current, health.max);
                    }

                    renderer::render_tooltip(&tooltip, mx, my);
                    return;
                }
            }
        }

        // Check tile
        if let Some(tile) = map.get_tile(tile_x, tile_y) {
            let tile_name = match tile.tile_type {
                crate::world::TileType::StairsDown => "Stairs Down",
                crate::world::TileType::StairsUp => "Stairs Up",
                crate::world::TileType::ShrineRest => "Rest Shrine",
                crate::world::TileType::ShrineSkill => "Skill Shrine",
                crate::world::TileType::ShrineEnchant => "Enchant Shrine",
                crate::world::TileType::ShrineCorruption => "Corruption Shrine",
                crate::world::TileType::DoorClosed => "Door (Closed)",
                crate::world::TileType::DoorOpen => "Door (Open)",
                _ => return,
            };
            renderer::render_tooltip(tile_name, mx, my);
        }
    }

    fn render_inventory_overlay(&self, _game: &Game) {
        let screen_w = screen_width();
        let screen_h = screen_height();

        // Dim background
        draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::new(0.0, 0.0, 0.0, 0.5));

        // Panel
        let panel_w = screen_w * 0.7;
        let panel_h = screen_h * 0.8;
        let panel_x = (screen_w - panel_w) / 2.0;
        let panel_y = (screen_h - panel_h) / 2.0;

        draw_rectangle(panel_x, panel_y, panel_w, panel_h, colors::PANEL_BG);
        draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, colors::PANEL_BORDER);

        draw_text("INVENTORY", panel_x + panel_w / 2.0 - 60.0, panel_y + 30.0, 24.0, colors::TEXT_PRIMARY);
        draw_text("[Esc] Close", panel_x + panel_w - 100.0, panel_y + panel_h - 20.0, 14.0, colors::TEXT_MUTED);

        // TODO: Implement full inventory display
        draw_text("(Inventory UI coming soon)", panel_x + 20.0, panel_y + 80.0, 16.0, colors::TEXT_SECONDARY);
    }

    fn render_character_overlay(&self, game: &Game) {
        let screen_w = screen_width();
        let screen_h = screen_height();

        // Dim background
        draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::new(0.0, 0.0, 0.0, 0.5));

        // Panel
        let panel_w = screen_w * 0.6;
        let panel_h = screen_h * 0.7;
        let panel_x = (screen_w - panel_w) / 2.0;
        let panel_y = (screen_h - panel_h) / 2.0;

        draw_rectangle(panel_x, panel_y, panel_w, panel_h, colors::PANEL_BG);
        draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, colors::PANEL_BORDER);

        draw_text("CHARACTER", panel_x + panel_w / 2.0 - 60.0, panel_y + 30.0, 24.0, colors::TEXT_PRIMARY);

        // Stats
        let stats = game.player_stats().unwrap_or(crate::ecs::Stats::player_base());
        let xp = game.player_experience().unwrap_or(crate::ecs::Experience::new());
        let stat_points = game.player_stat_points().unwrap_or(0);

        let mut y = panel_y + 70.0;
        let line_height = 30.0;

        draw_text(&format!("Level: {}", xp.level), panel_x + 30.0, y, 20.0, colors::XP);
        y += line_height;

        if stat_points > 0 {
            draw_text(&format!("Stat Points: {} (Press 1-4 to allocate)", stat_points), panel_x + 30.0, y, 18.0, colors::HEALTH_MED);
            y += line_height;
        }

        y += 10.0;
        draw_text(&format!("[1] Strength: {}", stats.strength), panel_x + 30.0, y, 18.0, Color::new(0.9, 0.4, 0.4, 1.0));
        y += line_height;
        draw_text(&format!("[2] Dexterity: {}", stats.dexterity), panel_x + 30.0, y, 18.0, Color::new(0.4, 0.9, 0.4, 1.0));
        y += line_height;
        draw_text(&format!("[3] Intelligence: {}", stats.intelligence), panel_x + 30.0, y, 18.0, Color::new(0.4, 0.4, 0.9, 1.0));
        y += line_height;
        draw_text(&format!("[4] Vitality: {}", stats.vitality), panel_x + 30.0, y, 18.0, Color::new(0.9, 0.9, 0.4, 1.0));

        draw_text("[Esc] Close", panel_x + panel_w - 100.0, panel_y + panel_h - 20.0, 14.0, colors::TEXT_MUTED);
    }
}

/// Main entry point for the graphical frontend
pub async fn run_graphical() {
    // Configure window
    request_new_screen_size(1280.0, 800.0);

    let mut app = GraphicalApp::new();
    let mut game = Game::new();

    loop {
        // Handle input
        if app.handle_input(&mut game) {
            break;
        }

        // Update game
        game.update(std::time::Duration::from_secs_f32(get_frame_time()));

        // Render
        app.render(&game);

        next_frame().await;
    }
}
