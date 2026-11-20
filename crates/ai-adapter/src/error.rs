//! Unified error taxonomy for AI provider adapters.

use thiserror::Error;

/// Categorized errors returned by AI provider adapters.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AiAdapterError {
    /// AI provider is not enabled or disabled by policy (zero-key fallback).
    #[error("AI provider is disabled: {0}")]
    ProviderDisabled(String),

    /// Missing, invalid, or expired API key or bearer token.
    #[error("AI provider authentication failed: {0}")]
    AuthenticationRequired(String),

    /// Low-level HTTP transport error, connection refused, or unreachable host.
    #[error("AI provider network error: {0}")]
    NetworkError(String),

    /// Rate limit or quota exhaustion with optional retry-after seconds.
    #[error("AI provider rate limit exceeded: {message} (retry after: {retry_after_secs:?}s)")]
    RateLimitExceeded {
        /// Suggested backoff period in seconds if provided by provider.
        retry_after_secs: Option<u64>,
        /// Descriptive message from provider.
        message: String,
    },

    /// The input prompt or request tokens exceed the model's context window.
    #[error("Context length exceeded: requested {requested_tokens} tokens, max allowed is {max_tokens}. {message}")]
    ContextLengthExceeded {
        /// Number of tokens requested.
        requested_tokens: usize,
        /// Maximum context window limit of the model.
        max_tokens: usize,
        /// Additional details.
        message: String,
    },

    /// Requested model name is unknown or unavailable on the provider.
    #[error("Model not found: '{model}' - {message}")]
    ModelNotFound {
        /// Requested model identifier.
        model: String,
        /// Detail message.
        message: String,
    },

    /// Malformed request parameters (invalid temperature, empty messages, etc.).
    #[error("Invalid request to AI provider: {0}")]
    InvalidRequest(String),

    /// Failed to parse provider response payload into expected schema.
    #[error("Failed to parse AI provider response: {0}")]
    ResponseParsingError(String),

    /// Provider returned a 5xx server-side error.
    #[error("AI provider server unavailable (status {status_code:?}): {message}")]
    ServerUnavailable {
        /// Optional HTTP status code (e.g. 500, 502, 503, 504).
        status_code: Option<u16>,
        /// Detail message.
        message: String,
    },

    /// Operation timed out before provider responded.
    #[error("AI provider operation timed out: {0}")]
    Timeout(String),

    /// Internal adapter error.
    #[error("Internal AI adapter error: {0}")]
    Internal(String),
}

impl AiAdapterError {
    /// Whether the error represents a transient failure that may succeed on retry.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::RateLimitExceeded { .. }
            | Self::NetworkError(_)
            | Self::ServerUnavailable { .. }
            | Self::Timeout(_) => true,
            Self::ProviderDisabled(_)
            | Self::AuthenticationRequired(_)
            | Self::ContextLengthExceeded { .. }
            | Self::ModelNotFound { .. }
            | Self::InvalidRequest(_)
            | Self::ResponseParsingError(_)
            | Self::Internal(_) => false,
        }
    }

    /// Associated HTTP-like status code if applicable.
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::AuthenticationRequired(_) => Some(401),
            Self::ProviderDisabled(_) => Some(403),
            Self::ModelNotFound { .. } => Some(404),
            Self::InvalidRequest(_) | Self::ResponseParsingError(_) => Some(400),
            Self::ContextLengthExceeded { .. } => Some(413),
            Self::RateLimitExceeded { .. } => Some(429),
            Self::ServerUnavailable { status_code, .. } => *status_code,
            Self::Timeout(_) => Some(504),
            Self::NetworkError(_) | Self::Internal(_) => Some(500),
        }
    }
}
