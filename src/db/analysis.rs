use rusqlite::Connection;
use rusqlite::Result as SqlResult;
use rusqlite::types::ToSql;

use super::types::{ColumnaInfo, ColumnaRaw, FkInfo, DependenciaInfo};
use super::constants;
use super::constants::AnalyseConfig;
use super::schema::obtener_pk_con_fallback;
use super::utils::{clean_identifier, safe_ident, strip_fk_prefix};
use std::collections::HashMap;

fn query_distinct_values(
    conn: &Connection,
    col_expr: &str,
    from_expr: &str,
    where_clause: Option<&str>,
    limit: u64,
) -> SqlResult<Vec<serde_json::Value>> {
    let where_sql = where_clause
        .map(|w| format!(" WHERE {}", w))
        .unwrap_or_default();
    let sql = format!(
        "SELECT DISTINCT {} FROM {} {} ORDER BY 1 LIMIT {}",
        col_expr, from_expr, where_sql, limit
    );
    let mut stmt = conn.prepare(&sql)?;
    let values = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .map(|v| serde_json::json!(v))
        .collect();
    Ok(values)
}

fn build_columna_info(
    nombre: String, tipo: &str,
    valores: Option<Vec<serde_json::Value>>,
    total_distintos: Option<u64>,
    min: Option<f64>, max: Option<f64>,
    fecha_min: Option<String>, fecha_max: Option<String>,
    col_original: Option<String>,
    tabla_catalogo: Option<String>,
    col_nombre_catalogo: Option<String>,
) -> ColumnaInfo {
    ColumnaInfo {
        nombre, tipo: tipo.to_string(), valores, total_distintos,
        min, max, fecha_min, fecha_max,
        col_original, tabla_catalogo, col_nombre_catalogo,
    }
}

fn query_min_max<T: rusqlite::types::FromSql>(
    conn: &Connection,
    tabla: &str,
    col: &str,
) -> SqlResult<(Option<T>, Option<T>)> {
    let st = safe_ident(tabla);
    let sc = safe_ident(col);
    conn.query_row(
        &format!("SELECT MIN({sc}), MAX({sc}) FROM {st}"),
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
}

pub fn analizar_columna(
    conn: &Connection,
    tabla: &str,
    col: &ColumnaRaw,
    fk_pairs: &[(String, FkInfo)],
    ac: &AnalyseConfig,
) -> SqlResult<Option<ColumnaInfo>> {
    let st = safe_ident(tabla);

    // Detect FK columns → show catalog display names
    let fk_match = fk_pairs.iter().find(|(k, _)| *k == format!("{}.{}", tabla, col.name));
    if let Some((_, fk_info)) = fk_match {
        if let Some(col_nombre) = detectar_columna_nombre(conn, &fk_info.tabla, ac)? {
            let tc = safe_ident(&fk_info.tabla);
            let cn = safe_ident(&col_nombre);
            let valores = query_distinct_values(
                conn, &cn, &tc,
                Some(&format!("{} IS NOT NULL", cn)),
                constants::MAX_CATEGORICAL_VALUES * 2,
            )?;
            let total_dist = valores.len() as u64;
            let nombre_display = strip_fk_prefix(&col.name, &ac.fk_id_prefix);
            return Ok(Some(build_columna_info(
                nombre_display, "categorical_fk",
                Some(valores), Some(total_dist),
                None, None, None, None,
                Some(col.name.clone()),
                Some(fk_info.tabla.clone()), Some(col_nombre),
            )));
        }
    }

    let sc = safe_ident(&col.name);

    match col.col_type.as_str() {
        "DATE" | "DATETIME" | "TIMESTAMP" => {
            let (min_v, max_v) = query_min_max::<String>(conn, tabla, &col.name)?;
            Ok(Some(build_columna_info(
                col.name.clone(), "date",
                None, None, None, None, min_v, max_v,
                None, None, None,
            )))
        }

        "REAL" | "FLOAT" | "DOUBLE" | "NUMERIC" | "DECIMAL" => {
            let (min_v, max_v) = query_min_max::<f64>(conn, tabla, &col.name)?;
            let distinct: i64 = conn
                .query_row(
                    &format!("SELECT COUNT(DISTINCT {sc}) FROM {st}"),
                    [],
                    |row| row.get(0),
                )?;

            if distinct as u64 <= constants::MAX_CATEGORICAL_VALUES && min_v.is_some() {
                let values = query_distinct_values(
                    conn, &sc.to_string(), &st.to_string(),
                    None,
                    constants::MAX_CATEGORICAL_VALUES * 2,
                )?;
                return Ok(Some(build_columna_info(
                    col.name.clone(), "categorical",
                    Some(values), Some(distinct as u64),
                    None, None, None, None,
                    None, None, None,
                )));
            }

            Ok(Some(build_columna_info(
                col.name.clone(), "numeric",
                None, None, min_v, max_v, None, None,
                None, None, None,
            )))
        }

        "INTEGER" | "BIGINT" | "SMALLINT" | "TINYINT" => {
            let (distinct, total): (i64, i64) = conn
                .query_row(
                    &format!("SELECT COUNT(DISTINCT {sc}), COUNT(*) FROM {st}"),
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )?;

            if total > 0 && (distinct as f64 / total as f64) > constants::PK_RATIO_THRESHOLD {
                return Ok(None);
            }

            if distinct as u64 <= constants::MAX_CATEGORICAL_VALUES {
                let values = query_distinct_values(
                    conn, &sc.to_string(), &st.to_string(),
                    None,
                    constants::MAX_CATEGORICAL_VALUES * 2,
                )?;
                return Ok(Some(build_columna_info(
                    col.name.clone(), "categorical",
                    Some(values), Some(distinct as u64),
                    None, None, None, None,
                    None, None, None,
                )));
            }

            let (min_v, max_v) = query_min_max::<i64>(conn, tabla, &col.name)?;
            return Ok(Some(build_columna_info(
                col.name.clone(), "numeric",
                None, None, min_v.map(|v| v as f64), max_v.map(|v| v as f64), None, None,
                None, None, None,
            )));
        }

        _ => {
            let row: std::result::Result<(i64, f64), _> = conn.query_row(
                &format!(
                    "SELECT COUNT(DISTINCT {sc}), AVG(LENGTH({sc})) \
                     FROM (SELECT {sc} FROM {st} LIMIT {})",
                    constants::SAMPLE_SIZE
                ),
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            );
            let (distinct, avg_len) = match row {
                Ok(r) => r,
                Err(_) => return Ok(None),
            };

            if avg_len > constants::MAX_TEXT_LENGTH_THRESHOLD {
                return Ok(None);
            }

            if distinct as u64 <= constants::MAX_CATEGORICAL_VALUES {
                let values = query_distinct_values(
                    conn, &sc.to_string(), &st.to_string(),
                    Some(&format!("{} IS NOT NULL AND {} != ''", sc, sc)),
                    constants::MAX_CATEGORICAL_VALUES * 2,
                )?;
                Ok(Some(build_columna_info(
                    col.name.clone(), "categorical",
                    Some(values), Some(distinct as u64),
                    None, None, None, None,
                    None, None, None,
                )))
            } else {
                Ok(Some(build_columna_info(
                    col.name.clone(), "text_search",
                    None, None, None, None, None, None,
                    None, None, None,
                )))
            }
        }
    }
}

pub fn detectar_columna_estado(
    conn: &Connection,
    tabla: &str,
    col_names: &[String],
    pending_pattern: &str,
    signed_pattern: &str,
) -> SqlResult<Option<String>> {
    let st = safe_ident(tabla);
    let mut best_col: Option<String> = None;
    let mut best_score = 0.0;

    let pend_like = format!("%{}%", pending_pattern.to_uppercase());
    let firm_like = format!("%{}%", signed_pattern.to_uppercase());

    for col in col_names {
        if !clean_identifier(col) {
            continue;
        }
        let sc = safe_ident(col);

        let (distinct_count, total_count): (i64, i64) = match conn.query_row(
            &format!(
                "SELECT COUNT(DISTINCT v), COUNT(*) FROM (SELECT CAST({sc} AS TEXT) AS v FROM {st}) \
                 WHERE v IS NOT NULL AND v != ''"
            ),
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ) {
            Ok(r) => r,
            Err(_) => continue,
        };

        if total_count == 0 {
            continue;
        }

        let distinct_ratio = distinct_count as f64 / total_count as f64;

        let mut short_values = 0i64;
        if let Ok(mut stmt) = conn.prepare(&format!(
            "SELECT COUNT(*) FROM (SELECT CAST({sc} AS TEXT) as v FROM {st} \
             WHERE v IS NOT NULL AND v != '' GROUP BY v) \
             WHERE LENGTH(v) <= {}",
            constants::STATUS_SHORT_LENGTH_THRESHOLD
        )) {
            short_values = stmt.query_row([], |row| row.get(0))
                .unwrap_or_else(|e| { eprintln!("[db/analysis] short_values query failed for col '{}': {}", col, e); 0 });
        }


        let short_ratio = if distinct_count > 0 {
            short_values as f64 / distinct_count as f64
        } else {
            0.0
        };

        let mut pend_ratio = 0.0;
        let mut firm_ratio = 0.0;
        if distinct_count <= constants::MAX_CATEGORICAL_VALUES as i64 {
            let sql = format!(
                "SELECT \
                 CAST(SUM(CASE WHEN UPPER({sc}) LIKE ? THEN 1 ELSE 0 END) AS REAL) / MAX(1.0 * COUNT(*), 1.0), \
                 CAST(SUM(CASE WHEN UPPER({sc}) LIKE ? THEN 1 ELSE 0 END) AS REAL) / MAX(1.0 * COUNT(*), 1.0) \
                 FROM {st}"
            );
            let (p, f): (f64, f64) = conn.query_row(
                &sql,
                &[&pend_like as &dyn ToSql, &firm_like as &dyn ToSql],
                |row| Ok((row.get(0)?, row.get(1)?)),
            ).unwrap_or_else(|e| { eprintln!("[db/analysis] status query failed for col '{}': {}", col, e); (0.0, 0.0) });
            pend_ratio = p;
            firm_ratio = f;
        }

        let status_combined = pend_ratio + firm_ratio;
        let both_statuses = pend_ratio > constants::STATUS_THRESHOLD && firm_ratio > constants::STATUS_THRESHOLD;

        let score = if distinct_ratio < constants::STATUS_DISTINCT_RATIO_THRESHOLD
            && distinct_count >= constants::STATUS_MIN_DISTINCT && distinct_count <= constants::STATUS_MAX_DISTINCT
            && short_ratio > constants::STATUS_SHORT_RATIO_THRESHOLD
        {
            if status_combined > constants::STATUS_COMBINED_THRESHOLD {
                constants::STATUS_SCORE_BASE + status_combined * if both_statuses { constants::STATUS_BOTH_MULTIPLIER } else { 1.0 }
            } else {
                let coverage = (distinct_count as f64) / total_count.min(distinct_count * 100) as f64;
                constants::STATUS_SCORE_BASE + coverage * constants::STATUS_COVERAGE_MULTIPLIER
            }
        } else if status_combined > constants::STATUS_COMBINED_THRESHOLD {
            status_combined * if both_statuses { constants::STATUS_BOTH_MULTIPLIER } else { 1.0 }
        } else {
            continue;
        };

        if score > best_score {
            best_score = score;
            best_col = Some(col.clone());
        }
    }

    Ok(best_col)
}

pub fn detectar_columna_nombre(
    conn: &Connection,
    tabla: &str,
    ac: &AnalyseConfig,
) -> SqlResult<Option<String>> {
    let st = safe_ident(tabla);
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", st))?;
    let col_names: Vec<String> = stmt.query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();
    for preferred in &ac.preferred_name_cols {
        if col_names.iter().any(|c| c.to_lowercase() == *preferred) {
            return Ok(Some(preferred.clone()));
        }
    }
    for c in &col_names {
        let cl = c.to_lowercase();
        if !cl.starts_with(&ac.exclude_id_prefix)
            && !ac.exclude_name_cols.iter().any(|e| cl == *e)
        {
            return Ok(Some(c.clone()));
        }
    }
    Ok(col_names.into_iter().next())
}

pub fn detectar_dependencias(
    conn: &Connection,
    columnas: &[ColumnaInfo],
    fk_pairs: &[(String, FkInfo)],
) -> SqlResult<Vec<DependenciaInfo>> {
    let cat_cols: Vec<&ColumnaInfo> = columnas.iter().filter(|c| c.tipo == "categorical").collect();
    let mut dependencias = Vec::new();

    for hijo in &cat_cols {
        for padre in &cat_cols {
            if padre.nombre == hijo.nombre {
                continue;
            }
            if let Some(mapeo) = construir_mapeo_dependencia(conn, &padre.nombre, &hijo.nombre, fk_pairs)? {
                if !mapeo.is_empty() {
                    dependencias.push(DependenciaInfo {
                        columna_padre: padre.nombre.clone(),
                        columna_dependiente: hijo.nombre.clone(),
                        mapeo,
                    });
                }
            }
        }
    }

    Ok(dependencias)
}

fn construir_mapeo_dependencia(
    conn: &Connection,
    col_padre: &str,
    col_hijo: &str,
    fk_pairs: &[(String, FkInfo)],
) -> SqlResult<Option<HashMap<String, Vec<String>>>> {
    let padre_keys: Vec<&(String, FkInfo)> = fk_pairs.iter()
        .filter(|(k, _)| k.to_lowercase().contains(&col_padre.to_lowercase()))
        .collect();
    let hijo_keys: Vec<&(String, FkInfo)> = fk_pairs.iter()
        .filter(|(k, _)| k.to_lowercase().contains(&col_hijo.to_lowercase()))
        .collect();

    if padre_keys.is_empty() || hijo_keys.is_empty() {
        return Ok(None);
    }

    for (pk_key, fk_info_pk) in &padre_keys {
        // split() always returns at least one element, unwrap_or("") is safety net
        let pt = pk_key.split(constants::FK_KEY_SEPARATOR).next().unwrap_or("");

        for (fk_key, fk_info_fk) in &hijo_keys {
            // split() always returns at least one element, unwrap_or("") is safety net
            let ht = fk_key.split(constants::FK_KEY_SEPARATOR).next().unwrap_or("");

            if fk_info_fk.tabla != fk_info_pk.tabla {
                continue;
            }
            let tabla_rel = &fk_info_fk.tabla;
            if [tabla_rel, pt, ht, col_padre, col_hijo].iter().any(|s| !clean_identifier(s)) {
                continue;
            }

            let scp = safe_ident(col_padre);
            let sch = safe_ident(col_hijo);
            let str_ = safe_ident(tabla_rel);
            let sa = safe_ident(&fk_info_pk.tabla);
            let sb = safe_ident(&fk_info_fk.tabla);
            let pk_col_name = if fk_info_pk.columna.is_empty() {
                obtener_pk_con_fallback(conn, &fk_info_pk.tabla, crate::db::constants::DEFAULT_PK_FALLBACK)
            } else {
                fk_info_pk.columna.clone()
            };
            let fk_col_name = if fk_info_fk.columna.is_empty() {
                obtener_pk_con_fallback(conn, &fk_info_fk.tabla, crate::db::constants::DEFAULT_PK_FALLBACK)
            } else {
                fk_info_fk.columna.clone()
            };
            let sapc = safe_ident(&pk_col_name);
            let saoc = safe_ident(&fk_col_name);

            let query = format!(
                "SELECT DISTINCT a.{scp}, b.{sch} \
                 FROM {str_} tr \
                 JOIN {sa} a ON tr.{sapc} = a.{sapc} \
                 JOIN {sb} b ON tr.{saoc} = b.{saoc} \
                 WHERE a.{scp} IS NOT NULL AND b.{sch} IS NOT NULL \
                 ORDER BY 1, 2"
            );

            if let Ok(mut stmt) = conn.prepare(&query) {
                if let Ok(rows) = stmt.query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                }) {
                    let mut mapeo: HashMap<String, Vec<String>> = HashMap::new();
                    for row in rows.flatten() {
                        mapeo.entry(row.0).or_default().push(row.1);
                    }
                    if !mapeo.is_empty() {
                        return Ok(Some(mapeo));
                    }
                }
            }
        }
    }

    Ok(None)
}

pub fn extraer_filtros_info(meta: &super::types::SchemaMetadata) -> Vec<super::types::FiltroInfo> {
    let mut filtros: Vec<super::types::FiltroInfo> = meta.columnas.iter()
        .map(|c| c.to_filtro_info())
        .collect();
    filtros.sort_by(|a, b| a.nombre.cmp(&b.nombre));
    filtros
}
