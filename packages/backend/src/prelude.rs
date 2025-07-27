use std::{env, path::Path};

use tracing::level_filters::LevelFilter;

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: AppConfig,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    // Log level
    pub log_level: LevelFilter,

    // HTTP server configuration
    pub port: String,

    // Local directories
    pub frontend_dir: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let log_level = env::var("CARDFOLIO_LOGLEVEL")
            .or_else(|_| env::var("LOGLEVEL"))
            .unwrap_or_else(|_| "info".to_string())
            .parse()?;

        let port = env::var("CARDFOLIO_PORT").unwrap_or("8000".to_string());

        let frontend_dir = env::var("CARDFOLIO_FRONTEND_DIR").unwrap_or("frontend/".to_string());

        Ok(Self {
            log_level,
            port,
            frontend_dir,
        })
    }

    pub fn get_frontend_path(&self) -> &Path {
        Path::new(&self.frontend_dir)
    }
}
