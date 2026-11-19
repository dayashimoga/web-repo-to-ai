//! # Pipeline Worker Daemon (`web-repo-to-ai-worker`)
//!
//! Provides the executable entry point for the background asynchronous conversion
//! pipeline daemon with bounded concurrency and graceful shutdown signal handling.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use tracing::info;
use web_repo_to_ai_core::Result;
use web_repo_to_ai_observability::{init_tracing, LoggingConfig};
use web_repo_to_ai_worker::{WorkerEngine, WorkerServiceConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging and tracing via observability crate
    let logging_config = LoggingConfig::from_env();
    init_tracing(&logging_config)?;

    info!(
        version = env!("CARGO_PKG_VERSION"),
        "Initializing web-repo-to-ai worker daemon"
    );

    let config = WorkerServiceConfig::default();
    let engine = WorkerEngine::new(config)?;

    let shutdown_token = engine.shutdown_token();

    // Spawn signal listener for graceful shutdown
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Shutdown signal received (Ctrl-C); requesting daemon drain");
                shutdown_token.cancel();
            }
            Err(err) => {
                tracing::error!(error = %err, "Failed to install Ctrl-C signal handler");
            }
        }
    });

    // Run daemon loop until shutdown token is cancelled
    engine.run_daemon().await;

    info!("web-repo-to-ai worker daemon terminated gracefully");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_worker_binary_metadata() {
        assert_eq!(env!("CARGO_PKG_NAME"), "web-repo-to-ai-worker");
    }
}
