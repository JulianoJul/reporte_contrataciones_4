use std::path::{Path, PathBuf};
use crate::db::AnalyseConfig;
use crate::db::constants;

pub struct Config {
    pub default_db: Option<PathBuf>,
    pub output_dir: PathBuf,
    pub pending_pattern: String,
    pub pending_label: String,
    pub signed_pattern: String,
    pub signed_label: String,
    pub analyse: AnalyseConfig,
}

impl Config {
    pub fn detect() -> Self {
        let project_root = detect_project_root();
        let default_db = find_default_db(&project_root);
        let output_dir = project_root.join("output");

        let pending_pattern = std::env::var("PENDING_PATTERN")
            .unwrap_or_else(|_| constants::DEFAULT_PENDING_PATTERN.to_string());
        let pending_label = std::env::var("PENDING_LABEL")
            .unwrap_or_else(|_| "Pendientes".to_string());
        let signed_pattern = std::env::var("SIGNED_PATTERN")
            .unwrap_or_else(|_| constants::DEFAULT_SIGNED_PATTERN.to_string());
        let signed_label = std::env::var("SIGNED_LABEL")
            .unwrap_or_else(|_| "Firmados".to_string());

        let dflt = AnalyseConfig::default();
        let analyse = AnalyseConfig {
            catalog_prefix: std::env::var("CATALOG_PREFIX")
                .unwrap_or_else(|_| dflt.catalog_prefix.clone()),
            fk_id_prefix: std::env::var("FK_ID_PREFIX")
                .unwrap_or_else(|_| dflt.fk_id_prefix.clone()),
            preferred_name_cols: {
                let s = std::env::var("PREFERRED_NAME_COLS")
                    .unwrap_or_else(|_| dflt.preferred_name_cols.join(","));
                s.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            },
            exclude_id_prefix: std::env::var("EXCLUDE_ID_PREFIX")
                .unwrap_or_else(|_| dflt.exclude_id_prefix.clone()),
            exclude_name_cols: {
                let s = std::env::var("EXCLUDE_NAME_COLS")
                    .unwrap_or_else(|_| dflt.exclude_name_cols.join(","));
                s.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            },
            view_keywords: {
                let s = std::env::var("VIEW_KEYWORDS")
                    .unwrap_or_else(|_| dflt.view_keywords.join(","));
                s.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            },
            fallback_pk_name: std::env::var("FALLBACK_PK_NAME")
                .unwrap_or_else(|_| dflt.fallback_pk_name.clone()),
        };

        Config {
            default_db,
            output_dir,
            pending_pattern,
            pending_label,
            signed_pattern,
            signed_label,
            analyse,
        }
    }
}

fn detect_project_root() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    if exe_dir.join("data").exists() {
        exe_dir
    } else if PathBuf::from("data").exists() {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    } else {
        exe_dir
    }
}

fn find_default_db(project_root: &Path) -> Option<PathBuf> {
    let data_dir = project_root.join("data");
    if !data_dir.exists() {
        return None;
    }
    crate::db::utils::db_paths_in(&data_dir).into_iter().next()
}
