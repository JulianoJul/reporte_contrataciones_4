use rusqlite::Connection;
use rusqlite::Result as SqlResult;

use super::types::{SchemaMetadata, FKOptimizada, ModoOptimizacion, FkInfo};
use super::constants::{self, AnalyseConfig};
use super::schema::{listar_tablas, encontrar_vista_principal, encontrar_tabla_principal, obtener_columnas, analizar_foreign_keys};
use super::analysis::{analizar_columna, detectar_dependencias, detectar_columna_nombre};
use super::utils::strip_fk_prefix;

pub fn explorar(conn: &Connection, ac: &AnalyseConfig) -> SqlResult<SchemaMetadata> {
    let tablas = listar_tablas(conn)?;
    let vista = encontrar_vista_principal(conn, &ac.view_keywords)?
        .or_else(|| encontrar_tabla_principal(conn, &tablas))
        .unwrap_or_default();

    if vista.is_empty() {
        return Ok(SchemaMetadata {
            vista_principal: String::new(),
            tablas,
            columnas: vec![],
            dependencias: vec![],
        });
    }

    let columnas_raw = obtener_columnas(conn, &vista)?;
    let fk_pairs = analizar_foreign_keys(conn, &tablas)?;

    let mut columnas = Vec::new();
    for col in &columnas_raw {
        if col.pk {
            continue;
        }
        if let Some(info) = analizar_columna(conn, &vista, col, &fk_pairs, ac)? {
            columnas.push(info);
        }
    }

    let cat_count = columnas.iter().filter(|c| c.tipo == "categorical" || c.tipo == "categorical_fk").count();
    let dependencias = if cat_count >= 2 {
        detectar_dependencias(conn, &columnas, &fk_pairs)?
    } else {
        Vec::new()
    };

    Ok(SchemaMetadata {
        vista_principal: vista,
        tablas,
        columnas,
        dependencias,
    })
}

pub fn detectar_patron_optimizable(
    conn: &Connection,
    vista: &str,
    ac: &AnalyseConfig,
) -> SqlResult<ModoOptimizacion> {
    let vl = vista.to_lowercase();

    // CRITERIO 1: Vistas con keywords conocidos → Universal
    if vista.is_empty() || ac.view_keywords.iter().any(|kw| vl.contains(kw.as_str())) {
        return Ok(ModoOptimizacion::Universal);
    }

    // CRITERIO 1b: Tablas catálogo → Universal (no tienen FKs de negocio, se consultan directas)
    if vl.starts_with(&ac.catalog_prefix) {
        return Ok(ModoOptimizacion::Universal);
    }

    // CRITERIO 2: Para tablas base, verificar si tienen suficientes FKs a catálogos
    let tablas = listar_tablas(conn)?;
    let tabla_base = encontrar_tabla_principal(conn, &tablas)
        .unwrap_or_else(|| vista.to_string());
    let fk_pairs = analizar_foreign_keys(conn, &tablas)?;

    let prefix = format!("{}.", tabla_base);
    let cat_fks: Vec<&(String, FkInfo)> = fk_pairs.iter()
        .filter(|(key, fk)| key.starts_with(&prefix) && fk.tabla.to_lowercase().starts_with(&ac.catalog_prefix))
        .collect();

    if cat_fks.len() < constants::MIN_FK_COUNT_FOR_OPTIMIZATION {
        return Ok(ModoOptimizacion::Universal);
    }

    let mut fks_optimizadas = Vec::new();
    for (key, fk_info) in &cat_fks {
        let Some(col_id) = key.split('.').last().map(|s| s.to_string()) else { continue };
        let Some(col_nombre) = detectar_columna_nombre(conn, &fk_info.tabla, ac)? else { continue };
        let nombre_display = strip_fk_prefix(&col_id, &ac.fk_id_prefix);
        let pk_col = super::schema::detectar_pk_columna(conn, &fk_info.tabla)?;
        fks_optimizadas.push(FKOptimizada {
            col_id,
            tabla_catalogo: fk_info.tabla.clone(),
            col_nombre,
            nombre_display,
            pk_col,
        });
    }

    Ok(ModoOptimizacion::VistaConFKs {
        tabla_base,
        fks: fks_optimizadas,
    })
}
