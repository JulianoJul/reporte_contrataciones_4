# Reporte de Contrataciones — Documentación

## Arquitectura

App 100% Rust. **egui (immediate-mode) = UI** | **rusqlite = Data Layer**.
Sin Python, sin bridge, sin runtime externo. Compila a un solo binario.

```
┌───────────────────────────────────┐
│  Rust (egui + eframe)             │
│  ├── src/app.rs  — estado + UI    │
│  ├── src/db/     — layer universal │
│  │   (sin modificar)              │
│  ├── src/export  — Excel/PDF      │
│  └── src/redactor — plantillas    │
│                                   │
│  SQLite ←────────────────────────┤
│  [cualquier .db / .sqlite]        │
└───────────────────────────────────┘
```

## Principio Fundamental

**Cero hardcodeo. Cero naming conventions. Cero assumptions del schema.**

Todo se genera dinámicamente analizando la BD al abrirla:
- Tablas/vistas disponibles → selector en UI
- Columnas → detección de tipo (categorical, categorical_fk, date, numeric, text_search)
- Filtros → widgets generados según tipo (ComboBox, sliders, búsqueda)
- La tabla muestra cualquier columna, cualquier schema

## Algoritmo Universal de Detección de Columnas

```
Para cada columna en la tabla/vista seleccionada:
┌───────────────────────────────────────────────────────────────┐
│  CONDICIÓN                          → TIPO FILTRO             │
├───────────────────────────────────────────────────────────────┤
│  TYPE = DATE/DATETIME/TIMESTAMP     → DateRange               │
│  TYPE = REAL/FLOAT/DOUBLE/NUMERIC   → Range Slider            │
│    con pocos valores distintos      → ComboBox (categórico)   │
│  TEXT + ≤50 valores distintos       → ComboBox (categórico)   │
│    + promedio largo < 80 chars                                │
│  TEXT + >50 valores distintos       → TextSearch              │
│  TEXT + promedio largo > 80 chars   → Omitido (descripción)   │
│  INTEGER + ≤50 valores distintos    → ComboBox                │
│  INTEGER + >50 valores             → Omitido (probable PK)    │
│  PK flag (PRAGMA table_info)       → Omitido                  │
│  FK ref → tabla_catalogo           → ComboBox con nombres     │
└───────────────────────────────────────────────────────────────┘
```

Sin mirar nombres de tablas, sin prefijos `cat_`, sin heurísticas de nombre.
Puro análisis de tipos + valores distintos + longitud promedio.

## Esquema de Colores

Nord Light — variante clara de la paleta Nord:
- Fondo: `#ECEFF4` | Superficie: `#E5E9F0` | Bordes: `#D8DEE9`
- Texto: `#2E3440` | Secundario: `#4C566A`
- Acento: `#88C0D0` (hover) | `#81A1C1` (active) | `#5E81AC` (selección)

## Estructura del Proyecto

```
reporte_contrataciones_4/
├── Cargo.toml            # eframe 0.29 + egui + rusqlite + serde + rfd + chrono
├── Makefile              # build / run / clean / combine
├── doc.md                # Esta documentación
├── src/
│   ├── main.rs           # Entry point, ventana 1400x900
│   ├── app.rs            # Estado global, UI panels, tema Nord Light
│   ├── config.rs         # Detección automática de rutas portable
│   ├── export.rs         # Exportación Excel + PDF
│   ├── redactor.rs       # Plantillas de texto con placeholders
│   ├── db/               # Layer universal (NO MODIFICAR)
│   │   ├── mod.rs, types.rs, schema.rs, analysis.rs
│   │   ├── dashboard.rs, explorer.rs, optimization.rs
│   │   ├── constants.rs, utils.rs
│   └── ui/               # UI panels + widgets
│       ├── sidebar.rs    # Panel de filtros laterales
│       ├── tabla.rs      # Tabla virtualizada
│       ├── redactor_window.rs
│       ├── widgets.rs    # Componentes reutilizables (metric_card)
│       ├── charts.rs     # Gráficos
│       └── theme.rs      # Paleta Nord Light
├── data/                 # Ubicación por defecto de .db / .sqlite
├── output/               # Archivos exportados
├── Tablas3.sql           # Schema de ejemplo
├── Inserts2.sql          # Datos de ejemplo
└── python/               # Scripts originales (legacy)
```

## UI

```
┌──────────────────────────────────────────────────────────┐
│  Explorador BD  [Abrir BD] [Excel] [PDF] [Redactor]      │
├──────────────┬───────────────────────────────────────────┤
│  FILTROS     │  Tabla/Vista: [vw_reporte...        ▼]   │
│  (dinámicos) │                                           │
│              │  Pendientes: 12 | Firmados: 45 | Total: 57│
│  columna1    │  ──────────────────────────────────────── │
│  [Todos ▼]   │  Agrupar por: [Ninguno     ▼]            │
│  columna2    │  ──────────────────────────────────────── │
│  [Todos ▼]   │  ┌── TABLA (scroll H+V) ────────────┐   │
│  col_fecha   │  │ Col1 │ Col2 │ Col3 │ Col4 │ ...  │   │
│  [desde]     │  ├──────┼──────┼──────┼──────┼──────┤   │
│  [hasta]     │  │ val1 │ val2 │ val3 │ val4 │ ...  │   │
│  col_monto   │  │ ...                              │   │
│  [min═══max] │  └──────────────────────────────────┘   │
│  col_texto   │  < Página 1 de 10 >  Tamano: [50][100]  │
│  [buscar]    │                                           │
└──────────────┴───────────────────────────────────────────┘
```

### Sidebar (filtros dinámicos)

| Tipo detectado    | Widget egui                |
|-------------------|----------------------------|
| `categorical`     | `ComboBox` con valores     |
| `categorical_fk`  | `ComboBox` con nombres cat |
| `date`            | 2x `text_edit` (desde/hasta) |
| `numeric`         | 2x `Slider` (min/max)      |
| `text_search`     | `text_edit_singleline`     |

### Panel central

1. **Selector de tabla/vista**: dropdown con todas las tablas y vistas de la BD
2. **Métricas**: Pendientes | Firmados | Total | Coinciden
3. **Group-By**: dropdown con todas las columnas de la tabla actual
4. **Tabla virtualizada**: scroll horizontal + vertical, columnas redimensionables
5. **Paginación**: < Página N de M > con selector de tamaño (50/100/200/500)

## Redactor de Reportes

Ventana flotante para escribir plantillas con placeholders:
- `#total`, `#pendientes`, `#firmados` → métricas
- `#nombre_columna` → valores de esa columna en los datos filtrados

## Dependencias (Cargo.toml)

| Crate            | Versión | Propósito                    |
|------------------|---------|------------------------------|
| `eframe`         | 0.29    | Ventana + loop de eventos    |
| `egui`           | 0.29    | UI immediate-mode            |
| `egui_extras`    | 0.29    | Tabla virtualizada           |
| `rusqlite`       | 0.34    | SQLite (bundled)             |
| `serde/serde_json`| 1      | Serialización                |
| `rfd`            | 0.15    | File dialog                  |
| `open`           | 5       | Abrir carpeta/archivos       |
| `chrono`         | 0.4     | Timestamps                   |
| `rust_xlsxwriter`| 0.82    | Exportar Excel               |
| `printpdf`       | 0.7     | Exportar PDF                 |

## Makefile

```bash
make build       # cargo build
make release     # cargo build --release
make run         # cargo run
make clean       # cargo clean + rm combined.txt
make combine     # concatena todo el código en combined.txt
```

## Reglas del Proceso

1. **doc.md primero**: antes de cualquier implementación o cambio de código, actualizar esta documentación con lo que se planea hacer.
2. **Makefile siempre**: después de cambios, ejecutar `make build` y `make combine`.
3. **Sin hardcodeo**: cero assumptions de naming conventions. Toda heurística debe ser configurable.

---

## Cambios Realizados (Septiembre 2026)

### Dead Code
El código muerto se **conserva** con `#[allow(dead_code)]` — son planned features pendientes de implementar.

### 1. `total_matching` corregido (dashboard.rs)
- Antes: siempre igual a `total_general` (ambos con filtros).
- Ahora: `total_matching` cuenta registros sin filtros, `total_general` con filtros. La tarjeta "Coinciden" ahora muestra el universo total vs. el subconjunto filtrado.

### 2. `PDF_ROW_LIMIT` usado (export.rs)
- Usa `db::constants::PDF_ROW_LIMIT` en vez del literal `200`.

### 3. Regla de documentación
- `doc.md` se actualiza **siempre antes** de cualquier implementación o cambio de código.
- `make build && make combine` se ejecuta después de cada cambio.

---

## Fixes Aplicados (Junio 2026)

### 🔴 SQL (Group A) — Completado

| # | Issue | Archivo | Cambio |
|---|---|---|---|
| 1.1 | SQL Injection en pattern | `dashboard.rs` | `pattern` escapado con `.replace('\'', "''")` antes de interpolarlo en LIKE |
| 1.2 | PK hardcodeado `id` en subquery FK | `dashboard.rs: construir_where` | Nueva función `detectar_pk_columna()` vía `PRAGMA table_info`. `construir_where` acepta `conn`, cachea PKs detectadas en un HashMap |
| 1.3 | PK hardcodeado `id` en JOINs | `dashboard.rs / explorer.rs` | Campo `pk_col` agregado a `FKOptimizada`. Se detecta en `explorer.rs` con `detectar_pk_columna()`. JOIN usa `fk.pk_col` |
| 1.5 | Patrones hardcodeados en detección | `analysis.rs: detectar_columna_estado` | Acepta `pending_pattern`/`signed_pattern` como parámetros. Caller `app.rs` pasa `config.pending_pattern/signed_pattern` |
| 1.6 | INTEGER >50 → None | `analysis.rs: analizar_columna` | Fall through a `numeric` con `MIN/MAX` y slider de rango |

### 🟠 UI/UX (Group B) — Completado

| # | Issue | Archivo | Cambio |
|---|---|---|---|
| 2.3 | Slider min > max | `sidebar.rs` | `if *min > *max { *max = *min; }` post-cambio |
| 2.4 | Slider min == max | `sidebar.rs` | Range padding: `lo..=lo + 1.0` si `(hi - lo).abs() < f64::EPSILON` |
| 2.5 | Paginación "Página 1 de 1" | `tabla.rs` | `if total_general == 0 { ui.label("Sin resultados"); return; }` |
| 2.6 | Columnas ancho fijo | `tabla.rs` | `Column::initial(100.0).at_least(60.0).resizable(true)` |

### 🟡 Cleanup (Group C) — Completado

| # | Issue | Archivo | Cambio |
|---|---|---|---|
| 6.6 | Magic string `"__todos__"` | `constants.rs`, `app.rs`, `sidebar.rs`, `dashboard.rs` | `pub const FILTRO_TODOS: &str = "__todos__"` usado en todos los lugares |

---

## Fixes Segunda Revisión (Julio 2026) — Completado ✓

**Verificado en combined.txt (Agosto 2026)**: Todos los fixes aplicados correctamente.

### 🔴 Crítico (SQL)

| # | Issue | Archivo | Fix | Estado |
|---|---|---|---|---|
| 1.1/1.2 | UPPER() sin normalizar pattern | `analysis.rs`, `dashboard.rs` | `.to_uppercase()` aplicado al pattern | ✓ |
| 1.7 | Fallback `"id"` revive bug PK | `dashboard.rs: construir_where` | Fallback cambiado a `"rowid"` | ✓ |

### 🟠 Importante (UX)

| # | Issue | Archivo | Fix | Estado |
|---|---|---|---|---|
| 1.5b | Label "Coinciden" engañoso | `app.rs` | Renombrado a "Universo" | ✓ |
| 2.1 | Loading overlay borra contenido | `app.rs` | Overlay semitransparente (alpha 180) | ✓ |
| 2.5 | Slider muestra `50.000000` | `sidebar.rs` | `fixed_decimals(0)` si enteros | ✓ |

### 🟡 Mantenibilidad

| # | Issue | Archivo | Fix | Estado |
|---|---|---|---|---|
| 5.1 | `detectar_dependencias` costoso | `explorer.rs` | Solo si ≥2 cols categóricas | ✓ |

**Nota**: La "Tercera Revisión (Agosto 2026)" describe bugs ya corregidos. El código actual en `combined.txt` tiene todos los fixes aplicados.

---

### Dead Code — Limpieza y Refactor SQL (Septiembre 2026) ✓

| # | Archivo | Cambio | Razón |
|---|---------|--------|-------|
| 1 | `src/redactor.rs` | Eliminado campo `visible` | Redundante: `redactor_open` en `App` ya controla la ventana |
| 2 | `src/db/constants.rs` | Eliminado `FK_SAMPLE_LIMIT` | Violaba DRY: `analysis.rs` ya usa `MAX_CATEGORICAL_VALUES * 2` |
| 3 | `src/db/types.rs` | Quitados `#[allow(dead_code)]` de `ModoOptimizacion` | Métodos integrados como ciudadanos de primera clase |
| 4 | `src/db/dashboard.rs` | Refactor: consultas parametrizadas (`?`) + uso de `es_universal()`, `tabla_base()`, `fks()` | Inmunidad SQL injection + eliminación de `match` anidados |
| 5 | `src/db/mod.rs` | Eliminado `pub mod optimization;` | Archivo removido |
| 6 | `src/db/optimization.rs` | **Borrado** | Violaba principio "Layer universal (sin modificar)" — la app no debe alterar el schema |

**SQL Parametrizado**: `construir_where` retorna `(String, Vec<Box<dyn ToSql>>)` con placeholders `?`. Los valores se pasan por separado a `rusqlite`. Se eliminó todo escape manual (`replace('\'', "''")`).
