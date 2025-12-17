//! Inventory system
//!
//! Manages player's item collection using a grid-based system (RE4 style).

use serde::{Deserialize, Serialize};
use super::item::{Item, ItemId, ItemCategory};
use super::grid::{InventoryGrid, PlacedItem, GRID_WIDTH, GRID_HEIGHT, SortMode};

/// Player inventory using a grid-based system
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Inventory {
    /// Grid-based item storage
    grid: InventoryGrid,
    /// Gold currency
    gold: u32,
}

impl Inventory {
    /// Create a new inventory
    pub fn new() -> Self {
        Self {
            grid: InventoryGrid::new(),
            gold: 0,
        }
    }

    /// Get current number of items
    pub fn count(&self) -> usize {
        self.grid.count()
    }

    /// Get grid dimensions
    pub fn grid_size() -> (usize, usize) {
        (GRID_WIDTH, GRID_HEIGHT)
    }

    /// Get total cell capacity
    pub fn capacity(&self) -> usize {
        GRID_WIDTH * GRID_HEIGHT
    }

    /// Check if inventory has any space
    pub fn is_full(&self) -> bool {
        !self.grid.has_space()
    }

    /// Check if inventory has space for an item
    pub fn has_space(&self) -> bool {
        self.grid.has_space()
    }

    /// Check if a specific item can fit
    pub fn can_fit(&self, item: &Item) -> bool {
        self.grid.find_space_for(item).is_some()
    }

    /// Add an item to inventory
    /// Returns false if no space available
    pub fn add_item(&mut self, item: Item) -> bool {
        self.grid.add_item(item)
    }

    /// Place an item at a specific grid position
    pub fn place_at(&mut self, item: Item, x: u8, y: u8, rotated: bool) -> bool {
        self.grid.place_at(item, x, y, rotated)
    }

    /// Remove an item by index (for legacy API)
    /// In grid inventory, this removes the nth item in iteration order
    pub fn remove_at(&mut self, index: usize) -> Option<Item> {
        let items: Vec<ItemId> = self.grid.placed_items().into_iter().map(|p| p.item.id).collect();
        if index < items.len() {
            self.grid.remove(items[index])
        } else {
            None
        }
    }

    /// Remove an item by ID
    pub fn remove_by_id(&mut self, id: ItemId) -> Option<Item> {
        self.grid.remove(id)
    }

    /// Remove item at a grid position
    pub fn remove_at_grid(&mut self, x: u8, y: u8) -> Option<Item> {
        self.grid.remove_at(x, y)
    }

    /// Get item by index (legacy API)
    pub fn get(&self, index: usize) -> Option<&Item> {
        self.grid.items().get(index).copied()
    }

    /// Get item by ID
    pub fn get_by_id(&self, id: ItemId) -> Option<&Item> {
        self.grid.get_by_id(id)
    }

    /// Get item at a grid position
    pub fn get_at_grid(&self, x: u8, y: u8) -> Option<&Item> {
        self.grid.get_at(x, y)
    }

    /// Get placed item at a grid position (includes position/rotation info)
    pub fn get_placed_at(&self, x: u8, y: u8) -> Option<&PlacedItem> {
        self.grid.get_placed_at(x, y)
    }

    /// Get all items (references)
    pub fn items(&self) -> Vec<&Item> {
        self.grid.items()
    }

    /// Get all items as owned values (for serialization)
    pub fn items_owned(&self) -> Vec<Item> {
        self.grid.items().into_iter().cloned().collect()
    }

    /// Get all placed items (with grid position info)
    pub fn placed_items(&self) -> Vec<&PlacedItem> {
        self.grid.placed_items()
    }

    /// Get access to the underlying grid
    pub fn grid(&self) -> &InventoryGrid {
        &self.grid
    }

    /// Get mutable access to the underlying grid
    pub fn grid_mut(&mut self) -> &mut InventoryGrid {
        &mut self.grid
    }

    /// Get items of a specific category
    pub fn items_of_category(&self, category: ItemCategory) -> Vec<&Item> {
        self.grid.items().into_iter()
            .filter(|i| i.category == category)
            .collect()
    }

    /// Find first consumable that matches a predicate
    pub fn find_consumable<F>(&self, f: F) -> Option<(usize, &Item)>
    where
        F: Fn(&Item) -> bool,
    {
        self.grid.items().into_iter()
            .enumerate()
            .find(|(_, i)| i.category == ItemCategory::Consumable && f(i))
    }

    /// Use (consume) one item from a stack at index
    /// Returns the consumed item if successful
    pub fn consume_at(&mut self, index: usize) -> Option<Item> {
        let items: Vec<ItemId> = self.grid.placed_items().into_iter().map(|p| p.item.id).collect();
        if index >= items.len() {
            return None;
        }

        let id = items[index];
        let item = self.grid.get_by_id_mut(id)?;

        if !item.is_consumable() {
            return None;
        }

        if item.stack_count > 1 {
            item.stack_count -= 1;
            let mut consumed = item.clone();
            consumed.stack_count = 1;
            Some(consumed)
        } else {
            self.grid.remove(id)
        }
    }

    /// Get current gold
    pub fn gold(&self) -> u32 {
        self.gold
    }

    /// Add gold
    pub fn add_gold(&mut self, amount: u32) {
        self.gold = self.gold.saturating_add(amount);
    }

    /// Spend gold, returns false if not enough
    pub fn spend_gold(&mut self, amount: u32) -> bool {
        if self.gold >= amount {
            self.gold -= amount;
            true
        } else {
            false
        }
    }

    /// Sort/organize items (auto-organize the grid)
    pub fn sort(&mut self) {
        self.grid.auto_organize();
    }

    /// Sort inventory by specified mode
    pub fn sort_by(&mut self, mode: SortMode) {
        self.grid.sort_by(mode);
    }

    /// Mark all items as seen
    pub fn mark_all_seen(&mut self) {
        self.grid.mark_all_seen();
    }

    /// Check if there are new items
    pub fn has_new_items(&self) -> bool {
        self.grid.has_new_items()
    }

    /// Count new items
    pub fn count_new(&self) -> usize {
        self.grid.count_new()
    }

    /// Rotate an item in the grid
    pub fn rotate_item(&mut self, id: ItemId) -> bool {
        self.grid.rotate_item(id)
    }

    /// Move an item to a new grid position
    pub fn move_item(&mut self, id: ItemId, x: u8, y: u8) -> bool {
        self.grid.move_item(id, x, y)
    }

    /// Get the grid cells for rendering
    pub fn cells(&self) -> &[[Option<ItemId>; GRID_WIDTH]; GRID_HEIGHT] {
        self.grid.cells()
    }
}
