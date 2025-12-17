//! Item system

pub mod item;
pub mod inventory;
pub mod equipment;
pub mod synergies;
pub mod loot;
pub mod grid;

pub use item::{Item, ItemId, ItemCategory, Rarity, EquipSlot, WeaponType, ArmorType, ConsumableEffect, Affix, AffixType, GemType, Gem};
pub use inventory::Inventory;
pub use equipment::Equipment;
pub use loot::{generate_enemy_loot, generate_floor_loot, generate_gold_drop, generate_weapon, generate_armor, generate_consumable, generate_boss_loot, generate_boss_gold_drop};
pub use synergies::{SynergyTag, SynergyBonus, Synergy, SynergyTier, SynergyBonuses, ActiveSynergy, calculate_synergies};
pub use grid::{InventoryGrid, GridPosition, PlacedItem, GRID_WIDTH, GRID_HEIGHT, SortMode};
