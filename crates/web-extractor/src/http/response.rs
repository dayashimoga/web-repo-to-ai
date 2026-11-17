//! # Fetched HTTP Response and Provenance Representation
//!
//! Encapsulates the complete terminal response of an acquisition chain, including body bytes,
//! preserved HTTP headers, status metadata, and complete redirect provenance.

use std::borrow::Cow;
use std::collections::HashMap;
use std::net::IpAddr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use web_repo_to_ai_url_security::ValidatedUrl;

use super::redirect::RedirectHop;

/// The complete response payload and acquisition provenance of an HTTP fetch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchedResponse {
    /// Terminal canonical URL after following all redirect hops.
    pub url: ValidatedUrl,
    /// Originating requested URL before any redirect transitions.
    pub initial_url: ValidatedUrl,
    /// HTTP status code (e.g. 200, 404, 500).
    pub status: u16,
    /// Standard HTTP status reason phrase (e.g. "OK", "Not Found").
    pub status_text: String,
    /// Preserved HTTP response headers normalized to lowercase keys.
    pub headers: HashMap<String, String>,
    /// Content-Type header value if present.
    pub content_type: Option<String>,
    /// Content-Length in bytes if present.
    pub content_length: Option<u64>,
    /// Content-Encoding header value if present (e.g. "gzip", "br").
    pub content_encoding: Option<String>,
    /// Raw byte size read from the network before decompression.
    pub raw_body_size: usize,
    /// ETag HTTP validator if present.
    pub etag: Option<String>,
    /// Last-Modified timestamp header if present.
    pub last_modified: Option<String>,
    /// Chronological list of redirect transitions leading to this response.
    pub redirect_history: Vec<RedirectHop>,
    /// Raw response body bytes.
    pub body: Vec<u8>,
    /// Pinned IP address from which this terminal response was retrieved.
    pub pinned_ip: IpAddr,
    /// Timestamp when this response was received.
    pub fetched_at: DateTime<Utc>,
}

impl FetchedResponse {
    /// Returns a slice of the raw body bytes.
    #[must_use]
    pub fn body_bytes(&self) -> &[u8] {
        &self.body
    }

    /// Converts body bytes to a UTF-8 String, returning an error if invalid UTF-8.
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    /// Converts body bytes to a UTF-8 string lossily, replacing malformed sequences with U+FFFD.
    #[must_use]
    pub fn text_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.body)
    }

    /// Checks if the response status code indicates success (200..=299).
    #[must_use]
    pub fn is_success(&self) -> bool {
        (200..=299).contains(&self.status)
    }

    /// Checks if the response status code indicates a redirect (300..=399).
    #[must_use]
    pub fn is_redirect(&self) -> bool {
        (300..=399).contains(&self.status)
    }

    /// Checks if the Content-Type header indicates HTML content.
    #[must_use]
    pub fn is_html(&self) -> bool {
        if let Some(ct) = &self.content_type {
            let lower = ct.to_ascii_lowercase();
            lower.contains("text/html") || lower.contains("application/xhtml+xml")
        } else {
            false
        }
    }

    /// Checks if the Content-Type header indicates JSON content.
    #[must_use]
    pub fn is_json(&self) -> bool {
        if let Some(ct) = &self.content_type {
            let lower = ct.to_ascii_lowercase();
            lower.contains("application/json") || lower.contains("+json")
        } else {
            false
        }
    }

    /// Returns the number of redirect hops traversed.
    #[must_use]
    pub fn redirect_count(&self) -> usize {
        self.redirect_history.len()
    }

    /// Retrieves the value of a header by name (case-insensitive).
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        let lower = name.to_ascii_lowercase();
        self.headers.get(&lower).map(|s| s.as_str())
    }
}
