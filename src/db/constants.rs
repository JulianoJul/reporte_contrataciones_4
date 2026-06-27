pub const MAX_CATEGORICAL_VALUES: u64 = 50;
pub const MAX_TEXT_LENGTH_THRESHOLD: f64 = 80.0;
pub const SAMPLE_SIZE: usize = 1000;
pub const TABLE_LIMIT: usize = 500;
pub const PDF_ROW_LIMIT: usize = 200;
pub const PK_RATIO_THRESHOLD: f64 = 0.8;
pub const STATUS_THRESHOLD: f64 = 0.01;
pub const STATUS_COMBINED_THRESHOLD: f64 = 0.5;
pub const STATUS_SHORT_LENGTH_THRESHOLD: u64 = 25;
pub const STATUS_SHORT_RATIO_THRESHOLD: f64 = 0.8;
pub const GROUP_BY_LIMIT: u64 = 50;
pub const MIN_FK_COUNT_FOR_OPTIMIZATION: usize = 3;
pub const DEFAULT_PENDING_PATTERN: &str = "PEND";
pub const DEFAULT_SIGNED_PATTERN: &str = "FIRM";
pub const FILTRO_TODOS: &str = "__todos__";
pub const DEFAULT_PAGE_SIZE: usize = 10;
pub const DEFAULT_PK_FALLBACK: &str = "rowid";

#[derive(Debug, Clone)]
pub struct AnalyseConfig {
    pub catalog_prefix: String,
    pub fk_id_prefix: String,
    pub preferred_name_cols: Vec<String>,
    pub exclude_id_prefix: String,
    pub exclude_name_cols: Vec<String>,
    pub view_keywords: Vec<String>,
    pub fallback_pk_name: String,
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
            exclude_id_prefix: "id".to_string(),
            exclude_name_cols: vec![
                "created_at".into(),
                "updated_at".into(),
            ],
            view_keywords: vec![
                "reporte".into(),
                "excel".into(),
                "vw_".into(),
                "vista".into(),
            ],
            fallback_pk_name: DEFAULT_PK_FALLBACK.to_string(),
        }
    }
}
