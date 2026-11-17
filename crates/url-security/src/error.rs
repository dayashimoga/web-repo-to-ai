//! # URL Security Errors
//!
//! Strongly-typed domain errors for URL syntax, scheme restriction, and SSRF defenses.

use std::fmt;
use web_repo_to_ai_core::error::CoreError;

/// Specific error conditions encountered during URL parsing, normalization, and SSRF evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlSecurityError {
    /// The URL exceeds the maximum allowed character limit.
    TooLong {
        /// Actual length of the submitted URL string.
        actual: usize,
        /// Maximum permitted length in characters.
        maximum: usize,
    },
    /// The URL uses a forbidden or unsupported URI scheme.
    ForbiddenScheme {
        /// The rejected scheme identifier (e.g., `file`, `gopher`, `ftp`).
        scheme: String,
        /// Explanation of why the scheme is forbidden.
        reason: String,
    },
    /// The URL string failed WHATWG standard syntax parsing.
    ParseError {
        /// Error description from standard parser.
        details: String,
    },
    /// The URL contains userinfo (credentials), which is prohibited by policy.
    UserinfoForbidden,
    /// The hostname or IP address resolves to a prohibited network or address space.
    BlockedIp {
        /// The prohibited IP address as formatted string.
        ip: String,
        /// Classification of the blocked address (e.g., Loopback, RFC1918 Private, Cloud Metadata).
        category: String,
    },
    /// The hostname is a blocked domain name (e.g., `localhost`, `*.internal`, cloud metadata names).
    BlockedHost {
        /// The rejected hostname.
        host: String,
        /// Reason for blocking the host.
        reason: String,
    },
    /// The URL targets a blocked or prohibited network port.
    BlockedPort {
        /// The rejected network port.
        port: u16,
        /// Reason why the port is prohibited.
        reason: String,
    },
    /// The URL lacks a valid network host component.
    MissingHost,
    /// The URL contains embedded null bytes or forbidden control characters.
    InvalidCharacters {
        /// Description of the invalid character violation.
        details: String,
    },
}

impl fmt::Display for UrlSecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLong { actual, maximum } => {
                write!(
                    f,
                    "URL length ({actual} chars) exceeds maximum allowed ({maximum} chars)"
                )
            }
            Self::ForbiddenScheme { scheme, reason } => {
                write!(f, "Forbidden URL scheme '{scheme}': {reason}")
            }
            Self::ParseError { details } => {
                write!(f, "Invalid URL syntax: {details}")
            }
            Self::UserinfoForbidden => {
                write!(
                    f,
                    "URL userinfo (embedded username/password) is strictly forbidden"
                )
            }
            Self::BlockedIp { ip, category } => {
                write!(f, "SSRF Defense: IP '{ip}' is blocked ({category})")
            }
            Self::BlockedHost { host, reason } => {
                write!(f, "SSRF Defense: Host '{host}' is blocked ({reason})")
            }
            Self::BlockedPort { port, reason } => {
                write!(f, "SSRF Defense: Port {port} is blocked ({reason})")
            }
            Self::MissingHost => {
                write!(f, "URL is missing a valid network hostname")
            }
            Self::InvalidCharacters { details } => {
                write!(f, "URL contains invalid characters: {details}")
            }
        }
    }
}

impl std::error::Error for UrlSecurityError {}

impl From<UrlSecurityError> for CoreError {
    fn from(err: UrlSecurityError) -> Self {
        match err {
            UrlSecurityError::BlockedIp { ip, category } => {
                CoreError::Security(format!("SSRF blocked IP '{ip}': {category}"))
            }
            UrlSecurityError::BlockedHost { host, reason } => {
                CoreError::Security(format!("SSRF blocked host '{host}': {reason}"))
            }
            UrlSecurityError::BlockedPort { port, reason } => {
                CoreError::Security(format!("SSRF blocked port {port}: {reason}"))
            }
            UrlSecurityError::ForbiddenScheme { scheme, reason } => {
                CoreError::Security(format!("Forbidden scheme '{scheme}': {reason}"))
            }
            UrlSecurityError::UserinfoForbidden => {
                CoreError::Security("URL userinfo credentials are not permitted".to_string())
            }
            UrlSecurityError::TooLong { actual, maximum } => CoreError::Validation {
                field: "url".to_string(),
                reason: format!("URL length ({actual}) exceeds limit ({maximum})"),
            },
            UrlSecurityError::ParseError { details } => CoreError::Validation {
                field: "url".to_string(),
                reason: format!("Invalid URL: {details}"),
            },
            UrlSecurityError::MissingHost => CoreError::Validation {
                field: "url".to_string(),
                reason: "URL must contain a valid hostname".to_string(),
            },
            UrlSecurityError::InvalidCharacters { details } => CoreError::Validation {
                field: "url".to_string(),
                reason: details,
            },
        }
    }
}
