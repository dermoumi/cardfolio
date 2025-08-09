use std::{net::SocketAddr, sync::Arc};

use anyhow::{Ok, Result};
use axum::{
    Router, ServiceExt,
    extract::Request,
    routing::{get, post},
};
use tokio::{net::TcpListener, signal};
use tower::Layer;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
    normalize_path::NormalizePathLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

mod api_v1;
mod database;
mod error;
mod migrations;
mod models;
mod prelude;
mod services;

#[cfg(test)]
mod test_utils;

use prelude::*;

fn app(state: AppState) -> Router {
    // Serve the frontend, and fallback all unknown routes to the index file
    let frontend_path = state.config.get_frontend_path();
    let frontend =
        ServeDir::new(frontend_path).fallback(ServeFile::new(frontend_path.join("index.html")));

    // API v1
    let api_v1 = Router::new()
        .route("/ygo/cards", get(api_v1::ygo_cards::list_ygo_cards))
        .route("/ygo/cards/{id}", get(api_v1::ygo_cards::get_ygo_card))
        .route(
            "/ygo/cards/import",
            post(api_v1::ygo_cards::import_ygo_cards),
        );

    Router::new()
        .nest("/api/v1", api_v1)
        .fallback_service(frontend)
        .with_state(state)
}

/// Waits for a shutdown signal from the OS.
pub async fn shutdown_signal() {
    // Wait on Ctrl+C signal asynchronously.
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C signal handler");
    };

    // Wait terminal signal asynchronously.
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install terminate signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // Wait for either signal future to complete.
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    std::process::exit(0);
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup app config
    let config = AppConfig::from_env()?;

    // Initialize tracing logs without timestamps
    tracing_subscriber::fmt()
        .with_max_level(config.log_level)
        .init();

    tracing::info!("Starting server.");

    // Postgresql connection pool
    let db_pool = database::init(&config.db_url, config.db_pool_size).await?;
    database::Migrate::new("migrations")
        .run(&db_pool, migrations::MIGRATIONS)
        .await?;
    tracing::info!("Connection to PostgreSQL established.");

    // Tracing layer for tower
    let trace = TraceLayer::new_for_http();

    // Compression layer for tower
    let compression = CompressionLayer::new();

    // Limit request size
    let size_limit_bytes = 1024 * 1024; // 1 MB
    let request_size = RequestBodyLimitLayer::new(size_limit_bytes);

    // Limit request rate
    let requests_per_second = 50;
    let rate_limiter_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(requests_per_second)
            .finish()
            .expect("Could not create rate limiter config"),
    );
    let rate_limiter = GovernorLayer {
        config: rate_limiter_config,
    };

    // Create the app
    let state = AppState {
        config: config.clone(),
        db: db_pool,
    };
    let app = app(state)
        .layer(trace)
        .layer(compression)
        .layer(request_size)
        .layer(rate_limiter);

    // Normalize path layer to trim trailing slashes
    let app = NormalizePathLayer::trim_trailing_slash().layer(app);

    // TCP Listener
    let listener = TcpListener::bind(format!("0.0.0.0:{}", &config.port)).await?;

    // Serve the application
    tracing::info!("Listening on http://{}", listener.local_addr()?);
    axum::serve(
        listener,
        ServiceExt::<Request>::into_make_service_with_connect_info::<SocketAddr>(app),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}
