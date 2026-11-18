//! # GitHub Extractor Errors
//!
//! Strongly-typed error taxonomy for GitHub repository URL identification,
//! target decomposition, REST API interactions, and ref discovery.

use chrono::{DateTime, Utc};
use thiserror::Error;
use web_repo_to_ai_core::CoreError;
use web_repo_to_ai_url_security::UrlSecurityError;

/// Redacts potential GitHub authentication tokens from strings.
#[must_use]
pub fn redact_secrets(input: &str) -> String {
    let prefixes = ["ghp_", "ghs_", "ghu_", "gho_", "ghr_", "github_pat_"];
    let mut result = input.to_string();

    // Redact token prefixes (ghp_..., ghs_..., etc.)
    for prefix in prefixes {
        let mut search_from = 0;
        while let Some(pos) = result[search_from..].find(prefix) {
            let start = search_from + pos;
            let secret_end = result[start..]
                .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == ',' || c == ';')
                .map(|len| start + len)
                .unwrap_or(result.len());

            result.replace_range(start..secret_end, "[REDACTED_TOKEN]");
            search_from = start + "[REDACTED_TOKEN]".len();
        }
    }

    // Redact Authorization headers: "Bearer ..." or "token ..."
    for auth_prefix in ["Bearer ", "token "] {
        let mut search_from = 0;
        while let Some(pos) = result[search_from..].find(auth_prefix) {
            let start = search_from + pos + auth_prefix.len();
            let secret_end = result[start..]
                .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == ',' || c == ';')
                .map(|len| start + len)
                .unwrap_or(result.len());

            result.replace_range(start..secret_end, "[REDACTED_TOKEN]");
            search_from = start + "[REDACTED_TOKEN]".len();
        }
    }

    result
}

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

    /// GitHub REST API rate limit exceeded.
    #[error("GitHub API rate limit exceeded (limit: {limit}, resets at: {reset_at}): {message}")]
    RateLimitExceeded {
        /// Total request limit for the window.
        limit: u32,
        /// Timestamp when the rate limit window resets.
        reset_at: DateTime<Utc>,
        /// Sanitized error message.
        message: String,
    },

    /// Requested GitHub repository or ref resource was not found (HTTP 404).
    #[error("GitHub resource not found at '{url}': {message}")]
    ResourceNotFound {
        /// Sanitized request URL.
        url: String,
        /// Sanitized error message.
        message: String,
    },

    /// Authentication failed due to invalid, revoked, or expired credentials (HTTP 401).
    #[error("GitHub authentication failed: {message}")]
    BadCredentials {
        /// Sanitized error message.
        message: String,
    },

    /// Request forbidden due to lack of permissions or secondary rate limiting (HTTP 403).
    #[error("GitHub request forbidden (HTTP 403): {message}")]
    Forbidden {
        /// Sanitized error message.
        message: String,
    },

    /// Non-success HTTP status returned from GitHub REST API.
    #[error("GitHub API HTTP error (status {status}): {message}")]
    HttpError {
        /// HTTP status code.
        status: u16,
        /// Sanitized error message.
        message: String,
    },

    /// Network or transport failure during HTTP execution.
    #[error("GitHub HTTP request failed: {0}")]
    RequestFailed(String),

    /// Failed to deserialize JSON payload from GitHub API response.
    #[error("Failed to decode GitHub JSON response: {0}")]
    JsonDecodeError(String),

    /// Internal core system error.
    #[error("Core system error: {0}")]
    Core(#[from] CoreError),
}
