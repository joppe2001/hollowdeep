//! Audio system
//!
//! Provides sound effects using the Kira audio library.

pub mod manager;
pub mod sounds;

pub use manager::AudioManager;
pub use sounds::{SoundId, SoundCategory};
