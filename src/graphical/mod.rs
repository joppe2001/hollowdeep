//! Graphical frontend using macroquad
//!
//! Alternative to the terminal UI, provides a windowed graphical experience.

mod app;
mod renderer;
mod input;
mod colors;

pub use app::run_graphical;
