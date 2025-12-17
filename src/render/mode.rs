//! Render mode detection and configuration
//!
//! Automatically detects terminal capabilities and selects the best rendering mode.

use std::env;

/// Available rendering modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderMode {
    /// Classic ASCII characters (@ # . etc.)
    /// Works everywhere, nostalgic feel
    #[default]
    Ascii,

    /// Unicode symbols (◆ ● ▲ ═ etc.)
    /// Better visuals, wide terminal support
    Unicode,

    /// Nerd Font icons
    /// Requires user to have Nerd Font installed
    NerdFont,

    /// Kitty Graphics Protocol
    /// Full image/sprite support, best visuals
    /// Supported by: Ghostty, Kitty, WezTerm, iTerm2
    Kitty,
}

impl RenderMode {
    /// Get a human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            RenderMode::Ascii => "ASCII",
            RenderMode::Unicode => "Unicode",
            RenderMode::NerdFont => "Nerd Font",
            RenderMode::Kitty => "Kitty Graphics",
        }
    }

    /// Check if this mode supports images
    pub fn supports_images(&self) -> bool {
        matches!(self, RenderMode::Kitty)
    }

    /// Check if this mode supports colors beyond basic 16
    pub fn supports_true_color(&self) -> bool {
        // All modes support true color in modern terminals
        true
    }
}

/// Detect the best rendering mode for the current terminal
pub fn detect_render_mode() -> RenderMode {
    // Check for Kitty graphics protocol support
    if is_kitty_supported() {
        log::info!("Detected Kitty graphics protocol support");
        return RenderMode::Kitty;
    }

    // Default to Unicode for modern terminals
    if is_unicode_supported() {
        log::info!("Using Unicode rendering mode");
        return RenderMode::Unicode;
    }

    // Fallback to ASCII
    log::info!("Falling back to ASCII rendering mode");
    RenderMode::Ascii
}

/// Check if Kitty graphics protocol is likely supported
fn is_kitty_supported() -> bool {
    // Check TERM environment variable
    if let Ok(term) = env::var("TERM") {
        let term_lower = term.to_lowercase();
        if term_lower.contains("kitty") || term_lower.contains("ghostty") {
            return true;
        }
    }

    // Check TERM_PROGRAM for additional terminals
    if let Ok(term_prog) = env::var("TERM_PROGRAM") {
        let prog_lower = term_prog.to_lowercase();
        if prog_lower.contains("kitty")
            || prog_lower.contains("ghostty")
            || prog_lower.contains("wezterm")
            || prog_lower.contains("iterm")
        {
            return true;
        }
    }

    // Check for KITTY_WINDOW_ID (set by Kitty terminal)
    if env::var("KITTY_WINDOW_ID").is_ok() {
        return true;
    }

    // Check for GHOSTTY_RESOURCES_DIR (set by Ghostty)
    if env::var("GHOSTTY_RESOURCES_DIR").is_ok() {
        return true;
    }

    // Check WEZTERM_PANE (set by WezTerm)
    if env::var("WEZTERM_PANE").is_ok() {
        return true;
    }

    false
}

/// Check if Unicode is well-supported
fn is_unicode_supported() -> bool {
    // Check LANG for UTF-8
    if let Ok(lang) = env::var("LANG") {
        if lang.to_uppercase().contains("UTF") {
            return true;
        }
    }

    // Check LC_ALL
    if let Ok(lc) = env::var("LC_ALL") {
        if lc.to_uppercase().contains("UTF") {
            return true;
        }
    }

    // Modern terminals generally support Unicode
    // Default to true unless on very old systems
    true
}

/// Terminal capabilities info
#[derive(Debug, Clone)]
pub struct TerminalCapabilities {
    pub render_mode: RenderMode,
    pub true_color: bool,
    pub cell_size: Option<(u16, u16)>, // Cell size in pixels (if known)
    pub terminal_name: String,
}

impl TerminalCapabilities {
    /// Detect all terminal capabilities
    pub fn detect() -> Self {
        let render_mode = detect_render_mode();

        let terminal_name = env::var("TERM_PROGRAM")
            .or_else(|_| env::var("TERM"))
            .unwrap_or_else(|_| "unknown".to_string());

        Self {
            render_mode,
            true_color: detect_true_color(),
            cell_size: None, // Could query via escape sequences
            terminal_name,
        }
    }
}

/// Check if true color (24-bit) is supported
fn detect_true_color() -> bool {
    if let Ok(colorterm) = env::var("COLORTERM") {
        let ct = colorterm.to_lowercase();
        if ct.contains("truecolor") || ct.contains("24bit") {
            return true;
        }
    }

    // Most modern terminals support true color
    true
}
