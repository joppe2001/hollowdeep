//! Grid-based inventory system (Resident Evil 4 style)
//!
//! Items occupy multiple cells based on their size, and can be rotated
//! to fit into available space.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::item::{Item, ItemId};

/// Sorting options for inventory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SortMode {
    /// Sort by grid size (largest first) - default tetris packing
    #[default]
    Size,
    /// Sort by rarity (legendary first), then by category
    Rarity,
    /// Sort by category (equipment first), then by rarity
    Category,
    /// Sort by name alphabetically
    Name,
    /// Sort with new items first, then by rarity
    New,
}

/// Grid dimensions
pub const GRID_WIDTH: usize = 8;
pub const GRID_HEIGHT: usize = 6;

/// Position in the inventory grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridPosition {
    pub x: u8,
    pub y: u8,
}

impl GridPosition {
    pub fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }
}

/// An item placed in the grid with its position and rotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacedItem {
    pub item: Item,
    pub position: GridPosition,
    pub rotated: bool, // If true, width and height are swapped
}

impl PlacedItem {
    /// Get the effective width (accounting for rotation)
    pub fn width(&self) -> u8 {
        if self.rotated {
            self.item.grid_size.1
        } else {
            self.item.grid_size.0
        }
    }

    /// Get the effective height (accounting for rotation)
    pub fn height(&self) -> u8 {
        if self.rotated {
            self.item.grid_size.0
        } else {
            self.item.grid_size.1
        }
    }

    /// Get all cells occupied by this item
    pub fn occupied_cells(&self) -> Vec<GridPosition> {
        let mut cells = Vec::new();
        for dy in 0..self.height() {
            for dx in 0..self.width() {
                cells.push(GridPosition::new(
                    self.position.x + dx,
                    self.position.y + dy,
                ));
            }
        }
        cells
    }
}

/// Grid-based inventory
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InventoryGrid {
    /// Grid cells - each cell contains Option<ItemId> pointing to the item that occupies it
    cells: [[Option<ItemId>; GRID_WIDTH]; GRID_HEIGHT],
    /// Items stored by ID
    items: HashMap<ItemId, PlacedItem>,
}

impl InventoryGrid {
    /// Create a new empty inventory grid
    pub fn new() -> Self {
        Self {
            cells: [[None; GRID_WIDTH]; GRID_HEIGHT],
            items: HashMap::new(),
        }
    }

    /// Check if a position is valid within the grid
    pub fn is_valid_position(&self, x: u8, y: u8) -> bool {
        (x as usize) < GRID_WIDTH && (y as usize) < GRID_HEIGHT
    }

    /// Check if an item can be placed at a position
    pub fn can_place_at(&self, x: u8, y: u8, width: u8, height: u8) -> bool {
        // Check all cells the item would occupy
        for dy in 0..height {
            for dx in 0..width {
                let nx = x + dx;
                let ny = y + dy;

                // Check bounds
                if !self.is_valid_position(nx, ny) {
                    return false;
                }

                // Check if cell is occupied
                if self.cells[ny as usize][nx as usize].is_some() {
                    return false;
                }
            }
        }
        true
    }

    /// Check if an item can be placed at a position (considering rotation)
    pub fn can_place_item_at(&self, item: &Item, x: u8, y: u8, rotated: bool) -> bool {
        let (width, height) = if rotated {
            (item.grid_size.1, item.grid_size.0)
        } else {
            item.grid_size
        };
        self.can_place_at(x, y, width, height)
    }

    /// Find the first available space for an item
    pub fn find_space_for(&self, item: &Item) -> Option<(GridPosition, bool)> {
        let (w, h) = item.grid_size;

        // Try without rotation first
        for y in 0..GRID_HEIGHT as u8 {
            for x in 0..GRID_WIDTH as u8 {
                if self.can_place_at(x, y, w, h) {
                    return Some((GridPosition::new(x, y), false));
                }
            }
        }

        // Try with rotation if item isn't square
        if w != h {
            for y in 0..GRID_HEIGHT as u8 {
                for x in 0..GRID_WIDTH as u8 {
                    if self.can_place_at(x, y, h, w) {
                        return Some((GridPosition::new(x, y), true));
                    }
                }
            }
        }

        None
    }

    /// Place an item at a specific position
    pub fn place_at(&mut self, item: Item, x: u8, y: u8, rotated: bool) -> bool {
        if !self.can_place_item_at(&item, x, y, rotated) {
            return false;
        }

        let item_id = item.id;
        let placed = PlacedItem {
            item,
            position: GridPosition::new(x, y),
            rotated,
        };

        // Mark cells as occupied
        for cell in placed.occupied_cells() {
            self.cells[cell.y as usize][cell.x as usize] = Some(item_id);
        }

        self.items.insert(item_id, placed);
        true
    }

    /// Add an item to the inventory, automatically finding space
    pub fn add_item(&mut self, item: Item) -> bool {
        // Check if stackable first
        if item.is_stackable() {
            for placed in self.items.values_mut() {
                if placed.item.base_name == item.base_name
                    && placed.item.stack_count < placed.item.max_stack
                {
                    let can_add = placed.item.max_stack - placed.item.stack_count;
                    let to_add = item.stack_count.min(can_add);
                    placed.item.stack_count += to_add;
                    if to_add >= item.stack_count {
                        return true;
                    }
                }
            }
        }

        // Find space and place
        if let Some((pos, rotated)) = self.find_space_for(&item) {
            self.place_at(item, pos.x, pos.y, rotated)
        } else {
            false
        }
    }

    /// Remove an item by ID
    pub fn remove(&mut self, id: ItemId) -> Option<Item> {
        if let Some(placed) = self.items.remove(&id) {
            // Clear cells
            for cell in placed.occupied_cells() {
                self.cells[cell.y as usize][cell.x as usize] = None;
            }
            Some(placed.item)
        } else {
            None
        }
    }

    /// Remove item at a grid position
    pub fn remove_at(&mut self, x: u8, y: u8) -> Option<Item> {
        if !self.is_valid_position(x, y) {
            return None;
        }

        if let Some(id) = self.cells[y as usize][x as usize] {
            self.remove(id)
        } else {
            None
        }
    }

    /// Get the item at a grid position
    pub fn get_at(&self, x: u8, y: u8) -> Option<&Item> {
        if !self.is_valid_position(x, y) {
            return None;
        }

        self.cells[y as usize][x as usize]
            .and_then(|id| self.items.get(&id))
            .map(|p| &p.item)
    }

    /// Get placed item at a grid position
    pub fn get_placed_at(&self, x: u8, y: u8) -> Option<&PlacedItem> {
        if !self.is_valid_position(x, y) {
            return None;
        }

        self.cells[y as usize][x as usize]
            .and_then(|id| self.items.get(&id))
    }

    /// Get item by ID
    pub fn get_by_id(&self, id: ItemId) -> Option<&Item> {
        self.items.get(&id).map(|p| &p.item)
    }

    /// Get mutable item by ID
    pub fn get_by_id_mut(&mut self, id: ItemId) -> Option<&mut Item> {
        self.items.get_mut(&id).map(|p| &mut p.item)
    }

    /// Get all items in the grid (in grid position order: top-left to bottom-right)
    pub fn items(&self) -> Vec<&Item> {
        // Collect unique items in grid position order (row by row, left to right)
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();

        for row in &self.cells {
            for cell in row {
                if let Some(id) = cell {
                    if seen.insert(*id) {
                        if let Some(placed) = self.items.get(id) {
                            result.push(&placed.item);
                        }
                    }
                }
            }
        }
        result
    }

    /// Get all placed items (in grid position order)
    pub fn placed_items(&self) -> Vec<&PlacedItem> {
        // Collect unique placed items in grid position order
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();

        for row in &self.cells {
            for cell in row {
                if let Some(id) = cell {
                    if seen.insert(*id) {
                        if let Some(placed) = self.items.get(id) {
                            result.push(placed);
                        }
                    }
                }
            }
        }
        result
    }

    /// Get item count
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Check if the grid is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Check if any space is available
    pub fn has_space(&self) -> bool {
        self.cells.iter().any(|row| row.iter().any(|c| c.is_none()))
    }

    /// Get the cell contents (for rendering)
    pub fn cells(&self) -> &[[Option<ItemId>; GRID_WIDTH]; GRID_HEIGHT] {
        &self.cells
    }

    /// Rotate an item in place (if possible)
    pub fn rotate_item(&mut self, id: ItemId) -> bool {
        let placed = match self.items.get(&id) {
            Some(p) => p.clone(),
            None => return false,
        };

        // Check if rotation would fit
        let new_rotated = !placed.rotated;
        let (new_w, new_h) = if new_rotated {
            (placed.item.grid_size.1, placed.item.grid_size.0)
        } else {
            placed.item.grid_size
        };

        // Clear current cells temporarily
        for cell in placed.occupied_cells() {
            self.cells[cell.y as usize][cell.x as usize] = None;
        }

        // Check if new rotation fits
        let can_rotate = self.can_place_at(placed.position.x, placed.position.y, new_w, new_h);

        if can_rotate {
            // Apply rotation
            let new_placed = PlacedItem {
                item: placed.item.clone(),
                position: placed.position,
                rotated: new_rotated,
            };

            for cell in new_placed.occupied_cells() {
                self.cells[cell.y as usize][cell.x as usize] = Some(id);
            }

            self.items.insert(id, new_placed);
            true
        } else {
            // Restore original cells
            for cell in placed.occupied_cells() {
                self.cells[cell.y as usize][cell.x as usize] = Some(id);
            }
            false
        }
    }

    /// Move an item to a new position (if possible)
    pub fn move_item(&mut self, id: ItemId, new_x: u8, new_y: u8) -> bool {
        let placed = match self.items.get(&id) {
            Some(p) => p.clone(),
            None => return false,
        };

        // Clear current cells temporarily
        for cell in placed.occupied_cells() {
            self.cells[cell.y as usize][cell.x as usize] = None;
        }

        // Check if new position fits
        let can_move = self.can_place_item_at(&placed.item, new_x, new_y, placed.rotated);

        if can_move {
            // Apply move
            let new_placed = PlacedItem {
                item: placed.item.clone(),
                position: GridPosition::new(new_x, new_y),
                rotated: placed.rotated,
            };

            for cell in new_placed.occupied_cells() {
                self.cells[cell.y as usize][cell.x as usize] = Some(id);
            }

            self.items.insert(id, new_placed);
            true
        } else {
            // Restore original cells
            for cell in placed.occupied_cells() {
                self.cells[cell.y as usize][cell.x as usize] = Some(id);
            }
            false
        }
    }

    /// Auto-organize inventory (tetris-style packing)
    pub fn auto_organize(&mut self) {
        self.sort_by(SortMode::Size);
    }

    /// Sort and organize inventory by the specified mode
    pub fn sort_by(&mut self, mode: SortMode) {
        // Collect all items
        let mut items: Vec<Item> = self.items.drain().map(|(_, p)| p.item).collect();

        // Sort based on mode
        match mode {
            SortMode::Size => {
                // Sort by area (largest first) then by height
                items.sort_by(|a, b| {
                    let area_a = (a.grid_size.0 as u16) * (a.grid_size.1 as u16);
                    let area_b = (b.grid_size.0 as u16) * (b.grid_size.1 as u16);
                    area_b.cmp(&area_a)
                        .then_with(|| b.grid_size.1.cmp(&a.grid_size.1))
                });
            }
            SortMode::Rarity => {
                // Sort by rarity (highest first), then category, then name
                items.sort_by(|a, b| {
                    b.rarity.sort_value().cmp(&a.rarity.sort_value())
                        .then_with(|| a.category.sort_value().cmp(&b.category.sort_value()))
                        .then_with(|| a.name.cmp(&b.name))
                });
            }
            SortMode::Category => {
                // Sort by category (equipment first), then rarity, then name
                items.sort_by(|a, b| {
                    a.category.sort_value().cmp(&b.category.sort_value())
                        .then_with(|| b.rarity.sort_value().cmp(&a.rarity.sort_value()))
                        .then_with(|| a.name.cmp(&b.name))
                });
            }
            SortMode::Name => {
                // Sort alphabetically by name
                items.sort_by(|a, b| a.name.cmp(&b.name));
            }
            SortMode::New => {
                // Sort new items first, then by rarity
                items.sort_by(|a, b| {
                    b.is_new.cmp(&a.is_new)
                        .then_with(|| b.rarity.sort_value().cmp(&a.rarity.sort_value()))
                        .then_with(|| a.category.sort_value().cmp(&b.category.sort_value()))
                });
            }
        }

        // Clear all cells
        self.cells = [[None; GRID_WIDTH]; GRID_HEIGHT];

        // Re-add all items
        for item in items {
            let _ = self.add_item(item);
        }
    }

    /// Mark all items as seen (not new)
    pub fn mark_all_seen(&mut self) {
        for placed in self.items.values_mut() {
            placed.item.is_new = false;
        }
    }

    /// Check if there are any new items
    pub fn has_new_items(&self) -> bool {
        self.items.values().any(|p| p.item.is_new)
    }

    /// Count new items
    pub fn count_new(&self) -> usize {
        self.items.values().filter(|p| p.item.is_new).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::item::{ItemCategory, Rarity};

    fn make_test_item(id: ItemId, width: u8, height: u8) -> Item {
        let mut item = Item::new(id, "Test Item", ItemCategory::Weapon);
        item.grid_size = (width, height);
        item.rarity = Rarity::Common;
        item
    }

    #[test]
    fn test_add_item() {
        let mut grid = InventoryGrid::new();
        let item = make_test_item(1, 2, 2);

        assert!(grid.add_item(item.clone()));
        assert_eq!(grid.count(), 1);
        assert!(grid.get_by_id(1).is_some());
    }

    #[test]
    fn test_remove_item() {
        let mut grid = InventoryGrid::new();
        let item = make_test_item(1, 2, 2);

        grid.add_item(item);
        let removed = grid.remove(1);

        assert!(removed.is_some());
        assert_eq!(grid.count(), 0);
    }

    #[test]
    fn test_grid_full() {
        let mut grid = InventoryGrid::new();

        // Fill grid with 1x1 items
        for i in 0..(GRID_WIDTH * GRID_HEIGHT) {
            let item = make_test_item(i as u64, 1, 1);
            assert!(grid.add_item(item));
        }

        // Should be full now
        let extra = make_test_item(999, 1, 1);
        assert!(!grid.add_item(extra));
    }

    #[test]
    fn test_rotation() {
        let mut grid = InventoryGrid::new();
        let item = make_test_item(1, 1, 3); // Tall item

        grid.add_item(item);
        let placed = grid.get_placed_at(0, 0).unwrap();
        assert!(!placed.rotated);

        // Rotate
        assert!(grid.rotate_item(1));
        let placed = grid.get_placed_at(0, 0).unwrap();
        assert!(placed.rotated);
    }
}
