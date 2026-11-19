//! Shared application state for the Axum HTTP service.
//!
//! Provides [`AppState`] holding configuration, Prometheus metrics registry,
//! startup timestamps, artifact storage, token storage, and GitHub client handles.

use chrono::{DateTime, Utc};
use std::fmt;
use std::sync::Arc;
use tokio::sync::Mutex;
use web_repo_to_ai_core::crypto::EncryptionKey;
use web_repo_to_ai_core::AppConfig;
use web_repo_to_ai_github_extractor::callback::InMemoryWebhookDeliveryTracker;
use web_repo_to_ai_github_extractor::client::GitHubApiClient;
use web_repo_to_ai_observability::AppMetrics;
use web_repo_to_ai_storage::{
    create_storage_backend, ApiKeyRepository, BatchRepository, DynStorage,
    InMemoryApiKeyRepository, InMemoryBatchRepository, InMemoryStorage, InMemoryTokenStorage,
    TokenStorageRepository,
};

/// Centralized thread-safe application state shared across all HTTP request handlers.
#[derive(Clone)]
pub struct AppState {
    /// Active application configuration.
    pub config: Arc<AppConfig>,
    /// Prometheus metrics registry.
    pub metrics: Arc<AppMetrics>,
    /// UTC timestamp of server initialization.
    pub start_time: DateTime<Utc>,
    /// Pluggable artifact storage driver.
    pub storage: DynStorage,
    /// Credential storage repository for encrypted GitHub App tokens.
    pub token_storage: Arc<dyn TokenStorageRepository>,
    /// Batch repository for multi-URL batch conversion tracking.
    pub batch_storage: Arc<dyn BatchRepository>,
    /// API key storage repository for authentication and tenant authorization.
    pub api_key_storage: Arc<dyn ApiKeyRepository>,
    /// GitHub API client for upstream token revocation and interactions.
    pub github_client: Arc<GitHubApiClient>,
    /// Webhook delivery GUID tracker preventing replay attacks.
    pub webhook_tracker: Arc<Mutex<InMemoryWebhookDeliveryTracker>>,
    /// Configured webhook secret for HMAC verification.
    pub webhook_secret: Option<String>,
}

impl fmt::Debug for AppState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppState")
            .field("config", &self.config)
            .field("metrics", &self.metrics)
            .field("start_time", &self.start_time)
            .field("storage", &self.storage)
            .field(
                "webhook_secret",
                &self.webhook_secret.as_ref().map(|_| "***[REDACTED]***"),
            )
            .finish_non_exhaustive()
    }
}

impl AppState {
    /// Creates a new `AppState` instance with the given configuration and default metrics and storage.
    #[must_use]
    pub fn new(config: AppConfig) -> Self {
        let storage = create_storage_backend(&config.storage)
            .unwrap_or_else(|_| Arc::new(InMemoryStorage::new()));
        let key =
            EncryptionKey::generate().unwrap_or_else(|_| EncryptionKey::from_bytes([0x42; 32]));
        let token_storage = Arc::new(InMemoryTokenStorage::new(key));
        let batch_storage = Arc::new(InMemoryBatchRepository::new());
        let api_key_storage = Arc::new(InMemoryApiKeyRepository::new());
        let github_client = Arc::new(GitHubApiClient::new());
        let webhook_tracker = Arc::new(Mutex::new(InMemoryWebhookDeliveryTracker::default()));
        let webhook_secret = std::env::var("GITHUB_APP_WEBHOOK_SECRET").ok();

        Self {
            config: Arc::new(config),
            metrics: Arc::new(AppMetrics::new()),
            start_time: Utc::now(),
            storage,
            token_storage,
            batch_storage,
            api_key_storage,
            github_client,
            webhook_tracker,
            webhook_secret,
        }
    }

    /// Creates an `AppState` with a custom metrics registry instance (useful in tests).
    #[must_use]
    pub fn with_metrics(config: AppConfig, metrics: Arc<AppMetrics>) -> Self {
        let mut state = Self::new(config);
        state.metrics = metrics;
        state
    }

    /// Creates an `AppState` with a custom storage driver (useful in tests).
    #[must_use]
    pub fn with_storage(config: AppConfig, storage: DynStorage) -> Self {
        let mut state = Self::new(config);
        state.storage = storage;
        state
    }

    /// Creates an `AppState` with custom metrics and storage drivers.
    #[must_use]
    pub fn with_metrics_and_storage(
        config: AppConfig,
        metrics: Arc<AppMetrics>,
        storage: DynStorage,
    ) -> Self {
        let mut state = Self::new(config);
        state.metrics = metrics;
        state.storage = storage;
        state
    }

    /// Configures a custom token storage repository.
    #[must_use]
    pub fn with_token_storage(mut self, token_storage: Arc<dyn TokenStorageRepository>) -> Self {
        self.token_storage = token_storage;
        self
    }

    /// Configures a custom batch storage repository.
    #[must_use]
    pub fn with_batch_storage(mut self, batch_storage: Arc<dyn BatchRepository>) -> Self {
        self.batch_storage = batch_storage;
        self
    }

    /// Configures a custom API key repository.
    #[must_use]
    pub fn with_api_key_storage(mut self, api_key_storage: Arc<dyn ApiKeyRepository>) -> Self {
        self.api_key_storage = api_key_storage;
        self
    }

    /// Configures a custom GitHub API client (e.g. pointing to a mock server).
    #[must_use]
    pub fn with_github_client(mut self, client: Arc<GitHubApiClient>) -> Self {
        self.github_client = client;
        self
    }

    /// Configures an explicit webhook secret.
    #[must_use]
    pub fn with_webhook_secret(mut self, secret: impl Into<String>) -> Self {
        self.webhook_secret = Some(secret.into());
        self
    }

    /// Returns the elapsed process uptime in seconds.
    #[must_use]
    pub fn uptime_seconds(&self) -> u64 {
        let now = Utc::now();
        u64::try_from((now - self.start_time).num_seconds().max(0)).unwrap_or(0)
    }
}
