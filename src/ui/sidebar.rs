use eframe::egui::{self, Context, Margin, Rounding, Stroke};
use crate::db::FiltroValor;
use crate::db::constants::FILTRO_TODOS;
use crate::ui::theme::C_BG;

pub fn ui_sidebar(app: &mut crate::app::App, ctx: &Context) {
    egui::SidePanel::left("sidebar")
        .resizable(true)
        .default_width(220.0)
        .width_range(180.0..=400.0)
        .show_separator_line(true)
        .frame(egui::Frame {
            inner_margin: Margin::same(8.0),
            outer_margin: Margin::default(),
            rounding: Rounding::ZERO,
            shadow: egui::Shadow::NONE,
            fill: C_BG,
            stroke: Stroke::NONE,
        })
        .show(ctx, |ui| {
            ui.heading("Filtros");

            // DB path display
            if let Some(path) = &app.config.default_db {
                let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default().to_string();
                ui.horizontal(|ui| {
                    ui.label("BD:");
                    ui.label(&name)
                        .on_hover_text(path.to_string_lossy());
                });
            }

            ui.separator();
            egui::ScrollArea::vertical().id_salt("sidebar_scroll").show(ui, |ui| {
                for fi in &app.filtros_info {
                    let nombre = &fi.nombre;
                    let display = nombre.replace('_', " ");
                    if let Some(valor) = app.filtros.get_mut(nombre) {
                        match valor {
                            FiltroValor::Categorical { selected } |
                            FiltroValor::CategoricalFK { selected, .. } => {
                                let prev = selected.clone();
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(&display).strong());
                                    if *selected != FILTRO_TODOS && !selected.is_empty() {
                                        if ui.button("✕").clicked() {
                                            *selected = FILTRO_TODOS.to_string();
                                            app.needs_refresh = true;
                                        }
                                    }
                                });
                                egui::ComboBox::from_id_salt(format!("flt_{}", nombre))
                                    .width(ui.available_width())
                                    .selected_text(if selected.is_empty() || selected == FILTRO_TODOS {
                                        "Todos".to_string()
                                    } else {
                                        selected.clone()
                                    })
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(selected, FILTRO_TODOS.to_string(), "Todos");
                                        if let Some(ref vals) = fi.valores {
                                            for v in vals {
                                                ui.selectable_value(selected, v.clone(), v);
                                            }
                                        }
                                    });
                                if prev != *selected { app.needs_refresh = true; }
                            }
                            FiltroValor::Date { desde, hasta } => {
                                ui.label(egui::RichText::new(&display).strong());
                                ui.horizontal(|ui| {
                                    ui.label("Desde:");
                                    let prev = desde.clone();
                                    ui.add(egui::TextEdit::singleline(desde).hint_text("DD/MM/AAAA"));
                                    if prev != *desde { app.needs_refresh = true; }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Hasta:");
                                    let prev = hasta.clone();
                                    ui.add(egui::TextEdit::singleline(hasta).hint_text("DD/MM/AAAA"));
                                    if prev != *hasta { app.needs_refresh = true; }
                                });
                            }
                            FiltroValor::Numeric { min, max, orig_min, orig_max } => {
                                let (lo, hi) = if *orig_min < *orig_max { (*orig_min, *orig_max) } else { (*orig_max, *orig_min) };
                                let range = if (hi - lo).abs() < f64::EPSILON { lo..=lo + 1.0 } else { lo..=hi };
                                let is_int = lo.fract() == 0.0 && hi.fract() == 0.0;
                                ui.label(egui::RichText::new(&display).strong());
                                let prev = [*min, *max];
                                ui.vertical(|ui| {
                                    let mut sl_min = egui::Slider::new(min, range.clone()).text("Mín");
                                    let mut sl_max = egui::Slider::new(max, range).text("Máx");
                                    if is_int {
                                        sl_min = sl_min.fixed_decimals(0);
                                        sl_max = sl_max.fixed_decimals(0);
                                    }
                                    ui.add(sl_min);
                                    ui.add(sl_max);
                                });
                                if *min > *max { *max = *min; }
                                if prev != [*min, *max] { app.needs_refresh = true; }
                            }
                            FiltroValor::TextSearch { query } => {
                                ui.label(egui::RichText::new(&display).strong());
                                let prev = query.clone();
                                ui.text_edit_singleline(query);
                                if prev != *query { app.needs_refresh = true; }
                            }
                        }
                    }
                }
            });
        });
}
