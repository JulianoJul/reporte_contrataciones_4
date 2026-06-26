use std::collections::HashMap;
use rusqlite::Connection;
use eframe::egui::{self, Context, Rounding, Stroke, Margin};

use crate::db::{self, SchemaMetadata, FiltroValor, FiltroInfo, DashboardData, ModoOptimizacion, constants};
use crate::db::analysis;
use crate::config::Config;
use crate::redactor::Redactor;
use crate::ui::charts::{self as chart_ui, ChartsData};
use crate::ui::theme::{self, C_BG, C_SURF, C_MUTED, C_GREY, C_DKBLUE, C_RED, C_GREEN, C_ORANGE};

pub enum ExportFormat {
    Pdf,
    Pptx,
}

pub struct App {
    pub(crate) conn: Option<Connection>,
    pub(crate) config: Config,
    pub(crate) meta: SchemaMetadata,
    pub(crate) vista: String,
    pub(crate) tablas_disponibles: Vec<String>,
    pub(crate) modo: ModoOptimizacion,
    pub(crate) filtros: HashMap<String, FiltroValor>,
    pub(crate) filtros_info: Vec<FiltroInfo>,
    pub(crate) data: DashboardData,
    pub(crate) status_col: Option<String>,
    pub(crate) group_by: String,
    pub(crate) error: Option<String>,
    pub(crate) redactor: Redactor,
    pub(crate) redactor_open: bool,
    pub(crate) needs_refresh: bool,
    pub(crate) needs_load: bool,
    pub(crate) is_loading: bool,
    pub(crate) last_update: String,
    pub(crate) pending_export: Option<ExportFormat>,
}

impl App {
    pub fn new() -> Self {
        let config = Config::detect();
        let mut app = App {
            conn: None,
            config,
            meta: SchemaMetadata::default(),
            vista: String::new(),
            tablas_disponibles: vec![],
            modo: ModoOptimizacion::Universal,
            filtros: HashMap::new(),
            filtros_info: vec![],
            data: DashboardData::default(),
            status_col: None,
            group_by: String::new(),
            error: None,
            redactor: Redactor::new(),
            redactor_open: false,
            needs_refresh: false,
            needs_load: false,
            is_loading: false,
            last_update: String::new(),
            pending_export: None,
        };
        if let Some(ref db_path) = app.config.default_db.clone() {
            app.open_database(db_path);
        }
        app
    }

    fn open_database(&mut self, path: &std::path::Path) {
        match Connection::open(path) {
            Ok(conn) => {
                self.conn = Some(conn);
                let conn = self.conn.as_ref().unwrap();
                self.tablas_disponibles = listar_tablas_vistas(conn);
                match db::explorar(conn, &self.config.analyse) {
                    Ok(meta) => {
                        self.meta = meta;
                        let vista = self.meta.vista_principal.clone();
                        if vista.is_empty() {
                            self.seleccionar_tabla(
                                self.tablas_disponibles.first().cloned().unwrap_or_default(),
                            );
                        } else {
                            self.seleccionar_tabla(vista);
                        }
                    }
                    Err(e) => self.error = Some(format!("Error explorando BD: {}", e)),
                }
            }
            Err(e) => self.error = Some(format!("Error abriendo BD: {}", e)),
        }
    }

    fn seleccionar_tabla(&mut self, tabla: String) {
        if tabla.is_empty() {
            self.vista.clear();
            self.meta = SchemaMetadata::default();
            self.filtros_info.clear();
            self.filtros.clear();
            self.data = DashboardData::default();
            self.status_col = None;
            return;
        }
        let conn = match self.conn.as_ref() {
            Some(c) => c,
            None => return,
        };
        self.vista = tabla.clone();
        self.modo = db::detectar_patron_optimizable(conn, &tabla, &self.config.analyse)
            .unwrap_or(ModoOptimizacion::Universal);
        let cols_raw = db::schema::obtener_columnas(conn, &tabla).unwrap_or_default();
        let all_tablas = db::schema::listar_tablas(conn).unwrap_or_default();
        let fk_pairs = db::schema::analizar_foreign_keys(conn, &all_tablas).unwrap_or_default();
        let mut columnas = Vec::new();
        for col in &cols_raw {
            if col.pk { continue; }
            if let Ok(Some(info)) = db::analysis::analizar_columna(conn, &tabla, col, &fk_pairs, &self.config.analyse.fk_id_prefix, &self.config.analyse.preferred_name_cols) {
                columnas.push(info);
            }
        }
        self.meta = SchemaMetadata {
            vista_principal: tabla.clone(),
            tablas: all_tablas,
            columnas,
            dependencias: vec![],
        };
        let col_names: Vec<String> = self.meta.columnas.iter().map(|c| c.nombre.clone()).collect();
        self.status_col = analysis::detectar_columna_estado(conn, &tabla, &col_names,
            &self.config.pending_pattern, &self.config.signed_pattern)
            .ok().flatten();
        self.filtros_info = db::extraer_filtros_info(&self.meta);
        self.init_filtros();
        self.needs_refresh = true;
    }

    fn init_filtros(&mut self) {
        self.filtros.clear();
        for fi in &self.filtros_info {
            match fi.tipo.as_str() {
                "categorical" => {
                    self.filtros.insert(fi.nombre.clone(),
                        FiltroValor::Categorical { selected: constants::FILTRO_TODOS.to_string() });
                }
                "categorical_fk" => {
                    self.filtros.insert(fi.nombre.clone(),
                        FiltroValor::CategoricalFK {
                            selected: constants::FILTRO_TODOS.to_string(),
                            col_original: fi.col_original.clone().unwrap_or_default(),
                            tabla_catalogo: fi.tabla_catalogo.clone().unwrap_or_default(),
                            col_nombre: fi.col_nombre_catalogo.clone().unwrap_or_default(),
                        });
                }
                "date" => {
                    self.filtros.insert(fi.nombre.clone(),
                        FiltroValor::Date { desde: String::new(), hasta: String::new() });
                }
                "numeric" => {
                    self.filtros.insert(fi.nombre.clone(),
                        FiltroValor::Numeric {
                            min: fi.min.unwrap_or(0.0), max: fi.max.unwrap_or(1.0),
                            orig_min: fi.min.unwrap_or(0.0), orig_max: fi.max.unwrap_or(1.0),
                        });
                }
                "text_search" => {
                    self.filtros.insert(fi.nombre.clone(),
                        FiltroValor::TextSearch { query: String::new() });
                }
                _ => {}
            }
        }
    }

    fn refresh_data(&mut self) {
        let conn = match self.conn.as_ref() {
            Some(c) => c,
            None => return,
        };
        if self.vista.is_empty() {
            self.data = DashboardData::default();
            return;
        }
        self.is_loading = true;
        self.data = db::dashboard(
            conn, &self.filtros, &self.vista,
            if self.group_by.is_empty() { None } else { Some(self.group_by.as_str()) },
            self.data.current_page, self.data.page_size,
            self.status_col.as_deref(), Some(&self.modo),
            Some(&self.config.pending_pattern), Some(&self.config.signed_pattern),
        ).unwrap_or_else(|e| {
            self.error = Some(format!("Error cargando datos: {}", e));
            DashboardData::default()
        });
        self.is_loading = false;
        self.last_update = chrono::Local::now().format("%H:%M:%S").to_string();
    }

    // ── Sidebar ──────────────────────────────────────────────
    fn ui_sidebar(&mut self, ctx: &Context) {
        crate::ui::sidebar::ui_sidebar(self, ctx);
    }

    // ── Top Bar ──────────────────────────────────────────────
    fn ui_top_panel(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame {
                inner_margin: Margin::symmetric(8.0, 4.0),
                outer_margin: Margin::default(),
                rounding: Rounding::ZERO,
                shadow: egui::Shadow::NONE,
                fill: C_SURF,
                stroke: Stroke::new(1.0, C_MUTED),
            })
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Explorador BD");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.conn.is_some() {
                            if ui.button("Redactor").on_hover_text("Editor de plantillas").clicked() {
                                self.redactor_open = !self.redactor_open;
                            }
                            if ui.button("Output").on_hover_text("Abrir carpeta de exportaciones").clicked() {
                                let _ = crate::export::abrir_carpeta_output(&self.config);
                            }
                            if ui.button("PDF").on_hover_text("Exportar dashboard como PDF (con captura)").clicked() {
                                self.pending_export = Some(ExportFormat::Pdf);
                                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
                            }
                            if ui.button("PPTX").on_hover_text("Exportar dashboard como PPTX (con captura)").clicked() {
                                self.pending_export = Some(ExportFormat::Pptx);
                                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
                            }
                            if ui.button("Excel").on_hover_text("Exportar a Excel").clicked() {
                                if let Some(ref conn) = self.conn {
                                    let p = self.config.output_dir.join("reporte.xlsx");
                                    match crate::export::exportar_excel(conn, &self.filtros, &p, &self.vista, &self.config) {
                                        Ok(p) => { let _ = open::that(p); }
                                        Err(e) => self.error = Some(e),
                                    }
                                }
                            }
                        }
                        if ui.button("Abrir BD").on_hover_text("Seleccionar base de datos").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("SQLite", &["db", "sqlite", "sqlite3"])
                                .pick_file()
                            {
                                self.open_database(&path);
                            }
                        }
                    });
                });
            });
    }

    // ── Status Bar ───────────────────────────────────────────
    fn ui_status_bar(&mut self, ctx: &Context) {
        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame {
                inner_margin: Margin::symmetric(8.0, 3.0),
                outer_margin: Margin::default(),
                rounding: Rounding::ZERO,
                shadow: egui::Shadow::NONE,
                fill: C_SURF,
                stroke: Stroke::new(1.0, C_MUTED),
            })
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if self.conn.is_some() {
                        ui.label(egui::RichText::new(format!("Ultima act: {}", self.last_update)).color(C_GREY));
                        ui.separator();
                        ui.label(egui::RichText::new(format!("Registros: {}", self.data.total_general)).color(C_GREY));
                        if self.data.total_general > self.data.page_size as u64 {
                            ui.separator();
                            ui.label(egui::RichText::new(format!("Mostrando: {}", self.data.tabla.len())).color(C_GREY));
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(&self.vista.replace('_', " ")).color(C_DKBLUE));
                        });
                    } else {
                        ui.label(egui::RichText::new("Sin base de datos abierta").color(C_GREY));
                    }
                });
            });
    }

    // ── Central Panel ────────────────────────────────────────
    fn ui_central_panel(&mut self, ctx: &Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame {
                inner_margin: Margin::symmetric(10.0, 8.0),
                outer_margin: Margin::default(),
                rounding: Rounding::ZERO,
                shadow: egui::Shadow::NONE,
                fill: C_BG,
                stroke: Stroke::NONE,
            })
            .show(ctx, |ui| {
                if self.conn.is_none() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);
                        ui.heading("Abrir una base de datos para comenzar");
                        if ui.button("Abrir BD").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("SQLite", &["db", "sqlite", "sqlite3"])
                                .pick_file() { self.open_database(&path); }
                        }
                    });
                    return;
                }

                // Error banner
                if let Some(ref err) = self.error.clone() {
                    egui::Frame::none()
                        .fill(C_RED.linear_multiply(0.1))
                        .stroke(egui::Stroke::new(1.0, C_RED))
                        .rounding(4.0)
                        .inner_margin(Margin::same(8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(err).color(C_RED).strong());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("✕").clicked() { self.error = None; }
                                });
                            });
                        });
                    ui.add_space(8.0);
                }

                // Table/view selector + refresh
                ui.horizontal(|ui| {
                    ui.label("Tabla / Vista:");
                    let prev = self.vista.clone();
                    egui::ComboBox::from_id_salt("tabla_selector")
                        .width(250.0)
                        .selected_text(if self.vista.is_empty() { "—".to_string() } else { self.vista.replace('_', " ") })
                        .show_ui(ui, |ui| {
                            for t in &self.tablas_disponibles {
                                ui.selectable_value(&mut self.vista, t.clone(), t.replace('_', " "));
                            }
                        });
                    if prev != self.vista { self.seleccionar_tabla(self.vista.clone()); }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Refrescar").clicked() { self.needs_refresh = true; }
                    });
                });
                ui.separator();

                if self.vista.is_empty() {
                    ui.label("Selecciona una tabla o vista del listado.");
                    return;
                }

                // Metric cards
                ui.columns(4, |cols| {
                    crate::ui::widgets::metric_card(&mut cols[0], &self.config.pending_label, self.data.total_pendientes, C_RED);
                    crate::ui::widgets::metric_card(&mut cols[1], &self.config.signed_label, self.data.total_firmados, C_GREEN);
                    crate::ui::widgets::metric_card(&mut cols[2], "Total", self.data.total_general, C_DKBLUE);
                    crate::ui::widgets::metric_card(&mut cols[3], "Universo", self.data.total_matching, C_ORANGE);
                });
                ui.separator();

                // Group-by selector
                ui.horizontal(|ui| {
                    ui.label("Agrupar por:");
                    let prev = self.group_by.clone();
                    egui::ComboBox::from_id_salt("group_by")
                        .selected_text(if self.group_by.is_empty() { "Ninguno".to_string() } else { self.group_by.replace('_', " ") })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.group_by, String::new(), "Ninguno");
                            for col in &self.data.columnas_tabla {
                                ui.selectable_value(&mut self.group_by, col.clone(), col.replace('_', " "));
                            }
                        });
                    if prev != self.group_by { self.needs_refresh = true; }
                });
                ui.separator();

                // Charts
                if !self.data.por_grupo.is_empty() {
                    chart_ui::show_charts(ui, &ChartsData {
                        por_grupo: &self.data.por_grupo,
                    });
                    ui.separator();
                }

                // Data table (renders beneath loading overlay)
                crate::ui::tabla::ui_tabla(self, ui);

                // Loading overlay — semi-transparent on top of existing data
                if self.is_loading {
                    let overlay_rect = ui.max_rect();
                    ui.painter().rect_filled(overlay_rect, egui::Rounding::ZERO, egui::Color32::from_black_alpha(180));
                    ui.vertical_centered(|ui| {
                        ui.add_space(overlay_rect.height() / 3.0);
                        ui.add(egui::Spinner::new().size(48.0));
                        ui.label("Cargando...");
                    });
                }
            });
    }

    // ── Redactor ─────────────────────────────────────────────
    fn ui_redactor_window(&mut self, ctx: &Context) {
        crate::ui::redactor_window::ui_redactor_window(self, ctx);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(theme::nord_light());

        if self.needs_load {
            self.needs_load = false;
            self.refresh_data();
        }

        self.ui_status_bar(ctx);
        self.ui_top_panel(ctx);
        self.ui_sidebar(ctx);
        self.ui_central_panel(ctx);
        self.ui_redactor_window(ctx);

        // Process screenshot for export
        if self.pending_export.is_some() {
            let events = ctx.input(|i| i.events.clone());
            for event in &events {
                if let egui::Event::Screenshot { image, .. } = event {
                    let format = self.pending_export.take();
                    let img = image.as_ref().clone();
                    let w = img.width();
                    let h = img.height();
                    let raw: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_array()).collect();
                    let png_data = {
                        let rgba = image::RgbaImage::from_raw(w as u32, h as u32, raw);
                        let mut buf = std::io::Cursor::new(Vec::new());
                        if let Some(rgba_img) = rgba {
                            let _ = rgba_img.write_to(&mut buf, image::ImageFormat::Png);
                        }
                        buf.into_inner()
                    };
                    if let Some(ExportFormat::Pdf) = format {
                        if let Some(ref conn) = self.conn {
                            let p = self.config.output_dir.join("dashboard.pdf");
                            match crate::export::exportar_pdf_with_screenshot(conn, &self.filtros, &p, &self.vista, &self.config, &png_data) {
                                Ok(path) => { let _ = open::that(path); }
                                Err(e) => self.error = Some(e),
                            }
                        }
                    } else if let Some(ExportFormat::Pptx) = format {
                        let p = self.config.output_dir.join("dashboard.pptx");
                        match crate::export::exportar_pptx_with_screenshot(&p, &png_data) {
                            Ok(path) => { let _ = open::that(path); }
                            Err(e) => self.error = Some(e),
                        }
                    }
                    break;
                }
            }
        }

        if self.needs_refresh {
            self.needs_refresh = false;
            self.needs_load = true;
            ctx.request_repaint();
        }
    }
}

fn listar_tablas_vistas(conn: &Connection) -> Vec<String> {
    let mut items = Vec::new();
    if let Ok(tablas) = db::schema::listar_tablas(conn) { items.extend(tablas); }
    if let Ok(vistas) = db::schema::listar_vistas(conn) { items.extend(vistas); }
    items.sort(); items.dedup(); items
}
