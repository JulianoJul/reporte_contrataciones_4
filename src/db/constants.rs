pub const MAX_CATEGORICAL_VALUES: u64 = 50;
pub const MAX_TEXT_LENGTH_THRESHOLD: f64 = 80.0;
pub const SAMPLE_SIZE: usize = 1000;
pub const TABLE_LIMIT: usize = 500;
pub const PDF_ROW_LIMIT: usize = 200;
pub const PK_RATIO_THRESHOLD: f64 = 0.8;
pub const STATUS_THRESHOLD: f64 = 0.01;
pub const STATUS_COMBINED_THRESHOLD: f64 = 0.5;
pub const GROUP_BY_LIMIT: u64 = 50;
pub const DEFAULT_PENDING_PATTERN: &str = "PEND";
pub const DEFAULT_SIGNED_PATTERN: &str = "FIRM";
pub const FILTRO_TODOS: &str = "__todos__";

#[derive(Debug, Clone)]
pub struct AnalyseConfig {
    pub catalog_prefix: String,
    pub fk_id_prefix: String,
    pub preferred_name_cols: Vec<String>,
}

impl Default for AnalyseConfig {
    fn default() -> Self {
        Self {
            catalog_prefix: "cat_".to_string(),
            fk_id_prefix: "id_".to_string(),
            preferred_name_cols: vec![
                "nombre".into(),
                "name".into(),
                "descripcion".into(),
                "desc".into(),
            ],
        }
    }
}
