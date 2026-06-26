-- ==========================================
-- 🔹 CONFIGURACIÓN INICIAL & LIMPIEZA TOTAL
-- ==========================================
PRAGMA foreign_keys = OFF;

DELETE FROM historial_expedientes;
DELETE FROM expedientes;
DELETE FROM cat_superintendencia;
DELETE FROM cat_gerencia;
DELETE FROM cat_documento;
DELETE FROM cat_plan_contratacion;
DELETE FROM cat_modalidad;
DELETE FROM cat_art;
DELETE FROM cat_tipo_contrato;
DELETE FROM cat_estatus_detalle;
DELETE FROM cat_resultado_proceso;
DELETE FROM cat_empresas;
DELETE FROM cat_responsables;
DELETE FROM cat_estado_accion;

PRAGMA foreign_keys = ON;

-- ==========================================
-- 🔹 1. GERENCIAS
-- ==========================================
INSERT INTO cat_gerencia (id, nombre) VALUES
(1, 'SIHO-A'), (2, 'TÉCNICA'), (3, 'OPERACIONES'), (4, 'SSGG'), (5, 'JURÍDICO'),
(6, 'FINANZAS'), (7, 'CONTRATACIÓN'), (8, 'RRHH'), (9, 'ASUNTOS GUBERNAMENTALES'), (10, 'COMISIÓN');

-- ==========================================
-- 🔹 2. SUPERINTENDENCIAS
-- ==========================================
INSERT INTO cat_superintendencia (id, nombre, id_gerencia) VALUES
(1, 'SIHO-A', 1),
(2, 'INFRAESTRUCTURA', 2), (3, 'PERFORACIÓN', 2), (4, 'YACIMIENTOS', 2), (5, 'OPTIMIZACIÓN', 2),
(6, 'OPERACIÓN DE PRODUCCIÓN', 3), (7, 'MANTENIMIENTO', 3),
(8, 'SSGG', 4), (9, 'JURÍDICO', 5), (10, 'FINANZAS', 6),
(11, 'CONTRATACIÓN', 7), (12, 'RRHH', 8),
(13, 'ASUNTOS GUBERNAMENTALES', 9), (14, 'COMISIÓN', 10);

-- ==========================================
-- 🔹 3. DOCUMENTOS (Manteniendo marcas (A), sin ruido/duplicados)
-- ==========================================
INSERT INTO cat_documento (id, nombre) VALUES
(1, 'ANÁLISIS ECONÓMICO / CONTRATO'),
(2, 'CONTRATO'),
(3, 'JUSTIFICACIÓN, MODIFICACIÓN Y ACTA DE OTRAS CONSIDERACIONES (A)'),
(4, 'DESCRIPCIÓN DE PROCESO Y ESPECIFICACIONES TÉCNICAS'),
(5, 'ACTO MOTIVADO Y ACTA DE OTRAS CONSIDERACIONES (A)'),
(6, 'ACTA DE OTRAS CONSIDERACIONES (A)'),
(7, 'ADDENDUM / DECISIÓN DE GERENCIA APROBACIÓN DE LA MODIFICACIÓN'),
(8, 'ACTUALIZACIÓN DE PRESUPUESTO BASE'),
(9, 'PRESUPUESTO BASE / ESPECIFICACIONES TÉCNICAS / ACTUALIZACIÓN DE PRESUPUESTO BASE'),
(10, 'DESCRIPCIÓN DE PROCESO, ESPECIFICACIONES TÉCNICAS Y JUSTIFICACIÓN'),
(11, 'SOLPED / PRESUPUESTO BASE / DESCRIPCIÓN DEL PROCESO / JUSTIFICACIÓN / INFORME TÉCNICO DE PRECALIFICACIÓN / ESPECIFICACIONES TÉCNICAS'),
(12, 'DECISIÓN DE GERENCIA INICIO'),
(13, 'ESPECIFICACIONES TÉCNICAS Y DESCRIPCIÓN DEL PROCESO'),
(14, 'PRESUPUESTO BASE / ESPECIFICACIONES TÉCNICAS Y DESCRIPCIÓN DEL PROCESO'),
(15, 'ACTA DE INICIO SOLICITUD (A)'),
(16, 'ACTA DE MODIFICACIÓN DEL CONTRATO (A)'),
(17, 'DECISIÓN DE GERENCIA MODIFICACIÓN / ADDENDUM'),
(18, 'ANÁLISIS ECONÓMICO / ACTA DE OTORGAMIENTO / CONTRATO'),
(19, 'ACTA DE OTRAS CONSIDERACIONES (A) / ACTA DE OTORGAMIENTO / NOTIFICACIÓN DE ADJUDICACIÓN'),
(20, 'ACTO MOTIVADO'),
(21, 'ACTA DE OTRAS CONSIDERACIONES / ANÁLISIS ECONÓMICO'),
(22, 'ACTA DE OTORGAMIENTO / NOTIFICACIÓN DE ADJUDICACIÓN'),
(23, 'ACTA DE OTRAS CONSIDERACIONES (A) / ACTO MOTIVADO / ACTA DE OTORGAMIENTO / NOTIFICACIÓN'),
(24, 'ANÁLISIS ECONÓMICO REV.1'),
(25, 'CONTRATO DE SERVICIOS'),
(26, 'ACTA DE RESULTADOS DE CALIFICACIÓN Y EVALUACIÓN (A)'),
(27, 'DECISIÓN DE GERENCIA'),
(28, 'ANÁLISIS ECONÓMICO / ACTA DE RESULTADOS DE CALIFICACIÓN Y EVALUACIÓN (A)');

-- ==========================================
-- 🔹 4. PLANES DE CONTRATACIÓN
-- ==========================================
INSERT INTO cat_plan_contratacion (id, nombre) VALUES
(1, 'ARRASTRE 2025'), (2, 'PLAN 2026'), (3, 'ADICIONAL (DIRECTOS) 2026'), (4, 'PLAN-ADICIONAL 2026');

-- ==========================================
-- 🔹 5. MODALIDADES
-- ==========================================
INSERT INTO cat_modalidad (id, nombre) VALUES
(1, 'CONCURSO ABIERTO'), (2, 'CONCURSO CERRADO'), (3, 'CONSULTA DE PRECIOS'), (4, 'CONTRATACIÓN DIRECTA');

-- ==========================================
-- 🔹 6. ARTÍCULOS NORMATIVA INTERNA
-- ==========================================
INSERT INTO cat_art (id, nombre) VALUES
(1, '5 N - 08'), (2, '77 N - 01'), (3, '77 N - 02'), (4, '77 N - 03'),
(5, '101 N - 01'), (6, '101 N - 02'), (7, '101 N - 03'), (8, '101 N - 04'), (9, '5 N - 06');

-- ==========================================
-- 🔹 7. TIPO DE CONTRATO
-- ==========================================
INSERT INTO cat_tipo_contrato (id, nombre) VALUES
(1, 'PU'), (2, 'SG'), (3, 'MIXTO');

-- ==========================================
-- 🔹 8. ESTATUS DETALLE
-- ==========================================
INSERT INTO cat_estatus_detalle (id, nombre) VALUES
(1, 'PENDIENTE'), (2, 'FIRMADO'), (3, 'DEVUELTO PARA CORRECCIÓN'), (4, 'DEVUELTO SIN FIRMA');

-- ==========================================
-- 🔹 9. RESULTADOS DEL PROCESO
-- ==========================================
INSERT INTO cat_resultado_proceso (id, nombre) VALUES
(1, 'ADJUDICADO'),
(2, 'DESIERTO 113 # 1'), (3, 'DESIERTO 113 # 2'), (4, 'DESIERTO 113 # 3'),
(5, 'DESIERTO 113 # 4'), (6, 'DESIERTO 113 # 5'), (7, 'DAR POR TERMINADO');

-- ==========================================
-- 🔹 10. EMPRESAS ADJUDICADAS (Placeholder eliminado, IDs renumerados 1-13)
-- ==========================================
INSERT INTO cat_empresas (id, nombre) VALUES
(1, 'PRODUCTORA Y DISTRIBUIDORA VENEZOLANA DE ALIMENTOS, S.A (PDVAL)'),
(2, 'TRANSPORTE ROJAS GARCÍA,C.A.'),
(3, 'CRANE & HEAVY SERVICE DE VENEZUELA'),
(4, 'AGROPECUARIA LA ROSALIERA'),
(5, 'SERVICIOS Y SUMINISTROS KAMULY K&M C.A'),
(6, 'IMSUPETROL, C.A'),
(7, 'CORPORACIÓN SAN REMO, C.A'),
(8, 'INVERSIONES ROYPA, S.A'),
(9, 'CONCRELAND, C.A'),
(10, 'SERVICIOS Y SUMINISTROS DAVNA, C.A.'),
(11, 'METALMECANICA CONTRERAS, C.A'),
(12, 'SERVICIOS Y TRANSPORTE LOS 2 HERMANOS, C.A'),
(13, 'POWERLINE CONSTRUCCIONES, C.A');

-- ==========================================
-- 🔹 11. ESTADO ACCIÓN (Valores iniciales)
-- ==========================================
INSERT INTO cat_estado_accion (id, nombre) VALUES
(1, 'SE ENTREGA CON LA FIRMA'),
(2, 'SE ENTREGA CON LA MODIFICACIÓN'),
(3, 'SE RECIBE PARA LA FIRMA');

-- ==========================================
-- 🔹 12. RESPONSABLES (Placeholder temporal)
-- ==========================================
INSERT INTO cat_responsables (id, nombre) VALUES
(1, 'EMISOR/RECEPTOR POR CONFIGURAR');