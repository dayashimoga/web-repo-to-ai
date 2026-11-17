//! Shared application state for the Axum HTTP service.
//!
//! Provides [`AppState`] holding configuration, Prometheus metrics registry,
//! and future database/storage connection pool handles.

use std::sync::Arc;
use web_repo_to_ai_core::AppConfig;
use web_repo_to_ai_observability::AppMetrics;

/// Centralized thread-safe application state shared across all HTTP request handlers.
#[derive(Clone, Debug)]
pub struct AppState {
    /// Active application configuration.
    pub config: Arc<AppConfig>,
    /// Prometheus metrics registry.
    pub metrics: Arc<AppMetrics>,
}

impl AppState {
    /// Creates a new `AppState` instance with the given configuration and default metrics.
    #[must_use]
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Arc::new(config),
            metrics: Arc::new(AppMetrics::new()),
        }
    }

    /// Creates an `AppState` with a custom metrics registry instance (useful in tests).
    #[must_use]
    pub fn with_metrics(config: AppConfig, metrics: Arc<AppMetrics>) -> Self {
        Self {
            config: Arc::new(config),
            metrics,
        }
    }
}
