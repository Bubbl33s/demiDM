use std::path::PathBuf;

use image::RgbaImage;
use mlua::Lua;
use tokio::sync::mpsc::Sender;

use crate::events::AppEvent;
use crate::graphics::backend::{self, BackendPreference, RenderOpts, ScaleMode};
use crate::graphics::handle::FbOverlayHandle;
use crate::graphics::scaler;
use crate::renderer::theme::parse_hex_color;

pub fn create_background_api(lua: &Lua, tx: Sender<AppEvent>) -> mlua::Result<mlua::Table<'_>> {
    let table = lua.create_table()?;

    let tx_set = tx.clone();
    let set_image_fn = lua.create_function(move |_lua, args: mlua::MultiValue| {
        let mut args_iter = args.into_iter();
        let path_val = args_iter
            .next()
            .ok_or_else(|| mlua::Error::external("set_image requires a path argument"))?;
        let opts_val = args_iter.next();

        let path: String = match path_val {
            mlua::Value::String(s) => s.to_str()?.to_string(),
            _ => return Err(mlua::Error::external("path must be a string")),
        };

        let mut mode = ScaleMode::Stretch;
        let mut _backend_pref = BackendPreference::Auto;

        if let Some(mlua::Value::Table(opts)) = opts_val {
            if let Some(m) = opts.get::<_, Option<String>>("mode")? {
                mode = match m.as_str() {
                    "stretch" => ScaleMode::Stretch,
                    "center" => ScaleMode::Center,
                    "fill" => ScaleMode::Fill,
                    "tile" => ScaleMode::Tile,
                    _ => ScaleMode::Stretch,
                };
            }
            if let Some(b) = opts.get::<_, Option<String>>("backend")? {
                _backend_pref = match b.as_str() {
                    "framebuffer" => BackendPreference::Framebuffer,
                    "kitty" => BackendPreference::Kitty,
                    "ueberzugpp" => BackendPreference::Ueberzugpp,
                    "ascii" => BackendPreference::AsciiArt,
                    _ => BackendPreference::Auto,
                };
            }
        }

        let path_buf = PathBuf::from(&path);
        let image = match scaler::load_image(&path_buf) {
            Ok(img) => img,
            Err(e) => {
                tracing::error!("Failed to load image {}: {}", path, e);
                return Err(mlua::Error::external(format!(
                    "Failed to load image: {}",
                    e
                )));
            }
        };

        let mut backend = backend::detect_backend(_backend_pref);
        let backend_name = backend.name().to_string();

        let opts = RenderOpts {
            mode,
            screen_cols: 80,
            screen_rows: 24,
        };

        if let Err(e) = backend.render(&image, &opts) {
            tracing::error!("Failed to render background: {}", e);
            return Err(mlua::Error::external(format!("Render failed: {}", e)));
        }

        let handle = FbOverlayHandle::new(backend_name, backend);
        let _ = tx_set.try_send(AppEvent::FbImageLoaded { handle });

        Ok(())
    })?;
    table.set("set_image", set_image_fn)?;

    let tx_color = tx.clone();
    let set_color_fn = lua.create_function(move |_lua, hex: String| {
        let (r, g, b) = match parse_hex_color(&hex) {
            Some(ratatui::style::Color::Rgb(r, g, b)) => (r, g, b),
            _ => return Err(mlua::Error::external(format!("Invalid color: {}", hex))),
        };

        let image = RgbaImage::from_pixel(1, 1, image::Rgba([r, g, b, 255]));
        let mut backend = backend::detect_backend(BackendPreference::Auto);
        let backend_name = backend.name().to_string();

        let opts = RenderOpts {
            mode: ScaleMode::Stretch,
            screen_cols: 80,
            screen_rows: 24,
        };

        if let Err(e) = backend.render(&image, &opts) {
            tracing::error!("Failed to render color background: {}", e);
            return Err(mlua::Error::external(format!("Render failed: {}", e)));
        }

        let handle = FbOverlayHandle::new(backend_name, backend);
        let _ = tx_color.try_send(AppEvent::FbImageLoaded { handle });

        Ok(())
    })?;
    table.set("set_color", set_color_fn)?;

    let tx_clear = tx.clone();
    let clear_fn = lua.create_function(move |_lua, ()| {
        let mut backend = backend::detect_backend(BackendPreference::Auto);
        if let Err(e) = backend.clear() {
            tracing::error!("Failed to clear background: {}", e);
            return Err(mlua::Error::external(format!("Clear failed: {}", e)));
        }
        let _ = tx_clear.try_send(AppEvent::Notification("Background cleared".to_string()));
        Ok(())
    })?;
    table.set("clear", clear_fn)?;

    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_mode_parsing() {
        assert!(matches!("stretch", "stretch"));
        assert!(matches!("center", "center"));
        assert!(matches!("fill", "fill"));
        assert!(matches!("tile", "tile"));
    }

    #[test]
    fn test_backend_preference_parsing() {
        let _auto = BackendPreference::Auto;
        let _fb = BackendPreference::Framebuffer;
        let _kitty = BackendPreference::Kitty;
        let _ueb = BackendPreference::Ueberzugpp;
        let _ascii = BackendPreference::AsciiArt;
    }
}
