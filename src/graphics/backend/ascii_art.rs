use std::fs::File;
use std::io::Write;

use image::RgbaImage;

use super::{BackgroundBackend, RenderOpts};
use crate::errors::{AuraError, AuraResult};
use crate::graphics::scaler;

pub struct AsciiArtBackend {
    tty: File,
}

impl AsciiArtBackend {
    pub fn new() -> Self {
        let tty = File::create("/dev/tty").unwrap_or_else(|_| File::create("/dev/null").unwrap());
        Self { tty }
    }

    #[allow(dead_code)]
    pub fn with_tty(tty: File) -> Self {
        Self { tty }
    }
}

impl Default for AsciiArtBackend {
    fn default() -> Self {
        Self::new()
    }
}

pub fn image_to_block_art(img: &RgbaImage, cols: u16, rows: u16) -> String {
    let pixel_rows = (rows as u32) * 2;
    let scaled = scaler::scale_image(
        img,
        cols as u32,
        pixel_rows,
        crate::graphics::backend::ScaleMode::Stretch,
    );

    let mut output = String::new();
    for y in (0..pixel_rows).step_by(2) {
        for x in 0..(cols as u32) {
            let top = scaled.get_pixel(x, y);
            let bottom = if y + 1 < pixel_rows {
                scaled.get_pixel(x, y + 1)
            } else {
                top
            };
            output.push_str(&format!(
                "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m▀",
                top[0], top[1], top[2], bottom[0], bottom[1], bottom[2]
            ));
        }
        output.push_str("\x1b[0m\n");
    }
    output
}

impl BackgroundBackend for AsciiArtBackend {
    fn name(&self) -> &'static str {
        "ascii_art"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn render(&mut self, image: &RgbaImage, opts: &RenderOpts) -> AuraResult<()> {
        let art = image_to_block_art(image, opts.screen_cols, opts.screen_rows);
        write!(self.tty, "\x1b[H{}", art)
            .map_err(|e| AuraError::Framebuffer(format!("ASCII art write error: {}", e)))?;
        self.tty
            .flush()
            .map_err(|e| AuraError::Framebuffer(format!("ASCII art flush error: {}", e)))?;
        Ok(())
    }

    fn clear(&mut self) -> AuraResult<()> {
        write!(self.tty, "\x1b[2J\x1b[H")
            .map_err(|e| AuraError::Framebuffer(format!("ASCII art clear error: {}", e)))?;
        self.tty
            .flush()
            .map_err(|e| AuraError::Framebuffer(format!("ASCII art flush error: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_block_art_generation_known_pixels() {
        let mut img = RgbaImage::new(2, 4);
        img.put_pixel(0, 0, Rgba([100, 150, 200, 255]));
        img.put_pixel(0, 1, Rgba([50, 75, 100, 255]));
        img.put_pixel(1, 0, Rgba([100, 150, 200, 255]));
        img.put_pixel(1, 1, Rgba([50, 75, 100, 255]));
        img.put_pixel(0, 2, Rgba([100, 150, 200, 255]));
        img.put_pixel(0, 3, Rgba([50, 75, 100, 255]));
        img.put_pixel(1, 2, Rgba([100, 150, 200, 255]));
        img.put_pixel(1, 3, Rgba([50, 75, 100, 255]));

        let art = image_to_block_art(&img, 2, 2);
        assert!(art.contains("▀"));
        assert!(art.contains("\x1b[38;2;100;150;200m"));
        assert!(art.contains("\x1b[48;2;50;75;100m"));
    }

    #[test]
    fn test_ansi_escape_format() {
        let img = RgbaImage::from_pixel(1, 2, Rgba([255, 0, 0, 255]));
        let art = image_to_block_art(&img, 1, 1);
        assert!(art.contains("\x1b[38;2;255;0;0m"));
        assert!(art.contains("▀"));
    }
}
