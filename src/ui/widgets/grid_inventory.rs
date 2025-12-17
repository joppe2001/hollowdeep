//! Grid inventory widget for ratatui
//!
//! Renders an 8x6 grid-based inventory (RE4 style).

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Widget},
};

use crate::items::{Inventory, ItemId, Item, Rarity, GRID_WIDTH, GRID_HEIGHT};

/// Grid cursor position
#[derive(Debug, Clone, Copy, Default)]
pub struct GridCursor {
    pub x: u8,
    pub y: u8,
}

impl GridCursor {
    pub fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }

    /// Move cursor up
    pub fn move_up(&mut self) {
        if self.y > 0 {
            self.y -= 1;
        }
    }

    /// Move cursor down
    pub fn move_down(&mut self) {
        if (self.y as usize) < GRID_HEIGHT - 1 {
            self.y += 1;
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self) {
        if self.x > 0 {
            self.x -= 1;
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        if (self.x as usize) < GRID_WIDTH - 1 {
            self.x += 1;
        }
    }
}

/// Widget for rendering grid inventory
pub struct GridInventoryWidget<'a> {
    inventory: &'a Inventory,
    cursor: GridCursor,
    selected_item: Option<ItemId>,
    title: &'a str,
}

impl<'a> GridInventoryWidget<'a> {
    pub fn new(inventory: &'a Inventory) -> Self {
        Self {
            inventory,
            cursor: GridCursor::default(),
            selected_item: None,
            title: "Inventory",
        }
    }

    pub fn cursor(mut self, cursor: GridCursor) -> Self {
        self.cursor = cursor;
        self
    }

    pub fn selected_item(mut self, item_id: Option<ItemId>) -> Self {
        self.selected_item = item_id;
        self
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    /// Get the cell character and style for a given position
    fn cell_style(&self, x: u8, y: u8) -> (char, Style) {
        let is_cursor = self.cursor.x == x && self.cursor.y == y;
        let cells = self.inventory.cells();

        if let Some(item_id) = cells[y as usize][x as usize] {
            // Cell occupied by an item
            if let Some(placed) = self.inventory.get_placed_at(x, y) {
                let is_origin = placed.position.x == x && placed.position.y == y;
                let item = &placed.item;

                // Get rarity color
                let color = rarity_color(item.rarity);

                let glyph = if is_origin {
                    item.glyph
                } else {
                    '█' // Fill character for multi-cell items
                };

                let mut style = Style::default().fg(color);

                if is_cursor {
                    style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
                }

                if Some(item_id) == self.selected_item {
                    style = style.add_modifier(Modifier::REVERSED);
                }

                (glyph, style)
            } else {
                ('?', Style::default().fg(Color::Red))
            }
        } else {
            // Empty cell
            let glyph = '·';
            let mut style = Style::default().fg(Color::Rgb(60, 60, 70));

            if is_cursor {
                style = style.bg(Color::Rgb(40, 40, 50)).fg(Color::White);
            }

            (glyph, style)
        }
    }
}

impl<'a> Widget for GridInventoryWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render border
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(100, 100, 120)));

        let inner = block.inner(area);
        block.render(area, buf);

        // Render grid cells
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let (glyph, style) = self.cell_style(x as u8, y as u8);

                // Each cell is 2 characters wide for better visibility
                let cell_x = inner.x + (x as u16 * 2);
                let cell_y = inner.y + y as u16;

                if cell_x < inner.x + inner.width && cell_y < inner.y + inner.height {
                    if let Some(cell) = buf.cell_mut((cell_x, cell_y)) {
                        cell.set_char(glyph).set_style(style);
                    }
                    if let Some(cell) = buf.cell_mut((cell_x + 1, cell_y)) {
                        cell.set_char(' ').set_style(style);
                    }
                }
            }
        }
    }
}

/// Get the color for an item rarity
pub fn rarity_color(rarity: Rarity) -> Color {
    let (r, g, b) = rarity.color();
    Color::Rgb(r, g, b)
}

/// Render item details panel
pub fn render_item_details(item: &Item, area: Rect, buf: &mut Buffer) {
    let block = Block::default()
        .title("Item Details")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(100, 100, 120)));

    let inner = block.inner(area);
    block.render(area, buf);

    let mut y = inner.y;

    // Item name with rarity color
    let name_style = Style::default()
        .fg(rarity_color(item.rarity))
        .add_modifier(Modifier::BOLD);
    let name_line = format!("{}", item.name);
    if y < inner.y + inner.height {
        buf.set_string(inner.x, y, &name_line, name_style);
        y += 1;
    }

    // Rarity
    if y < inner.y + inner.height {
        let rarity_text = format!("[{}]", item.rarity.name());
        buf.set_string(inner.x, y, &rarity_text, Style::default().fg(rarity_color(item.rarity)));
        y += 1;
    }

    // Category and slot
    if y < inner.y + inner.height {
        let cat_text = format!("{:?}", item.category);
        buf.set_string(inner.x, y, &cat_text, Style::default().fg(Color::White));
        y += 1;
    }

    // Stats
    y += 1; // Blank line

    if item.base_damage > 0 && y < inner.y + inner.height {
        buf.set_string(inner.x, y, &format!("Damage: {}", item.base_damage), Style::default().fg(Color::Rgb(255, 150, 150)));
        y += 1;
    }

    if item.base_armor > 0 && y < inner.y + inner.height {
        buf.set_string(inner.x, y, &format!("Armor: {}", item.base_armor), Style::default().fg(Color::Rgb(150, 150, 255)));
        y += 1;
    }

    // Affixes
    for affix in &item.affixes {
        if y >= inner.y + inner.height {
            break;
        }
        let affix_text = format!("+{} {}", affix.value, affix.affix_type.description());
        buf.set_string(inner.x, y, &affix_text, Style::default().fg(Color::Rgb(100, 255, 100)));
        y += 1;
    }

    // Value
    y += 1;
    if y < inner.y + inner.height {
        buf.set_string(inner.x, y, &format!("Value: {} gold", item.value), Style::default().fg(Color::Yellow));
        y += 1;
    }

    // Grid size
    if y < inner.y + inner.height {
        buf.set_string(inner.x, y, &format!("Size: {}x{}", item.grid_size.0, item.grid_size.1), Style::default().fg(Color::DarkGray));
    }
}

/// Render help text for grid inventory controls
pub fn render_grid_help(area: Rect, buf: &mut Buffer) {
    let help_lines = vec![
        ("↑↓←→", "Move cursor"),
        ("Enter", "Select/Use"),
        ("R", "Rotate item"),
        ("D", "Drop item"),
        ("S", "Sort inventory"),
        ("Esc", "Close"),
    ];

    let mut y = area.y;
    for (key, desc) in help_lines {
        if y >= area.y + area.height {
            break;
        }
        let key_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(Color::DarkGray);

        buf.set_string(area.x, y, key, key_style);
        buf.set_string(area.x + 6, y, desc, desc_style);
        y += 1;
    }
}
