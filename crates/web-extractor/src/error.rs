//! # Web Extractor Errors
//!
//! Strongly-typed domain errors for HTTP acquisition, streaming, redirect handling, and content extraction.

use std::fmt;
use web_repo_to_ai_core::error::CoreError;
use web_repo_to_ai_url_security::UrlSecurityError;

/// Error conditions encountered during web content acquisition, redirect revalidation, and extraction.
#[derive(Debug, Clone, PartialEq)]
pub enum WebExtractorError {
    /// URL security or SSRF validation failure.
    UrlSecurity(UrlSecurityError),
    /// Underlying HTTP protocol or network transport failure.
    Http(String),
    /// Cyclic redirect loop detected during acquisition.
    RedirectLoop {
        /// The repeated target URL where the loop occurred.
        url: String,
        /// Number of hops traversed before detecting the loop.
        hops: usize,
    },
    /// The number of redirect hops exceeded the configured policy limit.
    TooManyRedirects {
        /// Maximum allowed redirect hops.
        limit: usize,
    },
    /// A redirect HTTP status code was returned without a `Location` header.
    MissingLocationHeader {
        /// HTTP status code (e.g. 301, 302).
        status: u16,
        /// Current URL that returned the redirect status.
        url: String,
    },
    /// The `Location` header could not be parsed or joined to the base URL.
    InvalidRedirectUrl {
        /// The raw `Location` header value.
        location: String,
        /// Reason why parsing failed.
        details: String,
    },
    /// The streaming response body exceeded the maximum configured byte ceiling.
    PayloadTooLarge {
        /// Actual bytes read before exceeding ceiling.
        actual: usize,
        /// Maximum allowed byte ceiling.
        maximum: usize,
    },
    /// The HTTP operation timed out.
    Timeout {
        /// Contextual timeout details.
        details: String,
    },
    /// Network connection or transport layer error.
    Network {
        /// Error description.
        details: String,
    },
    /// Decompression bomb detected: expansion ratio exceeded the configured safety threshold.
    DecompressionBomb {
        /// Observed expansion ratio.
        ratio: f64,
        /// Maximum allowed expansion ratio.
        max_ratio: f64,
        /// Compressed bytes consumed.
        compressed_bytes: usize,
        /// Decompressed bytes produced.
        decompressed_bytes: usize,
    },
    /// Decompression failed due to corrupt data or decoder error.
    DecompressionFailed {
        /// Content encoding format (e.g. gzip, brotli, zstd).
        encoding: String,
        /// Underlying failure details.
        details: String,
    },
    /// The response declared an unsupported Content-Encoding.
    UnsupportedContentEncoding {
        /// The unsupported encoding token.
        encoding: String,
    },
    /// Mismatch between declared Content-Length header and actual bytes read.
    ContentLengthMismatch {
        /// Declared Content-Length in header.
        expected: u64,
        /// Actual raw bytes read from the network.
        actual: u64,
    },
    /// HTML DOM parsing failure.
    HtmlParseFailed(String),
    /// Content extraction or readability processing failure.
    ExtractionFailed {
        /// Failure details.
        details: String,
    },
}

impl fmt::Display for WebExtractorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UrlSecurity(err) => write!(f, "URL Security Error: {err}"),
            Self::Http(err) => write!(f, "HTTP error: {err}"),
            Self::HtmlParseFailed(details) => write!(f, "HTML parse failed: {details}"),
            Self::ExtractionFailed { details } => write!(f, "Extraction failed: {details}"),
            Self::RedirectLoop { url, hops } => {
                write!(f, "Redirect loop detected at '{url}' after {hops} hops")
            }
            Self::TooManyRedirects { limit } => {
                write!(f, "Redirect limit exceeded ({limit} hops)")
            }
            Self::MissingLocationHeader { status, url } => {
                write!(
                    f,
                    "Redirect response ({status}) missing Location header at '{url}'"
                )
            }
            Self::InvalidRedirectUrl { location, details } => {
                write!(f, "Invalid redirect Location '{location}': {details}")
            }
            Self::PayloadTooLarge { actual, maximum } => {
                write!(
                    f,
                    "Payload size ({actual} bytes) exceeded maximum ceiling ({maximum} bytes)"
                )
            }
            Self::Timeout { details } => write!(f, "HTTP request timed out: {details}"),
            Self::Network { details } => write!(f, "Network error: {details}"),
            Self::DecompressionBomb {
                ratio,
                max_ratio,
                compressed_bytes,
                decompressed_bytes,
            } => write!(
                f,
                "Decompression bomb detected: expansion ratio {ratio:.2} exceeds limit {max_ratio:.2} ({decompressed_bytes} decompressed bytes from {compressed_bytes} compressed bytes)"
            ),
            Self::DecompressionFailed { encoding, details } => {
                write!(f, "Decompression failed for '{encoding}': {details}")
            }
            Self::UnsupportedContentEncoding { encoding } => {
                write!(f, "Unsupported Content-Encoding: '{encoding}'")
            }
            Self::ContentLengthMismatch { expected, actual } => write!(
                f,
                "Content-Length mismatch: expected {expected} bytes, received {actual} bytes"
            ),
        }
    }
}

impl std::error::Error for WebExtractorError {}

impl From<UrlSecurityError> for WebExtractorError {
    fn from(err: UrlSecurityError) -> Self {
        Self::UrlSecurity(err)
    }
}

impl From<reqwest::Error> for WebExtractorError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout {
                details: err.to_string(),
            }
        } else if err.is_connect() {
            Self::Network {
                details: format!("Connection failed: {err}"),
            }
        } else {
            Self::Http(err.to_string())
        }
    }
}

impl From<WebExtractorError> for CoreError {
    fn from(err: WebExtractorError) -> Self {
        match err {
            WebExtractorError::UrlSecurity(sec) => CoreError::from(sec),
            WebExtractorError::RedirectLoop { url, hops } => CoreError::Security(format!(
                "Redirect loop detected targeting '{url}' after {hops} hops"
            )),
            WebExtractorError::TooManyRedirects { limit } => CoreError::Validation {
                field: "redirects".to_string(),
                reason: format!("Redirect limit of {limit} exceeded"),
            },
            WebExtractorError::PayloadTooLarge { actual, maximum } => CoreError::Validation {
                field: "body".to_string(),
                reason: format!("Payload size ({actual} bytes) exceeds limit ({maximum} bytes)"),
            },
            WebExtractorError::MissingLocationHeader { status, url } => CoreError::Extraction(
                format!("HTTP {status} redirect missing Location header from '{url}'"),
            ),
            WebExtractorError::InvalidRedirectUrl { location, details } => CoreError::Validation {
                field: "location".to_string(),
                reason: format!("Invalid redirect Location '{location}': {details}"),
            },
            WebExtractorError::Timeout { details } => {
                CoreError::Internal(format!("HTTP timeout: {details}"))
            }
            WebExtractorError::Network { details } => {
                CoreError::Internal(format!("Network error: {details}"))
            }
            WebExtractorError::Http(details) => {
                CoreError::Internal(format!("HTTP error: {details}"))
            }
            WebExtractorError::DecompressionBomb {
                ratio,
                max_ratio,
                compressed_bytes,
                decompressed_bytes,
            } => CoreError::Security(format!(
                "Decompression bomb blocked: ratio {ratio:.2} > {max_ratio:.2} ({decompressed_bytes} decompressed bytes from {compressed_bytes} compressed bytes)"
            )),
            WebExtractorError::DecompressionFailed { encoding, details } => {
                CoreError::Extraction(format!("Decompression failed for {encoding}: {details}"))
            }
            WebExtractorError::UnsupportedContentEncoding { encoding } => {
                CoreError::Validation {
                    field: "content-encoding".to_string(),
                    reason: format!("Unsupported Content-Encoding: '{encoding}'"),
                }
            }
            WebExtractorError::ContentLengthMismatch { expected, actual } => {
                CoreError::Validation {
                    field: "content-length".to_string(),
                    reason: format!("Content-Length mismatch: expected {expected}, actual {actual}"),
                }
            }
            WebExtractorError::HtmlParseFailed(details) => {
                CoreError::Extraction(format!("HTML parse error: {details}"))
            }
            WebExtractorError::ExtractionFailed { details } => {
                CoreError::Extraction(format!("Extraction error: {details}"))
            }
        }
    }
}
