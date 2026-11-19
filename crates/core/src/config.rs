//! Hierarchical application configuration loader.
//!
//! Provides [`AppConfig`], sub-configurations for server, database, storage,
//! security, extraction, and context optimization, with environment variable loading
//! and domain validation rules.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use crate::error::{CoreError, Result};
use crate::traits::Validatable;

/// Deployment environment tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    /// Local development environment.
    #[default]
    Development,
    /// Automated integration and staging environment.
    Staging,
    /// Production deployment environment.
    Production,
    /// Ephemeral unit and integration test environment.
    Test,
}

impl Environment {
    /// Returns true if running in production.
    #[must_use]
    pub const fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Development => f.write_str("development"),
            Self::Staging => f.write_str("staging"),
            Self::Production => f.write_str("production"),
            Self::Test => f.write_str("test"),
        }
    }
}

impl FromStr for Environment {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "development" | "dev" => Ok(Self::Development),
            "staging" | "stage" => Ok(Self::Staging),
            "production" | "prod" => Ok(Self::Production),
            "test" => Ok(Self::Test),
            _ => Err(CoreError::Config(format!(
                "Invalid environment '{}': expected development, staging, production, or test",
                s
            ))),
        }
    }
}

/// Artifact storage driver backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    /// Local filesystem storage.
    #[default]
    LocalFs,
    /// S3-compatible object storage (MinIO, AWS S3, Cloudflare R2).
    S3,
}

impl fmt::Display for StorageBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LocalFs => f.write_str("localfs"),
            Self::S3 => f.write_str("s3"),
        }
    }
}

impl FromStr for StorageBackend {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "localfs" | "local" | "filesystem" => Ok(Self::LocalFs),
            "s3" => Ok(Self::S3),
            _ => Err(CoreError::Config(format!(
                "Invalid storage backend '{}': expected localfs or s3",
                s
            ))),
        }
    }
}

/// HTTP API server configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host interface to bind to (e.g. "0.0.0.0" or "127.0.0.1").
    pub host: String,
    /// Listening TCP port.
    pub port: u16,
    /// Allowed CORS origins.
    pub cors_allowed_origins: Vec<String>,
    /// Global request timeout in seconds.
    pub request_timeout_secs: u64,
    /// Maximum allowable HTTP request body size in bytes.
    pub max_payload_bytes: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
            cors_allowed_origins: vec![
                "http://localhost:5173".to_string(),
                "http://127.0.0.1:5173".to_string(),
            ],
            request_timeout_secs: 30,
            max_payload_bytes: 10 * 1024 * 1024, // 10 MB
        }
    }
}

/// PostgreSQL database connection configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Full PostgreSQL connection URL.
    pub url: String,
    /// Maximum connection pool size.
    pub max_connections: u32,
    /// Minimum idle connection pool size.
    pub min_connections: u32,
    /// Connection timeout in seconds.
    pub connect_timeout_secs: u64,
    /// Idle connection timeout in seconds.
    pub idle_timeout_secs: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://postgres:postgres@localhost:5432/web_repo_to_ai".to_string(),
            max_connections: 10,
            min_connections: 2,
            connect_timeout_secs: 10,
            idle_timeout_secs: 600,
        }
    }
}

/// Artifact and blob storage configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Active storage backend.
    pub backend: StorageBackend,
    /// Base filesystem path when using [`StorageBackend::LocalFs`].
    pub base_dir: PathBuf,
    /// Maximum storage ceiling in bytes (default 15 GB, within 30 GB budget).
    pub max_storage_bytes: u64,
    /// Retention period for converted artifacts in days.
    pub artifact_retention_days: u32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::LocalFs,
            base_dir: PathBuf::from("./data/storage"),
            max_storage_bytes: 15 * 1024 * 1024 * 1024, // 15 GB
            artifact_retention_days: 7,
        }
    }
}

/// Network security and SSRF protection configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Explicitly allow private and loopback IP addresses (forbidden in production).
    pub allow_private_ips: bool,
    /// DNS resolution timeout in seconds.
    pub dns_timeout_secs: u64,
    /// Maximum HTTP redirect hops allowed before aborting.
    pub max_redirects: usize,
    /// Optional GitHub App ID.
    pub github_app_id: Option<String>,
    /// Optional GitHub App Client ID.
    pub github_client_id: Option<String>,
    /// Optional GitHub App Client Secret.
    pub github_client_secret: Option<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allow_private_ips: false,
            dns_timeout_secs: 5,
            max_redirects: 5,
            github_app_id: None,
            github_client_id: None,
            github_client_secret: None,
        }
    }
}

/// Extraction pipeline limits and worker configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractionConfig {
    /// Maximum allowable single file size in bytes (default 5 MB).
    pub max_file_size_bytes: u64,
    /// Maximum allowable total repository or website size in bytes (default 250 MB).
    pub max_total_repository_bytes: u64,
    /// Maximum concurrent downloads or fetch requests.
    pub max_concurrent_fetches: usize,
    /// Individual fetch timeout in seconds.
    pub fetch_timeout_secs: u64,
    /// URL of the isolated browser worker service (Playwright).
    pub browser_worker_url: Option<String>,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            max_file_size_bytes: 5 * 1024 * 1024,          // 5 MB
            max_total_repository_bytes: 250 * 1024 * 1024, // 250 MB
            max_concurrent_fetches: 8,
            fetch_timeout_secs: 30,
            browser_worker_url: Some("http://localhost:3001".to_string()),
        }
    }
}

/// AI context and token budget optimization configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Default target model identifier for token budgeting.
    pub default_model: String,
    /// Default token budget if unspecified by user.
    pub default_token_budget: usize,
    /// Maximum allowable token budget ceiling.
    pub max_token_budget: usize,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            default_model: "claude-3-5-sonnet".to_string(),
            default_token_budget: 128_000,
            max_token_budget: 2_000_000,
        }
    }
}

/// API rate limiting configuration (Token bucket / Sliding window).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Whether rate limiting is enabled globally (default: true).
    pub enabled: bool,
    /// Default requests per minute allowed for authenticated tenants / API keys (default: 120).
    pub requests_per_minute: u32,
    /// Burst capacity in token bucket for authenticated clients (default: 30).
    pub burst_capacity: u32,
    /// Rate limit for unauthenticated anonymous IP clients in requests per minute (default: 60).
    pub anonymous_requests_per_minute: u32,
    /// Burst capacity for anonymous clients (default: 20).
    pub anonymous_burst_capacity: u32,
    /// Sliding window duration in seconds (default: 60).
    pub window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 120,
            burst_capacity: 30,
            anonymous_requests_per_minute: 60,
            anonymous_burst_capacity: 20,
            window_seconds: 60,
        }
    }
}

/// Comprehensive root application configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Current deployment environment tier.
    pub environment: Environment,
    /// Server network configuration.
    pub server: ServerConfig,
    /// PostgreSQL persistence configuration.
    pub database: DatabaseConfig,
    /// Storage subsystem configuration.
    pub storage: StorageConfig,
    /// Network security and SSRF protection configuration.
    pub security: SecurityConfig,
    /// Web and repository extraction pipeline configuration.
    pub extraction: ExtractionConfig,
    /// AI context and token budgeting configuration.
    pub optimization: OptimizationConfig,
    /// API rate limiting configuration.
    pub rate_limit: RateLimitConfig,
}

impl AppConfig {
    /// Loads configuration from process environment variables, applying default fallbacks.
    ///
    /// # Errors
    /// Returns [`CoreError::Config`] if any provided environment variable cannot be parsed.
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        if let Ok(val) = std::env::var("APP_ENV").or_else(|_| std::env::var("ENVIRONMENT")) {
            config.environment = val.parse()?;
        }

        // Server
        if let Ok(val) = std::env::var("HOST") {
            config.server.host = val;
        }
        if let Ok(val) = std::env::var("PORT") {
            config.server.port = parse_env("PORT", &val)?;
        }
        if let Ok(val) = std::env::var("CORS_ALLOWED_ORIGINS") {
            config.server.cors_allowed_origins = val
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        if let Ok(val) = std::env::var("REQUEST_TIMEOUT_SECS") {
            config.server.request_timeout_secs = parse_env("REQUEST_TIMEOUT_SECS", &val)?;
        }
        if let Ok(val) = std::env::var("MAX_PAYLOAD_BYTES") {
            config.server.max_payload_bytes = parse_env("MAX_PAYLOAD_BYTES", &val)?;
        }

        // Database
        if let Ok(val) = std::env::var("DATABASE_URL") {
            config.database.url = val;
        }
        if let Ok(val) = std::env::var("DB_MAX_CONNECTIONS") {
            config.database.max_connections = parse_env("DB_MAX_CONNECTIONS", &val)?;
        }
        if let Ok(val) = std::env::var("DB_MIN_CONNECTIONS") {
            config.database.min_connections = parse_env("DB_MIN_CONNECTIONS", &val)?;
        }

        // Storage
        if let Ok(val) = std::env::var("STORAGE_BACKEND") {
            config.storage.backend = val.parse()?;
        }
        if let Ok(val) = std::env::var("STORAGE_DIR") {
            config.storage.base_dir = PathBuf::from(val);
        }
        if let Ok(val) = std::env::var("MAX_STORAGE_BYTES") {
            config.storage.max_storage_bytes = parse_env("MAX_STORAGE_BYTES", &val)?;
        }
        if let Ok(val) = std::env::var("ARTIFACT_RETENTION_DAYS") {
            config.storage.artifact_retention_days = parse_env("ARTIFACT_RETENTION_DAYS", &val)?;
        }

        // Security
        if let Ok(val) = std::env::var("ALLOW_PRIVATE_IPS") {
            config.security.allow_private_ips = parse_bool("ALLOW_PRIVATE_IPS", &val)?;
        }
        if let Ok(val) = std::env::var("DNS_TIMEOUT_SECS") {
            config.security.dns_timeout_secs = parse_env("DNS_TIMEOUT_SECS", &val)?;
        }
        if let Ok(val) = std::env::var("MAX_REDIRECTS") {
            config.security.max_redirects = parse_env("MAX_REDIRECTS", &val)?;
        }
        if let Ok(val) = std::env::var("GITHUB_APP_ID") {
            config.security.github_app_id = Some(val);
        }
        if let Ok(val) = std::env::var("GITHUB_CLIENT_ID") {
            config.security.github_client_id = Some(val);
        }
        if let Ok(val) = std::env::var("GITHUB_CLIENT_SECRET") {
            config.security.github_client_secret = Some(val);
        }

        // Extraction
        if let Ok(val) = std::env::var("MAX_FILE_SIZE_BYTES") {
            config.extraction.max_file_size_bytes = parse_env("MAX_FILE_SIZE_BYTES", &val)?;
        }
        if let Ok(val) = std::env::var("MAX_TOTAL_REPOSITORY_BYTES") {
            config.extraction.max_total_repository_bytes =
                parse_env("MAX_TOTAL_REPOSITORY_BYTES", &val)?;
        }
        if let Ok(val) = std::env::var("MAX_CONCURRENT_FETCHES") {
            config.extraction.max_concurrent_fetches = parse_env("MAX_CONCURRENT_FETCHES", &val)?;
        }
        if let Ok(val) = std::env::var("FETCH_TIMEOUT_SECS") {
            config.extraction.fetch_timeout_secs = parse_env("FETCH_TIMEOUT_SECS", &val)?;
        }
        if let Ok(val) = std::env::var("BROWSER_WORKER_URL") {
            config.extraction.browser_worker_url = Some(val);
        }

        // Optimization
        if let Ok(val) = std::env::var("DEFAULT_MODEL") {
            config.optimization.default_model = val;
        }
        if let Ok(val) = std::env::var("DEFAULT_TOKEN_BUDGET") {
            config.optimization.default_token_budget = parse_env("DEFAULT_TOKEN_BUDGET", &val)?;
        }
        if let Ok(val) = std::env::var("MAX_TOKEN_BUDGET") {
            config.optimization.max_token_budget = parse_env("MAX_TOKEN_BUDGET", &val)?;
        }

        // Rate Limiting
        if let Ok(val) = std::env::var("RATE_LIMIT_ENABLED") {
            config.rate_limit.enabled = parse_env("RATE_LIMIT_ENABLED", &val)?;
        }
        if let Ok(val) = std::env::var("RATE_LIMIT_RPM") {
            config.rate_limit.requests_per_minute = parse_env("RATE_LIMIT_RPM", &val)?;
        }
        if let Ok(val) = std::env::var("RATE_LIMIT_BURST") {
            config.rate_limit.burst_capacity = parse_env("RATE_LIMIT_BURST", &val)?;
        }
        if let Ok(val) = std::env::var("RATE_LIMIT_ANON_RPM") {
            config.rate_limit.anonymous_requests_per_minute =
                parse_env("RATE_LIMIT_ANON_RPM", &val)?;
        }
        if let Ok(val) = std::env::var("RATE_LIMIT_ANON_BURST") {
            config.rate_limit.anonymous_burst_capacity = parse_env("RATE_LIMIT_ANON_BURST", &val)?;
        }
        if let Ok(val) = std::env::var("RATE_LIMIT_WINDOW_SECS") {
            config.rate_limit.window_seconds = parse_env("RATE_LIMIT_WINDOW_SECS", &val)?;
        }

        config.validate()?;
        Ok(config)
    }
}

impl Validatable for AppConfig {
    fn validate(&self) -> Result<()> {
        if self.server.port == 0 {
            return Err(CoreError::Validation {
                field: "server.port".to_string(),
                reason: "Port must be greater than 0".to_string(),
            });
        }

        if self.server.max_payload_bytes == 0 {
            return Err(CoreError::Validation {
                field: "server.max_payload_bytes".to_string(),
                reason: "Max payload bytes must be greater than 0".to_string(),
            });
        }

        if self.database.min_connections > self.database.max_connections {
            return Err(CoreError::Validation {
                field: "database.min_connections".to_string(),
                reason: format!(
                    "min_connections ({}) cannot exceed max_connections ({})",
                    self.database.min_connections, self.database.max_connections
                ),
            });
        }

        if self.storage.max_storage_bytes == 0 {
            return Err(CoreError::Validation {
                field: "storage.max_storage_bytes".to_string(),
                reason: "Max storage bytes must be greater than 0".to_string(),
            });
        }

        if self.extraction.max_file_size_bytes > self.extraction.max_total_repository_bytes {
            return Err(CoreError::Validation {
                field: "extraction.max_file_size_bytes".to_string(),
                reason: format!(
                    "max_file_size_bytes ({}) cannot exceed max_total_repository_bytes ({})",
                    self.extraction.max_file_size_bytes, self.extraction.max_total_repository_bytes
                ),
            });
        }

        if self.optimization.default_token_budget > self.optimization.max_token_budget {
            return Err(CoreError::Validation {
                field: "optimization.default_token_budget".to_string(),
                reason: format!(
                    "default_token_budget ({}) cannot exceed max_token_budget ({})",
                    self.optimization.default_token_budget, self.optimization.max_token_budget
                ),
            });
        }

        // Strict security invariant: private IP relaxation forbidden in production
        if self.environment.is_production() && self.security.allow_private_ips {
            return Err(CoreError::Security(
                "SSRF protection invariant violated: allow_private_ips cannot be true in production".to_string(),
            ));
        }

        Ok(())
    }
}

fn parse_env<T: FromStr>(key: &str, val: &str) -> Result<T>
where
    T::Err: fmt::Display,
{
    val.parse().map_err(|err| {
        CoreError::Config(format!(
            "Failed to parse environment variable '{}' with value '{}': {}",
            key, val, err
        ))
    })
}

fn parse_bool(key: &str, val: &str) -> Result<bool> {
    match val.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(CoreError::Config(format!(
            "Invalid boolean for environment variable '{}': '{}'",
            key, val
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validates() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.environment, Environment::Development);
        assert!(!config.security.allow_private_ips);
    }

    #[test]
    fn test_invalid_port_validation() {
        let config = AppConfig {
            server: ServerConfig {
                port: 0,
                ..Default::default()
            },
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            CoreError::Validation { field, .. } => assert_eq!(field, "server.port"),
            other => panic!("Expected Validation error, got {:?}", other),
        }
    }

    #[test]
    fn test_inverted_db_connections_validation() {
        let config = AppConfig {
            database: DatabaseConfig {
                min_connections: 20,
                max_connections: 5,
                ..Default::default()
            },
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            CoreError::Validation { field, .. } => assert_eq!(field, "database.min_connections"),
            other => panic!("Expected Validation error, got {:?}", other),
        }
    }

    #[test]
    fn test_production_ssrf_invariant() {
        let config = AppConfig {
            environment: Environment::Production,
            security: SecurityConfig {
                allow_private_ips: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            CoreError::Security(msg) => assert!(msg.contains("SSRF protection invariant")),
            other => panic!("Expected Security error, got {:?}", other),
        }
    }

    #[test]
    fn test_environment_parsing() {
        assert_eq!(
            "prod".parse::<Environment>().unwrap(),
            Environment::Production
        );
        assert_eq!(
            "development".parse::<Environment>().unwrap(),
            Environment::Development
        );
        assert_eq!("test".parse::<Environment>().unwrap(), Environment::Test);
        assert!("invalid".parse::<Environment>().is_err());
    }

    #[test]
    fn test_storage_backend_parsing() {
        assert_eq!(
            "localfs".parse::<StorageBackend>().unwrap(),
            StorageBackend::LocalFs
        );
        assert_eq!("s3".parse::<StorageBackend>().unwrap(), StorageBackend::S3);
        assert!("ftp".parse::<StorageBackend>().is_err());
    }
}
