//! Entity creation and management

pub mod player;
pub mod enemies;
pub mod bosses;
pub mod npcs;
pub mod chests;
pub mod spawner;

pub use player::spawn_player;
pub use enemies::{spawn_enemy, spawn_enemy_scaled, spawn_enemies_for_floor, spawn_enemies_for_floor_with_zones, enemies_for_biome};
pub use bosses::{BossType, BossComponent, spawn_boss, boss_for_biome, update_boss_phase};
pub use npcs::{NpcType, NpcComponent, NpcMarker, ShopItem, spawn_npc, spawn_npcs_for_floor, get_npc_at};
pub use chests::{spawn_chest, spawn_chests_for_floor, generate_chest_loot, get_chest_at, mark_chest_opened};
