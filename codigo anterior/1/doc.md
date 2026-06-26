# Reporte de Contrataciones — Documentación

## Arquitectura

Aplicación portable para generación de reportes de contrataciones.
**Rust (egui) = UI** | **Python (pandas) = Data Layer**.
Comunicación via **stdin/stdout pipe persistente**.

```
┌────────────────────────┐     JSON/stdin      ┌─────────────────────┐
│  Rust (egui)           │ ──────────────────→  │  Python             │
│  - Dashboard + gráficos │                     │  - pandas + SQLite  │
│  - Filtros dinámicos    │ ←──────────────────  │  - Export Excel     │
│  (data-driven)          │     JSON/stdout      │  - Explorador BD    │
│  - Drag reorder         │                      │  (universal)        │
└────────────────────────┘                      └─────────────────────┘
                                                         │
                                                         ▼
                                                   [cualquier .db]
```

## Principio Fundamental

**Cero hardcodeo. Cero naming conventions. Cero assumptions del schema.**

El sidebar se genera automáticamente analizando los datos de la BD al abrirla.
No busca nombres de columna, no asume tablas `cat_*`, no sabe nada del dominio.

## Algoritmo Universal de Detección de Columnas

```
Para cada columna en la tabla/vista principal:
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
└───────────────────────────────────────────────────────────────┘
```

Sin mirar nombres de tablas, sin prefijos `cat_`, sin heurísticas de nombre.
Puro análisis de tipos + valores distintos + longitud promedio.

## Detección de Dependencias en Cascada

Usa `PRAGMA foreign_key_list` para detectar cadenas FK entre tablas.
Ejemplo detectado automáticamente:

```
expedientes.id_superintendencia → cat_superintendencia.id
                                  cat_superintendencia.id_gerencia → cat_gerencia.id
→ superintendencia DEPENDE de gerencia (ComboBox en cascada)
```

Esto funciona para **cualquier schema** que tenga FK constraints. Sin FK,
los filtros son independientes.

## Estructura del Proyecto

```
reporte_contrataciones/
├── Cargo.toml              # egui 0.31 + egui_plot + serde + rfd + chrono
├── Makefile                # build / run / clean / combine
├── doc.md                  # Esta documentación
├── build_portable.sh       # Script de build Linux
├── src/
│   ├── main.rs             # Entry point, ventana 1280x720
│   ├── app.rs              # Estado global, orquestación bridge + explorar
│   ├── bridge.rs           # Pipe persistente stdin/stdout + tipos schema
│   ├── config.rs           # Detección automática de rutas portable
│   ├── dashboard.rs        # Gráficos (barras + pastel manual) + tabla dinámica
│   ├── export.rs           # Exportación a Excel + abrir carpeta
│   └── widgets.rs          # Sidebar dinámico data-driven + StatusBar
├── python/
│   ├── data_service.py     # Bucle de comandos JSON via stdin
│   ├── queries.py          # Explorador universal + dashboard + filtros
│   ├── exporters.py        # Exportación a Excel con openpyxl
│   └── requirements.txt    # pandas, openpyxl, python-pptx
├── venv/                   # Python virtualenv
├── output/                 # Archivos exportados
├── data/                   # Ubicación por defecto de expedientes.db
├── Tablas3.sql             # Schema de ejemplo
└── Inserts2.sql            # Datos de ejemplo
```

## Comunicación Rust ↔ Python

### Pipe persistente (stdin/stdout)

**Ventajas sobre CLI restart por llamada:**
- No hay overhead de arranque de Python (~300-500ms en i5 2da gen)
- Comunicación bidireccional estructurada
- Python mantiene la conexión a SQLite abierta

**Ventajas sobre HTTP:**
- Cero configuración de red, puertos, firewalls
- No requiere async en Rust
- Menor consumo de RAM (sin servidor HTTP)
- Debug simple (todo es texto plano)

### Protocolo

```
Request  →  {"id": 1, "tipo": "explorar", "db": "ruta"}
Response ←  {"id": 1, "ok": true, "data": {vista, columnas[], dependencias[]}}

Request  →  {"id": 2, "tipo": "dashboard", "db": "ruta", "filtros": {col: {tipo, valor}}}
Response ←  {"id": 2, "ok": true, "data": {total, pendientes, grupos, tabla, columnas_tabla}}

Request  →  {"tipo": "quit"}
Response ←  (Python cierra proceso)
```

### Flujo de datos completo

```
1. App inicia → bridge.spawn("python data_service.py")
2. Usuario selecciona .db → Rust envía "explorar"
3. Python analiza schema + datos → devuelve columnas con tipo y valores
4. Rust construye Sidebar dinámico (ComboBox / Date / Slider / TextSearch)
5. Usuario ajusta filtros → click "Actualizar"
6. Rust serializa filtros activos como JSON → envía "dashboard"
7. Python aplica filtros con pandas → devuelve agrupaciones + tabla
8. Rust renderiza gráficos (barras + pastel) + tabla dinámica
```

## Dependencias

### Rust (Cargo.toml)

| Crate | Versión | Propósito |
|-------|---------|-----------|
| `eframe` | 0.31 | Ventana + loop de eventos egui |
| `egui` | 0.31 | UI framework immediate mode |
| `egui_plot` | 0.31 | Gráficos de barras |
| `serde` / `serde_json` | 1 | Serialización pipe JSON |
| `rfd` | 0.15 | File dialog selector .db |
| `open` | 5 | Abrir carpeta output |
| `chrono` | 0.4 | Timestamps |

### Python (requirements.txt)

| Paquete | Propósito |
|---------|-----------|
| `pandas` | Leer SQLite, agrupar, transformar |
| `openpyxl` | Exportar Excel con formato |
| `python-pptx` | Exportar PowerPoint (✓ implementado en queries.py) |
| `tabulate` | Formatear tablas (futuro) |

> **Nota**: DatePicker y Selector de Vista múltiple están identificados como mejoras futuras.

## Pantalla / UI

### Pantalla Única con Sidebar Dinámico

```
┌───────────────────────────────────────────────────┐
│  📋 Reporte de Contrataciones                     │
├───────────────┬───────────────────────────────────┤
│  FILTROS      │  DASHBOARD                        │
│  (dinámicos)  │                                   │
│               │  [Pendientes] [Firmados] [Total]  │
│  Base:        │                                   │
│  [📁 ____]    │  ┌── GRAFICOS ──────────────┐    │
│  ─────────    │  │  Barras por columna1      │    │
│  ≡ columna1   │  │  Pastel por columna2      │    │
│    [Todos ▼]  │  └──────────────────────────┘    │
│  ≡ columna2   │                                   │
│    [Todos ▼]  │  ┌── TABLA DINÁMICA ───────┐    │
│  ≡ col_fecha  │  │  columnas = schema      │    │
│    [desde]    │  │  filas                  │    │
│    [hasta]    │  └──────────────────────────┘    │
│  ≡ col_monto  │                                   │
│    [===slider]│                                   │
│  ≡ col_texto  │                                   │
│    [buscar]   │                                   │
│  ─────────    │                                   │
│  [🔄 Act.]    │                                   │
│  [📊 Excel]   │                                   │
│  [📁 Output]  │                                   │
├───────────────┴───────────────────────────────────┤
│  Última act.: 14:32:05  │  Registros: 6,912       │
└───────────────────────────────────────────────────┘
```

### Dashboard — Group-By Selector

En la parte superior del dashboard hay un dropdown **"Agrupar por:"** que permite elegir
cualquier columna **categorical** del schema. Al seleccionar una, el gráfico de barras
y el pastel se actualizan mostrando la distribución por ese campo.

- Los datos se recalculan del lado de Python con `df.groupby(columna)`.
- El selector solo muestra columnas categoricales detectadas automáticamente.
- Al abrir una BD, se selecciona por defecto la primera columna categorical.

### Sidebar Inteligente (widgets.rs)

Construido 100% desde la metadata devuelta por `explorar`:

| Dato de Python | Widget en Rust |
|---------------|----------------|
| `{tipo: "categorical", valores: [...]}` | `egui::ComboBox` con valores |
| `{tipo: "categorical"}` + dependencia | ComboBox filtrado por padre |
| `{tipo: "date"}` | 2x `text_edit` (desde/hasta) |
| `{tipo: "numeric", min, max}` | 2x `egui::Slider` (min/max) |
| `{tipo: "text_search"}` | `text_edit_singleline` |

**Reordenación**: Cada filtro tiene handle "≡". El orden inicial es alfabético.

**Reset rápido**: Botón "✕" en cada filtro activo lo resetea a su valor por defecto.

### Dashboard (dashboard.rs)

Tres secciones:
1. **Cards métricas**: Pendientes (rojo) | Firmados (verde) | Total (azul)
   - Columna de estatus detectada automáticamente por contenido (busca "PEND"/"FIRM")
2. **Gráficos**: Barras (egui_plot) + Pastel (manual con `Shape::Path`)
   - Columnas de agrupación detectadas automáticamente
3. **Tabla dinámica**: Columnas = las del schema, sin hardcodeo

## Módulos Python

### `data_service.py` — Entry point

Bucle de comandos via stdin/stdout. Soporta:
- `explorar` → introspección universal de schema
- `dashboard` → consulta con filtros dinámicos
- `exportar_excel` → genera .xlsx
- `quit` → cierra el bucle

### `queries.py` — Núcleo de datos

Tres funciones principales:

```python
def explorar(conn) -> dict:
    """Analiza la BD y devuelve columnas con tipos y valores."""

def dashboard(conn, filtros, vista=None) -> dict:
    """Aplica filtros genéricos (columna → tipo + valor)."""

def _detectar_metricas(df) -> dict:
    """Encuentra columnas de estatus/gerencia/modalidad por contenido."""
```

Los filtros se aplican universalmente:
```python
def _aplicar_filtros(df, filtros):
    for col_name, f_info in filtros.items():
        if f_info["tipo"] == "categorical" and valor != "__todos__":
            df = df[df[col_name] == valor]
        elif f_info["tipo"] == "date":
            # filtro por rango de fechas
        elif f_info["tipo"] == "numeric":
            # filtro por rango numérico
        elif f_info["tipo"] == "text_search":
            # búsqueda parcial case-insensitive
```

## Redactor de Reportes (`redactor.rs`)

Permite al usuario escribir una **plantilla de texto** con placeholders `#columna`
que se reemplazan automáticamente con los valores reales de los datos filtrados.

### Sintaxis de placeholders

| Placeholder | Reemplazo |
|-------------|-----------|
| `#total` | Total de registros (`total_general`) |
| `#pendientes` | Cantidad de pendientes (`total_pendientes`) |
| `#firmados` | Cantidad de firmados (`total_firmados`) |
| `#nombre_columna` | Valor(es) de esa columna en los datos |

### Comportamiento

- Si el placeholder corresponde a una columna con un **solo valor** → se reemplaza
  directamente.
- Si hay **múltiples filas** → se lista cada valor en una línea nueva con guion.
- Si no hay registros → muestra "(sin registros)".
- Los nombres de columna se matchean **con o sin guiones bajos**: `#acta de inicio`
  funciona aunque la columna se llame `acta_de_inicio`.

### UI

- Botón "📝 Redactor" en la barra lateral.
- Abre una ventana flotante con:
  - Área de texto para la plantilla.
  - Lista colapsable de columnas disponibles.
  - Botón "🖊 Redactar".
  - Área de resultado con botón "📋 Copiar al portapapeles".

### Futuras mejoras

- **DatePicker**: Reemplazar inputs de texto para fechas con un calendario.
- **Selector de Vista**: Si la BD tiene múltiples vistas, permitir elegir cuál
  usar (dropdown en sidebar).
- **Exportación PPTX/PDF**: Actualmente solo Excel está implementado.
- **Reordenación por arrastre**: Los handles "≡" en filtros son placeholder visual.

---

### `exporters.py` — Exportación

```python
def excel(conn, filtros, ruta) -> dict:
    # Filtra datos → escribe Excel con openpyxl → ajusta anchos
```

## Pipeline de Desarrollo (Linux → Windows)

### Portabilidad

1. **Python**: `venv/` portable (se distribuye con la app)
2. **Rust**: Compilación cruzada para `x86_64-pc-windows-gnu`
3. **Auto-detección** (`config.rs`):
   - Busca Python en: `./venv/bin/python3` → `./venv/Scripts/python.exe` → `python3`
   - Busca `.db` en `./data/` o ruta seleccionada por usuario

### Makefile

```bash
make build       # cargo build
make release     # cargo build --release
make run         # cargo run
make clean       # cargo clean + rm combined.txt
make combine     # concatena todo el código en combined.txt
```
