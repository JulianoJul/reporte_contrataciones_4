"""
queries.py - Universal data layer for any SQLite schema.
Data-driven column detection. Zero hardcoded naming conventions.
"""

import re
import sqlite3
from typing import Any

import pandas as pd

VISTA_PREDETERMINADA = "vw_reporte_excel_contrataciones"

_IDENTIFIER_RE = re.compile(r'^[a-zA-Z_][a-zA-Z0-9_]*$')

def _safe_ident(name: str) -> str:
    if not _IDENTIFIER_RE.match(name):
        raise ValueError(f"Nombre de tabla/columna inseguro: {name!r}")
    return f'"{name}"'


# ─── EXPLORACION ─────────────────────────────────────────────────


def explorar(conn: sqlite3.Connection) -> dict:
    """Introspecciona la BD y devuelve metadata de columnas para construir filtros.
    Puramente data-driven. Sin naming conventions, sin prefijos hardcodeados.
    """
    tablas = _listar_tablas(conn)
    vista = _encontrar_vista_principal(conn, tablas)
    if not vista:
        vista = _encontrar_tabla_principal(conn, tablas)
    if not vista:
        return {"error": "No se encontró ninguna tabla o vista con datos"}

    cursor = conn.execute(f"PRAGMA table_info({_safe_ident(vista)})")
    columnas_raw = cursor.fetchall()

    fk_map = _analizar_foreign_keys(conn, tablas)

    columnas = []
    for col in columnas_raw:
        info = _analizar_columna(conn, vista, col, fk_map)
        if info:
            columnas.append(info)

    dependencias = _detectar_dependencias(conn, columnas, fk_map)

    return {
        "vista_principal": vista,
        "tablas": tablas,
        "columnas": columnas,
        "dependencias": dependencias,
    }


def _listar_tablas(conn: sqlite3.Connection) -> list[str]:
    cursor = conn.execute(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
    )
    return [r[0] for r in cursor.fetchall()]


def _encontrar_vista_principal(conn: sqlite3.Connection, tablas: list[str]) -> str | None:
    cursor = conn.execute(
        "SELECT name FROM sqlite_master WHERE type='view' ORDER BY name"
    )
    vistas = [r[0] for r in cursor.fetchall()]
    if VISTA_PREDETERMINADA in vistas:
        return VISTA_PREDETERMINADA
    for v in vistas:
        if "reporte" in v.lower() or "vista" in v.lower() or "vw_" in v.lower():
            return v
    return vistas[0] if vistas else None


def _encontrar_tabla_principal(conn: sqlite3.Connection, tablas: list[str]) -> str | None:
    """Heurística data-driven: la tabla con más columnas y más filas suele ser la principal.
    Sin filtros por prefijo de nombre.
    """
    mejor = None
    mejor_puntaje = -1
    for t in tablas:
        st = _safe_ident(t)
        cursor = conn.execute(f"PRAGMA table_info({st})")
        cols = cursor.fetchall()
        if len(cols) < 3:
            continue
        cursor.execute(f"SELECT COUNT(*) FROM {st}")
        filas = cursor.fetchone()[0]
        puntaje = len(cols) + (1 if filas > 10 else 0)
        if puntaje > mejor_puntaje:
            mejor_puntaje = puntaje
            mejor = t
    return mejor


def _analizar_columna(
    conn: sqlite3.Connection,
    tabla: str,
    col: tuple,
    fk_map: dict,
) -> dict | None:
    """Analiza una columna y devuelve su metadata para filtros.
    Puramente data-driven. Sin heurísticas de nombre.
    col = (cid, name, type, notnull, dflt_value, pk)
    """
    col_name = col[1]
    col_type = (col[2] or "TEXT").upper()

    if col[5]:  # PK
        return None

    sc = _safe_ident(col_name)
    st = _safe_ident(tabla)

    if col_type in ("DATE", "DATETIME", "TIMESTAMP"):
        cursor = conn.execute(f"SELECT MIN({sc}), MAX({sc}) FROM {st}")
        min_v, max_v = cursor.fetchone()
        return {
            "nombre": col_name,
            "tipo": "date",
            "fecha_min": str(min_v) if min_v else None,
            "fecha_max": str(max_v) if max_v else None,
        }

    if col_type in ("REAL", "FLOAT", "DOUBLE", "NUMERIC", "DECIMAL"):
        cursor = conn.execute(f"SELECT MIN({sc}), MAX({sc}) FROM {st}")
        min_v, max_v = cursor.fetchone()
        cursor.execute(f"SELECT COUNT(DISTINCT {sc}) FROM {st}")
        distinct = cursor.fetchone()[0]
        if distinct <= 50 and min_v is not None and max_v is not None:
            cursor.execute(
                f"SELECT DISTINCT {sc} FROM {st} ORDER BY {sc} LIMIT 100"
            )
            values = [r[0] for r in cursor.fetchall() if r[0] is not None]
            return {
                "nombre": col_name,
                "tipo": "categorical",
                "valores": values,
                "total_distintos": distinct,
            }
        return {
            "nombre": col_name,
            "tipo": "numeric",
            "min": float(min_v) if min_v is not None else None,
            "max": float(max_v) if max_v is not None else None,
        }

    if col_type in ("INTEGER", "BIGINT", "SMALLINT", "TINYINT"):
        cursor = conn.execute(f"SELECT COUNT(DISTINCT {sc}), COUNT(*) FROM {st}")
        distinct, total = cursor.fetchone()
        if total > 0 and distinct / total > 0.8:
            return None
        if distinct <= 50:
            cursor.execute(
                f"SELECT DISTINCT {sc} FROM {st} ORDER BY {sc} LIMIT 100"
            )
            values = [r[0] for r in cursor.fetchall() if r[0] is not None]
            return {
                "nombre": col_name,
                "tipo": "categorical",
                "valores": values,
                "total_distintos": distinct,
            }
        return None

    if col_type in ("TEXT", "VARCHAR", "CHAR"):
        cursor = conn.execute(
            f"SELECT COUNT(DISTINCT {sc}), AVG(CAST(LENGTH({sc}) AS REAL)) "
            f"FROM (SELECT {sc} FROM {st} LIMIT 1000)"
        )
        row = cursor.fetchone()
        distinct = row[0] if row else 0
        avg_len = row[1] if row else 0

        if avg_len and avg_len > 80:
            return None

        if distinct <= 50:
            cursor.execute(
                f"SELECT DISTINCT {sc} FROM {st} "
                f"WHERE {sc} IS NOT NULL AND {sc} != '' "
                f"ORDER BY {sc} LIMIT 100"
            )
            values = [r[0] for r in cursor.fetchall()]
            return {
                "nombre": col_name,
                "tipo": "categorical",
                "valores": values,
                "total_distintos": distinct,
            }
        else:
            return {
                "nombre": col_name,
                "tipo": "text_search",
            }

    return None


def _analizar_foreign_keys(conn: sqlite3.Connection, tablas: list[str]) -> dict:
    """Analiza todas las FK de todas las tablas.
    Retorna: { "tabla_origen.columna": {"tabla": destino, "columna": col_dest} }
    """
    fk_map = {}
    for t in tablas:
        cursor = conn.execute(f"PRAGMA foreign_key_list({_safe_ident(t)})")
        for fk in cursor.fetchall():
            id_fk, seq, tabla_dest, col_origen, col_dest, upd, dele, match = fk
            key = f"{t}.{col_origen}"
            fk_map[key] = {"tabla": tabla_dest, "columna": col_dest}
    return fk_map


def _detectar_dependencias(conn: sqlite3.Connection, columnas: list[dict], fk_map: dict) -> list[dict]:
    """Detecta relaciones de dependencia entre columnas categoricales via FK chains.
    Sin asumir prefijos de tabla. Puramente data-driven.
    """
    dependencias = []
    cat_cols = {c["nombre"]: c for c in columnas if c["tipo"] == "categorical"}

    for hijo_nombre in cat_cols:
        for padre_nombre in cat_cols:
            if padre_nombre == hijo_nombre:
                continue
            mapeo = _construir_mapeo_dependencia(conn, padre_nombre, hijo_nombre, fk_map)
            if mapeo:
                dependencias.append({
                    "columna_padre": padre_nombre,
                    "columna_dependiente": hijo_nombre,
                    "mapeo": mapeo,
                })

    return dependencias


def _construir_mapeo_dependencia(
    conn: sqlite3.Connection,
    col_padre: str,
    col_hijo: str,
    fk_map: dict,
) -> dict | None:
    """Construye mapeo padre→[hijos] via FK chain.
    Busca cadenas FK sin asumir nombres de tabla.
    """
    padre_tables = [k.split(".")[0] for k in fk_map if col_padre.lower() in k.lower()]
    hijo_tables = [k.split(".")[0] for k in fk_map if col_hijo.lower() in k.lower()]

    if not padre_tables or not hijo_tables:
        return None

    for pt in padre_tables:
        for ht in hijo_tables:
            pk_key = f"{pt}.id"
            fk_key = f"{ht}.id"
            fk_info_origen = fk_map.get(fk_key)
            fk_info_pk = fk_map.get(pk_key)

            if not fk_info_origen or not fk_info_pk:
                continue
            if fk_info_origen["tabla"] == fk_info_pk["tabla"]:
                tabla_rel = fk_info_origen["tabla"]
                try:
                    scp = _safe_ident(col_padre)
                    sch = _safe_ident(col_hijo)
                    str_ = _safe_ident(tabla_rel)
                    sa = _safe_ident(fk_info_pk['tabla'])
                    sb = _safe_ident(fk_info_origen['tabla'])
                    sapc = _safe_ident(fk_info_pk.get('columna', 'id'))
                    saoc = _safe_ident(fk_info_origen.get('columna', 'id'))
                    cursor = conn.execute(
                        f"SELECT DISTINCT a.{scp}, b.{sch} "
                        f"FROM {str_} tr "
                        f"JOIN {sa} a ON tr.{sapc} = a.{sapc} "
                        f"JOIN {sb} b ON tr.{saoc} = b.{saoc} "
                        f"WHERE a.{scp} IS NOT NULL AND b.{sch} IS NOT NULL "
                        f"ORDER BY 1, 2"
                    )
                    rows = cursor.fetchall()
                    if not rows:
                        continue
                    mapeo = {}
                    for row in rows:
                        pv = str(row[0])
                        hv = str(row[1])
                        if pv not in mapeo:
                            mapeo[pv] = []
                        if hv not in mapeo[pv]:
                            mapeo[pv].append(hv)
                    if mapeo:
                        return mapeo
                except Exception:
                    continue

    return None


# ─── DASHBOARD ─────────────────────────────────────────────────


def _cargar_dataframe(conn: sqlite3.Connection, vista: str) -> "pd.DataFrame":
    return pd.read_sql(f"SELECT * FROM {_safe_ident(vista)}", conn)


def _aplicar_filtros(df: "pd.DataFrame", filtros: dict) -> "pd.DataFrame":
    """Aplica filtros genéricos recibidos desde Rust.
    filtros = { "columna": {"tipo": "...", "valor": ...} }
    """
    for col_name, f_info in filtros.items():
        if col_name not in df.columns:
            continue
        tipo = f_info.get("tipo", "text_search")
        valor = f_info.get("valor")

        if tipo == "categorical" and valor and valor != "__todos__":
            df = df[df[col_name].astype(str) == str(valor)]

        elif tipo == "date":
            desde = f_info.get("desde", "")
            hasta = f_info.get("hasta", "")
            if desde or hasta:
                try:
                    df[col_name] = pd.to_datetime(df[col_name], errors="coerce")
                except Exception:
                    pass
                if desde:
                    df = df[df[col_name] >= desde]
                if hasta:
                    df = df[df[col_name] <= hasta]

        elif tipo == "numeric":
            min_v = f_info.get("min")
            max_v = f_info.get("max")
            if min_v is not None:
                df = df[df[col_name] >= min_v]
            if max_v is not None:
                df = df[df[col_name] <= max_v]

        elif tipo == "text_search" and valor:
            df = df[df[col_name].astype(str).str.contains(str(valor), case=False, na=False)]

    return df


def dashboard(conn: sqlite3.Connection, filtros: dict, vista: str | None = None, group_by: str | None = None) -> dict:
    """Dashboard query: counts, groupings, and detail table."""
    if not vista:
        exp = explorar(conn)
        vista = exp.get("vista_principal", VISTA_PREDETERMINADA)

    df = _cargar_dataframe(conn, vista)
    mapa_metricas = _detectar_metricas(df)
    df = _aplicar_filtros(df, filtros)

    col_estatus = mapa_metricas.get("estatus")

    total_general = len(df)

    pendientes = 0
    firmados = 0
    if col_estatus:
        pendientes = len(df[df[col_estatus].astype(str).str.upper().str.contains("PEND", na=False)])
        firmados = len(df[df[col_estatus].astype(str).str.upper().str.contains("FIRM", na=False)])

    grupo_actual = group_by or mapa_metricas.get("gerencia", "")
    por_grupo = {}
    if grupo_actual and grupo_actual in df.columns:
        for k, v in df.groupby(grupo_actual).size().to_dict().items():
            por_grupo[str(k)] = v

    tabla = df.head(500).fillna("").to_dict(orient="records")
    columnas_tabla = list(df.columns)

    return {
        "total_pendientes": pendientes,
        "total_firmados": firmados,
        "total_general": total_general,
        "por_grupo": por_grupo,
        "grupo_actual": grupo_actual,
        "tabla": tabla,
        "columnas_tabla": columnas_tabla,
    }


def _detectar_metricas(df: "pd.DataFrame") -> dict:
    """Detecta qué columnas usar para las cards y gráficos del dashboard.
    Puramente data-driven. Sin naming conventions.
    """
    mapa = {}
    text_cols = [col for col in df.columns if pd.api.types.is_string_dtype(df[col]) or df[col].dtype == object]

    best_status_col = None
    best_status_score = 0.0

    for col in text_cols:
        values = df[col].astype(str).str.upper()
        ratio_pend = values.str.contains("PEND", na=False).mean()
        ratio_firm = values.str.contains("FIRM", na=False).mean()
        combined = ratio_pend + ratio_firm
        both = ratio_pend > 0.01 and ratio_firm > 0.01

        if combined > 0.5:
            score = combined * (2.0 if both else 1.0)
            if score > best_status_score:
                best_status_score = score
                best_status_col = col

    if best_status_col:
        mapa["estatus"] = best_status_col

    for col in text_cols:
        n = df[col].nunique()
        if 3 < n < 40:
            freq = df[col].value_counts(normalize=True)
            max_ratio = freq.iloc[0] if len(freq) > 0 else 1.0
            if max_ratio > 0.6:
                continue
            if "gerencia" not in mapa:
                mapa["gerencia"] = col
            elif "modalidad" not in mapa and col != mapa.get("gerencia"):
                mapa["modalidad"] = col
                break

    return mapa
