//! # API Server (`web-repo-to-ai-api`)
//!
//! Provides the entry point for the Axum REST and SSE server.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use tracing::info;
use web_repo_to_ai_api::run_server;
use web_repo_to_ai_core::{AppConfig, Result};
use web_repo_to_ai_observability::{init_tracing, LoggingConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let logging_config = LoggingConfig::from_env();
    init_tracing(&logging_config)?;

    let config = AppConfig::from_env()?;
    info!(
        "Initializing web-repo-to-ai-api server (v{})",
        env!("CARGO_PKG_VERSION")
    );

    run_server(config).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_api_service_metadata() {
        assert_eq!(env!("CARGO_PKG_NAME"), "web-repo-to-ai-api");
    }
}
