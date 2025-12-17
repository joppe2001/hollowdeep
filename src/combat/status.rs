//! Status effects system
//!
//! Handles DoT effects (poison, burn, bleed) and buff/debuff application.

use crate::ecs::{StatusEffects, StatusEffect, StatusEffectType, Health};

/// Result of ticking status effects
#[derive(Debug, Clone, Default)]
pub struct StatusTickResult {
    /// Total damage dealt from DoT effects
    pub damage_dealt: i32,
    /// Messages to display
    pub messages: Vec<String>,
    /// Whether any effects expired
    pub effects_expired: bool,
}

impl StatusEffects {
    /// Add a status effect (stacking or refreshing duration)
    pub fn add_effect(&mut self, effect_type: StatusEffectType, duration: f32, intensity: i32) {
        // Check if effect already exists
        if let Some(existing) = self.effects.iter_mut().find(|e| e.effect_type == effect_type) {
            // Refresh duration and increase intensity (stacking)
            existing.duration = existing.duration.max(duration);
            existing.intensity = (existing.intensity + intensity / 2).min(intensity * 3); // Cap stacking
        } else {
            self.effects.push(StatusEffect {
                effect_type,
                duration,
                intensity,
            });
        }
    }

    /// Remove all effects of a specific type
    pub fn remove_effect(&mut self, effect_type: StatusEffectType) {
        self.effects.retain(|e| e.effect_type != effect_type);
    }

    /// Check if an effect is active
    pub fn has_effect(&self, effect_type: StatusEffectType) -> bool {
        self.effects.iter().any(|e| e.effect_type == effect_type)
    }

    /// Get intensity of an effect (0 if not present)
    pub fn effect_intensity(&self, effect_type: StatusEffectType) -> i32 {
        self.effects
            .iter()
            .find(|e| e.effect_type == effect_type)
            .map(|e| e.intensity)
            .unwrap_or(0)
    }

    /// Tick all status effects (call on movement/turn)
    /// Returns total DoT damage and messages
    pub fn tick(&mut self, entity_name: &str) -> StatusTickResult {
        let mut result = StatusTickResult::default();

        for effect in &mut self.effects {
            // Reduce duration (1.0 per tick = 1 second worth)
            effect.duration -= 1.0;

            // Apply DoT damage based on effect type
            match effect.effect_type {
                StatusEffectType::Poison => {
                    result.damage_dealt += effect.intensity;
                    result.messages.push(format!("{} takes {} poison damage!", entity_name, effect.intensity));
                }
                StatusEffectType::Burn => {
                    let burn_dmg = effect.intensity + 1; // Burns hit a bit harder
                    result.damage_dealt += burn_dmg;
                    result.messages.push(format!("{} burns for {} damage!", entity_name, burn_dmg));
                }
                StatusEffectType::Bleed => {
                    result.damage_dealt += effect.intensity;
                    result.messages.push(format!("{} bleeds for {} damage!", entity_name, effect.intensity));
                }
                StatusEffectType::Regeneration => {
                    // Negative damage = healing
                    result.damage_dealt -= effect.intensity;
                    result.messages.push(format!("{} regenerates {} HP!", entity_name, effect.intensity));
                }
                // Other effects don't do damage per tick
                _ => {}
            }
        }

        // Remove expired effects
        let before_count = self.effects.len();
        self.effects.retain(|e| e.duration > 0.0);
        if self.effects.len() < before_count {
            result.effects_expired = true;
        }

        result
    }

    /// Clear all effects
    pub fn clear(&mut self) {
        self.effects.clear();
    }
}

impl StatusEffectType {
    /// Get display name for this effect
    pub fn name(&self) -> &'static str {
        match self {
            StatusEffectType::Poison => "Poison",
            StatusEffectType::Burn => "Burn",
            StatusEffectType::Bleed => "Bleed",
            StatusEffectType::Slow => "Slow",
            StatusEffectType::Weakness => "Weakness",
            StatusEffectType::Curse => "Curse",
            StatusEffectType::Regeneration => "Regen",
            StatusEffectType::Haste => "Haste",
            StatusEffectType::Shield => "Shield",
            StatusEffectType::Strength => "Strength",
        }
    }

    /// Get display color for this effect (RGB)
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            StatusEffectType::Poison => (100, 200, 100),   // Green
            StatusEffectType::Burn => (255, 100, 50),      // Orange-red
            StatusEffectType::Bleed => (200, 50, 50),      // Dark red
            StatusEffectType::Slow => (100, 100, 200),     // Blue
            StatusEffectType::Weakness => (150, 100, 150), // Purple-gray
            StatusEffectType::Curse => (150, 50, 150),     // Dark purple
            StatusEffectType::Regeneration => (100, 255, 100), // Bright green
            StatusEffectType::Haste => (255, 255, 100),    // Yellow
            StatusEffectType::Shield => (100, 200, 255),   // Light blue
            StatusEffectType::Strength => (255, 150, 100), // Orange
        }
    }

    /// Is this a beneficial effect?
    pub fn is_buff(&self) -> bool {
        matches!(
            self,
            StatusEffectType::Regeneration
                | StatusEffectType::Haste
                | StatusEffectType::Shield
                | StatusEffectType::Strength
        )
    }

    /// Is this a DoT effect?
    pub fn is_dot(&self) -> bool {
        matches!(
            self,
            StatusEffectType::Poison | StatusEffectType::Burn | StatusEffectType::Bleed
        )
    }
}

/// Apply status effect tick to an entity's health
/// Returns the actual damage/healing applied
pub fn apply_status_damage(health: &mut Health, tick_result: &StatusTickResult) -> i32 {
    if tick_result.damage_dealt > 0 {
        // Damage
        health.take_damage(tick_result.damage_dealt)
    } else if tick_result.damage_dealt < 0 {
        // Healing (negative damage)
        health.heal(-tick_result.damage_dealt)
    } else {
        0
    }
}
