//! Embedded images, decoded once into GPU textures.
//!
//! All artwork comes from public-domain scans of the 1910 Rider–Waite–Smith
//! cards by Pamela Colman Smith; see `tools/` for how each piece was made.

use eframe::egui::{self, ColorImage, TextureFilter, TextureHandle, TextureOptions};

pub const ICON_SEASON: &[u8] = include_bytes!("../../assets/icons/season.png");
pub const ICON_MOON: &[u8] = include_bytes!("../../assets/icons/moon.png");
pub const ICON_WHEEL: &[u8] = include_bytes!("../../assets/icons/wheel.png");
pub const ICON_SABBAT: &[u8] = include_bytes!("../../assets/icons/sabbat.png");

/// Mipmapped linear filtering: the art is drawn smaller than its pixels.
const SMOOTH: TextureOptions = TextureOptions {
    magnification: TextureFilter::Linear,
    minification: TextureFilter::Linear,
    mipmap_mode: Some(TextureFilter::Linear),
    ..TextureOptions::LINEAR
};

/// Decode an 8-bit RGBA or RGB PNG.
pub fn decode_png(bytes: &[u8]) -> Result<ColorImage, String> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut buf = vec![0; reader.output_buffer_size().ok_or("PNG too large")?];
    let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
    let size = [info.width as usize, info.height as usize];
    let data = &buf[..info.buffer_size()];
    match info.color_type {
        png::ColorType::Rgba => Ok(ColorImage::from_rgba_unmultiplied(size, data)),
        png::ColorType::Rgb => Ok(ColorImage::from_rgb(size, data)),
        other => Err(format!("unsupported PNG colour type {other:?}")),
    }
}

pub fn texture(ctx: &egui::Context, name: &str, bytes: &[u8]) -> TextureHandle {
    let image = decode_png(bytes).unwrap_or_else(|e| {
        // Embedded at build time, so this only fires on a corrupted build.
        eprintln!("sabbatia: image {name}: {e}");
        ColorImage::filled([1, 1], egui::Color32::TRANSPARENT)
    });
    ctx.load_texture(name, image, SMOOTH)
}

pub struct Icons {
    pub season: TextureHandle,
    pub moon: TextureHandle,
    pub wheel: TextureHandle,
    pub sabbat: TextureHandle,
}

impl Icons {
    pub fn load(ctx: &egui::Context) -> Icons {
        Icons {
            season: texture(ctx, "icon-season", ICON_SEASON),
            moon: texture(ctx, "icon-moon", ICON_MOON),
            wheel: texture(ctx, "icon-wheel", ICON_WHEEL),
            sabbat: texture(ctx, "icon-sabbat", ICON_SABBAT),
        }
    }
}

/// Window icon from the moon crop.
pub fn window_icon() -> egui::IconData {
    match decode_png(ICON_MOON) {
        Ok(img) => egui::IconData {
            width: img.size[0] as u32,
            height: img.size[1] as u32,
            rgba: img.pixels.iter().flat_map(|c| c.to_srgba_unmultiplied()).collect(),
        },
        Err(_) => egui::IconData::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_images_decode() {
        for (name, bytes) in [("season", ICON_SEASON), ("moon", ICON_MOON), ("wheel", ICON_WHEEL), ("sabbat", ICON_SABBAT)] {
            let img = decode_png(bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(img.size, [96, 96], "{name}");
        }
    }
}
