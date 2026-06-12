use std::fs::File;
use std::io::Write;

use base64::Engine;
use image::RgbaImage;

use super::{BackgroundBackend, RenderOpts};
use crate::errors::{AuraError, AuraResult};
use crate::graphics::scaler;

const KITTY_CHUNK_SIZE: usize = 4096;

pub struct KittyBackend {
    tty: File,
}

impl KittyBackend {
    pub fn new() -> Self {
        let tty = File::create("/dev/tty").unwrap_or_else(|_| File::create("/dev/null").unwrap());
        Self { tty }
    }

    #[allow(dead_code)]
    pub fn with_tty(tty: File) -> Self {
        Self { tty }
    }
}

impl Default for KittyBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl BackgroundBackend for KittyBackend {
    fn name(&self) -> &'static str {
        "kitty"
    }

    fn is_available(&self) -> bool {
        std::env::var("TERM")
            .map(|t| t.contains("kitty"))
            .unwrap_or(false)
    }

    fn render(&mut self, image: &RgbaImage, opts: &RenderOpts) -> AuraResult<()> {
        let scaled = scaler::scale_image(
            image,
            opts.screen_cols as u32,
            (opts.screen_rows as u32) * 2,
            opts.mode,
        );
        let (width, height) = scaled.dimensions();
        let raw_data = scaled.into_raw();
        let encoded = base64::engine::general_purpose::STANDARD.encode(&raw_data);

        let chunks: Vec<&str> = encoded
            .as_bytes()
            .chunks(KITTY_CHUNK_SIZE)
            .map(|c| std::str::from_utf8(c).unwrap_or(""))
            .collect();

        let total_chunks = chunks.len();
        for (i, chunk) in chunks.iter().enumerate() {
            let more = if i < total_chunks - 1 { 1 } else { 0 };
            if i == 0 {
                write!(
                    self.tty,
                    "\x1b_Ga=T,f=32,s={},v={},m={};{}\x1b\\\\",
                    width, height, more, chunk
                )
                .map_err(|e| AuraError::Framebuffer(format!("Kitty write error: {}", e)))?;
            } else {
                write!(self.tty, "\x1b_Gm={};{}\x1b\\\\", more, chunk)
                    .map_err(|e| AuraError::Framebuffer(format!("Kitty write error: {}", e)))?;
            }
        }
        self.tty
            .flush()
            .map_err(|e| AuraError::Framebuffer(format!("Kitty flush error: {}", e)))?;
        Ok(())
    }

    fn clear(&mut self) -> AuraResult<()> {
        write!(self.tty, "\x1b_Ga=d,d=A\x1b\\\\")
            .map_err(|e| AuraError::Framebuffer(format!("Kitty clear error: {}", e)))?;
        self.tty
            .flush()
            .map_err(|e| AuraError::Framebuffer(format!("Kitty flush error: {}", e)))?;
        Ok(())
    }
}

#[allow(dead_code)]
pub fn format_kitty_escape(width: u32, height: u32, more: u8, data: &str) -> String {
    format!(
        "\x1b_Ga=T,f=32,s={},v={},m={};{}\x1b\\\\",
        width, height, more, data
    )
}

#[allow(dead_code)]
pub fn format_kitty_continuation(more: u8, data: &str) -> String {
    format!("\x1b_Gm={};{}\x1b\\\\", more, data)
}

#[allow(dead_code)]
pub fn split_into_chunks(encoded: &str, chunk_size: usize) -> Vec<String> {
    encoded
        .as_bytes()
        .chunks(chunk_size)
        .map(|c| String::from_utf8_lossy(c).to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encoding() {
        let data = vec![255u8, 0, 128, 255];
        let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
        assert!(!encoded.is_empty());
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_chunk_splitting() {
        let data = "a".repeat(10000);
        let chunks = split_into_chunks(&data, KITTY_CHUNK_SIZE);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].len(), KITTY_CHUNK_SIZE);
        assert_eq!(chunks[1].len(), KITTY_CHUNK_SIZE);
        assert_eq!(chunks[2].len(), 10000 - 2 * KITTY_CHUNK_SIZE);
    }

    #[test]
    fn test_escape_sequence_format() {
        let esc = format_kitty_escape(100, 50, 1, "AAAA");
        assert!(esc.contains("a=T"));
        assert!(esc.contains("f=32"));
        assert!(esc.contains("s=100"));
        assert!(esc.contains("v=50"));
        assert!(esc.contains("m=1"));
        assert!(esc.contains("AAAA"));
    }

    #[test]
    fn test_escape_sequence_last_chunk() {
        let esc = format_kitty_escape(100, 50, 0, "AAAA");
        assert!(esc.contains("m=0"));
    }
}
