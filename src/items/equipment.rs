//! Equipment system
//!
//! Manages equipped items and calculates total bonuses.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::item::{Item, EquipSlot, AffixType};
use super::synergies::{SynergyTag, SynergyBonuses, ActiveSynergy, calculate_synergies};

/// Player equipment slots
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Equipment {
    /// Items in each slot
    slots: HashMap<EquipSlot, Item>,
}

impl Equipment {
    pub fn new() -> Self {
        Self {
            slots: HashMap::new(),
        }
    }

    /// Equip an item, returning the previously equipped item if any
    pub fn equip(&mut self, item: Item) -> Option<Item> {
        if let Some(slot) = item.equip_slot {
            self.slots.insert(slot, item)
        } else {
            None // Item not equippable
        }
    }

    /// Unequip an item from a slot
    pub fn unequip(&mut self, slot: EquipSlot) -> Option<Item> {
        self.slots.remove(&slot)
    }

    /// Get item in a slot
    pub fn get(&self, slot: EquipSlot) -> Option<&Item> {
        self.slots.get(&slot)
    }

    /// Get mutable reference to item in a slot
    pub fn get_mut(&mut self, slot: EquipSlot) -> Option<&mut Item> {
        self.slots.get_mut(&slot)
    }

    /// Check if a slot is empty
    pub fn is_empty(&self, slot: EquipSlot) -> bool {
        !self.slots.contains_key(&slot)
    }

    /// Get all equipped items
    pub fn all_items(&self) -> impl Iterator<Item = &Item> {
        self.slots.values()
    }

    /// Calculate total bonus damage from all equipment
    pub fn total_damage_bonus(&self) -> i32 {
        self.slots.values()
            .map(|item| item.total_damage())
            .sum()
    }

    /// Calculate total armor from all equipment
    pub fn total_armor(&self) -> i32 {
        self.slots.values()
            .map(|item| item.total_armor())
            .sum()
    }

    /// Calculate total stat bonus
    pub fn stat_bonus(&self, stat: AffixType) -> i32 {
        self.slots.values()
            .map(|item| item.stat_bonus(stat))
            .sum()
    }

    /// Get main hand weapon damage (or 0 if unarmed)
    pub fn weapon_damage(&self) -> i32 {
        self.get(EquipSlot::MainHand)
            .map(|w| w.total_damage())
            .unwrap_or(2) // Unarmed = 2 damage
    }

    /// Get weapon crit bonus
    pub fn weapon_crit_bonus(&self) -> f32 {
        self.get(EquipSlot::MainHand)
            .and_then(|w| w.weapon_type)
            .map(|wt| wt.crit_bonus())
            .unwrap_or(0.0)
    }

    /// Get strength bonus from equipment
    pub fn strength_bonus(&self) -> i32 {
        self.stat_bonus(AffixType::BonusStrength)
    }

    /// Get dexterity bonus from equipment
    pub fn dexterity_bonus(&self) -> i32 {
        self.stat_bonus(AffixType::BonusDexterity)
    }

    /// Get intelligence bonus from equipment
    pub fn intelligence_bonus(&self) -> i32 {
        self.stat_bonus(AffixType::BonusIntelligence)
    }

    /// Get vitality bonus from equipment
    pub fn vitality_bonus(&self) -> i32 {
        self.stat_bonus(AffixType::BonusVitality)
    }

    /// Get HP bonus from equipment
    pub fn hp_bonus(&self) -> i32 {
        self.stat_bonus(AffixType::BonusHP)
    }

    /// Get MP bonus from equipment
    pub fn mp_bonus(&self) -> i32 {
        self.stat_bonus(AffixType::BonusMP)
    }

    /// Get all synergy tags from equipped items
    pub fn synergy_tags(&self) -> Vec<SynergyTag> {
        let mut tags = Vec::new();
        for item in self.slots.values() {
            tags.extend(item.all_synergy_tags());
        }
        tags
    }

    /// Get active synergies from equipped items
    pub fn active_synergies(&self) -> Vec<ActiveSynergy> {
        let tags = self.synergy_tags();
        calculate_synergies(&tags)
    }

    /// Get aggregated synergy bonuses
    pub fn synergy_bonuses(&self) -> SynergyBonuses {
        let tags = self.synergy_tags();
        SynergyBonuses::from_tags(&tags)
    }
}

/// Equipment slot display info
impl EquipSlot {
    pub fn name(&self) -> &'static str {
        match self {
            EquipSlot::MainHand => "Main Hand",
            EquipSlot::OffHand => "Off Hand",
            EquipSlot::Head => "Head",
            EquipSlot::Body => "Body",
            EquipSlot::Hands => "Hands",
            EquipSlot::Feet => "Feet",
            EquipSlot::Amulet => "Amulet",
            EquipSlot::Ring1 => "Ring 1",
            EquipSlot::Ring2 => "Ring 2",
        }
    }

    pub fn glyph(&self) -> char {
        match self {
            EquipSlot::MainHand => '/',
            EquipSlot::OffHand => ')',
            EquipSlot::Head => '^',
            EquipSlot::Body => '[',
            EquipSlot::Hands => '{',
            EquipSlot::Feet => '"',
            EquipSlot::Amulet => '♦',
            EquipSlot::Ring1 => '◯',
            EquipSlot::Ring2 => '◯',
        }
    }

    /// Get all slots in display order
    pub fn all() -> &'static [EquipSlot] {
        &[
            EquipSlot::MainHand,
            EquipSlot::OffHand,
            EquipSlot::Head,
            EquipSlot::Body,
            EquipSlot::Hands,
            EquipSlot::Feet,
            EquipSlot::Amulet,
            EquipSlot::Ring1,
            EquipSlot::Ring2,
        ]
    }
}
