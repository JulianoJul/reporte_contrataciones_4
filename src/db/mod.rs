pub mod constants;
pub mod types;
pub mod utils;
pub mod schema;
pub mod analysis;
pub mod dashboard;
pub mod explorer;

pub use types::*;
pub use constants::AnalyseConfig;
pub use explorer::explorar;
pub use explorer::detectar_patron_optimizable;
pub use dashboard::dashboard;
pub use analysis::extraer_filtros_info;

