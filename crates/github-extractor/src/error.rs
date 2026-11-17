//! # GitHub Extractor Errors
//!
//! Strongly-typed error taxonomy for GitHub repository URL identification,
//! target decomposition, and API extraction.

use thiserror::Error;
use web_repo_to_ai_core::CoreError;
use web_repo_to_ai_url_security::UrlSecurityError;

/// Error conditions encountered during GitHub URL decomposition and extraction.
#[derive(Debug, Error)]
pub enum GitHubExtractorError {
    /// URL is syntactically invalid or unparseable.
    #[error("Invalid GitHub URL: {0}")]
    InvalidUrl(String),

    /// Repository owner does not meet GitHub naming constraints.
    #[error("Invalid GitHub owner '{owner}': {reason}")]
    InvalidOwner {
        /// The invalid owner string.
        owner: String,
        /// The specific validation reason.
        reason: String,
    },

    /// Repository name does not meet GitHub naming constraints.
    #[error("Invalid GitHub repository '{repo}': {reason}")]
    InvalidRepository {
        /// The invalid repo string.
        repo: String,
        /// The specific validation reason.
        reason: String,
    },

    /// Specified git ref (branch, tag, or commit) is invalid or malformed.
    #[error("Invalid git ref '{ref_name}': {reason}")]
    InvalidRef {
        /// The invalid ref string.
        ref_name: String,
        /// The specific validation reason.
        reason: String,
    },

    /// Subpath within the repository contains forbidden elements or path traversal.
    #[error("Invalid subpath '{path}': {reason}")]
    InvalidSubpath {
        /// The invalid subpath string.
        path: String,
        /// The specific validation reason.
        reason: String,
    },

    /// URL targets a reserved or non-repository GitHub path (e.g. /features, /pricing, /login).
    #[error("Reserved GitHub system path rejected: '{0}'")]
    ReservedPath(String),

    /// URL host is not an allowed GitHub domain.
    #[error("Unsupported or disallowed GitHub host: '{0}'")]
    UnsupportedHost(String),

    /// Credentials (user/password) detected in URL.
    #[error("URL contains embedded credentials which is forbidden for security")]
    EmbeddedCredentialsForbidden,

    /// Target URL was blocked by zero-trust URL security policy or SSRF filter.
    #[error("URL security policy violation: {0}")]
    UrlSecurity(#[from] UrlSecurityError),

    /// Internal core system error.
    #[error("Core system error: {0}")]
    Core(#[from] CoreError),
}
