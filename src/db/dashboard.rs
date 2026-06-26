use rusqlite::Connection;
use rusqlite::Result as SqlResult;
use rusqlite::types::ToSql;

use super::types::{DashboardData, FiltroValor, ModoOptimizacion};
use super::constants;
use super::utils::{clean_identifier, safe_ident};
use super::schema::{obtener_columnas, detectar_pk_columna};
use std::collections::HashMap;

fn col_prefix(modo: Option<&ModoOptimizacion>) -> &'static str {
    if modo.map_or(false, |m| !m.es_universal()) { "tb." } else { "" }
}

fn add_categorical_filter(sc: &str, selected: &str, clauses: &mut Vec<String>, params: &mut Vec<Box<dyn ToSql>>) {
    if !selected.is_empty() && selected != constants::FILTRO_TODOS {
        clauses.push(format!("{} = ?", sc));
        params.push(Box::new(selected.to_string()));
    }
}

fn add_categorical_fk_filter(
    p: &str, col_original: &str, tabla_catalogo: &str, col_nombre: &str,
    selected: &str, conn: Option<&Connection>, pk_cache: &mut HashMap<String, String>,
    clauses: &mut Vec<String>, params: &mut Vec<Box<dyn ToSql>>,
) {
    if !selected.is_empty() && selected != constants::FILTRO_TODOS {
        let tc = safe_ident(tabla_catalogo);
        let cn = safe_ident(col_nombre);
        let co = format!("{}{}", p, safe_ident(col_original));
        let pk_col = pk_cache.entry(tabla_catalogo.to_string()).or_insert_with(|| {
            conn.and_then(|c| detectar_pk_columna(c, tabla_catalogo).ok()).unwrap_or_else(|| "rowid".to_string())
        });
        clauses.push(format!("{co} = (SELECT {pk_col} FROM {tc} WHERE {cn} = ?)"));
        params.push(Box::new(selected.to_string()));
    }
}

fn add_date_filter(sc: &str, desde: &str, hasta: &str, clauses: &mut Vec<String>, params: &mut Vec<Box<dyn ToSql>>) {
    if !desde.is_empty() {
        clauses.push(format!("{} >= ?", sc));
        params.push(Box::new(desde.to_string()));
    }
    if !hasta.is_empty() {
        clauses.push(format!("{} <= ?", sc));
        params.push(Box::new(hasta.to_string()));
    }
}

fn add_numeric_filter(sc: &str, min: f64, max: f64, clauses: &mut Vec<String>, params: &mut Vec<Box<dyn ToSql>>) {
    clauses.push(format!("{} >= ?", sc));
    params.push(Box::new(min));
    clauses.push(format!("{} <= ?", sc));
    params.push(Box::new(max));
}

fn add_text_search_filter(sc: &str, query: &str, clauses: &mut Vec<String>, params: &mut Vec<Box<dyn ToSql>>) {
    if !query.is_empty() {
        clauses.push(format!("CAST({} AS TEXT) LIKE ?", sc));
        params.push(Box::new(format!("%{}%", query)));
    }
}

fn construir_where(
    filtros: &HashMap<String, FiltroValor>,
    modo: Option<&ModoOptimizacion>,
    conn: Option<&Connection>,
) -> (String, Vec<Box<dyn ToSql>>) {
    let p = col_prefix(modo);
    let mut pk_cache: HashMap<String, String> = HashMap::new();
    let mut where_clauses: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn ToSql>> = Vec::new();

    for (col_name, filtro) in filtros {
        let sc = format!("{}{}", p, safe_ident(col_name));
        match filtro {
            FiltroValor::Categorical { selected } =>
                add_categorical_filter(&sc, selected, &mut where_clauses, &mut params),
            FiltroValor::CategoricalFK { selected, col_original, tabla_catalogo, col_nombre } =>
                add_categorical_fk_filter(&p, col_original, tabla_catalogo, col_nombre, selected, conn, &mut pk_cache, &mut where_clauses, &mut params),
            FiltroValor::Date { desde, hasta } =>
                add_date_filter(&sc, desde, hasta, &mut where_clauses, &mut params),
            FiltroValor::Numeric { min, max, .. } =>
                add_numeric_filter(&sc, *min, *max, &mut where_clauses, &mut params),
            FiltroValor::TextSearch { query } =>
                add_text_search_filter(&sc, query, &mut where_clauses, &mut params),
        }
    }

    if where_clauses.is_empty() {
        (String::new(), params)
    } else {
        (format!("WHERE {}", where_clauses.join(" AND ")), params)
    }
}

pub fn dashboard(
    conn: &Connection,
    filtros: &HashMap<String, FiltroValor>,
    vista: &str,
    group_by: Option<&str>,
    page: usize,
    page_size: usize,
    status_col: Option<&str>,
    modo: Option<&ModoOptimizacion>,
    pending_pattern: Option<&str>,
    signed_pattern: Option<&str>,
) -> SqlResult<DashboardData> {
    if vista.is_empty() || !clean_identifier(vista) {
        return Ok(DashboardData::default());
    }

    let st = safe_ident(vista);
    let cols = obtener_columnas(conn, vista)?;
    let col_names: Vec<String> = cols.iter().map(|c| c.name.clone()).collect();
    let (where_sql, params) = construir_where(filtros, modo, Some(conn));
    let params_refs: Vec<&dyn ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let from_clause = match modo {
        Some(m) if !m.es_universal() => {
            let tb = safe_ident(m.tabla_base().unwrap());
            let mut joins = Vec::new();
            for fk in m.fks() {
                let alias = format!("c_{}", fk.col_id);
                joins.push(format!(
                    "LEFT JOIN {} AS {} ON {}.{} = {}.{}",
                    safe_ident(&fk.tabla_catalogo),
                    safe_ident(&alias),
                    "tb", safe_ident(&fk.col_id),
                    safe_ident(&alias), safe_ident(&fk.pk_col),
                ));
            }
            if joins.is_empty() {
                format!("FROM {tb} AS tb")
            } else {
                format!("FROM {tb} AS tb {}", joins.join(" "))
            }
        }
        _ => format!("FROM {st}"),
    };

    let count_sql = format!("SELECT COUNT(*) {from_clause} {where_sql}");
    let total_general: i64 = conn.query_row(&count_sql, params_refs.as_slice(), |row| row.get(0))?;
    let total_all_sql = format!("SELECT COUNT(*) {from_clause}");
    let total_matching: i64 = conn.query_row(&total_all_sql, [], |row| row.get(0))?;

    let pending_pat = pending_pattern.unwrap_or(constants::DEFAULT_PENDING_PATTERN);
    let signed_pat = signed_pattern.unwrap_or(constants::DEFAULT_SIGNED_PATTERN);
    let pendientes = contar_por_estado(conn, &format!("{from_clause}"), &where_sql, params_refs.as_slice(), status_col, pending_pat, modo)?;
    let firmados = contar_por_estado(conn, &format!("{from_clause}"), &where_sql, params_refs.as_slice(), status_col, signed_pat, modo)?;

    let grupo_actual = group_by.unwrap_or("").to_string();
    let por_grupo = if !grupo_actual.is_empty() && clean_identifier(&grupo_actual) {
        let p = col_prefix(modo);
        let sc = format!("{}{}", p, safe_ident(&grupo_actual));
        let group_where = if where_sql.is_empty() {
            "WHERE".to_string()
        } else {
            format!("{} AND", where_sql)
        };
        let group_sql = format!(
            "SELECT CAST({sc} AS TEXT), COUNT(*) {from_clause} {group_where} \
             {sc} IS NOT NULL AND CAST({sc} AS TEXT) != '' \
             GROUP BY {sc} ORDER BY COUNT(*) DESC LIMIT {}",
            constants::GROUP_BY_LIMIT
        );
        let mut stmt = conn.prepare(&group_sql)?;
        let mut map = HashMap::new();
        if let Ok(rows) = stmt.query_map(params_refs.as_slice(), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        }) {
            for row in rows.flatten() {
                map.insert(row.0, row.1 as u64);
            }
        }
        map
    } else {
        HashMap::new()
    };

    let select_cols: Vec<String> = match modo {
        Some(m) if !m.es_universal() => {
            col_names.iter().map(|c| {
                if let Some(fk) = m.fks().iter().find(|fk| &fk.nombre_display == c) {
                    let alias = format!("c_{}", fk.col_id);
                    format!("{}.{} AS \"{}\"", safe_ident(&alias), safe_ident(&fk.col_nombre), c)
                } else {
                    format!("tb.{}", safe_ident(c))
                }
            }).collect()
        }
        _ => {
            let p = col_prefix(modo);
            col_names.iter().map(|c| format!("{}{}", p, safe_ident(c))).collect()
        }
    };

    let offset = page.saturating_sub(1) * page_size;
    let limit = page_size.min(constants::TABLE_LIMIT);
    let table_sql = format!(
        "SELECT {} {from_clause} {where_sql} LIMIT {} OFFSET {}",
        select_cols.join(", "),
        limit,
        offset,
    );
    let mut stmt = conn.prepare(&table_sql)?;
    let mut tabla = Vec::new();
    if let Ok(rows) = stmt.query_map(params_refs.as_slice(), |row| {
        let mut map = HashMap::new();
        for (i, name) in col_names.iter().enumerate() {
            let val: String = row.get::<_, Option<String>>(i).unwrap_or_default().unwrap_or_default();
            map.insert(name.clone(), serde_json::Value::String(val));
        }
        Ok(map)
    }) {
        for row in rows.flatten() {
            tabla.push(row);
        }
    }

    Ok(DashboardData {
        total_pendientes: pendientes as u64,
        total_firmados: firmados as u64,
        total_general: total_general as u64,
        total_matching: total_matching as u64,
        current_page: page,
        page_size,
        por_grupo,
        grupo_actual,
        tabla,
        columnas_tabla: col_names,
    })
}

fn contar_por_estado(
    conn: &Connection, from_clause: &str, where_sql: &str,
    params: &[&dyn ToSql],
    status_col: Option<&str>, pattern: &str,
    modo: Option<&ModoOptimizacion>,
) -> SqlResult<i64> {
    let col = match status_col {
        Some(c) => c,
        None => return Ok(0),
    };
    if !clean_identifier(col) {
        return Ok(0);
    }
    let p = col_prefix(modo);
    let sc = format!("{}{}", p, safe_ident(col));
    let escaped_pattern = format!("%{}%", pattern.to_uppercase());
    let cond = if where_sql.is_empty() { "WHERE".to_string() } else { format!("{} AND", where_sql) };
    let sql = format!(
        "SELECT COUNT(*) {from_clause} {cond} UPPER(CAST({sc} AS TEXT)) LIKE ?"
    );

    let mut local_params: Vec<&dyn ToSql> = params.to_vec();
    local_params.push(&escaped_pattern);
    conn.query_row(&sql, local_params.as_slice(), |row| row.get::<_, i64>(0))
}
