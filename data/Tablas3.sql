PRAGMA foreign_keys = ON;

-- ==========================================
-- 🔹 1. CATÁLOGOS MAESTROS (Independientes)
-- ==========================================
CREATE TABLE cat_gerencia (id INTEGER PRIMARY KEY, nombre TEXT UNIQUE);
CREATE TABLE cat_documento (id INTEGER PRIMARY KEY, nombre TEXT UNIQUE);
CREATE TABLE cat_plan_contratacion (id INTEGER PRIMARY KEY, nombre TEXT UNIQUE);
CREATE TABLE cat_modalidad (id INTEGER PRIMARY KEY, nombre TEXT UNIQUE);
CREATE TABLE cat_art (id INTEGER PRIMARY KEY, nombre TEXT UNIQUE);
CREATE TABLE cat_tipo_contrato (id INTEGER PRIMARY KEY, nombre TEXT UNIQUE);
CREATE TABLE cat_estatus_detalle (id INTEGER PRIMARY KEY, nombre TEXT UNIQUE);
CREATE TABLE cat_resultado_proceso (id INTEGER PRIMARY KEY, nombre TEXT UNIQUE);
CREATE TABLE cat_empresas (id INTEGER PRIMARY KEY, nombre TEXT UNIQUE);
CREATE TABLE cat_responsables (id INTEGER PRIMARY KEY, nombre TEXT UNIQUE);
CREATE TABLE cat_estado_accion (id INTEGER PRIMARY KEY, nombre TEXT UNIQUE);

-- ==========================================
-- 🔹 2. CATÁLOGOS CON RELACIONES
-- ==========================================
CREATE TABLE cat_superintendencia (
    id INTEGER PRIMARY KEY,
    nombre TEXT UNIQUE,
    id_gerencia INTEGER,
    CONSTRAINT fk_sup_ger FOREIGN KEY (id_gerencia) REFERENCES cat_gerencia(id)
);

-- ==========================================
-- 🔹 3. TABLA PRINCIPAL: EXPEDIENTES
-- ==========================================
CREATE TABLE expedientes (
    id_expediente           INTEGER PRIMARY KEY AUTOINCREMENT,
    solped                  TEXT UNIQUE,
    id_gerencia             INTEGER,
    id_superintendencia     INTEGER,
    id_emisor               INTEGER,
    id_documento            INTEGER,
    fecha_presupuesto_base  DATE,
    presupuesto_base_usd    REAL,
    tipo_cambio             REAL,
    presupuesto_base_bs     REAL,
    id_plan                 INTEGER,
    descripcion_proceso     TEXT,
    id_modalidad            INTEGER,
    id_art                  INTEGER,
    id_tipo_contrato        INTEGER,
    nro_acta_apertura       TEXT,
    cantidad_frentes        INTEGER,
    nro_resolucion_jd       TEXT,
    id_estatus              INTEGER DEFAULT 1,
    id_estado_accion        INTEGER,
    fecha_recibido          DATE,
    fecha_devuelto          DATE,
    id_receptor             INTEGER,
    nro_proceso             TEXT,
    id_resultado            INTEGER,
    nro_contrato_sicac      TEXT,
    nro_contrato_sap        INTEGER,
    id_empresa              INTEGER,
    tiempo_ejecucion        TEXT,
    monto_adjudicado_bs     REAL,
    monto_adjudicado_usd    REAL,
    fecha_firma_contrato    DATE,
    observaciones_generales TEXT,
    fecha_creacion          DATE DEFAULT CURRENT_DATE,
    fecha_actualizacion     DATE DEFAULT CURRENT_DATE,

    CONSTRAINT fk_exp_ger      FOREIGN KEY (id_gerencia)         REFERENCES cat_gerencia(id),
    CONSTRAINT fk_exp_sup      FOREIGN KEY (id_superintendencia) REFERENCES cat_superintendencia(id),
    CONSTRAINT fk_exp_emisor   FOREIGN KEY (id_emisor)           REFERENCES cat_responsables(id),
    CONSTRAINT fk_exp_receptor FOREIGN KEY (id_receptor)         REFERENCES cat_responsables(id),
    CONSTRAINT fk_exp_doc      FOREIGN KEY (id_documento)        REFERENCES cat_documento(id),
    CONSTRAINT fk_exp_plan     FOREIGN KEY (id_plan)             REFERENCES cat_plan_contratacion(id),
    CONSTRAINT fk_exp_mod      FOREIGN KEY (id_modalidad)        REFERENCES cat_modalidad(id),
    CONSTRAINT fk_exp_art      FOREIGN KEY (id_art)              REFERENCES cat_art(id),
    CONSTRAINT fk_exp_tc       FOREIGN KEY (id_tipo_contrato)    REFERENCES cat_tipo_contrato(id),
    CONSTRAINT fk_exp_est      FOREIGN KEY (id_estatus)          REFERENCES cat_estatus_detalle(id),
    CONSTRAINT fk_exp_res      FOREIGN KEY (id_resultado)        REFERENCES cat_resultado_proceso(id),
    CONSTRAINT fk_exp_emp      FOREIGN KEY (id_empresa)          REFERENCES cat_empresas(id),
    CONSTRAINT fk_exp_accion   FOREIGN KEY (id_estado_accion)    REFERENCES cat_estado_accion(id)
);

-- ==========================================
-- 🔹 4. AUDIT LOG (Snapshot - NO normalizada)
-- ==========================================
CREATE TABLE historial_expedientes (
    id_historial            INTEGER PRIMARY KEY AUTOINCREMENT,
    id_expediente           INTEGER NOT NULL,
    fecha_snapshot          DATETIME DEFAULT CURRENT_TIMESTAMP,

    -- 📸 Copia exacta de los campos de datos
    solped                  TEXT,
    id_gerencia             INTEGER,
    id_superintendencia     INTEGER,
    id_emisor               INTEGER,
    id_documento            INTEGER,
    fecha_presupuesto_base  DATE,
    presupuesto_base_usd    REAL,
    tipo_cambio             REAL,
    presupuesto_base_bs     REAL,
    id_plan                 INTEGER,
    descripcion_proceso     TEXT,
    id_modalidad            INTEGER,
    id_art                  INTEGER,
    id_tipo_contrato        INTEGER,
    nro_acta_apertura       TEXT,
    cantidad_frentes        INTEGER,
    nro_resolucion_jd       TEXT,
    id_estatus              INTEGER,
    id_estado_accion        INTEGER,
    fecha_recibido          DATE,
    fecha_devuelto          DATE,
    id_receptor             INTEGER,
    nro_proceso             TEXT,
    id_resultado            INTEGER,
    nro_contrato_sicac      TEXT,
    nro_contrato_sap        INTEGER,
    id_empresa              INTEGER,
    tiempo_ejecucion        TEXT,
    monto_adjudicado_bs     REAL,
    monto_adjudicado_usd    REAL,
    fecha_firma_contrato    DATE,
    observaciones_generales TEXT
);

-- ==========================================
-- 🔹 5. ÍNDICES PARA RENDIMIENTO
-- ==========================================
CREATE INDEX idx_exp_solped          ON expedientes(solped);
CREATE INDEX idx_exp_gerencia        ON expedientes(id_gerencia);
CREATE INDEX idx_exp_estatus         ON expedientes(id_estatus);
CREATE INDEX idx_exp_empresa         ON expedientes(id_empresa);
CREATE INDEX idx_exp_fecha_presup    ON expedientes(fecha_presupuesto_base);
CREATE INDEX idx_exp_fecha_creacion  ON expedientes(fecha_creacion);
CREATE INDEX idx_hist_expediente     ON historial_expedientes(id_expediente);
CREATE INDEX idx_hist_fecha          ON historial_expedientes(fecha_snapshot);

-- ==========================================
-- 🔹 6. TRIGGER DE AUDITORÍA Y LÓGICA DE FIRMA
-- ==========================================
CREATE TRIGGER trg_exp_auditoria AFTER UPDATE ON expedientes
FOR EACH ROW
BEGIN
    -- 1️⃣ Guardar snapshot del estado ANTERIOR (OLD)
    INSERT INTO historial_expedientes (
        id_expediente, solped, id_gerencia, id_superintendencia,
        id_emisor, id_documento, fecha_presupuesto_base,
        presupuesto_base_usd, tipo_cambio, presupuesto_base_bs,
        id_plan, descripcion_proceso, id_modalidad, id_art,
        id_tipo_contrato, nro_acta_apertura, cantidad_frentes,
        nro_resolucion_jd, id_estatus, id_estado_accion,
        fecha_recibido, fecha_devuelto, id_receptor, nro_proceso,
        id_resultado, nro_contrato_sicac, nro_contrato_sap,
        id_empresa, tiempo_ejecucion, monto_adjudicado_bs,
        monto_adjudicado_usd, fecha_firma_contrato, observaciones_generales
    ) VALUES (
        OLD.id_expediente, OLD.solped, OLD.id_gerencia, OLD.id_superintendencia,
        OLD.id_emisor, OLD.id_documento, OLD.fecha_presupuesto_base,
        OLD.presupuesto_base_usd, OLD.tipo_cambio, OLD.presupuesto_base_bs,
        OLD.id_plan, OLD.descripcion_proceso, OLD.id_modalidad, OLD.id_art,
        OLD.id_tipo_contrato, OLD.nro_acta_apertura, OLD.cantidad_frentes,
        OLD.nro_resolucion_jd, OLD.id_estatus, OLD.id_estado_accion,
        OLD.fecha_recibido, OLD.fecha_devuelto, OLD.id_receptor, OLD.nro_proceso,
        OLD.id_resultado, OLD.nro_contrato_sicac, OLD.nro_contrato_sap,
        OLD.id_empresa, OLD.tiempo_ejecucion, OLD.monto_adjudicado_bs,
        OLD.monto_adjudicado_usd, OLD.fecha_firma_contrato, OLD.observaciones_generales
    );

    -- 2️⃣ Si se QUITÓ la firma -> regresar estatus a PENDIENTE
    UPDATE expedientes
    SET id_estatus = (SELECT id FROM cat_estatus_detalle WHERE nombre = 'PENDIENTE' LIMIT 1)
    WHERE NEW.fecha_firma_contrato IS NULL
      AND OLD.fecha_firma_contrato IS NOT NULL
      AND id_expediente = NEW.id_expediente;

    -- 3️⃣ Actualizar fecha de modificación
    UPDATE expedientes
    SET fecha_actualizacion = CURRENT_DATE
    WHERE id_expediente = NEW.id_expediente;
END;

-- ==========================================
-- 🔹 7. VISTA PARA EXPORTAR A EXCEL
-- ==========================================
CREATE VIEW vw_reporte_excel_contrataciones AS
SELECT 
    e.id_expediente,
    COALESCE(e.solped, 'SIN_SOLPED')            AS solped,
    g.nombre                                     AS gerencia,
    s.nombre                                     AS superintendencia,
    emisor.nombre                                AS emisor,
    d.nombre                                     AS documento,
    e.fecha_presupuesto_base,
    e.presupuesto_base_usd,
    e.tipo_cambio,
    e.presupuesto_base_bs,
    p.nombre                                     AS plan_contrataciones,
    e.descripcion_proceso,
    m.nombre                                     AS modalidad_contratacion,
    a.nombre                                     AS art,
    tc.nombre                                    AS tipo_contrato,
    COALESCE(e.nro_acta_apertura, 'NO POSEE')    AS nro_acta_apertura,
    e.cantidad_frentes,
    COALESCE(e.nro_resolucion_jd, 'NO APLICA')   AS nro_resolucion_jd,
    COALESCE(ed.nombre, 'NO APLICA')             AS estatus_detalle,
    ea.nombre                                    AS estado_accion,
    e.fecha_recibido,
    e.fecha_devuelto,
    COALESCE(receptor.nombre, 'NO APLICA')       AS receptor,
    COALESCE(e.nro_proceso, 'NO APLICA')         AS nro_proceso,
    COALESCE(rp.nombre, 'NO APLICA')             AS resultados_proceso,
    COALESCE(e.nro_contrato_sicac, 'NO POSEE')   AS nro_contrato_sicac,
    e.nro_contrato_sap,
    COALESCE(emp.nombre, 'NO APLICA')            AS empresa_adjudicada,
    e.tiempo_ejecucion,
    e.monto_adjudicado_bs,
    e.monto_adjudicado_usd,
    COALESCE(e.fecha_firma_contrato, 'NO APLICA') AS fecha_firma_contrato,
    e.observaciones_generales,
    e.fecha_creacion,
    e.fecha_actualizacion
FROM expedientes e
LEFT JOIN cat_gerencia g          ON e.id_gerencia         = g.id
LEFT JOIN cat_superintendencia s  ON e.id_superintendencia = s.id
LEFT JOIN cat_documento d         ON e.id_documento        = d.id
LEFT JOIN cat_plan_contratacion p ON e.id_plan             = p.id
LEFT JOIN cat_modalidad m         ON e.id_modalidad        = m.id
LEFT JOIN cat_art a               ON e.id_art              = a.id
LEFT JOIN cat_tipo_contrato tc    ON e.id_tipo_contrato    = tc.id
LEFT JOIN cat_estatus_detalle ed  ON e.id_estatus          = ed.id
LEFT JOIN cat_estado_accion ea    ON e.id_estado_accion    = ea.id
LEFT JOIN cat_resultado_proceso rp ON e.id_resultado       = rp.id
LEFT JOIN cat_empresas emp        ON e.id_empresa          = emp.id
LEFT JOIN cat_responsables emisor ON e.id_emisor           = emisor.id
LEFT JOIN cat_responsables receptor ON e.id_receptor       = receptor.id;