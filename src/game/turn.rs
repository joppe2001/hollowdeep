//! Turn management for combat
//!
//! Handles turn order, action points, and combat flow.

use hecs::Entity;

/// Manages turn order during combat
pub struct TurnManager {
    /// Entities in turn order (sorted by speed)
    turn_order: Vec<Entity>,
    /// Index of current entity's turn
    current_index: usize,
    /// Current round number
    round: u32,
}

impl TurnManager {
    /// Create a new turn manager
    pub fn new() -> Self {
        Self {
            turn_order: Vec::new(),
            current_index: 0,
            round: 1,
        }
    }

    /// Initialize turn order with combatants sorted by speed
    pub fn initialize(&mut self, mut combatants: Vec<(Entity, i32)>) {
        // Sort by speed (higher = earlier in turn order)
        combatants.sort_by(|a, b| b.1.cmp(&a.1));
        self.turn_order = combatants.into_iter().map(|(e, _)| e).collect();
        self.current_index = 0;
        self.round = 1;
    }

    /// Get the entity whose turn it is
    pub fn current_entity(&self) -> Option<Entity> {
        self.turn_order.get(self.current_index).copied()
    }

    /// Advance to the next turn
    pub fn next_turn(&mut self) {
        self.current_index += 1;
        if self.current_index >= self.turn_order.len() {
            self.current_index = 0;
            self.round += 1;
        }
    }

    /// Remove an entity from combat (died, fled, etc.)
    pub fn remove_entity(&mut self, entity: Entity) {
        if let Some(pos) = self.turn_order.iter().position(|&e| e == entity) {
            self.turn_order.remove(pos);
            // Adjust current index if needed
            if pos < self.current_index && self.current_index > 0 {
                self.current_index -= 1;
            }
            // Handle case where we removed the last entity
            if self.current_index >= self.turn_order.len() && !self.turn_order.is_empty() {
                self.current_index = 0;
                self.round += 1;
            }
        }
    }

    /// Check if combat is over (only one faction remains)
    pub fn is_combat_over(&self) -> bool {
        // TODO: Check if only player entities or only enemy entities remain
        self.turn_order.len() <= 1
    }

    /// Get current round number
    pub fn round(&self) -> u32 {
        self.round
    }

    /// Get remaining combatants
    pub fn combatants(&self) -> &[Entity] {
        &self.turn_order
    }
}

impl Default for TurnManager {
    fn default() -> Self {
        Self::new()
    }
}
