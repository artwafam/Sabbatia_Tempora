//! Small vector drawings: the moon-phase disk and text helpers.

use eframe::egui::{self, Color32, FontId, Mesh, Painter, Pos2, Stroke, vec2};

use super::theme;

/// Moon disk showing the illuminated fraction, in the card palette.
///
/// `lit_right` is true when the bright limb faces right as seen by the
/// observer: waxing in the Northern Hemisphere, waning in the Southern.
pub fn moon(painter: &Painter, center: Pos2, r: f32, illumination: f32, lit_right: bool) {
    painter.circle_filled(center, r, theme::NIGHT);
    // The terminator is a half-ellipse; at height y the lit band spans
    // x ∈ [−cos(i)·w, w] (lit on the right), w the disk half-width, and
    // cos(i) = 2k − 1 for illuminated fraction k.
    let cos_i = 2.0 * illumination.clamp(0.0, 1.0) - 1.0;
    let sign = if lit_right { 1.0 } else { -1.0 };
    const ROWS: u32 = 64;
    let mut mesh = Mesh::default();
    for j in 0..=ROWS {
        let y = -r + 2.0 * r * j as f32 / ROWS as f32;
        let w = (r * r - y * y).max(0.0).sqrt();
        mesh.colored_vertex(center + vec2(-sign * cos_i * w, y), theme::MOONLIGHT);
        mesh.colored_vertex(center + vec2(sign * w, y), theme::MOONLIGHT);
        if j > 0 {
            let b = 2 * j;
            mesh.add_triangle(b - 2, b - 1, b);
            mesh.add_triangle(b - 1, b + 1, b);
        }
    }
    painter.add(mesh);
    painter.circle_stroke(center, r, Stroke::new(1.5, theme::INK));
}

/// Thin rounded progress bar.
pub fn progress(painter: &Painter, rect: egui::Rect, fraction: f32, fill: Color32, track: Color32) {
    painter.rect_filled(rect, rect.height() / 2.0, track);
    let mut done = rect;
    done.set_width(rect.width() * fraction.clamp(0.0, 1.0));
    painter.rect_filled(done, rect.height() / 2.0, fill);
}

/// Small-caps style label: heavy, letter-spaced.
pub fn label_job(text: &str, size: f32, color: Color32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.append(
        text,
        0.0,
        egui::TextFormat { font_id: theme::bold(size), color, extra_letter_spacing: 0.8, ..Default::default() },
    );
    job
}

/// Wrapped text job.
pub fn wrapped(text: &str, font: FontId, color: Color32, width: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::simple(text.to_owned(), font, color, width);
    job.wrap.max_width = width;
    job
}
