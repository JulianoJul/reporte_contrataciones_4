use eframe::egui::{self, Color32, Rounding, Stroke, PopupCloseBehavior};
use chrono::{NaiveDate, Datelike, Local, Duration};

use crate::db::constants::{DATE_FORMAT, DATE_FORMAT_HINT};
use crate::ui::theme::{C_SURF, C_MUTED, C_GREY, C_BG};

fn safe_date_parse(date_str: &str, fallback: NaiveDate) -> NaiveDate {
    NaiveDate::parse_from_str(date_str, DATE_FORMAT).unwrap_or(fallback)
}

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

pub fn hoy_button(ui: &mut egui::Ui, date_str: &mut String, needs_refresh: &mut bool) {
    if ui.button("Hoy").clicked() {
        *date_str = Local::now().format(DATE_FORMAT).to_string();
        *needs_refresh = true;
    }
}

pub fn date_picker_widget(ui: &mut egui::Ui, date_str: &mut String, _label: &str, popup_id: &str, needs_refresh: &mut bool) {
    let resp = ui.add(egui::TextEdit::singleline(date_str).hint_text(DATE_FORMAT_HINT));
    if resp.clicked() {
        ui.memory_mut(|mem| mem.toggle_popup(egui::Id::new(popup_id)));
    }
    egui::popup::popup_below_widget(ui, egui::Id::new(popup_id), &resp, PopupCloseBehavior::CloseOnClick, |ui: &mut egui::Ui| {
        ui.set_min_width(200.0);
        let today = Local::now().naive_local().date();
        let current = safe_date_parse(date_str, today);

        let mut year = current.year();
        let mut month = current.month();
        let day = current.day();

        ui.horizontal(|ui| {
            if ui.button("◀").clicked() {
                if month == 1 { month = 12; year -= 1; } else { month -= 1; }
            }
            ui.label(format!("{:02}/{}", month, year));
            if ui.button("▶").clicked() {
                if month == 12 { month = 1; year += 1; } else { month += 1; }
            }
        });

        let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(today);
        let days_in_month = {
            let next = if month == 12 {
                NaiveDate::from_ymd_opt(year + 1, 1, 1)
            } else {
                NaiveDate::from_ymd_opt(year, month + 1, 1)
            }.unwrap_or_else(|| today + Duration::days(32));
            (next - Duration::days(1)).day()
        };
        let start_wd = first.weekday().num_days_from_monday() as usize;

        ui.horizontal(|ui| {
            for wd in &["L", "M", "M", "J", "V", "S", "D"] {
                ui.label(egui::RichText::new(*wd).weak().size(10.0));
            }
        });
        let cell_w = 28.0;
        let mut day_idx = 0i64;
        let mut day_pos = 0usize;
        for row in 0..6 {
            if day_idx >= days_in_month as i64 { break; }
            ui.horizontal(|ui| {
                for col in 0..7 {
                    if row == 0 && col < start_wd {
                        ui.add_sized(egui::vec2(cell_w, 18.0), egui::Label::new(""));
                    } else if day_idx < days_in_month as i64 {
                        let d = (day_idx + 1) as u32;
                        let selected = d == day && month == current.month() && year == current.year();
                        let btn = egui::Button::new(format!("{}", d))
                            .min_size(egui::vec2(cell_w, 18.0))
                            .fill(if selected { C_SURF } else { C_BG });
                        if ui.add(btn).clicked() {
                            if let Some(date) = NaiveDate::from_ymd_opt(year, month, d) {
                                *date_str = date.format(DATE_FORMAT).to_string();
                            }
                            *needs_refresh = true;
                            ui.memory_mut(|mem| mem.close_popup());
                        }
                        day_idx += 1;
                        day_pos += 1;
                    }
                }
            });
        }
    });
}
