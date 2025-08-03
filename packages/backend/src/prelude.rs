use std::{
    env::{self, VarError},
    path::Path,
    result,
};

use tracing::level_filters::LevelFilter;

use crate::database::Pool;
pub use crate::error::AppError;

/// Shortcut for the Result types
pub type Result<T, E = AppError> = result::Result<T, E>;

/// Common struct for request state
#[derive(Debug, Clone)]
pub struct AppState {
    pub config: AppConfig,
    #[allow(dead_code)] // TODO: Remove this when an endpoint uses the database
    pub db: Pool,
}

/// App configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    // Log level
    pub log_level: LevelFilter,

    // HTTP server configuration
    pub port: String,

    // Database
    pub db_url: String,
    pub db_pool_size: u32,

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

        let db_url = env::var("CARDFOLIO_DB").map_err(|error| match error {
            VarError::NotPresent => anyhow::anyhow!("CARDFOLIO_DB must be set"),
            VarError::NotUnicode(_) => anyhow::anyhow!("CARDFOLIO_DB must be valid UTF-8"),
        })?;
        let db_pool_size = env::var("CARDFOLIO_DB_POOL_SIZE")
            .unwrap_or("16".to_string())
            .parse()?;

        let frontend_dir = env::var("CARDFOLIO_FRONTEND_DIR").unwrap_or("frontend/".to_string());

        Ok(Self {
            log_level,
            port,
            db_url,
            db_pool_size,
            frontend_dir,
        })
    }

    pub fn get_frontend_path(&self) -> &Path {
        Path::new(&self.frontend_dir)
    }
}

#[cfg(test)]
mod tests {
    use temp_env::{with_var, with_vars};

    use super::*;

    #[test]
    fn test_app_config_default_values() {
        with_vars(
            [
                ("CARDFOLIO_LOGLEVEL", None::<&str>),
                ("CARDFOLIO_PORT", None),
                ("CARDFOLIO_FRONTEND_DIR", None),
            ],
            || {
                let config = AppConfig::from_env().unwrap();
                assert_eq!(config.log_level, LevelFilter::INFO);
                assert_eq!(config.port, "8000");
                assert_eq!(config.frontend_dir, "frontend/");
            },
        );
    }

    #[test]
    fn test_app_config_from_env() {
        with_vars(
            [
                ("CARDFOLIO_LOGLEVEL", Some("debug")),
                ("CARDFOLIO_PORT", Some("8080")),
                ("CARDFOLIO_FRONTEND_DIR", Some("test_frontend/")),
            ],
            || {
                let config = AppConfig::from_env().unwrap();
                assert_eq!(config.log_level, LevelFilter::DEBUG);
                assert_eq!(config.port, "8080");
                assert_eq!(config.frontend_dir, "test_frontend/");
            },
        );
    }

    #[test]
    fn test_app_config_get_frontend_path() {
        with_var("CARDFOLIO_FRONTEND_DIR", Some("frontend/"), || {
            let config = AppConfig::from_env().unwrap();
            let path = config.get_frontend_path();
            assert_eq!(path, Path::new("frontend/"));
        });
    }
}
