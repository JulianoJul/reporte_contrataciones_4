use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Default)]
pub struct SchemaMetadata {
    pub vista_principal: String,
    pub tablas: Vec<String>,
    pub columnas: Vec<ColumnaInfo>,
    #[serde(default)]
    pub dependencias: Vec<DependenciaInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ColumnaInfo {
    pub nombre: String,
    pub tipo: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valores: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_distintos: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fecha_min: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fecha_max: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub col_original: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tabla_catalogo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub col_nombre_catalogo: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DependenciaInfo {
    pub columna_padre: String,
    pub columna_dependiente: String,
    pub mapeo: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardData {
    pub total_pendientes: u64,
    pub total_firmados: u64,
    pub total_general: u64,
    pub total_matching: u64,
    pub current_page: usize,
    pub page_size: usize,
    #[serde(default)]
    pub por_grupo: HashMap<String, u64>,
    #[serde(default)]
    pub grupo_actual: String,
    pub tabla: Vec<HashMap<String, serde_json::Value>>,
    pub columnas_tabla: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum FiltroValor {
    Categorical { selected: String },
    CategoricalFK { selected: String, col_original: String, tabla_catalogo: String, col_nombre: String },
    Date { desde: String, hasta: String },
    Numeric { min: f64, max: f64, orig_min: f64, orig_max: f64 },
    TextSearch { query: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct FiltroInfo {
    pub nombre: String,
    pub tipo: String,
    pub valores: Option<Vec<String>>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub fecha_min: Option<String>,
    pub fecha_max: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub col_original: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tabla_catalogo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub col_nombre_catalogo: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FkInfo {
    pub tabla: String,
    pub columna: String,
}

#[derive(Debug)]
pub struct ColumnaRaw {
    pub _cid: i32,
    pub name: String,
    pub col_type: String,
    pub _notnull: bool,
    pub _dflt_value: Option<String>,
    pub pk: bool,
}

impl Default for DashboardData {
    fn default() -> Self {
        DashboardData {
            total_pendientes: 0,
            total_firmados: 0,
            total_general: 0,
            total_matching: 0,
            current_page: 1,
            page_size: super::constants::DEFAULT_PAGE_SIZE,
            por_grupo: HashMap::new(),
            grupo_actual: String::new(),
            tabla: vec![],
            columnas_tabla: vec![],
        }
    }
}

impl ModoOptimizacion {
    pub fn es_universal(&self) -> bool {
        matches!(self, ModoOptimizacion::Universal)
    }
    pub fn fks(&self) -> &[FKOptimizada] {
        match self {
            ModoOptimizacion::Universal => &[],
            ModoOptimizacion::VistaConFKs { fks, .. } => fks.as_slice(),
        }
    }
    pub fn tabla_base(&self) -> Option<&str> {
        match self {
            ModoOptimizacion::Universal => None,
            ModoOptimizacion::VistaConFKs { tabla_base, .. } => Some(tabla_base.as_str()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FKOptimizada {
    pub col_id: String,
    pub tabla_catalogo: String,
    pub col_nombre: String,
    pub nombre_display: String,
    pub pk_col: String,
}

#[derive(Debug, Clone)]
pub enum ModoOptimizacion {
    Universal,
    VistaConFKs {
        tabla_base: String,
        fks: Vec<FKOptimizada>,
    },
}


