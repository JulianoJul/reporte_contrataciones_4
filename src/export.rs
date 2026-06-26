use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;

use crate::config::Config;
use crate::db;
use rusqlite::Connection;

pub fn abrir_carpeta_output(config: &Config) -> Result<(), String> {
    let path = &config.output_dir;
    if !path.exists() {
        std::fs::create_dir_all(path).map_err(|e| format!("Error creando output dir: {}", e))?;
    }
    open::that(path).map_err(|e| format!("Error abriendo carpeta: {}", e))
}

pub fn exportar_excel(
    conn: &Connection,
    filtros: &HashMap<String, db::FiltroValor>,
    output_path: &Path,
    vista: &str,
    config: &Config,
) -> Result<String, String> {
    let data = db::dashboard(
        conn, filtros, vista, None, 1, 500, None, None,
        Some(&config.pending_pattern), Some(&config.signed_pattern)
    ).map_err(|e| e.to_string())?;

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Error creando directorio: {}", e))?;
    }

    let mut workbook = rust_xlsxwriter::Workbook::new();
    let sheet = workbook
        .add_worksheet()
        .set_name("Reporte")
        .map_err(|e| e.to_string())?;

    // Headers
    for (i, col_name) in data.columnas_tabla.iter().enumerate() {
        sheet
            .write_string(0, i as u16, col_name)
            .map_err(|e| e.to_string())?;
    }

    // Data rows
    for (row_idx, row) in data.tabla.iter().enumerate() {
        for (col_idx, col_name) in data.columnas_tabla.iter().enumerate() {
            let val = row
                .get(col_name)
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            sheet
                .write_string((row_idx + 1) as u32, col_idx as u16, &val)
                .map_err(|e| e.to_string())?;
        }
    }

    // Auto-width
    for (i, col_name) in data.columnas_tabla.iter().enumerate() {
        let max_len = col_name.len().max(10).min(50) as f64;
        sheet.set_column_width(i as u16, max_len + 2.0)
            .map_err(|e| e.to_string())?;
    }

    workbook
        .save(output_path)
        .map_err(|e| format!("Error guardando Excel: {}", e))?;

    Ok(output_path.to_string_lossy().to_string())
}

pub fn exportar_pdf_with_screenshot(
    conn: &Connection,
    filtros: &HashMap<String, db::FiltroValor>,
    output_path: &Path,
    vista: &str,
    config: &Config,
    png_bytes: &[u8],
) -> Result<String, String> {
    use printpdf::{BuiltinFont, Image, ImageTransform, Mm, PdfDocument};

    let data = db::dashboard(
        conn, filtros, vista, None, 1, db::constants::PDF_ROW_LIMIT, None, None,
        Some(&config.pending_pattern), Some(&config.signed_pattern)
    ).map_err(|e| e.to_string())?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Error creando directorio: {}", e))?;
    }

    let (doc, page1, layer1) = PdfDocument::new(
        "Dashboard de Contrataciones",
        Mm(297.0),
        Mm(210.0),
        "Layer 1",
    );

    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| format!("Error con fuente: {}", e))?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| format!("Error con fuente: {}", e))?;

    let layer = doc.get_page(page1).get_layer(layer1);

    layer.use_text("Dashboard de Contrataciones", 14.0, Mm(10.0), Mm(195.0), &font_bold);
    layer.use_text(
        &format!("Registros: {} | Pendientes: {} | Firmados: {}",
                 data.total_general, data.total_pendientes, data.total_firmados),
        10.0, Mm(10.0), Mm(188.0), &font,
    );

    let stats = format!(
        "Total: {} | Pendientes: {} | Firmados: {} | Universo: {}",
        data.total_general, data.total_pendientes, data.total_firmados, data.total_matching
    );
    layer.use_text(&stats, 8.0, Mm(10.0), Mm(4.0), &font);

    // Embed screenshot image after all text (consumes layer)
    let dynamic_img = ::image::load_from_memory(png_bytes)
        .map_err(|e| format!("Error cargando imagen: {}", e))?;
    let print_img = Image::from_dynamic_image(&dynamic_img);
    let dpi = 300.0;
    let natural_w_pt = dynamic_img.width() as f32 * 72.0 / dpi;
    let natural_h_pt = dynamic_img.height() as f32 * 72.0 / dpi;
    let target_w_pt = Mm(277.0).into_pt().0;
    let target_h_pt = Mm(175.0).into_pt().0;
    let scale = (target_w_pt / natural_w_pt).min(target_h_pt / natural_h_pt);
    print_img.add_to_layer(layer, ImageTransform {
        translate_x: Some(Mm(10.0)),
        translate_y: Some(Mm(10.0)),
        scale_x: Some(scale),
        scale_y: Some(scale),
        dpi: Some(dpi),
        ..Default::default()
    });

    let file = File::create(output_path)
        .map_err(|e| format!("Error creando PDF: {}", e))?;
    let mut buf_writer = BufWriter::new(file);
    doc.save(&mut buf_writer)
        .map_err(|e| format!("Error guardando PDF: {}", e))?;

    Ok(output_path.to_string_lossy().to_string())
}

pub fn exportar_pptx_with_screenshot(
    output_path: &Path,
    png_bytes: &[u8],
) -> Result<String, String> {
    use pptx::opc::PackURI;
    use pptx::presentation::Presentation;
    use pptx::shapes::ShapeTree;
    use pptx::units::Emu;
    use pptx::Image;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Error creando directorio: {}", e))?;
    }

    let mut pres = Presentation::new().map_err(|e| format!("Error creando PPTX: {}", e))?;
    let layouts = pres.slide_layouts().map_err(|e| format!("Error obteniendo layouts: {}", e))?;
    let slide_ref = pres.add_slide(&layouts[0]).map_err(|e| format!("Error agregando slide: {}", e))?;

    let img = Image::from_bytes(png_bytes.to_vec(), "image/png");
    let img_partname_str = pres.add_image(&img).map_err(|e| format!("Error agregando imagen: {}", e))?;
    let img_partname = PackURI::new(&img_partname_str).map_err(|e| format!("Error URI imagen: {}", e))?;

    let slide_part = pres.package_mut().part_mut(&slide_ref.partname)
        .ok_or_else(|| "Error accediendo al slide".to_string())?;
    let relative_ref = img_partname.relative_ref(slide_part.partname.base_uri());
    let r_id = slide_part.rels.add_relationship(
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image",
        relative_ref, false,
    );

    let new_xml = ShapeTree::add_picture(
        &slide_part.blob, &r_id,
        Emu(0), Emu(0),
        Emu(9_144_000), Emu(6_858_000),
    ).map_err(|e| format!("Error agregando imagen al slide: {}", e))?;
    slide_part.blob = new_xml;

    pres.save(output_path).map_err(|e| format!("Error guardando PPTX: {}", e))?;

    Ok(output_path.to_string_lossy().to_string())
}
