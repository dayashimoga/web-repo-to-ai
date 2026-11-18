//! # GitHub Extractor Errors
//!
//! Strongly-typed error taxonomy for GitHub repository URL identification,
//! target decomposition, REST API interactions, and ref discovery.

use chrono::{DateTime, Utc};
use thiserror::Error;
use web_repo_to_ai_core::CoreError;
use web_repo_to_ai_url_security::UrlSecurityError;

/// Redacts potential GitHub authentication tokens, API keys, private keys, and credentials from strings.
#[must_use]
pub fn redact_secrets(input: &str) -> String {
    web_repo_to_ai_observability::redact_sensitive_text(input)
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

    /// Failure during extraction or parsing of an archive file (e.g. zipball or tarball fallback).
    #[error("GitHub archive processing error: {0}")]
    ArchiveError(String),

    /// Git blob retrieval or integrity validation error.
    #[error("GitHub blob error for '{sha}': {message}")]
    BlobError {
        /// Git blob object SHA.
        sha: String,
        /// Detail error message.
        message: String,
    },

    /// GitHub App configuration, manifest, or credential validation error.
    #[error("GitHub App configuration error: {0}")]
    AppConfiguration(String),

    /// GitHub App permission violation under Rule 05 (insufficient, excessive, or forbidden scopes).
    #[error("GitHub App permission violation: {0}")]
    PermissionViolation(String),

    /// Callback security violation (CSRF mismatch, open redirect, invalid webhook signature, or replay).
    #[error("Callback security violation: {0}")]
    CallbackSecurity(String),

    /// Internal core system error.
    #[error("Core system error: {0}")]
    Core(#[from] CoreError),
}

impl GitHubExtractorError {
    /// Returns a sanitized copy of the error where all sensitive credentials and tokens are scrubbed.
    #[must_use]
    pub fn sanitized(&self) -> Self {
        match self {
            Self::InvalidUrl(u) => Self::InvalidUrl(redact_secrets(u)),
            Self::InvalidOwner { owner, reason } => Self::InvalidOwner {
                owner: redact_secrets(owner),
                reason: redact_secrets(reason),
            },
            Self::InvalidRepository { repo, reason } => Self::InvalidRepository {
                repo: redact_secrets(repo),
                reason: redact_secrets(reason),
            },
            Self::InvalidRef { ref_name, reason } => Self::InvalidRef {
                ref_name: redact_secrets(ref_name),
                reason: redact_secrets(reason),
            },
            Self::InvalidSubpath { path, reason } => Self::InvalidSubpath {
                path: redact_secrets(path),
                reason: redact_secrets(reason),
            },
            Self::ReservedPath(p) => Self::ReservedPath(redact_secrets(p)),
            Self::UnsupportedHost(h) => Self::UnsupportedHost(redact_secrets(h)),
            Self::EmbeddedCredentialsForbidden => Self::EmbeddedCredentialsForbidden,
            Self::UrlSecurity(e) => Self::UrlSecurity(e.clone()),
            Self::RateLimitExceeded {
                limit,
                reset_at,
                message,
            } => Self::RateLimitExceeded {
                limit: *limit,
                reset_at: *reset_at,
                message: redact_secrets(message),
            },
            Self::ResourceNotFound { url, message } => Self::ResourceNotFound {
                url: redact_secrets(url),
                message: redact_secrets(message),
            },
            Self::BadCredentials { message } => Self::BadCredentials {
                message: redact_secrets(message),
            },
            Self::Forbidden { message } => Self::Forbidden {
                message: redact_secrets(message),
            },
            Self::HttpError { status, message } => Self::HttpError {
                status: *status,
                message: redact_secrets(message),
            },
            Self::RequestFailed(msg) => Self::RequestFailed(redact_secrets(msg)),
            Self::JsonDecodeError(msg) => Self::JsonDecodeError(redact_secrets(msg)),
            Self::ArchiveError(msg) => Self::ArchiveError(redact_secrets(msg)),
            Self::BlobError { sha, message } => Self::BlobError {
                sha: sha.clone(),
                message: redact_secrets(message),
            },
            Self::AppConfiguration(msg) => Self::AppConfiguration(redact_secrets(msg)),
            Self::PermissionViolation(msg) => Self::PermissionViolation(redact_secrets(msg)),
            Self::CallbackSecurity(msg) => Self::CallbackSecurity(redact_secrets(msg)),
            Self::Core(c) => Self::Core(CoreError::Internal(redact_secrets(&c.to_string()))),
        }
    }

    /// Formats the error as a sanitized string guaranteed to have all credentials redacted.
    #[must_use]
    pub fn to_sanitized_string(&self) -> String {
        redact_secrets(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_sanitization_removes_tokens() {
        let err = GitHubExtractorError::RequestFailed(
            "HTTP failure while querying https://api.github.com with token ghs_123456789012345678901234567890123456".to_string()
        );

        let sanitized = err.sanitized();
        let display = sanitized.to_string();
        assert!(!display.contains("ghs_123456789012345678901234567890123456"));
        assert!(display.contains("[REDACTED_GITHUB_TOKEN]"));

        let sanitized_str = err.to_sanitized_string();
        assert!(!sanitized_str.contains("ghs_123456789012345678901234567890123456"));
        assert!(sanitized_str.contains("[REDACTED_GITHUB_TOKEN]"));
    }

    #[test]
    fn test_http_error_redaction() {
        let err = GitHubExtractorError::HttpError {
            status: 401,
            message: "Bad credentials for Bearer ghp_999999999999999999999999999999999999"
                .to_string(),
        };

        let sanitized = err.sanitized();
        let display = sanitized.to_string();
        assert!(!display.contains("ghp_999999999999999999999999999999999999"));
        assert!(display.contains("[REDACTED_GITHUB_TOKEN]"));
    }
}
