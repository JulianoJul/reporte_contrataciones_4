use rusqlite::Connection;
use rusqlite::Result as SqlResult;

use super::types::{ColumnaRaw, FkInfo};
use super::utils::{clean_identifier, safe_ident};

pub fn listar_tablas(conn: &Connection) -> SqlResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name",
    )?;
    let tablas = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(tablas)
}

pub fn listar_vistas(conn: &Connection) -> SqlResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='view' ORDER BY name",
    )?;
    let vistas: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(vistas)
}

pub fn encontrar_vista_principal(conn: &Connection, view_keywords: &[String]) -> SqlResult<Option<String>> {
    let vistas = listar_vistas(conn)?;

    for v in &vistas {
        let vl = v.to_lowercase();
        if view_keywords.iter().any(|kw| vl.contains(kw.as_str())) {
            return Ok(Some(v.clone()));
        }
    }
    Ok(vistas.into_iter().next())
}

pub fn encontrar_tabla_principal(conn: &Connection, tablas: &[String]) -> Option<String> {
    let mut mejor: Option<(i64, String)> = None;
    for t in tablas {
        if !clean_identifier(t) {
            continue;
        }
        let st = safe_ident(t);
        let Ok(cols) = conn.prepare(&format!("PRAGMA table_info({})", st))
            .and_then(|mut s| s.query_map([], |row| row.get::<_, String>(1)).map(|r| r.filter_map(|x| x.ok()).collect::<Vec<_>>()))
        else { continue; };
        if cols.len() < 3 {
            continue;
        }
        let Ok(filas) = conn.query_row(&format!("SELECT COUNT(*) FROM {}", st), [], |r| r.get::<_, i64>(0)) else { continue; };
        let puntaje = cols.len() as i64 + if filas > 10 { 1 } else { 0 };
        if puntaje > mejor.as_ref().map(|m| m.0).unwrap_or(-1) {
            mejor = Some((puntaje, t.clone()));
        }
    }
    mejor.map(|m| m.1)
}

pub fn obtener_columnas(conn: &Connection, tabla: &str) -> SqlResult<Vec<ColumnaRaw>> {
    let st = safe_ident(tabla);
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", st))?;
    let cols = stmt
        .query_map([], |row| {
            Ok(ColumnaRaw {
                _cid: row.get(0)?,
                name: row.get(1)?,
                col_type: row.get::<_, Option<String>>(2)?.unwrap_or_default().to_uppercase(),
                _notnull: row.get::<_, bool>(3)?,
                _dflt_value: row.get(4)?,
                pk: row.get::<_, bool>(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(cols)
}

pub fn analizar_foreign_keys(
    conn: &Connection,
    tablas: &[String],
) -> SqlResult<Vec<(String, FkInfo)>> {
    let mut result = Vec::new();
    for t in tablas {
        if !clean_identifier(t) {
            continue;
        }
        let st = safe_ident(t);
        let mut stmt = conn.prepare(&format!("PRAGMA foreign_key_list({})", st))?;
        let fks = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;
        for fk in fks.flatten() {
            let key = format!("{}.{}", t, fk.1);
            result.push((
                key,
                FkInfo {
                    tabla: fk.0,
                    columna: fk.2,
                },
            ));
        }
    }
    Ok(result)
}

pub fn detectar_pk_columna(conn: &Connection, tabla: &str) -> SqlResult<String> {
    let st = safe_ident(tabla);
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", st))?;
    let cols: Vec<(String, bool)> = stmt
        .query_map([], |row| Ok((row.get::<_, String>(1)?, row.get::<_, bool>(5)?)))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(cols
        .iter()
        .find(|(_, is_pk)| *is_pk)
        .map(|(name, _)| name.clone())
        .unwrap_or_else(|| "rowid".to_string()))
}

pub fn obtener_pk_con_fallback(conn: &Connection, tabla: &str, fallback: &str) -> String {
    detectar_pk_columna(conn, tabla).unwrap_or_else(|_| fallback.to_string())
}
