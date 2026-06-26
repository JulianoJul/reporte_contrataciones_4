use std::collections::HashMap;
use eframe::egui::{self, Color32, Stroke, Shape};

use crate::ui::theme::{C_MUTED, C_TEXT, C_BG, C_GREY, nord_colors};


pub fn bar_chart_view(ui: &mut egui::Ui, data: &HashMap<String, u64>, _title: &str) {
    use egui_plot::{Bar, BarChart, Plot};

    if data.is_empty() {
        ui.label("Sin datos para graficar");
        return;
    }

    let mut sorted: Vec<(&String, &u64)> = data.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    let colors = nord_colors();

    let bars: Vec<Bar> = sorted
        .iter()
        .enumerate()
        .map(|(i, (label, count))| {
            let display = label.replace('_', " ");
            Bar::new(i as f64, **count as f64)
                .name(format!("{}: {}", display, count))
                .width(0.7)
                .fill(colors[i % colors.len()])
        })
        .collect();

    let chart = BarChart::new(bars).width(0.7);

    Plot::new("bar_chart_grupos")
        .height(200.0)
        .show(ui, |plot_ui| {
            plot_ui.bar_chart(chart);
        });
}

pub fn pie_chart_view(ui: &mut egui::Ui, data: &HashMap<String, u64>, _title: &str) {
    let total: u64 = data.values().sum();
    if data.is_empty() || total == 0 {
        ui.label("Sin datos para graficar");
        return;
    }

    let mut sorted: Vec<(&String, &u64)> = data.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    let colors = nord_colors();

    let size = egui::vec2(ui.available_width().min(300.0), 200.0);
    let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());

    let pie_frac = 0.55_f32;
    let pie_right = rect.left() + rect.width() * pie_frac;
    let pie_rect = egui::Rect::from_min_max(rect.min, egui::pos2(pie_right, rect.max.y));
    let legend_rect = egui::Rect::from_min_max(egui::pos2(pie_right, rect.min.y), rect.max);

    let center = pie_rect.center();
    let radius = pie_rect.width().min(pie_rect.height()) * 0.35;
    let painter = ui.painter();
    let mut start_angle = -std::f32::consts::FRAC_PI_2;

    for (i, (_label, count)) in sorted.iter().enumerate() {
        let fraction = **count as f32 / total as f32;
        let sweep = fraction * 2.0 * std::f32::consts::PI;
        let color = colors[i % colors.len()];

        let n = (30.0 * fraction).ceil().max(3.0) as usize;
        let mut arc_points = vec![center];
        for j in 0..=n {
            let angle = start_angle + sweep * (j as f32 / n as f32);
            arc_points.push(center + radius * egui::vec2(angle.cos(), angle.sin()));
        }
        arc_points.push(center);

        painter.add(Shape::convex_polygon(arc_points, color, Stroke::new(1.0, C_MUTED)));

        if fraction > 0.05 {
            let mid_angle = start_angle + sweep / 2.0;
            let label_pos = center + (radius * 0.65) * egui::vec2(mid_angle.cos(), mid_angle.sin());
            let pct = (fraction * 100.0) as u32;
            painter.text(
                label_pos,
                egui::Align2::CENTER_CENTER,
                format!("{}%", pct),
                egui::FontId::proportional(10.0),
                Color32::WHITE,
            );
        }

        start_angle += sweep;
    }

    painter.circle_stroke(center, radius, Stroke::new(1.0, C_MUTED));
    painter.circle_filled(center, radius * 0.35, C_BG);
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        &total.to_string(),
        egui::FontId::proportional(18.0),
        C_TEXT,
    );

    // Inline legend on the right with truncation
    let text_h = 16.0;
    let mut ly = legend_rect.top() + 4.0;
    let mut truncated = false;
    for (i, (label, count)) in sorted.iter().enumerate() {
        if ly + text_h > legend_rect.bottom() {
            truncated = true;
            break;
        }
        let color = colors[i % colors.len()];
        painter.rect_filled(
            egui::Rect::from_min_size(egui::pos2(legend_rect.left() + 2.0, ly + 2.0), egui::vec2(10.0, 10.0)),
            2.0,
            color,
        );
        painter.text(
            egui::pos2(legend_rect.left() + 16.0, ly + 7.0),
            egui::Align2::LEFT_CENTER,
            format!("{}: {}", label.replace('_', " "), count),
            egui::FontId::proportional(11.0),
            C_TEXT,
        );
        ly += text_h;
    }
    if truncated {
        painter.text(
            egui::pos2(legend_rect.left() + 2.0, ly + 2.0),
            egui::Align2::LEFT_TOP,
            format!("... y {} mas", sorted.len() - (ly - legend_rect.top() - 4.0) as usize / text_h as usize),
            egui::FontId::proportional(9.0),
            C_GREY,
        );
    }
}

pub struct ChartsData<'a> {
    pub por_grupo: &'a HashMap<String, u64>,
}

pub fn show_charts(ui: &mut egui::Ui, data: &ChartsData) {
    if data.por_grupo.is_empty() {
        return;
    }

    egui::CollapsingHeader::new("Graficos")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Distribucion por grupo").strong().size(12.0));
                    bar_chart_view(ui, data.por_grupo, "grupos");
                });
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Proporcion").strong().size(12.0));
                    pie_chart_view(ui, data.por_grupo, "proporcion");
                });
            });
        });
}
