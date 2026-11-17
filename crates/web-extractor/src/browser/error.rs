//! # Browser Worker Error Types
//!
//! Error representations for IPC communication and browser execution failures.

use std::fmt;

/// Errors arising during communication with or execution inside the browser worker service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserWorkerError {
    /// Client-side invalid request parameter (HTTP 400).
    InvalidRequest(String),
    /// Navigation timed out before reaching target wait condition.
    NavigationTimeout(String),
    /// Target URL or subresource was blocked by SSRF/security policy.
    BlockedUrl(String),
    /// Chromium browser process crashed or failed unexpectedly.
    BrowserCrash(String),
    /// Underlying HTTP network transport error connecting to browser-worker service.
    HttpTransportError(String),
    /// Browser worker returned an unexpected HTTP status code.
    UnexpectedStatus {
        /// HTTP status code.
        status: u16,
        /// Server response body or error message.
        message: String,
    },
    /// Response payload deserialization error.
    DeserializationFailed(String),
}

impl fmt::Display for BrowserWorkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(msg) => write!(f, "Invalid browser render request: {msg}"),
            Self::NavigationTimeout(msg) => write!(f, "Browser navigation timeout: {msg}"),
            Self::BlockedUrl(msg) => write!(f, "Browser URL blocked by security policy: {msg}"),
            Self::BrowserCrash(msg) => write!(f, "Browser worker crashed: {msg}"),
            Self::HttpTransportError(msg) => write!(f, "HTTP transport error: {msg}"),
            Self::UnexpectedStatus { status, message } => {
                write!(f, "Browser worker returned HTTP {status}: {message}")
            }
            Self::DeserializationFailed(msg) => {
                write!(f, "Failed to deserialize browser worker response: {msg}")
            }
        }
    }
}

impl std::error::Error for BrowserWorkerError {}
