use crate::db::{DashboardData, SchemaMetadata};

fn limpiar_nombre(n: &str) -> String {
    n.replace('_', " ")
}

pub struct Redactor {
    pub template: String,
    pub output: String,
}

impl Redactor {
    pub fn new() -> Self {
        Redactor {
            template: String::new(),
            output: String::new(),
        }
    }

    pub fn redactar(&mut self, data: &DashboardData, meta: &SchemaMetadata) {
        if self.template.trim().is_empty() {
            self.output = "Escribe una plantilla primero.".to_string();
            return;
        }

        let mut result = self.template.clone();

        result = result.replace("#total", &data.total_general.to_string());
        result = result.replace("#pendientes", &data.total_pendientes.to_string());
        result = result.replace("#firmados", &data.total_firmados.to_string());

        let display_cols: Vec<(String, String)> = meta
            .columnas
            .iter()
            .map(|c| (c.nombre.clone(), limpiar_nombre(&c.nombre)))
            .collect();

        for (col_name, display_name) in &display_cols {
            let placeholder = format!("#{}", display_name);
            let alt_placeholder = format!("#{}", col_name);

            let ph = if result.contains(&placeholder) {
                placeholder
            } else if result.contains(&alt_placeholder) {
                alt_placeholder
            } else {
                continue;
            };

            let values: Vec<String> = data
                .tabla
                .iter()
                .map(|row| {
                    row.get(col_name)
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                        .unwrap_or_default()
                })
                .collect();

            let replacement = if values.is_empty() {
                if data.total_general == 0 {
                    "(sin registros)".to_string()
                } else {
                    format!("({} valores)", values.len())
                }
            } else if values.len() == 1 {
                values[0].clone()
            } else {
                let mut lines: Vec<String> = values.iter().map(|v| format!("  - {}", v)).collect();
                lines.insert(0, format!("({} registros):", values.len()));
                lines.join("\n")
            };

            result = result.replace(&ph, &replacement);
        }

        self.output = result;
    }
}
