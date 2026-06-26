use eframe::egui;
use egui_extras::{Column, TableBuilder};

pub fn ui_tabla(app: &mut crate::app::App, ui: &mut egui::Ui) {
    let col_names = app.data.columnas_tabla.clone();
    let rows = app.data.tabla.clone();
    if col_names.is_empty() {
        ui.label("Sin datos");
        return;
    }

    egui::ScrollArea::horizontal().show(ui, |ui| {
        let mut table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .min_scrolled_height(300.0);
        for _ in &col_names {
            table = table.column(Column::initial(100.0).at_least(60.0).resizable(true));
        }
        table
            .header(22.0, |mut header| {
                for col in &col_names {
                    header.col(|ui| {
                        ui.strong(col.replace('_', " "));
                    });
                }
            })
            .body(|mut body| {
                for row_data in &rows {
                    body.row(18.0, |mut row| {
                        for col in &col_names {
                            row.col(|ui| {
                                let val = row_data.get(col).map(|v| match v {
                                    serde_json::Value::String(s) => s.clone(),
                                    serde_json::Value::Number(n) => n.to_string(),
                                    serde_json::Value::Null => String::new(),
                                    _ => format!("{}", v),
                                }).unwrap_or_default();
                                ui.label(val);
                            });
                        }
                    });
                }
            });
    });

    // Pagination
    ui.separator();
    if app.data.total_general == 0 {
        ui.label("Sin resultados");
        return;
    }
    ui.horizontal(|ui| {
        let total_pages = (app.data.total_general as f64 / app.data.page_size as f64).ceil() as usize;
        
        let prev_btn = ui.add_enabled(app.data.current_page > 1, egui::Button::new("⬅ Anterior"));
        if prev_btn.clicked() {
            app.data.current_page -= 1;
            app.needs_refresh = true;
        }

        ui.label(format!("Página {} de {}", app.data.current_page, total_pages.max(1)));

        let next_btn = ui.add_enabled(app.data.current_page < total_pages, egui::Button::new("Siguiente ➡"));
        if next_btn.clicked() {
            app.data.current_page += 1;
            app.needs_refresh = true;
        }

        ui.separator();
        ui.label("Tamaño de Página:");
        for s in &[50usize, 100, 200, 500] {
            if ui.selectable_label(app.data.page_size == *s, s.to_string()).clicked() {
                app.data.page_size = *s;
                app.data.current_page = 1;
                app.needs_refresh = true;
            }
        }
    });
}
