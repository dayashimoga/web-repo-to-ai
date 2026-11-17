//! HTTP server lifecycle and graceful shutdown handling.
//!
//! Provides [`build_app`] to assemble routers and middleware, and [`run_server`]
//! for binding TCP listeners with asynchronous graceful shutdown signals.

use axum::middleware as axum_mw;
use axum::Router;
use tower_http::limit::RequestBodyLimitLayer;
use tracing::info;
use web_repo_to_ai_core::{AppConfig, CoreError, Result};

use crate::middleware::{build_cors_layer, build_timeout_layer, trace_id_middleware};
use crate::routes::create_router;
use crate::state::AppState;

/// Assembles the complete Axum application router with all middleware layers attached.
pub fn build_app(state: AppState) -> Router {
    let cors = build_cors_layer(&state.config);
    let timeout = build_timeout_layer(&state.config);
    let body_limit = RequestBodyLimitLayer::new(10 * 1024 * 1024);

    create_router(state)
        .layer(axum_mw::from_fn(trace_id_middleware))
        .layer(cors)
        .layer(timeout)
        .layer(body_limit)
}

/// Binds the HTTP listener and starts the Axum web server with graceful shutdown.
///
/// # Errors
/// Returns [`CoreError::Io`] if socket binding or serving fails.
pub async fn run_server(config: AppConfig) -> Result<()> {
    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    let state = AppState::new(config);
    let app = build_app(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .map_err(CoreError::Io)?;

    info!(
        "web-repo-to-ai-api server listening on http://{}",
        bind_addr
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(CoreError::Io)?;

    info!("web-repo-to-ai-api server gracefully shut down");
    Ok(())
}

/// Listens for termination signals (`SIGINT` / `Ctrl+C` across platforms, plus `SIGTERM` on Unix).
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C / SIGINT signal, initiating graceful shutdown");
        },
        _ = terminate => {
            info!("Received SIGTERM signal, initiating graceful shutdown");
        },
    }
}
