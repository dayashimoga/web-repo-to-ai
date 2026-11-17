//! Unified domain error taxonomy and standard result types.
//!
//! Provides [`CoreError`], [`ErrorCode`], [`ApiError`], [`ApiErrorEnvelope`],
//! and the standard [`Result`] type alias used across all crates.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Machine-readable standard error codes matching RFC 7807 style API envelopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// Request schema, parameters, or JSON body is invalid.
    InvalidRequestSchema,
    /// Requested URL scheme is unsupported (e.g. not http/https/github).
    UnsupportedUrlScheme,
    /// Target address resolved to a blocked loopback, private, or link-local IP.
    SsrfAddressBlocked,
    /// Access to private GitHub repository is unauthorized or token expired.
    UnauthorizedGithubRepo,
    /// Requested job, document, or artifact was not found.
    ResourceNotFound,
    /// HTTP method not allowed for the target endpoint.
    MethodNotAllowed,
    /// Request rate limit or quota exceeded.
    RateLimitExceeded,
    /// Target file or repository exceeds maximum allowed size budget.
    PayloadTooLarge,
    /// Web acquisition or browser worker operation timed out.
    ExtractionTimeout,
    /// Configuration or environment parameter is missing or invalid.
    ConfigError,
    /// Input or domain invariant validation failed.
    ValidationError,
    /// Storage read, write, or quota enforcement error.
    StorageError,
    /// Job state machine transition or lease acquisition failure.
    JobEngineError,
    /// Unhandled internal system error.
    InternalServerError,
}

impl ErrorCode {
    /// Returns the exact machine-readable string representation of this error code.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidRequestSchema => "INVALID_REQUEST_SCHEMA",
            Self::UnsupportedUrlScheme => "UNSUPPORTED_URL_SCHEME",
            Self::SsrfAddressBlocked => "SSRF_ADDRESS_BLOCKED",
            Self::UnauthorizedGithubRepo => "UNAUTHORIZED_GITHUB_REPO",
            Self::ResourceNotFound => "RESOURCE_NOT_FOUND",
            Self::MethodNotAllowed => "METHOD_NOT_ALLOWED",
            Self::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            Self::PayloadTooLarge => "PAYLOAD_TOO_LARGE",
            Self::ExtractionTimeout => "EXTRACTION_TIMEOUT",
            Self::ConfigError => "CONFIG_ERROR",
            Self::ValidationError => "VALIDATION_ERROR",
            Self::StorageError => "STORAGE_ERROR",
            Self::JobEngineError => "JOB_ENGINE_ERROR",
            Self::InternalServerError => "INTERNAL_SERVER_ERROR",
        }
    }

    /// Maps this error code to its standard HTTP status code.
    #[must_use]
    pub const fn http_status(&self) -> u16 {
        match self {
            Self::InvalidRequestSchema | Self::UnsupportedUrlScheme | Self::ValidationError => 400,
            Self::UnauthorizedGithubRepo => 401,
            Self::SsrfAddressBlocked => 403,
            Self::ResourceNotFound => 404,
            Self::MethodNotAllowed => 405,
            Self::PayloadTooLarge => 413,
            Self::RateLimitExceeded => 429,
            Self::ExtractionTimeout => 504,
            Self::ConfigError
            | Self::StorageError
            | Self::JobEngineError
            | Self::InternalServerError => 500,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Unified domain error type representing all failure modes in `web-repo-to-ai`.
#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    /// Configuration parameter missing, malformed, or out of range.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Request schema, parameters, or JSON body is malformed or invalid.
    #[error("Invalid request schema: {0}")]
    InvalidRequestSchema(String),

    /// Input or domain model validation failed for a specific field.
    #[error("Validation error for field '{field}': {reason}")]
    Validation {
        /// Name of the invalid field or parameter.
        field: String,
        /// Reason the validation failed.
        reason: String,
    },

    /// Security policy violation or untrusted input.
    #[error("Security violation: {0}")]
    Security(String),

    /// Target URL resolved to a restricted IP address.
    #[error("SSRF blocked: URL '{url}' resolved to restricted address '{address}'")]
    SsrfBlocked {
        /// Target URL requested.
        url: String,
        /// Blocked IP address it resolved to.
        address: String,
    },

    /// Unsupported URL scheme.
    #[error("Unsupported URL scheme: {0}")]
    UnsupportedUrlScheme(String),

    /// Target resource was not found.
    #[error("Resource '{resource_type}' with identifier '{id}' was not found")]
    NotFound {
        /// Resource category (e.g. "Job", "Artifact", "Document").
        resource_type: &'static str,
        /// Unique identifier of the missing entity.
        id: String,
    },

    /// HTTP method not allowed for the requested path.
    #[error("HTTP method '{method}' is not allowed for path '{path}'")]
    MethodNotAllowed {
        /// HTTP method attempted.
        method: String,
        /// Request path requested.
        path: String,
    },

    /// Rate limit or concurrency permit exhausted.
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Payload or repository size exceeds limit.
    #[error(
        "Payload too large: actual {actual_bytes} bytes exceeds maximum allowed {max_bytes} bytes"
    )]
    PayloadTooLarge {
        /// Actual size encountered.
        actual_bytes: u64,
        /// Maximum allowable bytes.
        max_bytes: u64,
    },

    /// Operation timed out.
    #[error("Extraction operation timed out after {timeout_secs}s: {context}")]
    Timeout {
        /// Configured timeout in seconds.
        timeout_secs: u64,
        /// Contextual description of the timed-out task.
        context: String,
    },

    /// GitHub authentication or authorization failed.
    #[error("Unauthorized GitHub access: {0}")]
    UnauthorizedGithub(String),

    /// Storage subsystem failure.
    #[error("Storage error: {0}")]
    Storage(String),

    /// Job engine or state machine error.
    #[error("Job engine error: {0}")]
    JobEngine(String),

    /// Web or repository extraction failure.
    #[error("Extraction error: {0}")]
    Extraction(String),

    /// Unhandled internal error.
    #[error("Internal error: {0}")]
    Internal(String),

    /// JSON serialization or deserialization failure.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Standard I/O failure.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl CoreError {
    /// Returns the corresponding [`ErrorCode`] for this domain error.
    #[must_use]
    pub const fn error_code(&self) -> ErrorCode {
        match self {
            Self::Config(_) => ErrorCode::ConfigError,
            Self::InvalidRequestSchema(_) => ErrorCode::InvalidRequestSchema,
            Self::Validation { .. } => ErrorCode::ValidationError,
            Self::Security(_) | Self::SsrfBlocked { .. } => ErrorCode::SsrfAddressBlocked,
            Self::UnsupportedUrlScheme(_) => ErrorCode::UnsupportedUrlScheme,
            Self::NotFound { .. } => ErrorCode::ResourceNotFound,
            Self::MethodNotAllowed { .. } => ErrorCode::MethodNotAllowed,
            Self::RateLimitExceeded(_) => ErrorCode::RateLimitExceeded,
            Self::PayloadTooLarge { .. } => ErrorCode::PayloadTooLarge,
            Self::Timeout { .. } => ErrorCode::ExtractionTimeout,
            Self::UnauthorizedGithub(_) => ErrorCode::UnauthorizedGithubRepo,
            Self::Storage(_) => ErrorCode::StorageError,
            Self::JobEngine(_) => ErrorCode::JobEngineError,
            Self::Extraction(_) | Self::Internal(_) | Self::Serialization(_) | Self::Io(_) => {
                ErrorCode::InternalServerError
            }
        }
    }

    /// Returns the standard HTTP status code for this error.
    #[must_use]
    pub const fn http_status(&self) -> u16 {
        self.error_code().http_status()
    }

    /// Converts this error into a serializable [`ApiErrorEnvelope`] with an optional request identifier.
    #[must_use]
    pub fn to_api_error(&self, request_id: Option<String>) -> ApiErrorEnvelope {
        let details = match self {
            Self::InvalidRequestSchema(reason) => Some(serde_json::json!({
                "reason": reason,
            })),
            Self::Validation { field, reason } => Some(serde_json::json!({
                "field": field,
                "reason": reason,
            })),
            Self::SsrfBlocked { url, address } => Some(serde_json::json!({
                "url": url,
                "address": address,
            })),
            Self::PayloadTooLarge {
                actual_bytes,
                max_bytes,
            } => Some(serde_json::json!({
                "actual_bytes": actual_bytes,
                "max_bytes": max_bytes,
            })),
            Self::NotFound { resource_type, id } => Some(serde_json::json!({
                "resource_type": resource_type,
                "id": id,
            })),
            Self::MethodNotAllowed { method, path } => Some(serde_json::json!({
                "method": method,
                "path": path,
            })),
            _ => None,
        };

        ApiErrorEnvelope {
            error: ApiError {
                code: self.error_code().as_str().to_string(),
                message: self.to_string(),
                details,
                request_id,
            },
        }
    }
}

/// Standard machine-readable API error payload (RFC 7807 style).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiError {
    /// Machine-readable uppercase error identifier.
    pub code: String,
    /// Human-readable description of the error.
    pub message: String,
    /// Contextual structured details about the failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// Correlated request identifier, if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Outermost JSON envelope wrapping an [`ApiError`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiErrorEnvelope {
    /// Enclosed API error details.
    pub error: ApiError,
}

/// Standard Result type alias with [`CoreError`] as the default error variant.
pub type Result<T, E = CoreError> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_mapping() {
        let err = CoreError::SsrfBlocked {
            url: "http://127.0.0.1/admin".to_string(),
            address: "127.0.0.1".to_string(),
        };
        assert_eq!(err.error_code(), ErrorCode::SsrfAddressBlocked);
        assert_eq!(err.http_status(), 403);
        assert_eq!(err.error_code().as_str(), "SSRF_ADDRESS_BLOCKED");
    }

    #[test]
    fn test_validation_error_envelope() {
        let err = CoreError::Validation {
            field: "token_budget".to_string(),
            reason: "Must be positive integer".to_string(),
        };
        let envelope = err.to_api_error(Some("req-12345".to_string()));
        assert_eq!(envelope.error.code, "VALIDATION_ERROR");
        assert_eq!(envelope.error.request_id.as_deref(), Some("req-12345"));
        let details = envelope.error.details.expect("Details should exist");
        assert_eq!(details["field"], "token_budget");
        assert_eq!(details["reason"], "Must be positive integer");
    }

    #[test]
    fn test_error_json_serialization() {
        let err = CoreError::NotFound {
            resource_type: "Job",
            id: "job-0193".to_string(),
        };
        let envelope = err.to_api_error(None);
        let json = serde_json::to_string(&envelope).expect("Serialization must succeed");
        assert!(json.contains("RESOURCE_NOT_FOUND"));
        assert!(json.contains("job-0193"));
        assert!(!json.contains("request_id")); // Skipped when None
    }

    #[test]
    fn test_invalid_request_schema_envelope() {
        let err = CoreError::InvalidRequestSchema(
            "Failed to parse JSON body at line 1 column 5".to_string(),
        );
        assert_eq!(err.error_code(), ErrorCode::InvalidRequestSchema);
        assert_eq!(err.http_status(), 400);
        let envelope = err.to_api_error(Some("req-test".to_string()));
        assert_eq!(envelope.error.code, "INVALID_REQUEST_SCHEMA");
        let details = envelope.error.details.expect("Details should exist");
        assert!(details["reason"]
            .as_str()
            .unwrap()
            .contains("line 1 column 5"));
    }

    #[test]
    fn test_method_not_allowed_envelope() {
        let err = CoreError::MethodNotAllowed {
            method: "DELETE".to_string(),
            path: "/api/v1/jobs".to_string(),
        };
        assert_eq!(err.error_code(), ErrorCode::MethodNotAllowed);
        assert_eq!(err.http_status(), 405);
        let envelope = err.to_api_error(Some("req-mna".to_string()));
        assert_eq!(envelope.error.code, "METHOD_NOT_ALLOWED");
        let details = envelope.error.details.expect("Details should exist");
        assert_eq!(details["method"], "DELETE");
        assert_eq!(details["path"], "/api/v1/jobs");
    }
}
