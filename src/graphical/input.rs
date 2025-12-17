//! Input handling for the graphical frontend

use macroquad::prelude::*;

/// Input action that can be triggered by the player
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    // Movement
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveUpLeft,
    MoveUpRight,
    MoveDownLeft,
    MoveDownRight,
    Wait,

    // Actions
    Interact,
    Attack,
    PickUp,

    // UI Navigation
    Confirm,
    Cancel,

    // Menus
    Inventory,
    Character,
    Skills,
    Map,
    Help,
    Pause,

    // Skills (1-5)
    Skill1,
    Skill2,
    Skill3,
    Skill4,
    Skill5,

    // Stat allocation
    StatStr,
    StatDex,
    StatInt,
    StatVit,

    // Scroll
    ScrollUp,
    ScrollDown,

    // Mouse
    LeftClick,
    RightClick,

    // System
    Quit,
}

/// Get the current input action based on keyboard state
pub fn get_input_action() -> Option<InputAction> {
    // Movement keys (arrow keys and vim-style)
    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::K) {
        return Some(InputAction::MoveUp);
    }
    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::J) {
        return Some(InputAction::MoveDown);
    }
    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::H) {
        return Some(InputAction::MoveLeft);
    }
    if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::L) {
        return Some(InputAction::MoveRight);
    }

    // Diagonal movement (vim-style)
    if is_key_pressed(KeyCode::Y) {
        return Some(InputAction::MoveUpLeft);
    }
    if is_key_pressed(KeyCode::U) {
        return Some(InputAction::MoveUpRight);
    }
    if is_key_pressed(KeyCode::B) {
        return Some(InputAction::MoveDownLeft);
    }
    if is_key_pressed(KeyCode::N) {
        return Some(InputAction::MoveDownRight);
    }

    // Wait
    if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Period) {
        return Some(InputAction::Wait);
    }

    // Interact / Confirm
    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::E) {
        return Some(InputAction::Confirm);
    }

    // Cancel / Back
    if is_key_pressed(KeyCode::Escape) {
        return Some(InputAction::Cancel);
    }

    // Menus
    if is_key_pressed(KeyCode::I) {
        return Some(InputAction::Inventory);
    }
    if is_key_pressed(KeyCode::C) {
        return Some(InputAction::Character);
    }
    if is_key_pressed(KeyCode::S) && is_key_down(KeyCode::LeftShift) {
        return Some(InputAction::Skills);
    }
    if is_key_pressed(KeyCode::M) {
        return Some(InputAction::Map);
    }
    if is_key_pressed(KeyCode::Slash) && is_key_down(KeyCode::LeftShift) {
        return Some(InputAction::Help);
    }
    if is_key_pressed(KeyCode::P) {
        return Some(InputAction::Pause);
    }

    // Skills
    if is_key_pressed(KeyCode::Key1) {
        return Some(InputAction::Skill1);
    }
    if is_key_pressed(KeyCode::Key2) {
        return Some(InputAction::Skill2);
    }
    if is_key_pressed(KeyCode::Key3) {
        return Some(InputAction::Skill3);
    }
    if is_key_pressed(KeyCode::Key4) {
        return Some(InputAction::Skill4);
    }
    if is_key_pressed(KeyCode::Key5) {
        return Some(InputAction::Skill5);
    }

    // Pick up
    if is_key_pressed(KeyCode::G) || is_key_pressed(KeyCode::Comma) {
        return Some(InputAction::PickUp);
    }

    // Quit
    if is_key_pressed(KeyCode::Q) && is_key_down(KeyCode::LeftControl) {
        return Some(InputAction::Quit);
    }

    // Mouse
    if is_mouse_button_pressed(MouseButton::Left) {
        return Some(InputAction::LeftClick);
    }
    if is_mouse_button_pressed(MouseButton::Right) {
        return Some(InputAction::RightClick);
    }

    None
}

/// Get menu navigation input
pub fn get_menu_input() -> Option<InputAction> {
    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::K) {
        return Some(InputAction::ScrollUp);
    }
    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::J) {
        return Some(InputAction::ScrollDown);
    }
    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
        return Some(InputAction::Confirm);
    }
    if is_key_pressed(KeyCode::Escape) {
        return Some(InputAction::Cancel);
    }

    // Number keys for stat allocation
    if is_key_pressed(KeyCode::Key1) {
        return Some(InputAction::StatStr);
    }
    if is_key_pressed(KeyCode::Key2) {
        return Some(InputAction::StatDex);
    }
    if is_key_pressed(KeyCode::Key3) {
        return Some(InputAction::StatInt);
    }
    if is_key_pressed(KeyCode::Key4) {
        return Some(InputAction::StatVit);
    }

    None
}

/// Get mouse position in screen coordinates
pub fn mouse_screen_pos() -> (f32, f32) {
    mouse_position()
}

/// Check if a point is within a rectangle
pub fn point_in_rect(x: f32, y: f32, rx: f32, ry: f32, rw: f32, rh: f32) -> bool {
    x >= rx && x <= rx + rw && y >= ry && y <= ry + rh
}
