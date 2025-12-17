//! Rendering utilities for the graphical frontend

use macroquad::prelude::*;
use crate::game::Game;
use crate::world::{TileType, Biome};
use crate::ecs::{Position, Renderable, Health, Mana, Stamina, Name, Player, Experience, EquipmentComponent};
use crate::game::message::MessageCategory;
use super::colors;

/// Tile size in pixels
pub const TILE_SIZE: f32 = 24.0;

/// Camera offset for centering on player
pub struct Camera {
    pub x: f32,
    pub y: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn center_on(&mut self, pos: Position, screen_width: f32, screen_height: f32) {
        self.x = pos.x as f32 * TILE_SIZE - screen_width / 2.0 + TILE_SIZE / 2.0;
        self.y = pos.y as f32 * TILE_SIZE - screen_height / 2.0 + TILE_SIZE / 2.0;
    }
}

/// Get the color for a tile type
pub fn tile_color(tile_type: TileType, biome: Biome) -> Color {
    let config = biome.config();

    match tile_type {
        TileType::Floor => colors::rgb(config.floor_color.0, config.floor_color.1, config.floor_color.2),
        TileType::Wall => colors::rgb(config.wall_color.0, config.wall_color.1, config.wall_color.2),
        TileType::Corridor => colors::rgb(config.corridor_color.0, config.corridor_color.1, config.corridor_color.2),
        TileType::StairsDown | TileType::StairsUp => colors::STAIRS,
        TileType::DoorClosed | TileType::DoorOpen => colors::DOOR,
        TileType::Lava => colors::LAVA,
        TileType::Pit => colors::PIT,
        TileType::Torch => colors::TORCH,
        TileType::Brazier => colors::BRAZIER,
        TileType::BloodStain => colors::BLOOD,
        TileType::Bones => colors::BONES,
        TileType::ShrineRest | TileType::ShrineSkill | TileType::ShrineEnchant => Color::new(0.6, 0.8, 0.6, 1.0),
        TileType::ShrineCorruption => colors::CORRUPTION,
        TileType::Rubble | TileType::Cracks => colors::rgb(config.floor_color_alt.0, config.floor_color_alt.1, config.floor_color_alt.2),
        TileType::Cobweb => Color::new(0.7, 0.7, 0.7, 0.6),
        TileType::Grime | TileType::Ashes => Color::new(0.3, 0.3, 0.25, 1.0),
        TileType::ShopFloor => Color::new(0.4, 0.35, 0.25, 1.0),
        _ => colors::FLOOR,
    }
}

/// Get the glyph character for a tile type (for text fallback)
pub fn tile_glyph(tile_type: TileType) -> char {
    match tile_type {
        TileType::Floor | TileType::Corridor => '.',
        TileType::Wall => '#',
        TileType::StairsDown => '>',
        TileType::StairsUp => '<',
        TileType::DoorClosed => '+',
        TileType::DoorOpen => '/',
        TileType::Lava => '~',
        TileType::Pit => ' ',
        TileType::Torch | TileType::Brazier => '†',
        TileType::BloodStain => '·',
        TileType::Bones => '%',
        TileType::ShrineRest => '☼',
        TileType::ShrineSkill => '☆',
        TileType::ShrineEnchant => '◊',
        TileType::ShrineCorruption => '✧',
        TileType::Rubble | TileType::Cracks => ',',
        TileType::Cobweb => '~',
        _ => '?',
    }
}

/// Render the game map
pub fn render_map(game: &Game, camera: &Camera, view_area: Rect) {
    let map = game.map();
    let biome = game.biome();

    // Calculate visible tile range
    let start_x = (camera.x / TILE_SIZE).floor() as i32 - 1;
    let start_y = (camera.y / TILE_SIZE).floor() as i32 - 1;
    let end_x = ((camera.x + view_area.w) / TILE_SIZE).ceil() as i32 + 1;
    let end_y = ((camera.y + view_area.h) / TILE_SIZE).ceil() as i32 + 1;

    // Draw tiles
    for y in start_y..=end_y {
        for x in start_x..=end_x {
            if x < 0 || y < 0 || x >= map.width as i32 || y >= map.height as i32 {
                continue;
            }

            let screen_x = view_area.x + x as f32 * TILE_SIZE - camera.x;
            let screen_y = view_area.y + y as f32 * TILE_SIZE - camera.y;

            // Skip if outside view area
            if screen_x + TILE_SIZE < view_area.x || screen_x > view_area.x + view_area.w
                || screen_y + TILE_SIZE < view_area.y || screen_y > view_area.y + view_area.h
            {
                continue;
            }

            let pos = Position::new(x, y);

            // Check visibility
            if !map.is_explored(pos) {
                draw_rectangle(screen_x, screen_y, TILE_SIZE, TILE_SIZE, colors::FOG_HIDDEN);
                continue;
            }

            let in_fov = map.is_visible(pos);
            let tile = map.get_tile(x, y);

            if let Some(tile) = tile {
                let mut color = tile_color(tile.tile_type, biome);

                // Dim if not in FOV
                if !in_fov {
                    color = Color::new(color.r * 0.4, color.g * 0.4, color.b * 0.4, color.a);
                }

                draw_rectangle(screen_x, screen_y, TILE_SIZE, TILE_SIZE, color);

                // Draw tile glyph for detailed tiles
                if in_fov {
                    match tile.tile_type {
                        TileType::StairsDown | TileType::StairsUp |
                        TileType::ShrineRest | TileType::ShrineSkill |
                        TileType::ShrineEnchant | TileType::ShrineCorruption => {
                            let glyph = tile_glyph(tile.tile_type);
                            draw_text(
                                &glyph.to_string(),
                                screen_x + 6.0,
                                screen_y + 18.0,
                                20.0,
                                colors::TEXT_PRIMARY,
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Render all visible entities
pub fn render_entities(game: &Game, camera: &Camera, view_area: Rect) {
    let map = game.map();
    let world = game.world();

    // Collect and sort entities by render order
    let mut entities: Vec<_> = world
        .query::<(&Position, &Renderable)>()
        .iter()
        .filter(|(_, (pos, _))| map.is_visible(**pos))
        .collect();

    // Sort by render order (lower first)
    entities.sort_by_key(|(_, (_, r))| r.render_order);

    for (entity, (pos, renderable)) in entities {
        let screen_x = view_area.x + pos.x as f32 * TILE_SIZE - camera.x;
        let screen_y = view_area.y + pos.y as f32 * TILE_SIZE - camera.y;

        // Skip if outside view area
        if screen_x + TILE_SIZE < view_area.x || screen_x > view_area.x + view_area.w
            || screen_y + TILE_SIZE < view_area.y || screen_y > view_area.y + view_area.h
        {
            continue;
        }

        // Get entity color
        let color = colors::rgb(renderable.color.0, renderable.color.1, renderable.color.2);

        // Draw entity background (slight highlight)
        let is_player = world.get::<&Player>(entity).is_ok();
        if is_player {
            draw_rectangle(screen_x + 2.0, screen_y + 2.0, TILE_SIZE - 4.0, TILE_SIZE - 4.0, Color::new(0.2, 0.2, 0.15, 0.5));
        }

        // Draw entity glyph
        draw_text(
            &renderable.glyph.to_string(),
            screen_x + 4.0,
            screen_y + 18.0,
            22.0,
            color,
        );

        // Draw health bar for enemies (not player)
        if !is_player {
            if let Ok(health) = world.get::<&Health>(entity) {
                let hp_pct = health.current as f32 / health.max as f32;
                if hp_pct < 1.0 {
                    let bar_width = TILE_SIZE - 4.0;
                    let bar_height = 3.0;
                    let bar_y = screen_y + TILE_SIZE - 5.0;

                    // Background
                    draw_rectangle(screen_x + 2.0, bar_y, bar_width, bar_height, Color::new(0.2, 0.0, 0.0, 0.8));
                    // Health
                    let hp_color = if hp_pct > 0.5 { colors::HEALTH_HIGH } else if hp_pct > 0.25 { colors::HEALTH_MED } else { colors::HEALTH_LOW };
                    draw_rectangle(screen_x + 2.0, bar_y, bar_width * hp_pct, bar_height, hp_color);
                }
            }
        }
    }
}

/// Render the player status panel
pub fn render_status_panel(game: &Game, area: Rect) {
    // Panel background
    draw_rectangle(area.x, area.y, area.w, area.h, colors::PANEL_BG);
    draw_rectangle_lines(area.x, area.y, area.w, area.h, 2.0, colors::PANEL_BORDER);

    let padding = 10.0;
    let mut y = area.y + padding;
    let line_height = 22.0;

    // Get player data
    let health = game.player_health().unwrap_or(Health::new(100));
    let mana = game.player_mana().unwrap_or(Mana::new(50));
    let stamina = game.player_stamina().unwrap_or(Stamina::new(50));
    let stats = game.player_stats().unwrap_or(crate::ecs::Stats::player_base());
    let xp = game.player_experience().unwrap_or(Experience::new());

    // Get equipment bonuses
    let (eq_hp, eq_mp) = if let Some(player) = game.player() {
        game.world().get::<&EquipmentComponent>(player)
            .map(|eq| (eq.equipment.hp_bonus(), eq.equipment.mp_bonus()))
            .unwrap_or((0, 0))
    } else {
        (0, 0)
    };

    let effective_max_hp = health.max + eq_hp;
    let effective_max_mp = mana.max + eq_mp;

    // Title
    draw_text("HERO", area.x + padding, y + 16.0, 24.0, colors::TEXT_PRIMARY);
    y += line_height + 5.0;

    // Level
    draw_text(&format!("Level {}", xp.level), area.x + padding, y + 14.0, 18.0, colors::XP);
    y += line_height;

    // Health bar
    let bar_width = area.w - padding * 2.0;
    let bar_height = 16.0;
    let hp_pct = health.current as f32 / effective_max_hp as f32;
    let hp_color = if hp_pct > 0.6 { colors::HEALTH_HIGH } else if hp_pct > 0.3 { colors::HEALTH_MED } else { colors::HEALTH_LOW };

    draw_text("HP", area.x + padding, y + 12.0, 14.0, colors::TEXT_MUTED);
    y += 14.0;
    draw_rectangle(area.x + padding, y, bar_width, bar_height, Color::new(0.15, 0.0, 0.0, 1.0));
    draw_rectangle(area.x + padding, y, bar_width * hp_pct, bar_height, hp_color);
    draw_text(
        &format!("{}/{}", health.current, effective_max_hp),
        area.x + padding + 4.0,
        y + 12.0,
        14.0,
        colors::TEXT_PRIMARY,
    );
    y += bar_height + 6.0;

    // Mana bar
    let mp_pct = mana.current as f32 / effective_max_mp as f32;
    draw_text("MP", area.x + padding, y + 12.0, 14.0, colors::TEXT_MUTED);
    y += 14.0;
    draw_rectangle(area.x + padding, y, bar_width, bar_height, Color::new(0.0, 0.0, 0.15, 1.0));
    draw_rectangle(area.x + padding, y, bar_width * mp_pct, bar_height, colors::MANA);
    draw_text(
        &format!("{}/{}", mana.current, effective_max_mp),
        area.x + padding + 4.0,
        y + 12.0,
        14.0,
        colors::TEXT_PRIMARY,
    );
    y += bar_height + 6.0;

    // Stamina bar
    let sp_pct = stamina.current as f32 / stamina.max as f32;
    draw_text("SP", area.x + padding, y + 12.0, 14.0, colors::TEXT_MUTED);
    y += 14.0;
    draw_rectangle(area.x + padding, y, bar_width, bar_height, Color::new(0.15, 0.12, 0.0, 1.0));
    draw_rectangle(area.x + padding, y, bar_width * sp_pct, bar_height, colors::STAMINA);
    draw_text(
        &format!("{}/{}", stamina.current, stamina.max),
        area.x + padding + 4.0,
        y + 12.0,
        14.0,
        colors::TEXT_PRIMARY,
    );
    y += bar_height + 10.0;

    // XP bar
    let xp_pct = xp.current_xp as f32 / xp.xp_to_next as f32;
    draw_text("XP", area.x + padding, y + 12.0, 14.0, colors::TEXT_MUTED);
    y += 14.0;
    draw_rectangle(area.x + padding, y, bar_width, 8.0, Color::new(0.0, 0.1, 0.1, 1.0));
    draw_rectangle(area.x + padding, y, bar_width * xp_pct, 8.0, colors::XP);
    y += 16.0;

    // Stats
    draw_text("─ Stats ─", area.x + padding, y + 14.0, 14.0, colors::TEXT_MUTED);
    y += line_height;

    let stat_col1 = area.x + padding;
    let stat_col2 = area.x + padding + 70.0;

    draw_text(&format!("STR {}", stats.strength), stat_col1, y + 14.0, 14.0, Color::new(0.9, 0.4, 0.4, 1.0));
    draw_text(&format!("DEX {}", stats.dexterity), stat_col2, y + 14.0, 14.0, Color::new(0.4, 0.9, 0.4, 1.0));
    y += line_height;

    draw_text(&format!("INT {}", stats.intelligence), stat_col1, y + 14.0, 14.0, Color::new(0.4, 0.4, 0.9, 1.0));
    draw_text(&format!("VIT {}", stats.vitality), stat_col2, y + 14.0, 14.0, Color::new(0.9, 0.9, 0.4, 1.0));
    y += line_height + 5.0;

    // Floor info
    draw_text("─ Location ─", area.x + padding, y + 14.0, 14.0, colors::TEXT_MUTED);
    y += line_height;

    draw_text(&format!("Floor {}", game.floor()), area.x + padding, y + 14.0, 16.0, colors::TEXT_PRIMARY);
    y += line_height;

    let biome_config = game.biome().config();
    let biome_color = colors::rgb(
        biome_config.ambient_color.0,
        biome_config.ambient_color.1,
        biome_config.ambient_color.2,
    );
    draw_text(biome_config.name, area.x + padding, y + 14.0, 14.0, biome_color);
}

/// Render the message log panel
pub fn render_messages(game: &Game, area: Rect) {
    // Panel background
    draw_rectangle(area.x, area.y, area.w, area.h, colors::PANEL_BG);
    draw_rectangle_lines(area.x, area.y, area.w, area.h, 2.0, colors::PANEL_BORDER);

    let padding = 8.0;
    let line_height = 18.0;
    let max_messages = ((area.h - padding * 2.0) / line_height) as usize;

    let messages: Vec<_> = game.messages().iter().rev().take(max_messages).collect();

    for (i, msg) in messages.iter().rev().enumerate() {
        let y = area.y + padding + (i as f32) * line_height;

        let color = match msg.category {
            MessageCategory::Combat => colors::MSG_COMBAT,
            MessageCategory::Item => colors::MSG_ITEM,
            MessageCategory::System => colors::MSG_SYSTEM,
            MessageCategory::Lore => colors::MSG_LORE,
            MessageCategory::Warning => colors::MSG_WARNING,
        };

        // Truncate message if too long
        let max_chars = (area.w / 8.0) as usize;
        let text = if msg.text.len() > max_chars {
            format!("{}...", &msg.text[..max_chars - 3])
        } else {
            msg.text.clone()
        };

        draw_text(&text, area.x + padding, y + 14.0, 14.0, color);
    }
}

/// Render a simple tooltip at mouse position
pub fn render_tooltip(text: &str, x: f32, y: f32) {
    let padding = 6.0;
    let text_width = text.len() as f32 * 8.0;
    let width = text_width + padding * 2.0;
    let height = 24.0;

    // Adjust position to stay on screen
    let screen_w = screen_width();
    let screen_h = screen_height();
    let tooltip_x = if x + width > screen_w { screen_w - width - 5.0 } else { x };
    let tooltip_y = if y + height > screen_h { y - height - 5.0 } else { y + 20.0 };

    draw_rectangle(tooltip_x, tooltip_y, width, height, Color::new(0.1, 0.1, 0.12, 0.95));
    draw_rectangle_lines(tooltip_x, tooltip_y, width, height, 1.0, colors::PANEL_BORDER);
    draw_text(text, tooltip_x + padding, tooltip_y + 17.0, 14.0, colors::TEXT_PRIMARY);
}

/// Render the minimap
pub fn render_minimap(game: &Game, area: Rect) {
    let map = game.map();
    let player_pos = game.player_position().unwrap_or(Position::new(0, 0));

    // Panel background
    draw_rectangle(area.x, area.y, area.w, area.h, Color::new(0.0, 0.0, 0.0, 0.8));
    draw_rectangle_lines(area.x, area.y, area.w, area.h, 1.0, colors::PANEL_BORDER);

    let scale = 2.0; // Each tile is 2 pixels on minimap
    let half_w = (area.w / scale / 2.0) as i32;
    let half_h = (area.h / scale / 2.0) as i32;

    for dy in -half_h..half_h {
        for dx in -half_w..half_w {
            let x = player_pos.x + dx;
            let y = player_pos.y + dy;

            if x < 0 || y < 0 || x >= map.width as i32 || y >= map.height as i32 {
                continue;
            }

            let pos = Position::new(x, y);
            if !map.is_explored(pos) {
                continue;
            }

            let screen_x = area.x + (dx + half_w) as f32 * scale;
            let screen_y = area.y + (dy + half_h) as f32 * scale;

            let color = if x == player_pos.x && y == player_pos.y {
                colors::PLAYER
            } else if let Some(tile) = map.get_tile(x, y) {
                match tile.tile_type {
                    TileType::Wall => Color::new(0.4, 0.4, 0.35, 1.0),
                    TileType::Floor | TileType::Corridor => Color::new(0.2, 0.2, 0.18, 1.0),
                    TileType::StairsDown | TileType::StairsUp => colors::STAIRS,
                    _ => Color::new(0.25, 0.25, 0.22, 1.0),
                }
            } else {
                continue;
            };

            draw_rectangle(screen_x, screen_y, scale, scale, color);
        }
    }
}

/// Render the main menu
pub fn render_main_menu(selected: usize, difficulty_popup: bool, difficulty_cursor: usize) {
    let screen_w = screen_width();
    let screen_h = screen_height();

    // Background
    clear_background(colors::BACKGROUND);

    // Title
    let title = "HOLLOWDEEP";
    let title_size = 64.0;
    let title_width = title.len() as f32 * title_size * 0.5;
    draw_text(title, (screen_w - title_width) / 2.0, screen_h * 0.25, title_size, colors::TEXT_PRIMARY);

    // Subtitle
    let subtitle = "A Grimdark Roguelike";
    let sub_size = 20.0;
    let sub_width = subtitle.len() as f32 * sub_size * 0.5;
    draw_text(subtitle, (screen_w - sub_width) / 2.0, screen_h * 0.25 + 40.0, sub_size, colors::TEXT_MUTED);

    // Menu options
    let options = ["New Game", "Continue", "Options", "Quit"];
    let menu_y = screen_h * 0.5;
    let line_height = 40.0;

    for (i, option) in options.iter().enumerate() {
        let y = menu_y + i as f32 * line_height;
        let color = if i == selected { colors::TEXT_PRIMARY } else { colors::TEXT_SECONDARY };
        let prefix = if i == selected { "> " } else { "  " };
        let text = format!("{}{}", prefix, option);
        let text_width = text.len() as f32 * 12.0;
        draw_text(&text, (screen_w - text_width) / 2.0, y, 24.0, color);
    }

    // Controls hint
    draw_text(
        "Arrow Keys / Enter to Select",
        screen_w / 2.0 - 120.0,
        screen_h - 50.0,
        16.0,
        colors::TEXT_MUTED,
    );

    // Difficulty popup
    if difficulty_popup {
        let popup_w = 300.0;
        let popup_h = 200.0;
        let popup_x = (screen_w - popup_w) / 2.0;
        let popup_y = (screen_h - popup_h) / 2.0;

        draw_rectangle(popup_x, popup_y, popup_w, popup_h, colors::PANEL_BG);
        draw_rectangle_lines(popup_x, popup_y, popup_w, popup_h, 2.0, colors::PANEL_BORDER);

        draw_text("Select Difficulty", popup_x + 60.0, popup_y + 30.0, 20.0, colors::TEXT_PRIMARY);

        let difficulties = ["Easy", "Normal", "Hard", "Nightmare"];
        let diff_colors = [colors::HEALTH_HIGH, colors::TEXT_PRIMARY, colors::HEALTH_MED, colors::HEALTH_LOW];

        for (i, (diff, color)) in difficulties.iter().zip(diff_colors.iter()).enumerate() {
            let y = popup_y + 60.0 + i as f32 * 30.0;
            let prefix = if i == difficulty_cursor { "> " } else { "  " };
            let text_color = if i == difficulty_cursor { *color } else { colors::TEXT_MUTED };
            draw_text(&format!("{}{}", prefix, diff), popup_x + 100.0, y + 20.0, 18.0, text_color);
        }
    }
}

/// Render a pause/game over overlay
pub fn render_overlay(title: &str, message: &str) {
    let screen_w = screen_width();
    let screen_h = screen_height();

    // Dim background
    draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::new(0.0, 0.0, 0.0, 0.7));

    // Panel
    let panel_w = 400.0;
    let panel_h = 200.0;
    let panel_x = (screen_w - panel_w) / 2.0;
    let panel_y = (screen_h - panel_h) / 2.0;

    draw_rectangle(panel_x, panel_y, panel_w, panel_h, colors::PANEL_BG);
    draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, colors::PANEL_BORDER);

    // Title
    let title_width = title.len() as f32 * 16.0;
    draw_text(title, (screen_w - title_width) / 2.0, panel_y + 50.0, 32.0, colors::TEXT_PRIMARY);

    // Message
    let msg_width = message.len() as f32 * 8.0;
    draw_text(message, (screen_w - msg_width) / 2.0, panel_y + 100.0, 16.0, colors::TEXT_SECONDARY);

    // Continue hint
    draw_text(
        "Press Enter to continue",
        panel_x + 110.0,
        panel_y + 160.0,
        16.0,
        colors::TEXT_MUTED,
    );
}
