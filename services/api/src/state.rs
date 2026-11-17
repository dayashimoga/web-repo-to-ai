//! Shared application state for the Axum HTTP service.
//!
//! Provides [`AppState`] holding configuration, Prometheus metrics registry,
//! startup timestamps, and future database/storage connection pool handles.

use chrono::{DateTime, Utc};
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
    /// UTC timestamp of server initialization.
    pub start_time: DateTime<Utc>,
}

impl AppState {
    /// Creates a new `AppState` instance with the given configuration and default metrics.
    #[must_use]
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Arc::new(config),
            metrics: Arc::new(AppMetrics::new()),
            start_time: Utc::now(),
        }
    }

    /// Creates an `AppState` with a custom metrics registry instance (useful in tests).
    #[must_use]
    pub fn with_metrics(config: AppConfig, metrics: Arc<AppMetrics>) -> Self {
        Self {
            config: Arc::new(config),
            metrics,
            start_time: Utc::now(),
        }
    }

    /// Returns the elapsed process uptime in seconds.
    #[must_use]
    pub fn uptime_seconds(&self) -> u64 {
        let now = Utc::now();
        u64::try_from((now - self.start_time).num_seconds().max(0)).unwrap_or(0)
    }
}
