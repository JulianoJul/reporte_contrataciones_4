#!/usr/bin/env python3
"""
Genera una base de datos SQLite de prueba.
Ejecutar: venv/bin/python generar_test_db.py
"""

import sqlite3
import random
from datetime import datetime, timedelta
from pathlib import Path

DB_PATH = Path(__file__).parent / "data" / "expedientes.db"
SCHEMA_PATH = Path(__file__).parent / "Tablas3.sql"
INSERTS_PATH = Path(__file__).parent / "Inserts2.sql"


def ejecutar_sql(conn, path):
    with open(path, encoding="utf-8") as f:
        sql = f.read()
    # sqlite3 executescript no soporta PRAGMA foreign_keys = ON dentro de transacción
    # Así que partimos por statements individuales
    for statement in sql.split(";"):
        stmt = statement.strip()
        if stmt:
            try:
                conn.execute(stmt)
            except sqlite3.Error as e:
                print(f"  ⚠ {e}")


def generar_expedientes(conn):
    cursor = conn.cursor()

    # Obtener IDs de catálogos
    gerencias = [r[0] for r in cursor.execute("SELECT id FROM cat_gerencia").fetchall()]
    supers = [r[0] for r in cursor.execute("SELECT id FROM cat_superintendencia").fetchall()]
    docs = [r[0] for r in cursor.execute("SELECT id FROM cat_documento").fetchall()]
    planes = [r[0] for r in cursor.execute("SELECT id FROM cat_plan_contratacion").fetchall()]
    mods = [r[0] for r in cursor.execute("SELECT id FROM cat_modalidad").fetchall()]
    arts = [r[0] for r in cursor.execute("SELECT id FROM cat_art").fetchall()]
    tcs = [r[0] for r in cursor.execute("SELECT id FROM cat_tipo_contrato").fetchall()]
    estatus_list = [r[0] for r in cursor.execute("SELECT id FROM cat_estatus_detalle").fetchall()]
    resultados = [r[0] for r in cursor.execute("SELECT id FROM cat_resultado_proceso").fetchall()]
    empresas = [r[0] for r in cursor.execute("SELECT id FROM cat_empresas").fetchall()]
    acciones = [r[0] for r in cursor.execute("SELECT id FROM cat_estado_accion").fetchall()]
    responsables = [r[0] for r in cursor.execute("SELECT id FROM cat_responsables").fetchall()]

    if not responsables:
        print("  ⚠ No hay responsables, insertando...")
        cursor.execute("INSERT INTO cat_responsables (nombre) VALUES ('EMISOR/RECEPTOR POR CONFIGURAR')")
        responsables = [cursor.lastrowid]

    # Relación superintendencia → gerencia
    sup_ger_map = {}
    for s in supers:
        row = cursor.execute("SELECT id_gerencia FROM cat_superintendencia WHERE id=?", (s,)).fetchone()
        if row:
            sup_ger_map[s] = row[0]

    now = datetime.now()

    # ── Generar 50 expedientes ──
    for i in range(1, 51):
        id_ger = random.choice(gerencias)
        # superintendencia que pertenezca a esa gerencia
        posibles_sup = [s for s, g in sup_ger_map.items() if g == id_ger]
        id_sup = random.choice(posibles_sup) if posibles_sup else random.choice(supers)
        id_est = random.choice(estatus_list)
        id_doc = random.choice(docs)
        id_plan = random.choice(planes)
        id_mod = random.choice(mods)
        id_art = random.choice(arts)
        id_tc = random.choice(tcs)
        id_res = random.choice(resultados)
        id_emp = random.choice(empresas)
        id_emisor = random.choice(responsables)
        id_receptor = random.choice(responsables)
        id_accion = random.choice(acciones)

        fecha_presup = now - timedelta(days=random.randint(30, 365))
        presup_base_usd = round(random.uniform(10000, 500000), 2)
        tc = round(random.uniform(40, 60), 2)
        presup_base_bs = round(presup_base_usd * tc, 2)

        monto_bs = round(presup_base_bs * random.uniform(0.8, 1.2), 2)
        monto_usd = round(monto_bs / tc, 2)

        solped = f"SOLPED-2025-{i:04d}"
        nro_proceso = f"PROC-{random.randint(2025, 2026)}-{random.randint(100, 999)}"
        nro_contrato = f"SICAC-{random.randint(10000, 99999)}"
        descripcion = random.choice([
            "Servicio de mantenimiento de equipos de perforación",
            "Suministro de materiales para construcción de pozos",
            "Servicio de transporte de personal",
            "Consultoría para optimización de producción",
            "Mantenimiento de infraestructura de oficinas",
            "Suministro de tuberías y accesorios",
            "Servicio de alimentación para personal",
            "Alquiler de equipos de izaje",
            "Servicio de vigilancia y seguridad",
            "Mantenimiento de plantas eléctricas",
        ])

        fecha_recibido = fecha_presup + timedelta(days=random.randint(5, 30))
        fecha_devuelto = fecha_recibido + timedelta(days=random.randint(3, 15)) if random.random() < 0.3 else None
        fecha_firma = None
        if id_est == 2:  # FIRMADO
            fecha_firma = fecha_recibido + timedelta(days=random.randint(10, 60))
        elif id_est == 1 and random.random() < 0.2:  # PENDIENTE con firma
            fecha_firma = now + timedelta(days=random.randint(5, 30))

        cursor.execute("""
            INSERT INTO expedientes (
                solped, id_gerencia, id_superintendencia, id_emisor,
                id_documento, fecha_presupuesto_base, presupuesto_base_usd,
                tipo_cambio, presupuesto_base_bs, id_plan, descripcion_proceso,
                id_modalidad, id_art, id_tipo_contrato, nro_acta_apertura,
                cantidad_frentes, nro_resolucion_jd, id_estatus, id_estado_accion,
                fecha_recibido, fecha_devuelto, id_receptor, nro_proceso,
                id_resultado, nro_contrato_sicac, nro_contrato_sap,
                id_empresa, tiempo_ejecucion, monto_adjudicado_bs,
                monto_adjudicado_usd, fecha_firma_contrato, observaciones_generales
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            solped, id_ger, id_sup, id_emisor,
            id_doc, fecha_presup.strftime("%Y-%m-%d"), presup_base_usd,
            tc, presup_base_bs, id_plan, descripcion,
            id_mod, id_art, id_tc, f"ACTA-{random.randint(1000,9999)}",
            random.randint(1, 5), f"RES-JD-{random.randint(100,999)}", id_est, id_accion,
            fecha_recibido.strftime("%Y-%m-%d"),
            fecha_devuelto.strftime("%Y-%m-%d") if fecha_devuelto else None,
            id_receptor, nro_proceso,
            id_res, nro_contrato, random.randint(100000, 999999),
            id_emp, f"{random.randint(3, 18)} meses", monto_bs,
            monto_usd, fecha_firma.strftime("%Y-%m-%d") if fecha_firma else None,
            random.choice(["", "Sin observaciones", "Pendiente de documento legal", "Requiere revisión jurídica"]),
        ))

    conn.commit()
    total = cursor.execute("SELECT COUNT(*) FROM expedientes").fetchone()[0]
    print(f"  ✅ {total} expedientes insertados")


def main():
    if DB_PATH.exists():
        print(f"🗑  Eliminando BD existente: {DB_PATH}")
        DB_PATH.unlink()

    print(f"🔨 Creando BD: {DB_PATH}")
    conn = sqlite3.connect(str(DB_PATH))
    conn.execute("PRAGMA foreign_keys = OFF")

    print("  📦 Creando schema desde Tablas3.sql...")
    ejecutar_sql(conn, SCHEMA_PATH)

    print("  📦 Cargando catálogos desde Inserts2.sql...")
    ejecutar_sql(conn, INSERTS_PATH)

    print("  📦 Insertando más responsables...")
    conn.execute("INSERT INTO cat_responsables (nombre) VALUES ('MARÍA GARCÍA')")
    conn.execute("INSERT INTO cat_responsables (nombre) VALUES ('PEDRO RODRÍGUEZ')")
    conn.execute("INSERT INTO cat_responsables (nombre) VALUES ('ANA MARTÍNEZ')")
    conn.execute("INSERT INTO cat_responsables (nombre) VALUES ('CARLOS LÓPEZ')")
    conn.execute("INSERT INTO cat_responsables (nombre) VALUES ('LUISA FERNÁNDEZ')")
    conn.execute("INSERT INTO cat_responsables (nombre) VALUES ('JOSÉ RAMÍREZ')")
    conn.execute("INSERT INTO cat_responsables (nombre) VALUES ('ELENA DÍAZ')")

    print("  📦 Generando expedientes de prueba...")
    generar_expedientes(conn)

    # Verificar vista
    cursor = conn.cursor()
    try:
        cursor.execute("SELECT COUNT(*) FROM vw_reporte_excel_contrataciones")
        total_vw = cursor.fetchone()[0]
        print(f"  ✅ Vista vw_reporte_excel_contrataciones: {total_vw} registros")
    except sqlite3.Error as e:
        print(f"  ⚠ Vista no disponible: {e}")

    conn.close()
    print(f"\n✅ BD lista: {DB_PATH}")
    print(f"   Tamaño: {DB_PATH.stat().st_size / 1024:.1f} KB")


if __name__ == "__main__":
    main()
