use eframe::egui::{self, Context};

pub fn ui_redactor_window(app: &mut crate::app::App, ctx: &Context) {
    let mut open = app.redactor_open;
    egui::Window::new("Redactor")
        .open(&mut open)
        .default_size([600.0, 400.0])
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                let ph = app.config.analyse.redactor_placeholders.join(", ");
                ui.label(format!("Plantilla (usa {}, #NombreColumna):", ph));

                ui.push_id("template_edit", |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut app.redactor.template)
                            .desired_rows(5)
                            .desired_width(f32::INFINITY)
                            .hint_text("Ej: Hay #pendientes documentos de #gerencia"),
                    );
                });

                // Available columns
                if !app.data.columnas_tabla.is_empty() {
                    egui::CollapsingHeader::new("Columnas disponibles")
                        .default_open(false)
                        .show(ui, |ui| {
                            let labels: Vec<String> = app
                                .data
                                .columnas_tabla
                                .iter()
                                .map(|c| format!("#{}", c.replace('_', " ")))
                                .collect();
                            ui.label(labels.join(", "));
                        });
                }

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    let has_data = !app.data.tabla.is_empty();
                    if ui
                        .add_enabled(has_data, egui::Button::new("Redactar"))
                        .clicked()
                    {
                        app.redactor.redactar(&app.data, &app.meta);
                    }
                    if !app.redactor.output.is_empty() {
                        if ui.button("Copiar").on_hover_text("Copiar resultado al portapapeles").clicked() {
                            ui.ctx().copy_text(app.redactor.output.clone());
                        }
                    }
                });

                if !app.redactor.output.is_empty() {
                    ui.separator();
                    ui.label("Resultado:");
                    ui.push_id("output_view", |ui| {
                        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            let mut out = app.redactor.output.clone();
                            ui.add(
                                egui::TextEdit::multiline(&mut out)
                                    .desired_rows(6)
                                    .desired_width(f32::INFINITY)
                                    .font(egui::TextStyle::Monospace)
                                    .interactive(false),
                            );
                        });
                    });
                }
            });
        });
    app.redactor_open = open;
}
