//! ECS Components
//!
//! All components used in the game's entity-component system.

use serde::{Deserialize, Serialize};

// ============================================================================
// Position & Movement
// ============================================================================

/// Position in the game world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Manhattan distance to another position
    pub fn distance(&self, other: &Position) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// Chebyshev distance (allows diagonal)
    pub fn chebyshev_distance(&self, other: &Position) -> i32 {
        (self.x - other.x).abs().max((self.y - other.y).abs())
    }
}

/// Velocity for moving entities
#[derive(Debug, Clone, Copy, Default)]
pub struct Velocity {
    pub dx: i32,
    pub dy: i32,
}

// ============================================================================
// Rendering
// ============================================================================

/// Visual representation of an entity
#[derive(Debug, Clone)]
pub struct Renderable {
    /// Character to display
    pub glyph: char,
    /// Foreground color (RGB)
    pub fg: (u8, u8, u8),
    /// Background color (RGB), None for transparent
    pub bg: Option<(u8, u8, u8)>,
    /// Render order (higher = on top)
    pub render_order: i32,
}

impl Renderable {
    pub fn new(glyph: char, fg: (u8, u8, u8)) -> Self {
        Self {
            glyph,
            fg,
            bg: None,
            render_order: 0,
        }
    }

    pub fn with_bg(mut self, bg: (u8, u8, u8)) -> Self {
        self.bg = Some(bg);
        self
    }

    pub fn with_order(mut self, order: i32) -> Self {
        self.render_order = order;
        self
    }
}

// ============================================================================
// Identity & Naming
// ============================================================================

/// Name component for entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Name(pub String);

impl Name {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

/// Marks an entity as the player
#[derive(Debug, Clone, Copy, Default)]
pub struct Player;

/// Marks an entity as an enemy
#[derive(Debug, Clone)]
pub struct Enemy {
    pub archetype: EnemyArchetype,
}

/// Enemy behavior archetypes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyArchetype {
    Melee,
    Ranged,
    Caster,
    Tank,
    Swarm,
    Elite,
    Boss,
}

/// Marks an entity as an NPC
#[derive(Debug, Clone)]
pub struct Npc {
    pub npc_type: NpcType,
}

/// Types of NPCs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcType {
    Merchant,
    QuestGiver,
    Lore,
}

// ============================================================================
// Combat Stats
// ============================================================================

/// Core RPG stats
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Stats {
    /// Strength - physical damage, carry weight
    pub strength: i32,
    /// Dexterity - speed, dodge, crit
    pub dexterity: i32,
    /// Intelligence - magic damage, mana
    pub intelligence: i32,
    /// Vitality - HP, resistances
    pub vitality: i32,
}

impl Stats {
    pub fn new(str: i32, dex: i32, int: i32, vit: i32) -> Self {
        Self {
            strength: str,
            dexterity: dex,
            intelligence: int,
            vitality: vit,
        }
    }

    /// Base player stats
    pub fn player_base() -> Self {
        Self::new(10, 10, 10, 10)
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new(10, 10, 10, 10)
    }
}

/// Health pool
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    pub fn take_damage(&mut self, amount: i32) -> i32 {
        let actual = amount.min(self.current);
        self.current -= actual;
        actual
    }

    pub fn heal(&mut self, amount: i32) -> i32 {
        let actual = amount.min(self.max - self.current);
        self.current += actual;
        actual
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0
    }

    pub fn percentage(&self) -> f32 {
        self.current as f32 / self.max as f32
    }
}

/// Mana pool for abilities
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Mana {
    pub current: i32,
    pub max: i32,
}

impl Mana {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    pub fn spend(&mut self, amount: i32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    pub fn restore(&mut self, amount: i32) {
        self.current = (self.current + amount).min(self.max);
    }
}

/// Stamina pool for physical abilities
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Stamina {
    pub current: i32,
    pub max: i32,
}

impl Stamina {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    pub fn spend(&mut self, amount: i32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    pub fn restore(&mut self, amount: i32) {
        self.current = (self.current + amount).min(self.max);
    }
}

/// Experience and level
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Experience {
    pub level: u32,
    pub current_xp: u32,
    pub xp_to_next: u32,
}

impl Experience {
    pub fn new() -> Self {
        Self {
            level: 1,
            current_xp: 0,
            xp_to_next: 100,
        }
    }

    /// Add XP and return true if leveled up
    pub fn add_xp(&mut self, amount: u32) -> bool {
        self.current_xp += amount;
        if self.current_xp >= self.xp_to_next {
            self.current_xp -= self.xp_to_next;
            self.level += 1;
            // XP curve: each level needs 50 more XP
            self.xp_to_next = 100 + (self.level - 1) * 50;
            true
        } else {
            false
        }
    }
}

// ============================================================================
// Combat
// ============================================================================

/// Combat-related derived stats
#[derive(Debug, Clone, Copy, Default)]
pub struct CombatStats {
    pub armor: i32,
    pub magic_resist: i32,
    pub crit_chance: f32,
    pub dodge_chance: f32,
    pub speed: i32,
}

/// Faction for determining hostility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Faction {
    Player,
    Enemy,
    Neutral,
}

#[derive(Debug, Clone, Copy)]
pub struct FactionComponent(pub Faction);

// ============================================================================
// Field of View
// ============================================================================

/// Field of view data
#[derive(Debug, Clone)]
pub struct FieldOfView {
    /// Tiles currently visible
    pub visible_tiles: Vec<Position>,
    /// Base vision range
    pub range: i32,
    /// Whether FOV needs recalculation
    pub dirty: bool,
}

impl FieldOfView {
    pub fn new(range: i32) -> Self {
        Self {
            visible_tiles: Vec::new(),
            range,
            dirty: true,
        }
    }
}

// ============================================================================
// Items
// ============================================================================

/// Marks an entity as an item that can be picked up
#[derive(Debug, Clone)]
pub struct Item {
    pub item_type: ItemType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    Weapon,
    Armor,
    Accessory,
    Consumable,
    Key,
    Lore,
}

/// Marks an item as equippable
#[derive(Debug, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    MainHand,
    OffHand,
    Head,
    Body,
    Hands,
    Feet,
    Amulet,
    Ring1,
    Ring2,
}

// ============================================================================
// Status Effects
// ============================================================================

/// A status effect on an entity
#[derive(Debug, Clone)]
pub struct StatusEffect {
    pub effect_type: StatusEffectType,
    pub duration: f32,      // Remaining duration in seconds
    pub intensity: i32,     // Effect strength
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusEffectType {
    // Debuffs
    Poison,
    Burn,
    Bleed,
    Slow,
    Weakness,
    Curse,
    // Buffs
    Regeneration,
    Haste,
    Shield,
    Strength,
}

/// Collection of active status effects
#[derive(Debug, Clone, Default)]
pub struct StatusEffects {
    pub effects: Vec<StatusEffect>,
}

// ============================================================================
// AI
// ============================================================================

/// AI behavior component
#[derive(Debug, Clone)]
pub struct AI {
    pub state: AIState,
    pub target: Option<Position>,
    pub home: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIState {
    Idle,
    Patrol,
    Chase,
    Attack,
    Flee,
}

// ============================================================================
// Blocking
// ============================================================================

/// Marks an entity as blocking movement
#[derive(Debug, Clone, Copy, Default)]
pub struct BlocksMovement;

/// Marks an entity as blocking line of sight
#[derive(Debug, Clone, Copy, Default)]
pub struct BlocksSight;

// ============================================================================
// Inventory & Equipment
// ============================================================================

/// Player's inventory component
#[derive(Debug, Clone, Default)]
pub struct InventoryComponent {
    pub inventory: crate::items::Inventory,
}

/// Player's equipment component
#[derive(Debug, Clone, Default)]
pub struct EquipmentComponent {
    pub equipment: crate::items::Equipment,
}

/// Marks an entity as an item that can be picked up from the ground
#[derive(Debug, Clone)]
pub struct GroundItem {
    pub item: crate::items::Item,
}

// ============================================================================
// Progression
// ============================================================================

/// XP reward for killing this entity
#[derive(Debug, Clone, Copy, Default)]
pub struct XpReward(pub u32);

/// Unspent stat points from leveling up
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct StatPoints(pub u32);

/// Player's equipped skills
#[derive(Debug, Clone, Default)]
pub struct SkillsComponent {
    pub skills: crate::progression::EquippedSkills,
}

// ============================================================================
// Chests
// ============================================================================

/// Chest rarity determines loot quality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChestRarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl ChestRarity {
    /// Get the glyph for this chest rarity
    pub fn glyph(&self) -> char {
        match self {
            ChestRarity::Common => '□',
            ChestRarity::Rare => '▣',
            ChestRarity::Epic => '◈',
            ChestRarity::Legendary => '◆',
        }
    }

    /// Get the color for this chest rarity
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            ChestRarity::Common => (180, 140, 100),  // Brown
            ChestRarity::Rare => (100, 150, 255),    // Blue
            ChestRarity::Epic => (180, 100, 220),    // Purple
            ChestRarity::Legendary => (255, 180, 50), // Orange/Gold
        }
    }

    /// Get minimum item rarity for this chest
    pub fn min_item_rarity(&self) -> crate::items::Rarity {
        match self {
            ChestRarity::Common => crate::items::Rarity::Common,
            ChestRarity::Rare => crate::items::Rarity::Uncommon,
            ChestRarity::Epic => crate::items::Rarity::Rare,
            ChestRarity::Legendary => crate::items::Rarity::Epic,
        }
    }

    /// Get item count range for this chest
    pub fn item_count(&self) -> (usize, usize) {
        match self {
            ChestRarity::Common => (1, 2),
            ChestRarity::Rare => (2, 3),
            ChestRarity::Epic => (2, 4),
            ChestRarity::Legendary => (3, 5),
        }
    }

    /// Get gold bonus multiplier for this chest
    pub fn gold_multiplier(&self) -> f32 {
        match self {
            ChestRarity::Common => 1.0,
            ChestRarity::Rare => 2.0,
            ChestRarity::Epic => 4.0,
            ChestRarity::Legendary => 8.0,
        }
    }
}

/// Marks an entity as a chest that can be opened
#[derive(Debug, Clone)]
pub struct Chest {
    pub rarity: ChestRarity,
    pub opened: bool,
}
