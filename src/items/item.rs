//! Item definitions
//!
//! Core item types, rarities, and properties.

use serde::{Deserialize, Serialize};
use super::synergies::SynergyTag;

/// Unique item ID for tracking
pub type ItemId = u64;

/// Item rarity tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rarity {
    Common,     // White - base stats only
    Uncommon,   // Green - 1 affix
    Rare,       // Blue - 2 affixes
    Epic,       // Purple - 3 affixes + unique property
    Legendary,  // Orange - fixed unique items
    Mythic,     // Cyan/White - 4 affixes + unique + mythic-only affixes, floor 20+
}

impl Rarity {
    /// Get display color RGB
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            Rarity::Common => (200, 200, 200),
            Rarity::Uncommon => (100, 255, 100),
            Rarity::Rare => (100, 150, 255),
            Rarity::Epic => (200, 100, 255),
            Rarity::Legendary => (255, 180, 50),
            Rarity::Mythic => (100, 255, 255), // Cyan - divine/transcendent
        }
    }

    /// Get rarity name
    pub fn name(&self) -> &'static str {
        match self {
            Rarity::Common => "Common",
            Rarity::Uncommon => "Uncommon",
            Rarity::Rare => "Rare",
            Rarity::Epic => "Epic",
            Rarity::Legendary => "Legendary",
            Rarity::Mythic => "Mythic",
        }
    }

    /// Get numeric value for sorting (higher = rarer)
    pub fn sort_value(&self) -> u8 {
        match self {
            Rarity::Common => 0,
            Rarity::Uncommon => 1,
            Rarity::Rare => 2,
            Rarity::Epic => 3,
            Rarity::Legendary => 4,
            Rarity::Mythic => 5,
        }
    }

    /// Get number of affix slots for this rarity
    pub fn affix_slots(&self) -> usize {
        match self {
            Rarity::Common => 0,
            Rarity::Uncommon => 1,
            Rarity::Rare => 2,
            Rarity::Epic => 3,
            Rarity::Legendary => 3,
            Rarity::Mythic => 4,
        }
    }

    /// Check if this rarity can have mythic-only affixes
    pub fn can_have_mythic_affixes(&self) -> bool {
        matches!(self, Rarity::Mythic)
    }
}

/// Main item categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemCategory {
    Weapon,
    Armor,
    Accessory,
    Consumable,
    Key,
    Lore,
}

impl ItemCategory {
    /// Get sort value for grouping (lower = appears first)
    /// Equipment (Weapon/Armor/Accessory) comes before consumables
    pub fn sort_value(&self) -> u8 {
        match self {
            ItemCategory::Weapon => 0,
            ItemCategory::Armor => 1,
            ItemCategory::Accessory => 2,
            ItemCategory::Key => 3,
            ItemCategory::Consumable => 4,
            ItemCategory::Lore => 5,
        }
    }

    /// Check if this is an equipment category
    pub fn is_equipment(&self) -> bool {
        matches!(self, ItemCategory::Weapon | ItemCategory::Armor | ItemCategory::Accessory)
    }
}

/// Equipment slot for wearable items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipSlot {
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

/// Weapon subtypes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponType {
    Sword,      // Balanced: moderate damage, some crit & double-strike
    Axe,        // Heavy: high damage, high stamina cost, some armor pen
    Dagger,     // Agile: low damage, low cost, high crit & double-strike
    Mace,       // Crushing: good damage, high armor penetration
    Staff,      // Arcane: magic damage scaling (INT based)
    Bow,        // Ranged: can attack from distance, good crit
}

impl WeaponType {
    pub fn base_damage(&self) -> i32 {
        match self {
            WeaponType::Sword => 8,
            WeaponType::Axe => 12,
            WeaponType::Dagger => 5,
            WeaponType::Mace => 10,
            WeaponType::Staff => 6,
            WeaponType::Bow => 7,
        }
    }

    /// Stamina cost per attack (turn-based balance)
    pub fn stamina_cost(&self) -> i32 {
        match self {
            WeaponType::Sword => 10,    // Balanced
            WeaponType::Axe => 15,      // Heavy, costs more
            WeaponType::Dagger => 5,    // Light, cheap attacks
            WeaponType::Mace => 12,     // Moderate
            WeaponType::Staff => 8,     // Light
            WeaponType::Bow => 8,       // Moderate
        }
    }

    /// Chance to strike twice in one turn (0-100)
    pub fn double_strike_chance(&self) -> i32 {
        match self {
            WeaponType::Dagger => 25,   // Fast weapon, high double-strike
            WeaponType::Sword => 10,    // Balanced
            WeaponType::Bow => 5,       // Slight chance
            _ => 0,                     // Heavy weapons don't double-strike
        }
    }

    /// Armor penetration percentage (ignores this % of enemy armor)
    pub fn armor_penetration(&self) -> i32 {
        match self {
            WeaponType::Mace => 40,     // Crushing weapons bypass armor
            WeaponType::Axe => 20,      // Heavy cleaving
            WeaponType::Dagger => 15,   // Finds gaps in armor
            _ => 0,
        }
    }

    pub fn crit_bonus(&self) -> f32 {
        match self {
            WeaponType::Dagger => 15.0, // +15% crit chance
            WeaponType::Sword => 5.0,
            WeaponType::Bow => 10.0,    // Ranged precision
            _ => 0.0,
        }
    }
}

/// Armor subtypes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArmorType {
    Cloth,      // Low armor, magic bonus
    Leather,    // Medium armor, dex bonus
    Chain,      // Good armor, balanced
    Plate,      // High armor, slow
}

impl ArmorType {
    pub fn base_armor(&self, slot: EquipSlot) -> i32 {
        let base = match self {
            ArmorType::Cloth => 1,
            ArmorType::Leather => 3,
            ArmorType::Chain => 5,
            ArmorType::Plate => 8,
        };
        // Body armor gives more
        match slot {
            EquipSlot::Body => base * 2,
            EquipSlot::Head | EquipSlot::Feet => base,
            EquipSlot::Hands => base / 2 + 1,
            _ => base,
        }
    }
}

/// Consumable effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsumableEffect {
    HealHP(i32),
    RestoreMP(i32),
    RestoreSP(i32),
    BuffStrength(i32, u32),   // amount, duration in turns
    BuffDexterity(i32, u32),
    BuffIntelligence(i32, u32),
    CurePoison,
    Teleport,
    RevealMap,
}

/// Item affixes (magical properties)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affix {
    pub affix_type: AffixType,
    pub value: i32,
}

/// Types of affixes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AffixType {
    // Offensive
    BonusDamage,
    BonusCritChance,
    BonusCritDamage,
    FireDamage,
    IceDamage,
    LightningDamage,
    PoisonDamage,
    LifeSteal,

    // Defensive
    BonusArmor,
    BonusHP,
    BonusMP,
    BonusDodge,
    FireResist,
    IceResist,
    PoisonResist,

    // Stats
    BonusStrength,
    BonusDexterity,
    BonusIntelligence,
    BonusVitality,

    // Utility
    BonusXP,
    GoldFind,
    MagicFind,

    // ===== MYTHIC-ONLY AFFIXES =====
    // These can only appear on Mythic rarity items

    /// All stats bonus (STR/DEX/INT/VIT)
    AllStats,
    /// Percentage damage reduction
    DamageReduction,
    /// Percentage of damage reflected to attackers
    Thorns,
    /// Chance to double the effect of potions
    PotionMastery,
    /// Increased experience from all sources (percentage)
    ExperienceMultiplier,
    /// Chance to not consume resources on skill use
    ResourceConservation,
    /// Bonus damage that scales with floor level
    AscendedPower,
    /// Health regeneration per turn
    Regeneration,
}

impl AffixType {
    pub fn is_prefix(&self) -> bool {
        matches!(self,
            AffixType::BonusDamage | AffixType::FireDamage |
            AffixType::IceDamage | AffixType::LightningDamage |
            AffixType::PoisonDamage | AffixType::LifeSteal |
            AffixType::BonusArmor | AffixType::BonusHP |
            // Mythic prefixes
            AffixType::AllStats | AffixType::DamageReduction |
            AffixType::Thorns | AffixType::Regeneration
        )
    }

    /// Check if this is a mythic-only affix
    pub fn is_mythic_only(&self) -> bool {
        matches!(self,
            AffixType::AllStats | AffixType::DamageReduction |
            AffixType::Thorns | AffixType::PotionMastery |
            AffixType::ExperienceMultiplier | AffixType::ResourceConservation |
            AffixType::AscendedPower | AffixType::Regeneration
        )
    }

    pub fn name(&self) -> &'static str {
        match self {
            AffixType::BonusDamage => "Sharp",
            AffixType::BonusCritChance => "Precise",
            AffixType::BonusCritDamage => "Deadly",
            AffixType::FireDamage => "Flaming",
            AffixType::IceDamage => "Frozen",
            AffixType::LightningDamage => "Shocking",
            AffixType::PoisonDamage => "Venomous",
            AffixType::LifeSteal => "Vampiric",
            AffixType::BonusArmor => "Fortified",
            AffixType::BonusHP => "Sturdy",
            AffixType::BonusMP => "Arcane",
            AffixType::BonusDodge => "Evasive",
            AffixType::FireResist => "Fireproof",
            AffixType::IceResist => "Insulated",
            AffixType::PoisonResist => "Antitoxin",
            AffixType::BonusStrength => "of Might",
            AffixType::BonusDexterity => "of Agility",
            AffixType::BonusIntelligence => "of Wisdom",
            AffixType::BonusVitality => "of Vitality",
            AffixType::BonusXP => "of Learning",
            AffixType::GoldFind => "of Greed",
            AffixType::MagicFind => "of Fortune",
            // Mythic affixes
            AffixType::AllStats => "Divine",
            AffixType::DamageReduction => "Impervious",
            AffixType::Thorns => "Retributive",
            AffixType::PotionMastery => "of Alchemy",
            AffixType::ExperienceMultiplier => "of Enlightenment",
            AffixType::ResourceConservation => "of Efficiency",
            AffixType::AscendedPower => "Ascended",
            AffixType::Regeneration => "Regenerating",
        }
    }

    /// Get a description of what this affix does
    pub fn description(&self) -> &'static str {
        match self {
            AffixType::BonusDamage => "Increases physical damage",
            AffixType::BonusCritChance => "Increases critical hit chance",
            AffixType::BonusCritDamage => "Increases critical hit damage",
            AffixType::FireDamage => "Adds fire damage to attacks",
            AffixType::IceDamage => "Adds ice damage, may slow enemies",
            AffixType::LightningDamage => "Adds lightning damage, may chain",
            AffixType::PoisonDamage => "Adds poison damage over time",
            AffixType::LifeSteal => "Heal 5% of damage dealt per point",
            AffixType::BonusArmor => "Increases armor rating",
            AffixType::BonusHP => "Increases maximum health",
            AffixType::BonusMP => "Increases maximum mana",
            AffixType::BonusDodge => "Increases dodge chance",
            AffixType::FireResist => "Reduces fire damage taken",
            AffixType::IceResist => "Reduces ice damage taken",
            AffixType::PoisonResist => "Reduces poison damage taken",
            AffixType::BonusStrength => "Increases STR (physical damage)",
            AffixType::BonusDexterity => "Increases DEX (crit & dodge)",
            AffixType::BonusIntelligence => "Increases INT (magic damage)",
            AffixType::BonusVitality => "Increases VIT (health & defense)",
            AffixType::BonusXP => "Increases experience gained",
            AffixType::GoldFind => "Increases gold from enemies",
            AffixType::MagicFind => "Increases rare item drop chance",
            // Mythic affix descriptions
            AffixType::AllStats => "Increases all stats (STR/DEX/INT/VIT)",
            AffixType::DamageReduction => "Reduces all damage taken by %",
            AffixType::Thorns => "Reflects % of damage back to attackers",
            AffixType::PotionMastery => "% chance to double potion effects",
            AffixType::ExperienceMultiplier => "% increased experience gain",
            AffixType::ResourceConservation => "% chance to not consume MP/SP",
            AffixType::AscendedPower => "Damage scales with floor depth",
            AffixType::Regeneration => "Regenerate HP each turn",
        }
    }
}

/// Gem types for socketing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GemType {
    /// Red - adds damage
    Ruby,
    /// Blue - adds mana
    Sapphire,
    /// Green - adds crit chance
    Emerald,
    /// Yellow - adds armor
    Topaz,
    /// Purple - adds lifesteal
    Amethyst,
    /// White - adds all stats
    Diamond,
    /// Black - adds corruption power
    Onyx,
}

impl GemType {
    /// Get the color of this gem
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            GemType::Ruby => (255, 80, 80),
            GemType::Sapphire => (80, 80, 255),
            GemType::Emerald => (80, 255, 80),
            GemType::Topaz => (255, 255, 80),
            GemType::Amethyst => (200, 80, 255),
            GemType::Diamond => (255, 255, 255),
            GemType::Onyx => (50, 50, 50),
        }
    }

    /// Get the name of this gem
    pub fn name(&self) -> &'static str {
        match self {
            GemType::Ruby => "Ruby",
            GemType::Sapphire => "Sapphire",
            GemType::Emerald => "Emerald",
            GemType::Topaz => "Topaz",
            GemType::Amethyst => "Amethyst",
            GemType::Diamond => "Diamond",
            GemType::Onyx => "Onyx",
        }
    }

    /// Get the bonus this gem provides
    pub fn description(&self) -> &'static str {
        match self {
            GemType::Ruby => "+5 Damage per tier",
            GemType::Sapphire => "+15 Max MP per tier",
            GemType::Emerald => "+5% Crit Chance per tier",
            GemType::Topaz => "+5 Armor per tier",
            GemType::Amethyst => "+3% Lifesteal per tier",
            GemType::Diamond => "+2 All Stats per tier",
            GemType::Onyx => "+8 Corruption Power per tier",
        }
    }
}

/// A gem that can be socketed into items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gem {
    pub gem_type: GemType,
    /// Gem tier (1-5), affects bonus strength
    pub tier: u8,
}

impl Gem {
    pub fn new(gem_type: GemType, tier: u8) -> Self {
        Self { gem_type, tier: tier.min(5).max(1) }
    }

    /// Get the bonus value based on tier
    pub fn bonus_value(&self) -> i32 {
        self.tier as i32
    }
}

/// The main Item struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// Unique instance ID
    pub id: ItemId,
    /// Base item template name
    pub base_name: String,
    /// Full generated name (with affixes)
    pub name: String,
    /// Description/flavor text
    pub description: String,
    /// Item category
    pub category: ItemCategory,
    /// Rarity tier
    pub rarity: Rarity,
    /// Equipment slot (if equippable)
    pub equip_slot: Option<EquipSlot>,
    /// Weapon type (if weapon)
    pub weapon_type: Option<WeaponType>,
    /// Armor type (if armor)
    pub armor_type: Option<ArmorType>,
    /// Consumable effect (if consumable)
    pub consumable_effect: Option<ConsumableEffect>,
    /// Base damage (weapons)
    pub base_damage: i32,
    /// Base armor (armor)
    pub base_armor: i32,
    /// Magical affixes
    pub affixes: Vec<Affix>,
    /// Synergy tags for combo bonuses
    pub synergy_tags: Vec<SynergyTag>,
    /// Grid size for inventory (width, height)
    pub grid_size: (u8, u8),
    /// Display glyph
    pub glyph: char,
    /// Stack count (for consumables)
    pub stack_count: u32,
    /// Max stack size
    pub max_stack: u32,
    /// Gold value
    pub value: u32,
    /// Maximum enchantment slots (default 5, can be increased)
    pub max_enchantments: usize,
    /// Whether this item is newly picked up (not yet viewed)
    #[serde(default)]
    pub is_new: bool,

    // ===== ENDGAME ENHANCEMENT FIELDS =====

    /// Enchantment level (+1, +2, etc.) - increases base stats
    /// Each level adds 10% to base damage/armor
    #[serde(default)]
    pub enchantment_level: u8,
    /// Awakening level (infinite progression) - multiplies ALL stats
    /// Each awakening adds 10% multiplicative bonus to everything
    #[serde(default)]
    pub awakening_level: u8,
    /// Sockets for gems (None = empty socket)
    #[serde(default)]
    pub sockets: Vec<Option<Gem>>,
    /// Corruption level (risk/reward - higher = more power but penalties)
    /// Each level: +15% damage, -5% max HP
    #[serde(default)]
    pub corruption_level: u8,
}

impl Item {
    /// Create a new item with a unique ID
    pub fn new(id: ItemId, base_name: impl Into<String>, category: ItemCategory) -> Self {
        let base = base_name.into();
        Self {
            id,
            name: base.clone(),
            base_name: base,
            description: String::new(),
            category,
            rarity: Rarity::Common,
            equip_slot: None,
            weapon_type: None,
            armor_type: None,
            consumable_effect: None,
            base_damage: 0,
            base_armor: 0,
            affixes: Vec::new(),
            synergy_tags: Vec::new(),
            grid_size: (1, 1),
            glyph: '?',
            stack_count: 1,
            max_stack: 1,
            value: 1,
            max_enchantments: 5,
            is_new: true,
            // Endgame enhancement defaults
            enchantment_level: 0,
            awakening_level: 0,
            sockets: Vec::new(),
            corruption_level: 0,
        }
    }

    /// Mark item as seen (no longer new)
    pub fn mark_seen(&mut self) {
        self.is_new = false;
    }

    /// Get enchantment multiplier (1.0 + 0.1 per level)
    pub fn enchantment_multiplier(&self) -> f32 {
        1.0 + (self.enchantment_level as f32 * 0.10)
    }

    /// Get awakening multiplier (1.0 + 0.1 per level, multiplicative)
    pub fn awakening_multiplier(&self) -> f32 {
        1.0 + (self.awakening_level as f32 * 0.10)
    }

    /// Get corruption damage bonus (15% per level)
    pub fn corruption_damage_bonus(&self) -> f32 {
        self.corruption_level as f32 * 0.15
    }

    /// Get corruption HP penalty (5% per level)
    pub fn corruption_hp_penalty(&self) -> f32 {
        self.corruption_level as f32 * 0.05
    }

    /// Get total damage including affixes, enchantments, awakening, and corruption
    pub fn total_damage(&self) -> i32 {
        let mut damage = self.base_damage as f32;

        // Add affix damage
        for affix in &self.affixes {
            if matches!(affix.affix_type, AffixType::BonusDamage) {
                damage += affix.value as f32;
            }
        }

        // Add gem damage (Ruby)
        for socket in &self.sockets {
            if let Some(gem) = socket {
                if matches!(gem.gem_type, GemType::Ruby) {
                    damage += (gem.tier as f32) * 5.0;
                }
            }
        }

        // Apply enchantment bonus (flat % increase)
        damage *= self.enchantment_multiplier();

        // Apply awakening bonus (multiplicative)
        damage *= self.awakening_multiplier();

        // Apply corruption bonus
        damage *= 1.0 + self.corruption_damage_bonus();

        damage as i32
    }

    /// Get total armor including affixes, enchantments, awakening
    pub fn total_armor(&self) -> i32 {
        let mut armor = self.base_armor as f32;

        // Add affix armor
        for affix in &self.affixes {
            if matches!(affix.affix_type, AffixType::BonusArmor) {
                armor += affix.value as f32;
            }
        }

        // Add gem armor (Topaz)
        for socket in &self.sockets {
            if let Some(gem) = socket {
                if matches!(gem.gem_type, GemType::Topaz) {
                    armor += (gem.tier as f32) * 5.0;
                }
            }
        }

        // Apply enchantment bonus
        armor *= self.enchantment_multiplier();

        // Apply awakening bonus
        armor *= self.awakening_multiplier();

        armor as i32
    }

    /// Get bonus from socketed gems for a specific type
    pub fn gem_bonus(&self, gem_type: GemType) -> i32 {
        self.sockets.iter()
            .filter_map(|s| s.as_ref())
            .filter(|g| g.gem_type == gem_type)
            .map(|g| g.bonus_value())
            .sum()
    }

    /// Get number of empty sockets
    pub fn empty_sockets(&self) -> usize {
        self.sockets.iter().filter(|s| s.is_none()).count()
    }

    /// Get number of filled sockets
    pub fn filled_sockets(&self) -> usize {
        self.sockets.iter().filter(|s| s.is_some()).count()
    }

    /// Add a socket to the item (returns false if max sockets reached)
    pub fn add_socket(&mut self) -> bool {
        let max_sockets = match self.rarity {
            Rarity::Common => 1,
            Rarity::Uncommon => 2,
            Rarity::Rare => 3,
            Rarity::Epic => 4,
            Rarity::Legendary => 5,
            Rarity::Mythic => 6,
        };
        if self.sockets.len() < max_sockets {
            self.sockets.push(None);
            true
        } else {
            false
        }
    }

    /// Socket a gem into the first empty socket (returns false if no empty sockets)
    pub fn socket_gem(&mut self, gem: Gem) -> bool {
        for socket in &mut self.sockets {
            if socket.is_none() {
                *socket = Some(gem);
                return true;
            }
        }
        false
    }

    /// Remove a gem from a specific socket index
    pub fn unsocket_gem(&mut self, index: usize) -> Option<Gem> {
        if index < self.sockets.len() {
            self.sockets[index].take()
        } else {
            None
        }
    }

    /// Enchant the item (increase enchantment level)
    /// Returns the gold cost for the enchantment
    /// Cost scales aggressively: 100 * (level+1) * 2^level
    /// This makes early enchants cheap but high enchants very expensive
    pub fn enchant_cost(&self) -> u32 {
        // Aggressive scaling: 100 * (level+1) * 2^level
        // +0→+1: 100g, +1→+2: 400g, +2→+3: 1200g, +5→+6: 19200g, +10→+11: 1.1M
        let level = self.enchantment_level as u32;
        let level_mult = level + 1;
        let exp_mult = 2u32.saturating_pow(level);
        100u32.saturating_mul(level_mult).saturating_mul(exp_mult)
    }

    /// Perform an enchantment upgrade
    pub fn enchant(&mut self) -> bool {
        if self.enchantment_level < 15 { // Max +15
            self.enchantment_level += 1;
            true
        } else {
            false
        }
    }

    /// Awakening cost (increases with level)
    pub fn awakening_cost(&self) -> u32 {
        // Cost: 500 * (level + 1)^2
        500 * ((self.awakening_level as u32 + 1).pow(2))
    }

    /// Perform an awakening upgrade
    pub fn awaken(&mut self) -> bool {
        // No max level - infinite progression
        self.awakening_level += 1;
        true
    }

    /// Corruption cost
    pub fn corruption_cost(&self) -> u32 {
        // Cost: 200 * (level + 1)
        200 * (self.corruption_level as u32 + 1)
    }

    /// Add corruption (risk/reward)
    pub fn corrupt(&mut self) -> bool {
        if self.corruption_level < 10 { // Max 10 corruption
            self.corruption_level += 1;
            true
        } else {
            false
        }
    }

    /// Get display name including enhancement levels
    pub fn display_name(&self) -> String {
        let mut name = self.name.clone();

        if self.enchantment_level > 0 {
            name = format!("+{} {}", self.enchantment_level, name);
        }

        if self.awakening_level > 0 {
            name = format!("{} [A{}]", name, self.awakening_level);
        }

        if self.corruption_level > 0 {
            name = format!("{} {{C{}}}", name, self.corruption_level);
        }

        name
    }

    /// Get stat bonus from affixes
    pub fn stat_bonus(&self, stat: AffixType) -> i32 {
        self.affixes.iter()
            .filter(|a| a.affix_type == stat)
            .map(|a| a.value)
            .sum()
    }

    /// Check if item is equippable
    pub fn is_equippable(&self) -> bool {
        self.equip_slot.is_some()
    }

    /// Check if item is consumable
    pub fn is_consumable(&self) -> bool {
        self.consumable_effect.is_some()
    }

    /// Check if item can stack
    pub fn is_stackable(&self) -> bool {
        self.max_stack > 1
    }

    /// Get all synergy tags (base + from affixes)
    pub fn all_synergy_tags(&self) -> Vec<SynergyTag> {
        let mut tags = self.synergy_tags.clone();

        // Add tags from elemental affixes
        for affix in &self.affixes {
            match affix.affix_type {
                AffixType::FireDamage | AffixType::FireResist => {
                    if !tags.contains(&SynergyTag::Fire) {
                        tags.push(SynergyTag::Fire);
                    }
                }
                AffixType::IceDamage | AffixType::IceResist => {
                    if !tags.contains(&SynergyTag::Ice) {
                        tags.push(SynergyTag::Ice);
                    }
                }
                AffixType::LightningDamage => {
                    if !tags.contains(&SynergyTag::Lightning) {
                        tags.push(SynergyTag::Lightning);
                    }
                }
                AffixType::PoisonDamage | AffixType::PoisonResist => {
                    if !tags.contains(&SynergyTag::Poison) {
                        tags.push(SynergyTag::Poison);
                    }
                }
                _ => {}
            }
        }

        tags
    }

    /// Generate display name with affixes
    pub fn generate_name(&mut self) {
        let mut prefix = String::new();
        let mut suffix = String::new();

        for affix in &self.affixes {
            if affix.affix_type.is_prefix() {
                if prefix.is_empty() {
                    prefix = affix.affix_type.name().to_string();
                }
            } else if suffix.is_empty() {
                suffix = affix.affix_type.name().to_string();
            }
        }

        self.name = match (prefix.is_empty(), suffix.is_empty()) {
            (true, true) => self.base_name.clone(),
            (false, true) => format!("{} {}", prefix, self.base_name),
            (true, false) => format!("{} {}", self.base_name, suffix),
            (false, false) => format!("{} {} {}", prefix, self.base_name, suffix),
        };
    }
}

/// Item templates for common items
pub mod templates {
    use super::*;

    pub fn iron_sword(id: ItemId) -> Item {
        let mut item = Item::new(id, "Iron Sword", ItemCategory::Weapon);
        item.equip_slot = Some(EquipSlot::MainHand);
        item.weapon_type = Some(WeaponType::Sword);
        item.base_damage = WeaponType::Sword.base_damage();
        item.glyph = '🗡';
        item.grid_size = (1, 2);
        item.value = 50;
        item.description = "A sturdy iron blade.".to_string();
        item.synergy_tags = vec![SynergyTag::Knight];
        item
    }

    pub fn rusty_dagger(id: ItemId) -> Item {
        let mut item = Item::new(id, "Rusty Dagger", ItemCategory::Weapon);
        item.equip_slot = Some(EquipSlot::MainHand);
        item.weapon_type = Some(WeaponType::Dagger);
        item.base_damage = WeaponType::Dagger.base_damage() - 1;
        item.glyph = '🔪';
        item.grid_size = (1, 1);
        item.value = 15;
        item.description = "A worn dagger, still sharp enough.".to_string();
        item.synergy_tags = vec![SynergyTag::Shadow];
        item
    }

    pub fn battle_axe(id: ItemId) -> Item {
        let mut item = Item::new(id, "Battle Axe", ItemCategory::Weapon);
        item.equip_slot = Some(EquipSlot::MainHand);
        item.weapon_type = Some(WeaponType::Axe);
        item.base_damage = WeaponType::Axe.base_damage();
        item.glyph = '🪓';
        item.grid_size = (2, 2);
        item.value = 80;
        item.description = "A heavy two-handed axe.".to_string();
        item.synergy_tags = vec![SynergyTag::TwoHanded];
        item
    }

    pub fn leather_armor(id: ItemId) -> Item {
        let mut item = Item::new(id, "Leather Armor", ItemCategory::Armor);
        item.equip_slot = Some(EquipSlot::Body);
        item.armor_type = Some(ArmorType::Leather);
        item.base_armor = ArmorType::Leather.base_armor(EquipSlot::Body);
        item.glyph = '🛡';
        item.grid_size = (2, 2);
        item.value = 60;
        item.description = "Supple leather armor.".to_string();
        item.synergy_tags = vec![SynergyTag::Shadow];
        item
    }

    pub fn chain_helm(id: ItemId) -> Item {
        let mut item = Item::new(id, "Chain Helm", ItemCategory::Armor);
        item.equip_slot = Some(EquipSlot::Head);
        item.armor_type = Some(ArmorType::Chain);
        item.base_armor = ArmorType::Chain.base_armor(EquipSlot::Head);
        item.glyph = '⛑';
        item.grid_size = (1, 1);
        item.value = 45;
        item.description = "A chain mail helmet.".to_string();
        item.synergy_tags = vec![SynergyTag::Knight];
        item
    }

    pub fn health_potion(id: ItemId) -> Item {
        let mut item = Item::new(id, "Health Potion", ItemCategory::Consumable);
        item.consumable_effect = Some(ConsumableEffect::HealHP(30));
        item.glyph = '❤';
        item.grid_size = (1, 1);
        item.max_stack = 10;
        item.value = 25;
        item.description = "Restores 30 HP.".to_string();
        item.rarity = Rarity::Common;
        item
    }

    pub fn mana_potion(id: ItemId) -> Item {
        let mut item = Item::new(id, "Mana Potion", ItemCategory::Consumable);
        item.consumable_effect = Some(ConsumableEffect::RestoreMP(25));
        item.glyph = '💧';
        item.grid_size = (1, 1);
        item.max_stack = 10;
        item.value = 30;
        item.description = "Restores 25 MP.".to_string();
        item.rarity = Rarity::Common;
        item
    }

    // Synergy-themed items
    pub fn flame_sword(id: ItemId) -> Item {
        let mut item = Item::new(id, "Flame Sword", ItemCategory::Weapon);
        item.equip_slot = Some(EquipSlot::MainHand);
        item.weapon_type = Some(WeaponType::Sword);
        item.base_damage = WeaponType::Sword.base_damage() + 2;
        item.glyph = '🗡';
        item.grid_size = (1, 2);
        item.value = 120;
        item.rarity = Rarity::Uncommon;
        item.description = "A blade wreathed in eternal flame.".to_string();
        item.synergy_tags = vec![SynergyTag::Fire];
        item.affixes.push(Affix { affix_type: AffixType::FireDamage, value: 4 });
        item.generate_name();
        item
    }

    pub fn frost_dagger(id: ItemId) -> Item {
        let mut item = Item::new(id, "Frost Dagger", ItemCategory::Weapon);
        item.equip_slot = Some(EquipSlot::MainHand);
        item.weapon_type = Some(WeaponType::Dagger);
        item.base_damage = WeaponType::Dagger.base_damage() + 1;
        item.glyph = '🔪';
        item.grid_size = (1, 1);
        item.value = 90;
        item.rarity = Rarity::Uncommon;
        item.description = "Bitter cold radiates from this blade.".to_string();
        item.synergy_tags = vec![SynergyTag::Ice, SynergyTag::Shadow];
        item.affixes.push(Affix { affix_type: AffixType::IceDamage, value: 3 });
        item.generate_name();
        item
    }

    pub fn cultist_robe(id: ItemId) -> Item {
        let mut item = Item::new(id, "Cultist Robe", ItemCategory::Armor);
        item.equip_slot = Some(EquipSlot::Body);
        item.armor_type = Some(ArmorType::Cloth);
        item.base_armor = ArmorType::Cloth.base_armor(EquipSlot::Body);
        item.glyph = '👘';
        item.grid_size = (2, 2);
        item.value = 75;
        item.rarity = Rarity::Uncommon;
        item.description = "Dark robes worn by the blood cult.".to_string();
        item.synergy_tags = vec![SynergyTag::Cultist, SynergyTag::Corruption];
        item
    }

    pub fn cultist_dagger(id: ItemId) -> Item {
        let mut item = Item::new(id, "Ritual Dagger", ItemCategory::Weapon);
        item.equip_slot = Some(EquipSlot::MainHand);
        item.weapon_type = Some(WeaponType::Dagger);
        item.base_damage = WeaponType::Dagger.base_damage() + 2;
        item.glyph = '🔪';
        item.grid_size = (1, 1);
        item.value = 85;
        item.rarity = Rarity::Uncommon;
        item.description = "A dagger used in dark rituals.".to_string();
        item.synergy_tags = vec![SynergyTag::Cultist];
        item.affixes.push(Affix { affix_type: AffixType::LifeSteal, value: 5 });
        item.generate_name();
        item
    }

    pub fn knight_helm(id: ItemId) -> Item {
        let mut item = Item::new(id, "Knight's Helm", ItemCategory::Armor);
        item.equip_slot = Some(EquipSlot::Head);
        item.armor_type = Some(ArmorType::Plate);
        item.base_armor = ArmorType::Plate.base_armor(EquipSlot::Head);
        item.glyph = '⛑';
        item.grid_size = (1, 1);
        item.value = 100;
        item.rarity = Rarity::Uncommon;
        item.description = "A knight's sturdy helm.".to_string();
        item.synergy_tags = vec![SynergyTag::Knight];
        item
    }

    pub fn knight_plate(id: ItemId) -> Item {
        let mut item = Item::new(id, "Knight's Plate", ItemCategory::Armor);
        item.equip_slot = Some(EquipSlot::Body);
        item.armor_type = Some(ArmorType::Plate);
        item.base_armor = ArmorType::Plate.base_armor(EquipSlot::Body);
        item.glyph = '🛡';
        item.grid_size = (2, 3);
        item.value = 180;
        item.rarity = Rarity::Rare;
        item.description = "Heavy plate armor worn by knights.".to_string();
        item.synergy_tags = vec![SynergyTag::Knight];
        item.affixes.push(Affix { affix_type: AffixType::BonusHP, value: 15 });
        item.generate_name();
        item
    }

    pub fn shadow_cloak(id: ItemId) -> Item {
        let mut item = Item::new(id, "Shadow Cloak", ItemCategory::Armor);
        item.equip_slot = Some(EquipSlot::Body);
        item.armor_type = Some(ArmorType::Cloth);
        item.base_armor = ArmorType::Cloth.base_armor(EquipSlot::Body) + 1;
        item.glyph = '👘';
        item.grid_size = (2, 2);
        item.value = 95;
        item.rarity = Rarity::Uncommon;
        item.description = "A cloak that drinks in light.".to_string();
        item.synergy_tags = vec![SynergyTag::Shadow];
        item.affixes.push(Affix { affix_type: AffixType::BonusDodge, value: 8 });
        item.generate_name();
        item
    }

    pub fn venom_blade(id: ItemId) -> Item {
        let mut item = Item::new(id, "Venom Blade", ItemCategory::Weapon);
        item.equip_slot = Some(EquipSlot::MainHand);
        item.weapon_type = Some(WeaponType::Sword);
        item.base_damage = WeaponType::Sword.base_damage();
        item.glyph = '🗡';
        item.grid_size = (1, 2);
        item.value = 110;
        item.rarity = Rarity::Uncommon;
        item.description = "Poison drips from this cruel blade.".to_string();
        item.synergy_tags = vec![SynergyTag::Poison];
        item.affixes.push(Affix { affix_type: AffixType::PoisonDamage, value: 5 });
        item.generate_name();
        item
    }

    pub fn corrupted_gauntlets(id: ItemId) -> Item {
        let mut item = Item::new(id, "Corrupted Gauntlets", ItemCategory::Armor);
        item.equip_slot = Some(EquipSlot::Hands);
        item.armor_type = Some(ArmorType::Leather);
        item.base_armor = ArmorType::Leather.base_armor(EquipSlot::Hands) + 2;
        item.glyph = '🧤';
        item.grid_size = (1, 1);
        item.value = 70;
        item.rarity = Rarity::Uncommon;
        item.description = "Gauntlets stained with dark power.".to_string();
        item.synergy_tags = vec![SynergyTag::Corruption];
        item.affixes.push(Affix { affix_type: AffixType::BonusDamage, value: 3 });
        item.generate_name();
        item
    }

    // Base equipment templates for all slots
    pub fn leather_boots(id: ItemId) -> Item {
        let mut item = Item::new(id, "Leather Boots", ItemCategory::Armor);
        item.equip_slot = Some(EquipSlot::Feet);
        item.armor_type = Some(ArmorType::Leather);
        item.base_armor = ArmorType::Leather.base_armor(EquipSlot::Feet);
        item.glyph = '👢';
        item.grid_size = (1, 1);
        item.value = 35;
        item.description = "Simple leather boots.".to_string();
        item
    }

    pub fn chain_boots(id: ItemId) -> Item {
        let mut item = Item::new(id, "Chain Boots", ItemCategory::Armor);
        item.equip_slot = Some(EquipSlot::Feet);
        item.armor_type = Some(ArmorType::Chain);
        item.base_armor = ArmorType::Chain.base_armor(EquipSlot::Feet);
        item.glyph = '👢';
        item.grid_size = (1, 1);
        item.value = 55;
        item.description = "Chain mail boots.".to_string();
        item.synergy_tags = vec![SynergyTag::Knight];
        item
    }

    pub fn leather_gloves(id: ItemId) -> Item {
        let mut item = Item::new(id, "Leather Gloves", ItemCategory::Armor);
        item.equip_slot = Some(EquipSlot::Hands);
        item.armor_type = Some(ArmorType::Leather);
        item.base_armor = ArmorType::Leather.base_armor(EquipSlot::Hands);
        item.glyph = '🧤';
        item.grid_size = (1, 1);
        item.value = 30;
        item.description = "Basic leather gloves.".to_string();
        item
    }

    pub fn wooden_shield(id: ItemId) -> Item {
        let mut item = Item::new(id, "Wooden Shield", ItemCategory::Armor);
        item.equip_slot = Some(EquipSlot::OffHand);
        item.armor_type = Some(ArmorType::Leather); // Wood similar to leather for stats
        item.base_armor = 3;
        item.glyph = '⛨';
        item.grid_size = (2, 2);
        item.value = 40;
        item.description = "A simple wooden shield.".to_string();
        item
    }

    pub fn iron_shield(id: ItemId) -> Item {
        let mut item = Item::new(id, "Iron Shield", ItemCategory::Armor);
        item.equip_slot = Some(EquipSlot::OffHand);
        item.armor_type = Some(ArmorType::Chain);
        item.base_armor = 5;
        item.glyph = '⛨';
        item.grid_size = (2, 2);
        item.value = 70;
        item.description = "A sturdy iron shield.".to_string();
        item.synergy_tags = vec![SynergyTag::Knight];
        item
    }

    pub fn bone_ring(id: ItemId) -> Item {
        let mut item = Item::new(id, "Bone Ring", ItemCategory::Accessory);
        item.equip_slot = Some(EquipSlot::Ring1);
        item.glyph = '◯';
        item.grid_size = (1, 1);
        item.value = 50;
        item.description = "A ring carved from ancient bone.".to_string();
        item
    }

    pub fn copper_amulet(id: ItemId) -> Item {
        let mut item = Item::new(id, "Copper Amulet", ItemCategory::Accessory);
        item.equip_slot = Some(EquipSlot::Amulet);
        item.glyph = '♦';
        item.grid_size = (1, 1);
        item.value = 45;
        item.description = "A tarnished copper amulet.".to_string();
        item
    }

    pub fn silver_ring(id: ItemId) -> Item {
        let mut item = Item::new(id, "Silver Ring", ItemCategory::Accessory);
        item.equip_slot = Some(EquipSlot::Ring1);
        item.glyph = '◯';
        item.grid_size = (1, 1);
        item.value = 80;
        item.description = "A polished silver ring.".to_string();
        item.synergy_tags = vec![SynergyTag::Knight];
        item
    }
}
