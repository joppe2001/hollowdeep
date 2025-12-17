//! Combat system

pub mod stats;
pub mod damage;
pub mod abilities;
pub mod status;

pub use damage::{calculate_attack, calculate_attack_with_equipment, calculate_enemy_attack, AttackResult, EquipmentBonuses, crit_chance, dodge_chance};
pub use status::{StatusTickResult, apply_status_damage};
