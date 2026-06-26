use eframe::egui::{self, Color32, Rounding, Stroke};

use crate::ui::theme::{C_SURF, C_MUTED, C_GREY};

pub fn metric_card(ui: &mut egui::Ui, title: &str, value: u64, accent: Color32) {
    let height = 64.0;
    let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), height), egui::Sense::hover());
    let bg_rect = r.shrink(2.0);
    ui.painter().rect_filled(bg_rect, Rounding::same(6.0), C_SURF);
    ui.painter().rect_stroke(bg_rect, Rounding::same(6.0), Stroke::new(1.0, C_MUTED));
    let accent_rect = egui::Rect::from_min_size(
        egui::pos2(bg_rect.left(), bg_rect.top() + 4.0),
        egui::vec2(3.0, bg_rect.height() - 8.0),
    );
    ui.painter().rect_filled(accent_rect, Rounding::same(2.0), accent);
    ui.painter().text(
        egui::pos2(bg_rect.left() + 12.0, bg_rect.top() + 16.0),
        egui::Align2::LEFT_TOP,
        title,
        egui::FontId::proportional(11.0),
        C_GREY,
    );
    ui.painter().text(
        egui::pos2(bg_rect.left() + 12.0, bg_rect.bottom() - 8.0),
        egui::Align2::LEFT_BOTTOM,
        &format!("{}", value),
        egui::FontId::proportional(22.0),
        accent,
    );
}
