//! ECS Systems
//!
//! Game logic systems that operate on entities with specific components.

use hecs::World;
use rand::Rng;
use crate::ecs::{Position, AI, AIState, Enemy, Health, Name, BlocksMovement, StatusEffects, StatusEffectType};
use crate::world::Map;

/// Detection range for enemies to notice the player
const DETECTION_RANGE: i32 = 8;

/// Run AI for all enemies
pub fn run_enemy_ai(
    world: &mut World,
    map: &Map,
    player_pos: Position,
    rng: &mut impl Rng,
) -> Vec<AIAction> {
    let mut actions = Vec::new();

    // Collect all enemies with AI and their slow status (need to collect first to avoid borrow issues)
    let enemies: Vec<(hecs::Entity, Position, AIState, i32)> = world
        .query::<(&Position, &AI, &Enemy)>()
        .iter()
        .map(|(entity, (pos, ai, _))| {
            // Check if enemy is slowed
            let slow_intensity = world
                .get::<&StatusEffects>(entity)
                .ok()
                .map(|effects| effects.effect_intensity(StatusEffectType::Slow))
                .unwrap_or(0);
            (entity, *pos, ai.state, slow_intensity)
        })
        .collect();

    for (entity, enemy_pos, _current_state, slow_intensity) in enemies {
        // If slowed, chance to skip turn based on intensity
        // Intensity 1 = 50% skip, intensity 2 = 66% skip, intensity 3+ = 75% skip
        if slow_intensity > 0 {
            let skip_chance = match slow_intensity {
                1 => 0.50,
                2 => 0.66,
                _ => 0.75,
            };
            if rng.gen_bool(skip_chance) {
                continue; // Skip this enemy's turn due to slow
            }
        }

        let distance = enemy_pos.chebyshev_distance(&player_pos);

        // Update AI state based on distance
        let new_state = if distance <= 1 {
            AIState::Attack
        } else if distance <= DETECTION_RANGE {
            AIState::Chase
        } else {
            AIState::Idle
        };

        // Update the entity's AI state
        if let Ok(mut ai) = world.get::<&mut AI>(entity) {
            ai.state = new_state;
            ai.target = if new_state != AIState::Idle {
                Some(player_pos)
            } else {
                None
            };
        }

        // Generate action based on state
        match new_state {
            AIState::Attack => {
                actions.push(AIAction::Attack { attacker: entity, target_pos: player_pos });
            }
            AIState::Chase => {
                // Calculate move towards player
                if let Some(move_to) = calculate_chase_move(enemy_pos, player_pos, map, world) {
                    actions.push(AIAction::Move { entity, to: move_to });
                }
            }
            _ => {}
        }
    }

    actions
}

/// Calculate the best move for chasing the player
fn calculate_chase_move(
    from: Position,
    target: Position,
    map: &Map,
    world: &World,
) -> Option<Position> {
    let dx = (target.x - from.x).signum();
    let dy = (target.y - from.y).signum();

    // Try cardinal directions first (more predictable movement)
    let candidates = if dx != 0 && dy != 0 {
        // Diagonal case - try both cardinal directions
        vec![
            Position::new(from.x + dx, from.y),
            Position::new(from.x, from.y + dy),
            Position::new(from.x + dx, from.y + dy),
        ]
    } else if dx != 0 {
        vec![
            Position::new(from.x + dx, from.y),
            Position::new(from.x + dx, from.y - 1),
            Position::new(from.x + dx, from.y + 1),
        ]
    } else if dy != 0 {
        vec![
            Position::new(from.x, from.y + dy),
            Position::new(from.x - 1, from.y + dy),
            Position::new(from.x + 1, from.y + dy),
        ]
    } else {
        return None; // Already at target
    };

    // Find first valid move
    for pos in candidates {
        if is_valid_move(pos, map, world) {
            return Some(pos);
        }
    }

    None
}

/// Check if a position is valid for an enemy to move to
fn is_valid_move(pos: Position, map: &Map, world: &World) -> bool {
    // Check map walkability
    if !map.is_walkable(pos.x, pos.y) {
        return false;
    }

    // Check for blocking entities
    for (_, (entity_pos, _)) in world.query::<(&Position, &BlocksMovement)>().iter() {
        if entity_pos.x == pos.x && entity_pos.y == pos.y {
            return false;
        }
    }

    true
}

/// AI actions that need to be executed
#[derive(Debug)]
pub enum AIAction {
    Move { entity: hecs::Entity, to: Position },
    Attack { attacker: hecs::Entity, target_pos: Position },
}

/// Execute AI actions after collecting them
pub fn execute_ai_actions(
    world: &mut World,
    actions: Vec<AIAction>,
    player_entity: Option<hecs::Entity>,
    rng: &mut impl rand::Rng,
) -> Vec<String> {
    use crate::combat::{calculate_attack_with_equipment, EquipmentBonuses};
    use crate::ecs::{Stats, EquipmentComponent};

    let mut messages = Vec::new();

    // Get player equipment bonuses once for all attacks
    let player_equipment = player_entity
        .and_then(|p| world.get::<&EquipmentComponent>(p).ok())
        .map(|eq| EquipmentBonuses {
            weapon_damage: 0, // Not used for defense
            armor: eq.equipment.total_armor(),
            str_bonus: eq.equipment.strength_bonus(),
            dex_bonus: eq.equipment.dexterity_bonus(),
            crit_bonus: 0.0, // Not used for defense
        })
        .unwrap_or_default();

    for action in actions {
        match action {
            AIAction::Move { entity, to } => {
                // Update position
                if let Ok(mut pos) = world.get::<&mut Position>(entity) {
                    pos.x = to.x;
                    pos.y = to.y;
                }
            }
            AIAction::Attack { attacker, target_pos: _ } => {
                // Get attacker info
                let attacker_name = world
                    .get::<&Name>(attacker)
                    .map(|n| n.0.clone())
                    .unwrap_or_else(|_| "Enemy".to_string());

                let attacker_stats = world
                    .get::<&Stats>(attacker)
                    .map(|s| *s)
                    .unwrap_or(Stats::new(8, 8, 8, 8));

                // Get player stats for defense calculation
                let player_stats = player_entity
                    .and_then(|p| world.get::<&Stats>(p).ok().map(|s| *s))
                    .unwrap_or(Stats::player_base());

                // Calculate attack with equipment bonuses
                let result = calculate_attack_with_equipment(
                    &attacker_stats,
                    &player_stats,
                    &EquipmentBonuses::default(), // Enemies don't have equipment
                    &player_equipment,            // Player armor reduces damage
                    rng,
                );

                // Handle dodge/miss
                if result.is_dodge {
                    messages.push(format!("You dodge the {}'s attack!", attacker_name));
                    continue;
                }
                if result.is_miss {
                    messages.push(format!("The {} misses you!", attacker_name));
                    continue;
                }

                // Apply damage to player
                if let Some(player) = player_entity {
                    if let Ok(mut health) = world.get::<&mut Health>(player) {
                        health.take_damage(result.final_damage);
                        let msg = if result.is_crit {
                            format!("The {} lands a CRITICAL HIT for {} damage!", attacker_name, result.final_damage)
                        } else {
                            format!("The {} attacks you for {} damage.", attacker_name, result.final_damage)
                        };
                        messages.push(msg);
                    }
                }
            }
        }
    }

    messages
}
