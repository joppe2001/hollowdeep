//! Audio manager using Kira
//!
//! Handles loading and playing sound effects.

use std::collections::HashMap;
use std::path::Path;

use kira::{
    manager::{AudioManager as KiraManager, AudioManagerSettings, backend::DefaultBackend},
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
    Volume,
};

use super::sounds::SoundId;

/// Audio manager that handles all sound playback
pub struct AudioManager {
    /// Kira audio manager
    manager: Option<KiraManager>,
    /// Preloaded sound data
    sounds: HashMap<SoundId, StaticSoundData>,
    /// Master volume (0.0 - 1.0)
    master_volume: f64,
    /// SFX volume multiplier (0.0 - 1.0)
    sfx_volume: f64,
    /// Whether audio is enabled
    enabled: bool,
}

impl AudioManager {
    /// Create a new audio manager
    pub fn new() -> Self {
        let manager = match KiraManager::<DefaultBackend>::new(AudioManagerSettings::default()) {
            Ok(m) => {
                log::info!("Audio manager initialized successfully");
                Some(m)
            }
            Err(e) => {
                log::warn!("Failed to initialize audio manager: {}. Audio disabled.", e);
                None
            }
        };

        let mut audio = Self {
            manager,
            sounds: HashMap::new(),
            master_volume: 1.0,
            sfx_volume: 0.7,
            enabled: true,
        };

        // Try to preload common sounds
        audio.preload_sounds();

        audio
    }

    /// Preload commonly used sounds
    fn preload_sounds(&mut self) {
        // List of sounds to preload
        let sounds_to_preload = [
            SoundId::Hit,
            SoundId::Miss,
            SoundId::Critical,
            SoundId::Dodge,
            SoundId::EnemyDeath,
            SoundId::PlayerHurt,
            SoundId::ItemPickup,
            SoundId::GoldPickup,
            SoundId::ChestOpen,
            SoundId::MenuMove,
            SoundId::MenuSelect,
            SoundId::MenuBack,
            SoundId::LevelUp,
        ];

        for sound_id in sounds_to_preload {
            if let Err(e) = self.load_sound(sound_id) {
                log::debug!("Could not preload sound {:?}: {}", sound_id, e);
            }
        }
    }

    /// Load a sound from file
    fn load_sound(&mut self, sound_id: SoundId) -> Result<(), String> {
        if self.sounds.contains_key(&sound_id) {
            return Ok(()); // Already loaded
        }

        let path = sound_id.file_path();
        if !Path::new(path).exists() {
            return Err(format!("Sound file not found: {}", path));
        }

        match StaticSoundData::from_file(path) {
            Ok(data) => {
                self.sounds.insert(sound_id, data);
                Ok(())
            }
            Err(e) => Err(format!("Failed to load sound {}: {:?}", path, e)),
        }
    }

    /// Play a sound effect
    pub fn play(&mut self, sound_id: SoundId) {
        if !self.enabled || self.manager.is_none() {
            return;
        }

        // Try to load if not already loaded (do this before getting manager reference)
        if !self.sounds.contains_key(&sound_id) {
            if let Err(e) = self.load_sound(sound_id) {
                log::debug!("Cannot play sound {:?}: {}", sound_id, e);
                return;
            }
        }

        // Get the sound data
        let sound_data = match self.sounds.get(&sound_id) {
            Some(data) => data.clone(),
            None => return,
        };

        // Calculate final volume
        let base_volume = sound_id.default_volume();
        let final_volume = base_volume * self.sfx_volume * self.master_volume;

        // Play the sound
        let settings = StaticSoundSettings::new().volume(Volume::Amplitude(final_volume));
        let sound_with_settings = sound_data.with_settings(settings);

        if let Some(manager) = &mut self.manager {
            if let Err(e) = manager.play(sound_with_settings) {
                log::debug!("Failed to play sound {:?}: {:?}", sound_id, e);
            }
        }
    }

    /// Play a sound with custom volume multiplier
    pub fn play_with_volume(&mut self, sound_id: SoundId, volume_multiplier: f64) {
        if !self.enabled || self.manager.is_none() {
            return;
        }

        // Try to load if not already loaded (do this before getting manager reference)
        if !self.sounds.contains_key(&sound_id) {
            if let Err(e) = self.load_sound(sound_id) {
                log::debug!("Cannot play sound {:?}: {}", sound_id, e);
                return;
            }
        }

        // Get the sound data
        let sound_data = match self.sounds.get(&sound_id) {
            Some(data) => data.clone(),
            None => return,
        };

        // Calculate final volume
        let base_volume = sound_id.default_volume();
        let final_volume = base_volume * self.sfx_volume * self.master_volume * volume_multiplier;

        // Play the sound
        let settings = StaticSoundSettings::new().volume(Volume::Amplitude(final_volume));
        let sound_with_settings = sound_data.with_settings(settings);

        if let Some(manager) = &mut self.manager {
            if let Err(e) = manager.play(sound_with_settings) {
                log::debug!("Failed to play sound {:?}: {:?}", sound_id, e);
            }
        }
    }

    /// Set master volume (0.0 - 1.0)
    pub fn set_master_volume(&mut self, volume: f64) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Get master volume
    pub fn master_volume(&self) -> f64 {
        self.master_volume
    }

    /// Set SFX volume (0.0 - 1.0)
    pub fn set_sfx_volume(&mut self, volume: f64) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
    }

    /// Get SFX volume
    pub fn sfx_volume(&self) -> f64 {
        self.sfx_volume
    }

    /// Enable or disable audio
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if audio is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled && self.manager.is_some()
    }

    /// Check if audio backend is available
    pub fn is_available(&self) -> bool {
        self.manager.is_some()
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

// Note: AudioManager contains Kira's manager which isn't Send/Sync,
// so we need to be careful about thread safety. In this single-threaded
// game, this isn't a concern.
