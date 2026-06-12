use image::RgbaImage;

use crate::graphics::backend::ScaleMode;

#[allow(dead_code)]
pub fn scale_image(image: &RgbaImage, width: u32, height: u32, mode: ScaleMode) -> RgbaImage {
    match mode {
        ScaleMode::Stretch => {
            image::imageops::resize(image, width, height, image::imageops::FilterType::Lanczos3)
        }
        ScaleMode::Center => {
            let mut canvas = RgbaImage::new(width, height);
            let (src_w, src_h) = image.dimensions();
            let scaled = if src_w > width || src_h > height {
                image::imageops::resize(
                    image,
                    src_w.min(width),
                    src_h.min(height),
                    image::imageops::FilterType::Lanczos3,
                )
            } else {
                image.clone()
            };
            let (sw, sh) = scaled.dimensions();
            let x_offset = (width.saturating_sub(sw)) / 2;
            let y_offset = (height.saturating_sub(sh)) / 2;
            image::imageops::overlay(
                &mut canvas,
                &scaled,
                i64::from(x_offset),
                i64::from(y_offset),
            );
            canvas
        }
        ScaleMode::Fill => {
            let (src_w, src_h) = image.dimensions();
            let scale_x = width as f64 / src_w as f64;
            let scale_y = height as f64 / src_h as f64;
            let scale = scale_x.max(scale_y);
            let new_w = (src_w as f64 * scale) as u32;
            let new_h = (src_h as f64 * scale) as u32;
            let resized =
                image::imageops::resize(image, new_w, new_h, image::imageops::FilterType::Lanczos3);
            let x_offset = (new_w.saturating_sub(width)) / 2;
            let y_offset = (new_h.saturating_sub(height)) / 2;
            image::imageops::crop_imm(&resized, x_offset, y_offset, width, height).to_image()
        }
        ScaleMode::Tile => {
            let mut canvas = RgbaImage::new(width, height);
            let (src_w, src_h) = image.dimensions();
            if src_w == 0 || src_h == 0 {
                return canvas;
            }
            let mut y = 0u32;
            while y < height {
                let mut x = 0u32;
                while x < width {
                    image::imageops::overlay(&mut canvas, image, i64::from(x), i64::from(y));
                    x += src_w;
                }
                y += src_h;
            }
            canvas
        }
    }
}

#[allow(dead_code)]
pub fn load_image(path: &std::path::Path) -> crate::errors::AuraResult<RgbaImage> {
    let img = image::open(path).map_err(|e| {
        crate::errors::AuraError::Framebuffer(format!("Failed to load image: {}", e))
    })?;
    Ok(img.to_rgba8())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    fn make_test_image(w: u32, h: u32) -> RgbaImage {
        RgbaImage::from_pixel(w, h, image::Rgba([255, 0, 0, 255]))
    }

    #[test]
    fn test_stretch_produces_correct_dimensions() {
        let img = make_test_image(100, 50);
        let result = scale_image(&img, 200, 100, ScaleMode::Stretch);
        assert_eq!(result.dimensions(), (200, 100));
    }

    #[test]
    fn test_center_preserves_aspect_ratio() {
        let img = make_test_image(50, 50);
        let result = scale_image(&img, 200, 100, ScaleMode::Center);
        assert_eq!(result.dimensions(), (200, 100));
    }

    #[test]
    fn test_fill_covers_target() {
        let img = make_test_image(100, 50);
        let result = scale_image(&img, 200, 100, ScaleMode::Fill);
        assert_eq!(result.dimensions(), (200, 100));
    }

    #[test]
    fn test_tile_fills_target() {
        let img = make_test_image(10, 10);
        let result = scale_image(&img, 50, 50, ScaleMode::Tile);
        assert_eq!(result.dimensions(), (50, 50));
    }
}
