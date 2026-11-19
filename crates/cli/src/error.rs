//! # CLI Error Types & Deterministic Process Exit Codes

use std::fmt;
use web_repo_to_ai_context_optimizer::ContextOptimizerError;
use web_repo_to_ai_exporters::ExporterError;
use web_repo_to_ai_github_extractor::GitHubExtractorError;
use web_repo_to_ai_url_security::UrlSecurityError;
use web_repo_to_ai_web_extractor::WebExtractorError;

/// Deterministic process exit code representations per CLI specifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {
    /// Command completed successfully (0).
    Success = 0,
    /// Invalid arguments, syntax error, or unsupported option (1).
    Validation = 1,
    /// Network error, SSRF violation, DNS lookup failure, or blocked port (2).
    SecurityOrNetwork = 2,
    /// Extraction failure, HTML parsing error, or non-success HTTP status (3).
    Extraction = 3,
    /// Token budget or size constraint exceeded (4).
    BudgetExceeded = 4,
}

impl ExitCode {
    /// Returns the integer exit code.
    #[must_use]
    pub const fn code(self) -> i32 {
        self as i32
    }
}

/// Errors originating from CLI argument parsing, network extraction, or conversion.
#[derive(Debug)]
pub enum CliError {
    /// Validation error (invalid URL syntax, invalid option, etc.).
    Validation(String),
    /// Security / SSRF violation.
    Security(String),
    /// Network or DNS failure.
    Network(String),
    /// Content extraction or parsing failure.
    Extraction(String),
    /// Token budget constraint exceeded.
    Budget(String),
    /// Input/output error.
    Io(std::io::Error),
    /// Remote API error.
    Remote(String),
}

impl CliError {
    /// Returns the appropriate process exit code for this error.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Validation(_) | Self::Io(_) => ExitCode::Validation.code(),
            Self::Security(_) | Self::Network(_) => ExitCode::SecurityOrNetwork.code(),
            Self::Extraction(_) | Self::Remote(_) => ExitCode::Extraction.code(),
            Self::Budget(_) => ExitCode::BudgetExceeded.code(),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(msg) => write!(f, "Validation error: {msg}"),
            Self::Security(msg) => write!(f, "Security violation: {msg}"),
            Self::Network(msg) => write!(f, "Network error: {msg}"),
            Self::Extraction(msg) => write!(f, "Extraction error: {msg}"),
            Self::Budget(msg) => write!(f, "Budget exceeded: {msg}"),
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Remote(msg) => write!(f, "Remote API error: {msg}"),
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<UrlSecurityError> for CliError {
    fn from(err: UrlSecurityError) -> Self {
        match err {
            UrlSecurityError::ForbiddenScheme { .. }
            | UrlSecurityError::ParseError { .. }
            | UrlSecurityError::UserinfoForbidden
            | UrlSecurityError::TooLong { .. }
            | UrlSecurityError::MissingHost
            | UrlSecurityError::InvalidCharacters { .. } => Self::Validation(err.to_string()),

            UrlSecurityError::BlockedIp { .. }
            | UrlSecurityError::BlockedHost { .. }
            | UrlSecurityError::BlockedPort { .. } => Self::Security(err.to_string()),

            UrlSecurityError::DnsResolutionFailed { .. }
            | UrlSecurityError::DnsRebindingDetected { .. }
            | UrlSecurityError::IpPinningMismatch { .. } => Self::Network(err.to_string()),
        }
    }
}

impl From<WebExtractorError> for CliError {
    fn from(err: WebExtractorError) -> Self {
        match err {
            WebExtractorError::UrlSecurity(sec) => Self::from(sec),
            WebExtractorError::Http(msg) => Self::Network(msg),
            WebExtractorError::Timeout { details } => Self::Network(details),
            WebExtractorError::Network { details } => Self::Network(details),
            WebExtractorError::PayloadTooLarge { actual, maximum } => Self::Budget(format!(
                "Payload size ({actual} bytes) exceeds maximum ({maximum} bytes)"
            )),
            WebExtractorError::DecompressionBomb { .. } => Self::Security(err.to_string()),
            WebExtractorError::HtmlParseFailed(msg) => Self::Extraction(msg),
            WebExtractorError::ExtractionFailed { details } => Self::Extraction(details),
            other => Self::Extraction(other.to_string()),
        }
    }
}

impl From<GitHubExtractorError> for CliError {
    fn from(err: GitHubExtractorError) -> Self {
        match err {
            GitHubExtractorError::InvalidUrl(_)
            | GitHubExtractorError::ReservedPath(_)
            | GitHubExtractorError::UnsupportedHost(_)
            | GitHubExtractorError::EmbeddedCredentialsForbidden
            | GitHubExtractorError::InvalidOwner { .. }
            | GitHubExtractorError::InvalidRepository { .. }
            | GitHubExtractorError::InvalidRef { .. } => Self::Validation(err.to_string()),

            GitHubExtractorError::InvalidSubpath { path, reason } => {
                if reason.contains("Directory traversal") || path.contains("..") {
                    Self::Security(format!("Directory traversal blocked: {reason}"))
                } else {
                    Self::Validation(format!("Invalid subpath: {reason}"))
                }
            }

            GitHubExtractorError::UrlSecurity(sec) => Self::from(sec),

            GitHubExtractorError::BadCredentials { message }
            | GitHubExtractorError::Forbidden { message } => Self::Security(message),

            GitHubExtractorError::RateLimitExceeded { message, .. }
            | GitHubExtractorError::RequestFailed(message) => Self::Network(message),

            GitHubExtractorError::ResourceNotFound { message, .. }
            | GitHubExtractorError::HttpError { message, .. }
            | GitHubExtractorError::ArchiveError(message)
            | GitHubExtractorError::BlobError { message, .. } => Self::Extraction(message),

            other => Self::Extraction(other.to_string()),
        }
    }
}

impl From<ContextOptimizerError> for CliError {
    fn from(err: ContextOptimizerError) -> Self {
        match err {
            ContextOptimizerError::BudgetTooSmall { .. }
            | ContextOptimizerError::InvalidBudget { .. } => Self::Budget(err.to_string()),
            other => Self::Extraction(other.to_string()),
        }
    }
}

impl From<ExporterError> for CliError {
    fn from(err: ExporterError) -> Self {
        Self::Extraction(err.to_string())
    }
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        Self::Validation(err.to_string())
    }
}
