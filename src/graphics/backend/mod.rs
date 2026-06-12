pub mod ascii_art;
pub mod framebuffer;
pub mod kitty;
pub mod ueberzugpp;

use image::RgbaImage;

use crate::errors::AuraResult;
use framebuffer::FramebufferBackend;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ScaleMode {
    Stretch,
    Center,
    Tile,
    Fill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BackendPreference {
    Auto,
    Framebuffer,
    Kitty,
    Ueberzugpp,
    AsciiArt,
}

#[allow(dead_code)]
pub struct RenderOpts {
    pub mode: ScaleMode,
    pub screen_cols: u16,
    pub screen_rows: u16,
}

#[allow(dead_code)]
pub trait BackgroundBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    fn render(&mut self, image: &RgbaImage, opts: &RenderOpts) -> AuraResult<()>;
    fn clear(&mut self) -> AuraResult<()>;
}

#[allow(dead_code)]
pub fn detect_backend(preferred: BackendPreference) -> Box<dyn BackgroundBackend> {
    match preferred {
        BackendPreference::Framebuffer => match FramebufferBackend::new() {
            Ok(fb) => Box::new(fb),
            Err(_) => Box::new(ascii_art::AsciiArtBackend::new()),
        },
        BackendPreference::Kitty => Box::new(kitty::KittyBackend::new()),
        BackendPreference::Ueberzugpp => Box::new(ueberzugpp::UeberzugppBackend::new()),
        BackendPreference::AsciiArt => Box::new(ascii_art::AsciiArtBackend::new()),
        BackendPreference::Auto => {
            if framebuffer::check_available() {
                if let Ok(fb) = FramebufferBackend::new() {
                    return Box::new(fb);
                }
            }
            let kitty = kitty::KittyBackend::new();
            if kitty.is_available() {
                return Box::new(kitty);
            }
            let ueberzugpp = ueberzugpp::UeberzugppBackend::new();
            if ueberzugpp.is_available() {
                return Box::new(ueberzugpp);
            }
            Box::new(ascii_art::AsciiArtBackend::new())
        }
    }
}
