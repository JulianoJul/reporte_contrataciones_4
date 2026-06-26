mod app;
mod config;
mod db;
mod export;
mod redactor;
mod ui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1400.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Reporte de Contrataciones",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::new()))),
    )
}
