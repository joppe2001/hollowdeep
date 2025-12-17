//! Sound definitions and mappings
//!
//! Defines all sound events used in the game.

use std::path::Path;

/// Sound event identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundId {
    // === Combat ===
    /// Player attacks and hits
    Hit,
    /// Attack misses
    Miss,
    /// Critical hit
    Critical,
    /// Player or enemy dodges
    Dodge,
    /// Enemy dies
    EnemyDeath,
    /// Player takes damage
    PlayerHurt,
    /// Player dies
    PlayerDeath,
    /// Boss defeated
    BossDefeat,

    // === Items ===
    /// Item picked up
    ItemPickup,
    /// Item dropped
    ItemDrop,
    /// Inventory full (can't pick up)
    InventoryFull,
    /// Item equipped
    Equip,
    /// Item unequipped
    Unequip,
    /// Consumable used (potion, scroll)
    UseConsumable,

    // === Chests & Loot ===
    /// Chest opened
    ChestOpen,
    /// Gold/coins collected
    GoldPickup,
    /// Rare item found
    RareLoot,
    /// Legendary item found
    LegendaryLoot,

    // === UI & Menu ===
    /// Menu navigation (cursor move)
    MenuMove,
    /// Menu selection confirmed
    MenuSelect,
    /// Menu cancelled/back
    MenuBack,
    /// Error/invalid action
    Error,

    // === Skills ===
    /// Healing skill used
    SkillHeal,
    /// Buff skill activated
    SkillBuff,
    /// Offensive skill used
    SkillAttack,
    /// Movement skill (teleport, dash)
    SkillMovement,

    // === Environment ===
    /// Shrine approached
    ShrineApproach,
    /// Shrine used
    ShrineUse,
    /// Stairs descended
    Descend,
    /// Door opened
    DoorOpen,
    /// Footstep (walking)
    Footstep,

    // === Ambient ===
    /// Level up
    LevelUp,
    /// New floor entered
    NewFloor,
    /// Low health warning
    LowHealth,
}

impl SoundId {
    /// Get the file path for this sound
    pub fn file_path(&self) -> &'static str {
        match self {
            // Combat
            SoundId::Hit => "assets/sounds/combat/hit.ogg",
            SoundId::Miss => "assets/sounds/combat/miss.ogg",
            SoundId::Critical => "assets/sounds/combat/critical.ogg",
            SoundId::Dodge => "assets/sounds/combat/dodge.ogg",
            SoundId::EnemyDeath => "assets/sounds/combat/enemy_death.ogg",
            SoundId::PlayerHurt => "assets/sounds/combat/player_hurt.ogg",
            SoundId::PlayerDeath => "assets/sounds/combat/player_death.ogg",
            SoundId::BossDefeat => "assets/sounds/combat/boss_defeat.ogg",

            // Items
            SoundId::ItemPickup => "assets/sounds/items/pickup.ogg",
            SoundId::ItemDrop => "assets/sounds/items/drop.ogg",
            SoundId::InventoryFull => "assets/sounds/items/inventory_full.ogg",
            SoundId::Equip => "assets/sounds/items/equip.ogg",
            SoundId::Unequip => "assets/sounds/items/unequip.ogg",
            SoundId::UseConsumable => "assets/sounds/items/consume.ogg",

            // Chests
            SoundId::ChestOpen => "assets/sounds/chests/open.ogg",
            SoundId::GoldPickup => "assets/sounds/chests/gold.ogg",
            SoundId::RareLoot => "assets/sounds/chests/rare.ogg",
            SoundId::LegendaryLoot => "assets/sounds/chests/legendary.ogg",

            // UI
            SoundId::MenuMove => "assets/sounds/ui/move.ogg",
            SoundId::MenuSelect => "assets/sounds/ui/select.ogg",
            SoundId::MenuBack => "assets/sounds/ui/back.ogg",
            SoundId::Error => "assets/sounds/ui/error.ogg",

            // Skills
            SoundId::SkillHeal => "assets/sounds/skills/heal.ogg",
            SoundId::SkillBuff => "assets/sounds/skills/buff.ogg",
            SoundId::SkillAttack => "assets/sounds/skills/attack.ogg",
            SoundId::SkillMovement => "assets/sounds/skills/movement.ogg",

            // Environment
            SoundId::ShrineApproach => "assets/sounds/environment/shrine_approach.ogg",
            SoundId::ShrineUse => "assets/sounds/environment/shrine_use.ogg",
            SoundId::Descend => "assets/sounds/environment/descend.ogg",
            SoundId::DoorOpen => "assets/sounds/environment/door.ogg",
            SoundId::Footstep => "assets/sounds/environment/footstep.ogg",

            // Ambient
            SoundId::LevelUp => "assets/sounds/ambient/level_up.ogg",
            SoundId::NewFloor => "assets/sounds/ambient/new_floor.ogg",
            SoundId::LowHealth => "assets/sounds/ambient/low_health.ogg",
        }
    }

    /// Get the default volume for this sound (0.0 - 1.0)
    pub fn default_volume(&self) -> f64 {
        match self {
            // Quieter ambient sounds
            SoundId::Footstep => 0.3,
            SoundId::MenuMove => 0.4,

            // Normal volume
            SoundId::Hit | SoundId::Miss | SoundId::Dodge => 0.6,
            SoundId::ItemPickup | SoundId::ItemDrop => 0.5,
            SoundId::GoldPickup => 0.5,
            SoundId::MenuSelect | SoundId::MenuBack => 0.5,

            // Louder important sounds
            SoundId::Critical | SoundId::PlayerHurt => 0.7,
            SoundId::EnemyDeath => 0.6,
            SoundId::ChestOpen => 0.6,
            SoundId::RareLoot => 0.7,
            SoundId::LegendaryLoot => 0.8,
            SoundId::LevelUp => 0.8,
            SoundId::BossDefeat => 0.9,
            SoundId::PlayerDeath => 0.8,

            // Default
            _ => 0.6,
        }
    }

    /// Check if the sound file exists
    pub fn exists(&self) -> bool {
        Path::new(self.file_path()).exists()
    }
}

/// Categories for organizing sounds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundCategory {
    Combat,
    Items,
    Chests,
    UI,
    Skills,
    Environment,
    Ambient,
}

impl SoundId {
    /// Get the category for this sound
    pub fn category(&self) -> SoundCategory {
        match self {
            SoundId::Hit | SoundId::Miss | SoundId::Critical | SoundId::Dodge |
            SoundId::EnemyDeath | SoundId::PlayerHurt | SoundId::PlayerDeath |
            SoundId::BossDefeat => SoundCategory::Combat,

            SoundId::ItemPickup | SoundId::ItemDrop | SoundId::InventoryFull |
            SoundId::Equip | SoundId::Unequip | SoundId::UseConsumable => SoundCategory::Items,

            SoundId::ChestOpen | SoundId::GoldPickup | SoundId::RareLoot |
            SoundId::LegendaryLoot => SoundCategory::Chests,

            SoundId::MenuMove | SoundId::MenuSelect | SoundId::MenuBack |
            SoundId::Error => SoundCategory::UI,

            SoundId::SkillHeal | SoundId::SkillBuff | SoundId::SkillAttack |
            SoundId::SkillMovement => SoundCategory::Skills,

            SoundId::ShrineApproach | SoundId::ShrineUse | SoundId::Descend |
            SoundId::DoorOpen | SoundId::Footstep => SoundCategory::Environment,

            SoundId::LevelUp | SoundId::NewFloor | SoundId::LowHealth => SoundCategory::Ambient,
        }
    }
}
