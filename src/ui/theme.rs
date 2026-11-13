//! Palette, fonts and egui style.
//!
//! Colours are taken from Pamela Colman Smith's card art: card cream, ink
//! line, leaf green, night blue, the Fool's yellow and poppy red.

use std::sync::Arc;

use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, FontId, TextStyle};

pub const CREAM: Color32 = Color32::from_rgb(0xFB, 0xF7, 0xEE);
pub const INK: Color32 = Color32::from_rgb(0x23, 0x20, 0x1C);
pub const INK_SOFT: Color32 = Color32::from_rgb(0x5A, 0x54, 0x4B);
pub const DIM: Color32 = Color32::from_rgb(0x6E, 0x66, 0x5B);
pub const TRACK: Color32 = Color32::from_rgb(0xEC, 0xE5, 0xD6);
pub const RULE: Color32 = Color32::from_rgb(0xE4, 0xDC, 0xCB);
pub const WHITE: Color32 = Color32::WHITE;
pub const LEAF: Color32 = Color32::from_rgb(0x6B, 0xA3, 0x68);
pub const NIGHT: Color32 = Color32::from_rgb(0x2F, 0x4F, 0x7F);
pub const MOONLIGHT: Color32 = Color32::from_rgb(0xF7, 0xE3, 0xA0);
pub const POPPY: Color32 = Color32::from_rgb(0xB4, 0x41, 0x2B);

/// Tint and label colour of each section tile.
#[derive(Clone, Copy)]
pub struct Tint {
    pub fill: Color32,
    pub hover: Color32,
    pub label: Color32,
}

pub const SEASON: Tint = Tint {
    fill: Color32::from_rgb(0xEA, 0xF2, 0xE3),
    hover: Color32::from_rgb(0xDF, 0xEC, 0xD5),
    label: Color32::from_rgb(0x41, 0x6E, 0x3E),
};
pub const MOON: Tint = Tint {
    fill: Color32::from_rgb(0xE6, 0xEE, 0xF8),
    hover: Color32::from_rgb(0xD9, 0xE5, 0xF4),
    label: Color32::from_rgb(0x35, 0x62, 0x9B),
};
pub const WHEEL: Tint = Tint {
    fill: Color32::from_rgb(0xFB, 0xF0, 0xD2),
    hover: Color32::from_rgb(0xF7, 0xE6, 0xBC),
    label: Color32::from_rgb(0x8A, 0x6A, 0x12),
};
pub const SABBAT: Tint = Tint {
    fill: Color32::from_rgb(0xF9, 0xE3, 0xDC),
    hover: Color32::from_rgb(0xF5, 0xD5, 0xCB),
    label: Color32::from_rgb(0xA8, 0x3B, 0x26),
};

pub const CARD_RADIUS: u8 = 22;
pub const TILE_RADIUS: u8 = 14;

pub fn heavy() -> FontFamily {
    FontFamily::Name("heavy".into())
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn bold(size: f32) -> FontId {
    FontId::new(size, heavy())
}

fn fonts() -> FontDefinitions {
    let mut f = FontDefinitions::default();
    f.font_data.insert(
        "nunito".into(),
        Arc::new(FontData::from_static(include_bytes!("../../assets/fonts/Nunito-SemiBold.ttf"))),
    );
    f.font_data.insert(
        "nunito-heavy".into(),
        Arc::new(FontData::from_static(include_bytes!("../../assets/fonts/Nunito-ExtraBold.ttf"))),
    );
    let fallbacks = f.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let mut prop = vec!["nunito".to_string()];
    prop.extend(fallbacks.iter().cloned());
    f.families.insert(FontFamily::Proportional, prop);
    let mut heavy_list = vec!["nunito-heavy".to_string(), "nunito".to_string()];
    heavy_list.extend(fallbacks);
    f.families.insert(heavy(), heavy_list);
    f
}

pub fn apply(ctx: &egui::Context) {
    ctx.set_fonts(fonts());
    ctx.set_theme(egui::Theme::Light);
    let mut style = (*ctx.global_style()).clone();
    style.text_styles = [
        (TextStyle::Small, semibold(11.0)),
        (TextStyle::Body, semibold(12.5)),
        (TextStyle::Button, semibold(12.5)),
        (TextStyle::Heading, bold(16.0)),
        (TextStyle::Monospace, FontId::monospace(11.5)),
    ]
    .into();
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.menu_margin = egui::Margin::same(6);
    style.interaction.selectable_labels = false;

    let v = &mut style.visuals;
    *v = egui::Visuals::light();
    v.override_text_color = Some(INK);
    v.panel_fill = CREAM;
    v.window_fill = CREAM;
    v.window_stroke = egui::Stroke::new(1.0, RULE);
    v.window_corner_radius = 14.into();
    v.menu_corner_radius = 14.into();
    v.window_shadow = egui::Shadow { offset: [0, 6], blur: 18, spread: 0, color: Color32::from_black_alpha(45) };
    v.popup_shadow = v.window_shadow;
    v.selection.bg_fill = TRACK;
    v.selection.stroke = egui::Stroke::new(1.0, INK);
    v.hyperlink_color = NIGHT;
    v.extreme_bg_color = WHITE;
    for w in [&mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = 9.into();
        w.fg_stroke.color = INK;
    }
    v.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    v.widgets.inactive.bg_stroke = egui::Stroke::NONE;
    v.widgets.hovered.weak_bg_fill = TRACK;
    v.widgets.hovered.bg_stroke = egui::Stroke::NONE;
    v.widgets.active.weak_bg_fill = RULE;
    v.widgets.noninteractive.fg_stroke.color = INK;
    ctx.set_global_style(style);
}
