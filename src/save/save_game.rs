//! Game save/load system
//!
//! Handles saving and loading game state to/from disk.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::ecs::{Position, Health, Mana, Stamina, Stats, Experience, StatPoints};
use crate::ecs::{InventoryComponent, EquipmentComponent, SkillsComponent, GroundItem};
use crate::items::Item;
use crate::progression::{Difficulty, EquippedSkills};
use crate::world::{Biome, TileType};

/// Save file version for compatibility checking
const SAVE_VERSION: u32 = 1;

/// Complete save data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub player: PlayerSaveData,
    pub game: GameSaveData,
    pub map: MapSaveData,
    pub enemies: Vec<EnemySaveData>,
    pub items_on_ground: Vec<ItemOnGround>,
}

/// Player-specific save data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSaveData {
    pub position: (i32, i32),
    pub health: (i32, i32),  // (current, max)
    pub mana: (i32, i32),
    pub stamina: (i32, i32),
    pub stats: StatsSaveData,
    pub experience: ExperienceSaveData,
    pub stat_points: u32,
    pub gold: u32,
    pub inventory: Vec<Item>,
    pub equipment: EquipmentSaveData,
    pub skills: EquippedSkills,
}

/// Stats save data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsSaveData {
    pub strength: i32,
    pub dexterity: i32,
    pub intelligence: i32,
    pub vitality: i32,
}

/// Experience save data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceSaveData {
    pub current: u32,
    pub level: u32,
    pub to_next_level: u32,
}

/// Equipment save data (items in each slot)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentSaveData {
    pub main_hand: Option<Item>,
    pub off_hand: Option<Item>,
    pub head: Option<Item>,
    pub body: Option<Item>,
    pub hands: Option<Item>,
    pub feet: Option<Item>,
    pub amulet: Option<Item>,
    pub ring1: Option<Item>,
    pub ring2: Option<Item>,
}

/// Game state save data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSaveData {
    pub floor: u32,
    pub difficulty: Difficulty,
    pub item_id_counter: u64,
    pub used_shrines: Vec<(u32, i32, i32)>,
    pub rng_seed: u64,
}

/// Map save data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapSaveData {
    pub width: i32,
    pub height: i32,
    pub floor_number: u32,
    pub biome: Biome,
    pub tiles: Vec<TileSaveData>,
    pub start_pos: (i32, i32),
    pub exit_pos: Option<(i32, i32)>,
    pub elite_rooms: Vec<(i32, i32)>,
}

/// Tile save data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileSaveData {
    pub tile_type: TileType,
    pub explored: bool,
    pub glyph_override: Option<char>,
}

/// Enemy save data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemySaveData {
    pub name: String,
    pub position: (i32, i32),
    pub health: (i32, i32),
    pub stats: StatsSaveData,
    pub xp_reward: u32,
    pub glyph: char,
    pub color: (u8, u8, u8),
}

/// Item on the ground
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemOnGround {
    pub position: (i32, i32),
    pub item: Item,
}

/// Get the save directory path
pub fn save_directory() -> PathBuf {
    use directories::ProjectDirs;

    if let Some(proj_dirs) = ProjectDirs::from("com", "hollowdeep", "Hollowdeep") {
        let mut path = proj_dirs.data_local_dir().to_path_buf();
        path.push("saves");
        path
    } else {
        // Fallback to current directory
        PathBuf::from("./saves")
    }
}

/// Get the path for a specific save slot
pub fn save_path(slot: u8) -> PathBuf {
    let mut path = save_directory();
    path.push(format!("save_{}.json", slot));
    path
}

/// Check if a save exists in the given slot
pub fn save_exists(slot: u8) -> bool {
    save_path(slot).exists()
}

/// List all available save slots (0-2)
pub fn list_saves() -> Vec<(u8, Option<SaveSummary>)> {
    (0..3).map(|slot| {
        let summary = if save_exists(slot) {
            load_save_summary(slot).ok()
        } else {
            None
        };
        (slot, summary)
    }).collect()
}

/// Brief summary of a save for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSummary {
    pub floor: u32,
    pub level: u32,
    pub difficulty: Difficulty,
}

/// Load just the summary from a save file
pub fn load_save_summary(slot: u8) -> Result<SaveSummary, SaveError> {
    let path = save_path(slot);
    let data = fs::read_to_string(&path).map_err(|e| SaveError::IoError(e.to_string()))?;
    let save: SaveData = serde_json::from_str(&data).map_err(|e| SaveError::ParseError(e.to_string()))?;

    Ok(SaveSummary {
        floor: save.game.floor,
        level: save.player.experience.level,
        difficulty: save.game.difficulty,
    })
}

/// Save error types
#[derive(Debug, Clone)]
pub enum SaveError {
    IoError(String),
    ParseError(String),
    VersionMismatch { expected: u32, found: u32 },
    InvalidData(String),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::IoError(e) => write!(f, "IO error: {}", e),
            SaveError::ParseError(e) => write!(f, "Parse error: {}", e),
            SaveError::VersionMismatch { expected, found } => {
                write!(f, "Save version mismatch: expected {}, found {}", expected, found)
            }
            SaveError::InvalidData(e) => write!(f, "Invalid save data: {}", e),
        }
    }
}

/// Save the game to a slot
pub fn save_game(game: &crate::game::Game, slot: u8) -> Result<(), SaveError> {
    let save_data = extract_save_data(game)?;

    // Ensure directory exists
    let dir = save_directory();
    fs::create_dir_all(&dir).map_err(|e| SaveError::IoError(e.to_string()))?;

    // Write save file
    let path = save_path(slot);
    let json = serde_json::to_string_pretty(&save_data)
        .map_err(|e| SaveError::ParseError(e.to_string()))?;
    fs::write(&path, json).map_err(|e| SaveError::IoError(e.to_string()))?;

    log::info!("Game saved to slot {}", slot);
    Ok(())
}

/// Load a game from a slot
pub fn load_game(slot: u8) -> Result<SaveData, SaveError> {
    let path = save_path(slot);
    let data = fs::read_to_string(&path).map_err(|e| SaveError::IoError(e.to_string()))?;
    let save: SaveData = serde_json::from_str(&data)
        .map_err(|e| SaveError::ParseError(e.to_string()))?;

    // Version check
    if save.version != SAVE_VERSION {
        return Err(SaveError::VersionMismatch {
            expected: SAVE_VERSION,
            found: save.version,
        });
    }

    log::info!("Game loaded from slot {}", slot);
    Ok(save)
}

/// Delete a save slot
pub fn delete_save(slot: u8) -> Result<(), SaveError> {
    let path = save_path(slot);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| SaveError::IoError(e.to_string()))?;
        log::info!("Deleted save slot {}", slot);
    }
    Ok(())
}

/// Extract save data from the current game state
fn extract_save_data(game: &crate::game::Game) -> Result<SaveData, SaveError> {
    use crate::ecs::{Name, Renderable, Enemy, XpReward};
    use crate::items::EquipSlot;

    let player = game.player().ok_or(SaveError::InvalidData("No player entity".to_string()))?;
    let world = game.world();

    // Extract player data
    let pos = world.get::<&Position>(player)
        .map_err(|_| SaveError::InvalidData("Missing player position".to_string()))?;
    let health = world.get::<&Health>(player)
        .map_err(|_| SaveError::InvalidData("Missing player health".to_string()))?;
    let mana = world.get::<&Mana>(player)
        .map_err(|_| SaveError::InvalidData("Missing player mana".to_string()))?;
    let stamina = world.get::<&Stamina>(player)
        .map_err(|_| SaveError::InvalidData("Missing player stamina".to_string()))?;
    let stats = world.get::<&Stats>(player)
        .map_err(|_| SaveError::InvalidData("Missing player stats".to_string()))?;
    let exp = world.get::<&Experience>(player)
        .map_err(|_| SaveError::InvalidData("Missing player experience".to_string()))?;
    let stat_points = world.get::<&StatPoints>(player).map(|sp| sp.0).unwrap_or(0);

    // Get inventory (includes gold and items)
    let inv_comp = world.get::<&InventoryComponent>(player);
    let gold = inv_comp.as_ref().map(|inv| inv.inventory.gold()).unwrap_or(0);
    let inventory = inv_comp.map(|inv| inv.inventory.items_owned()).unwrap_or_default();

    // Equipment
    let equipment = world.get::<&EquipmentComponent>(player)
        .map(|eq| EquipmentSaveData {
            main_hand: eq.equipment.get(EquipSlot::MainHand).cloned(),
            off_hand: eq.equipment.get(EquipSlot::OffHand).cloned(),
            head: eq.equipment.get(EquipSlot::Head).cloned(),
            body: eq.equipment.get(EquipSlot::Body).cloned(),
            hands: eq.equipment.get(EquipSlot::Hands).cloned(),
            feet: eq.equipment.get(EquipSlot::Feet).cloned(),
            amulet: eq.equipment.get(EquipSlot::Amulet).cloned(),
            ring1: eq.equipment.get(EquipSlot::Ring1).cloned(),
            ring2: eq.equipment.get(EquipSlot::Ring2).cloned(),
        })
        .unwrap_or(EquipmentSaveData {
            main_hand: None, off_hand: None, head: None, body: None,
            hands: None, feet: None, amulet: None, ring1: None, ring2: None,
        });

    // Skills
    let skills = world.get::<&SkillsComponent>(player)
        .map(|sk| sk.skills.clone())
        .unwrap_or_default();

    let player_data = PlayerSaveData {
        position: (pos.x, pos.y),
        health: (health.current, health.max),
        mana: (mana.current, mana.max),
        stamina: (stamina.current, stamina.max),
        stats: StatsSaveData {
            strength: stats.strength,
            dexterity: stats.dexterity,
            intelligence: stats.intelligence,
            vitality: stats.vitality,
        },
        experience: ExperienceSaveData {
            current: exp.current_xp,
            level: exp.level,
            to_next_level: exp.xp_to_next,
        },
        stat_points,
        gold,
        inventory,
        equipment,
        skills,
    };

    // Game data
    let game_data = GameSaveData {
        floor: game.floor(),
        difficulty: game.difficulty(),
        item_id_counter: 0, // Will need accessor
        used_shrines: Vec::new(), // Will need accessor
        rng_seed: 0, // Can't easily extract RNG state
    };

    // Map data
    let map = game.map().ok_or(SaveError::InvalidData("No map".to_string()))?;
    let map_data = MapSaveData {
        width: map.width,
        height: map.height,
        floor_number: map.floor_number,
        biome: map.biome,
        tiles: map.tiles.iter().map(|t| TileSaveData {
            tile_type: t.tile_type,
            explored: t.explored,
            glyph_override: t.glyph,
        }).collect(),
        start_pos: (map.start_pos.x, map.start_pos.y),
        exit_pos: map.exit_pos.map(|p| (p.x, p.y)),
        elite_rooms: map.elite_rooms.iter().map(|p| (p.x, p.y)).collect(),
    };

    // Enemies
    let mut enemies = Vec::new();
    for (_, (epos, name, ehealth, estats, xp, renderable, _)) in world.query::<(
        &Position, &Name, &Health, &Stats, &XpReward, &Renderable, &Enemy
    )>().iter() {
        enemies.push(EnemySaveData {
            name: name.0.clone(),
            position: (epos.x, epos.y),
            health: (ehealth.current, ehealth.max),
            stats: StatsSaveData {
                strength: estats.strength,
                dexterity: estats.dexterity,
                intelligence: estats.intelligence,
                vitality: estats.vitality,
            },
            xp_reward: xp.0,
            glyph: renderable.glyph,
            color: renderable.fg,
        });
    }

    // Items on ground
    let mut items_on_ground = Vec::new();
    for (_, (ipos, ground_item)) in world.query::<(&Position, &GroundItem)>().iter() {
        items_on_ground.push(ItemOnGround {
            position: (ipos.x, ipos.y),
            item: ground_item.item.clone(),
        });
    }

    Ok(SaveData {
        version: SAVE_VERSION,
        player: player_data,
        game: game_data,
        map: map_data,
        enemies,
        items_on_ground,
    })
}
