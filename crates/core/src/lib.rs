//! # Core Foundation (`web-repo-to-ai-core`)
//!
//! Provides the foundational primitives for the `web-repo-to-ai` monorepo:
//! - **Unified Error Taxonomy:** [`CoreError`], [`ErrorCode`], [`ApiError`], [`ApiErrorEnvelope`], and [`Result`].
//! - **Strongly-Typed Domain Identifiers:** [`JobId`], [`DocumentId`], [`TenantId`], [`RequestId`], and [`ArtifactId`].
//! - **Hierarchical Configuration:** [`AppConfig`], sub-configurations, and environment variable loaders.
//! - **Core Traits:** [`Identifiable`], [`Clock`], [`SystemClock`], and [`Validatable`].

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod config;
pub mod error;
pub mod id;
pub mod traits;

// Re-exports for clean ergonomic use
pub use config::{
    AppConfig, DatabaseConfig, Environment, ExtractionConfig, OptimizationConfig, SecurityConfig,
    ServerConfig, StorageBackend, StorageConfig,
};
pub use error::{ApiError, ApiErrorEnvelope, CoreError, ErrorCode, Result};
pub use id::{ArtifactId, DocumentId, JobId, RequestId, TenantId};
pub use traits::{Clock, Identifiable, SystemClock, Validatable};

/// Returns the semantic crate version.
#[must_use]
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Returns the unique crate identifier name.
#[must_use]
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-core");
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn test_core_reexports() {
        let job_id = JobId::new();
        let err = CoreError::NotFound {
            resource_type: "Job",
            id: job_id.to_string(),
        };
        assert_eq!(err.error_code(), ErrorCode::ResourceNotFound);
        assert_eq!(err.http_status(), 404);

        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }
}
