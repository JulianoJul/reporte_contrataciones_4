use std::path::Path;

pub fn clean_identifier(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

pub fn safe_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

pub fn db_paths_in(data_dir: &Path) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    if !data_dir.exists() {
        return paths;
    }
    let Ok(entries) = std::fs::read_dir(data_dir) else { return paths; };
    for entry in entries.flatten() {
        let ext = entry.path().extension().map(|e| e.to_string_lossy().to_lowercase());
        match ext.as_deref() {
            Some("db") | Some("sqlite") | Some("sqlite3") => paths.push(entry.path()),
            _ => {}
        }
    }
    paths
}
