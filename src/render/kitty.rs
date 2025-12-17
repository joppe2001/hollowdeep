//! Kitty Graphics Protocol implementation
//!
//! Allows rendering actual images/sprites in supporting terminals.
//! Supported by: Ghostty, Kitty, WezTerm, iTerm2
//!
//! Protocol documentation: https://sw.kovidgoyal.net/kitty/graphics-protocol/

use std::collections::HashMap;
use std::io::{self, Write};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use image::{DynamicImage, RgbaImage};

/// Kitty Graphics Protocol handler
pub struct KittyGraphics {
    /// Uploaded image IDs mapped to their dimensions
    uploaded_images: HashMap<u32, (u32, u32)>,
    /// Next available image ID
    next_id: u32,
    /// Cell size in pixels (width, height) - defaults to 8x16
    cell_size: (u16, u16),
    /// Whether to suppress terminal responses
    quiet: bool,
}

impl KittyGraphics {
    /// Create a new Kitty graphics handler
    pub fn new() -> Self {
        Self {
            uploaded_images: HashMap::new(),
            next_id: 1,
            cell_size: (8, 16), // Common default
            quiet: true,
        }
    }

    /// Set the cell size in pixels
    pub fn set_cell_size(&mut self, width: u16, height: u16) {
        self.cell_size = (width, height);
    }

    /// Upload an image and get an ID for later use
    /// This is efficient for sprites that are reused many times
    pub fn upload_image(&mut self, image: &DynamicImage) -> io::Result<u32> {
        let id = self.next_id;
        self.next_id += 1;

        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();

        // Encode as PNG for compression
        let png_data = encode_png(&rgba)?;
        let encoded = BASE64.encode(&png_data);

        // Build the upload command
        // a=t (transmit), t=d (direct), f=100 (PNG format), i=ID, q=quiet
        let mut stdout = io::stdout();

        // Send in chunks if large (Kitty has a limit per escape sequence)
        let chunk_size = 4096;
        let chunks: Vec<&str> = encoded
            .as_bytes()
            .chunks(chunk_size)
            .map(|c| std::str::from_utf8(c).unwrap())
            .collect();

        for (i, chunk) in chunks.iter().enumerate() {
            let is_last = i == chunks.len() - 1;
            let more = if is_last { 0 } else { 1 };

            if i == 0 {
                // First chunk includes all parameters
                write!(
                    stdout,
                    "\x1b_Ga=t,t=d,f=100,i={},q={},m={};{}\x1b\\",
                    id,
                    if self.quiet { 2 } else { 0 },
                    more,
                    chunk
                )?;
            } else {
                // Continuation chunks
                write!(stdout, "\x1b_Gm={};{}\x1b\\", more, chunk)?;
            }
        }

        stdout.flush()?;

        self.uploaded_images.insert(id, (width, height));
        Ok(id)
    }

    /// Display an uploaded image at the current cursor position
    pub fn display_image(&self, image_id: u32, cols: u16, rows: u16) -> io::Result<()> {
        let mut stdout = io::stdout();

        // a=p (put), i=ID, c=columns, r=rows, q=quiet
        write!(
            stdout,
            "\x1b_Ga=p,i={},c={},r={},q={}\x1b\\",
            image_id,
            cols,
            rows,
            if self.quiet { 2 } else { 0 }
        )?;

        stdout.flush()
    }

    /// Display an uploaded image at specific cell coordinates
    pub fn display_image_at(
        &self,
        image_id: u32,
        col: u16,
        row: u16,
        cols: u16,
        rows: u16,
    ) -> io::Result<()> {
        let mut stdout = io::stdout();

        // Move cursor to position first
        write!(stdout, "\x1b[{};{}H", row + 1, col + 1)?;

        // Then display image
        self.display_image(image_id, cols, rows)?;

        stdout.flush()
    }

    /// Upload and immediately display an image (one-shot)
    pub fn display_image_direct(
        &self,
        image: &DynamicImage,
        cols: u16,
        rows: u16,
    ) -> io::Result<()> {
        let rgba = image.to_rgba8();
        let png_data = encode_png(&rgba)?;
        let encoded = BASE64.encode(&png_data);

        let mut stdout = io::stdout();

        // a=T (transmit and display), t=d, f=100, c=cols, r=rows, q=quiet
        let chunk_size = 4096;
        let chunks: Vec<&str> = encoded
            .as_bytes()
            .chunks(chunk_size)
            .map(|c| std::str::from_utf8(c).unwrap())
            .collect();

        for (i, chunk) in chunks.iter().enumerate() {
            let is_last = i == chunks.len() - 1;
            let more = if is_last { 0 } else { 1 };

            if i == 0 {
                write!(
                    stdout,
                    "\x1b_Ga=T,t=d,f=100,c={},r={},q={},m={};{}\x1b\\",
                    cols,
                    rows,
                    if self.quiet { 2 } else { 0 },
                    more,
                    chunk
                )?;
            } else {
                write!(stdout, "\x1b_Gm={};{}\x1b\\", more, chunk)?;
            }
        }

        stdout.flush()
    }

    /// Delete an uploaded image
    pub fn delete_image(&mut self, image_id: u32) -> io::Result<()> {
        let mut stdout = io::stdout();

        // a=d (delete), d=I (by ID), i=ID, q=quiet
        write!(
            stdout,
            "\x1b_Ga=d,d=I,i={},q={}\x1b\\",
            image_id,
            if self.quiet { 2 } else { 0 }
        )?;

        stdout.flush()?;

        self.uploaded_images.remove(&image_id);
        Ok(())
    }

    /// Delete all uploaded images
    pub fn clear_all(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();

        // a=d, d=A (delete all)
        write!(
            stdout,
            "\x1b_Ga=d,d=A,q={}\x1b\\",
            if self.quiet { 2 } else { 0 }
        )?;

        stdout.flush()?;

        self.uploaded_images.clear();
        Ok(())
    }

    /// Check if an image is uploaded
    pub fn is_uploaded(&self, image_id: u32) -> bool {
        self.uploaded_images.contains_key(&image_id)
    }

    /// Get dimensions of an uploaded image
    pub fn get_dimensions(&self, image_id: u32) -> Option<(u32, u32)> {
        self.uploaded_images.get(&image_id).copied()
    }

    /// Calculate how many cells an image should occupy
    pub fn image_to_cells(&self, width: u32, height: u32) -> (u16, u16) {
        let cols = (width as u16 + self.cell_size.0 - 1) / self.cell_size.0;
        let rows = (height as u16 + self.cell_size.1 - 1) / self.cell_size.1;
        (cols.max(1), rows.max(1))
    }
}

impl Default for KittyGraphics {
    fn default() -> Self {
        Self::new()
    }
}

/// Encode an RGBA image as PNG bytes
fn encode_png(image: &RgbaImage) -> io::Result<Vec<u8>> {
    use std::io::Cursor;
    use image::ImageEncoder;

    let mut buffer = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(Cursor::new(&mut buffer));

    encoder
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(buffer)
}

/// Helper to create a simple colored square (useful for testing)
pub fn create_colored_square(size: u32, r: u8, g: u8, b: u8, a: u8) -> DynamicImage {
    let mut img = RgbaImage::new(size, size);

    for pixel in img.pixels_mut() {
        *pixel = image::Rgba([r, g, b, a]);
    }

    DynamicImage::ImageRgba8(img)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_calculation() {
        let kitty = KittyGraphics::new();

        // 16x16 sprite with 8x16 cells = 2x1 cells
        assert_eq!(kitty.image_to_cells(16, 16), (2, 1));

        // 32x32 sprite = 4x2 cells
        assert_eq!(kitty.image_to_cells(32, 32), (4, 2));
    }
}
