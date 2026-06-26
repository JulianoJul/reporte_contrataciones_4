#!/usr/bin/env python3
"""
data_service.py - Pipe-based data service for Reporte de Contrataciones.
Protocol: JSON lines via stdin/stdout.
Keeps SQLite connection open for the lifetime of the process.
"""

import json
import sqlite3
import sys
import traceback
from pathlib import Path

# Redirect stderr to stdout so Rust can see startup errors
sys.stderr = sys.stdout

sys.path.insert(0, str(Path(__file__).parent))
try:
    import queries
    import exporters
except Exception:
    sys.stdout.write(json.dumps({
        "id": 0, "ok": False,
        "error": f"Import error:\n{traceback.format_exc()}"
    }, ensure_ascii=False) + "\n")
    sys.stdout.flush()
    sys.exit(1)

_db_conn: sqlite3.Connection | None = None
_db_path: str | None = None


def get_connection(db_path: str) -> sqlite3.Connection:
    global _db_conn, _db_path
    if _db_conn is None or _db_path != db_path:
        if _db_conn is not None:
            try:
                _db_conn.close()
            except Exception:
                pass
        _db_conn = sqlite3.connect(db_path)
        _db_path = db_path
    return _db_conn


def procesar(request: dict) -> dict:
    request_id = request.get("id", 0)
    tipo = request.get("tipo", "")
    db_path = request.get("db", "")
    filtros = request.get("filtros", {})
    output = request.get("output")
    vista = request.get("vista")
    group_by = request.get("group_by")

    if tipo == "quit":
        return {"id": request_id, "ok": True}

    if not db_path:
        return {"id": request_id, "ok": False, "error": "No se especificó ruta de BD"}
    if not Path(db_path).exists():
        return {"id": request_id, "ok": False, "error": f"Archivo no encontrado: {db_path}"}

    try:
        conn = get_connection(db_path)

        if tipo == "explorar":
            data = queries.explorar(conn)
            if "error" in data:
                return {"id": request_id, "ok": False, "error": data["error"]}
            return {"id": request_id, "ok": True, "data": data}

        elif tipo == "dashboard":
            data = queries.dashboard(conn, filtros, vista=vista, group_by=group_by)
            return {"id": request_id, "ok": True, "data": data}

        elif tipo == "exportar_excel":
            if not output:
                return {"id": request_id, "ok": False, "error": "No se especificó ruta de salida"}
            resultado = exporters.excel(conn, filtros, output, vista=vista)
            return {"id": request_id, "ok": True, "data": resultado}

        elif tipo == "exportar_pdf":
            if not output:
                return {"id": request_id, "ok": False, "error": "No se especificó ruta de salida"}
            resultado = exporters.pdf(conn, filtros, output, vista=vista)
            return {"id": request_id, "ok": True, "data": resultado}

        elif tipo == "exportar_pptx":
            if not output:
                return {"id": request_id, "ok": False, "error": "No se especificó ruta de salida"}
            resultado = exporters.pptx(conn, filtros, output, vista=vista)
            return {"id": request_id, "ok": True, "data": resultado}

        else:
            return {"id": request_id, "ok": False, "error": f"Tipo desconocido: {tipo}"}

    except Exception as e:
        return {"id": request_id, "ok": False, "error": str(e)}


def main():
    sys.stdin.reconfigure(encoding="utf-8")
    sys.stdout.reconfigure(encoding="utf-8")

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue

        try:
            request = json.loads(line)
        except json.JSONDecodeError as e:
            response = {"id": 0, "ok": False, "error": f"JSON inválido: {e}"}
            sys.stdout.write(json.dumps(response, ensure_ascii=False) + "\n")
            sys.stdout.flush()
            continue

        if request.get("tipo") == "quit":
            break

        response = procesar(request)
        sys.stdout.write(json.dumps(response, ensure_ascii=False) + "\n")
        sys.stdout.flush()

    global _db_conn
    if _db_conn is not None:
        try:
            _db_conn.close()
        except Exception:
            pass


if __name__ == "__main__":
    main()
