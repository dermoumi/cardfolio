use std::sync::Arc;

use tracing::level_filters::LevelFilter;

use crate::prelude::*;

pub use http_body_util::BodyExt;
pub use tower::ServiceExt; // Import for .oneshot()

/// Utility to create an AppState for testing
pub async fn with_app_state<Fn, Fut>(f: Fn)
where
    Fn: FnOnce(Arc<AppState>) -> Fut,
    Fut: Future<Output = ()>,
{
    let config = AppConfig {
        log_level: LevelFilter::ERROR,
        port: "8000".to_string(),
        frontend_dir: "frontend/".to_string(),
    };

    // Run the wrapped function
    let state = Arc::new(AppState { config });

    f(state).await
}
