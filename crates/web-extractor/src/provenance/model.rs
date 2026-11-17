//! # Source Attribution and Provenance Models
//!
//! Strongly-typed domain representations for source attribution, HTTP headers,
//! cryptographic content hashes, JSON-LD structured data, and citations.

use std::collections::HashMap;
use std::net::IpAddr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::browser::model::{BlockedSubresource, BrowserTiming, ConsoleMessage, PageError};
use crate::browser::ExtractionMethod;
use crate::dedup::CanonicalResolution;

/// Cryptographic content hashes computed over raw response bytes and extracted clean text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentHashes {
    /// SHA-256 lowercase hex digest of the raw response markup bytes.
    pub raw_html_sha256: String,
    /// BLAKE3 lowercase hex digest of the raw response markup bytes.
    pub raw_html_blake3: String,
    /// SHA-256 lowercase hex digest of the extracted, cleaned plain text.
    pub cleaned_text_sha256: String,
    /// BLAKE3 lowercase hex digest of the extracted, cleaned plain text.
    pub cleaned_text_blake3: String,
}

impl ContentHashes {
    /// Computes dual SHA-256 and BLAKE3 digests over raw markup bytes and cleaned text.
    #[must_use]
    pub fn compute(raw_html: &[u8], cleaned_text: &str) -> Self {
        let mut sha256_hasher = Sha256::new();
        sha256_hasher.update(raw_html);
        let raw_html_sha256 = format!("{:x}", sha256_hasher.finalize());

        let raw_html_blake3 = blake3::hash(raw_html).to_hex().to_string();

        let mut text_sha256_hasher = Sha256::new();
        text_sha256_hasher.update(cleaned_text.as_bytes());
        let cleaned_text_sha256 = format!("{:x}", text_sha256_hasher.finalize());

        let cleaned_text_blake3 = blake3::hash(cleaned_text.as_bytes()).to_hex().to_string();

        Self {
            raw_html_sha256,
            raw_html_blake3,
            cleaned_text_sha256,
            cleaned_text_blake3,
        }
    }
}

/// A recorded redirect transition within an acquisition chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedirectHopRecord {
    /// Source URL before redirect hop.
    pub from_url: String,
    /// Destination URL specified by the redirect `Location` header.
    pub to_url: String,
    /// HTTP status code of the redirect (e.g. 301, 302, 307, 308).
    pub status_code: u16,
    /// Pinned IP address connected to for this hop if known.
    pub pinned_ip: Option<IpAddr>,
    /// Timestamp when this redirect hop occurred.
    pub timestamp: DateTime<Utc>,
}

/// Comprehensive HTTP transport provenance captured during content acquisition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpProvenance {
    /// Terminal HTTP response status code (e.g. 200, 304).
    pub status_code: u16,
    /// Standard HTTP status reason phrase (e.g. "OK").
    pub status_text: String,
    /// Preserved HTTP response headers normalized to lowercase keys.
    pub headers: HashMap<String, String>,
    /// Content-Type header value if present.
    pub content_type: Option<String>,
    /// Content-Length in bytes if present.
    pub content_length: Option<u64>,
    /// ETag HTTP validator if present.
    pub etag: Option<String>,
    /// Last-Modified timestamp header if present.
    pub last_modified: Option<String>,
    /// Cache-Control header if present.
    pub cache_control: Option<String>,
    /// Server header if present.
    pub server: Option<String>,
    /// Pinned IP address from which this response was retrieved.
    pub pinned_ip: Option<IpAddr>,
    /// Chronological list of redirect transitions leading to this response.
    pub redirect_history: Vec<RedirectHopRecord>,
}

/// Diagnostic browser execution details captured when dynamic rendering is invoked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserDiagnostics {
    /// Execution method used.
    pub method: ExtractionMethod,
    /// Performance and lifecycle timing metrics.
    pub timing: Option<BrowserTiming>,
    /// Console messages emitted by page scripts.
    pub console_messages: Vec<ConsoleMessage>,
    /// Uncaught JavaScript errors during page execution.
    pub page_errors: Vec<PageError>,
    /// Subresources blocked by SSRF/sandbox limits.
    pub blocked_subresources: Vec<BlockedSubresource>,
}

/// Severity classification for provenance warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningSeverity {
    /// Informational notice (e.g. missing optional header).
    Info,
    /// Potential inconsistency (e.g. canonical link differs from fetched URL).
    Warning,
    /// Significant extraction degradation (e.g. browser render timed out, degraded to static).
    Critical,
}

/// Diagnostic warning or anomaly detected during acquisition, parsing, or metadata capture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceWarning {
    /// Machine-readable warning code (e.g. "CANONICAL_URL_MISMATCH", "MISSING_ETAG").
    pub code: String,
    /// Human-readable diagnostic explanation.
    pub message: String,
    /// Diagnostic severity level.
    pub severity: WarningSeverity,
}

impl ProvenanceWarning {
    /// Constructs a new provenance warning.
    #[must_use]
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        severity: WarningSeverity,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity,
        }
    }
}

/// Comprehensive metadata extracted from HTML `<head>`, OpenGraph, Twitter Cards, and Schema.org JSON-LD.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CapturedMetadata {
    /// Best-effort resolved title following standard precedence rules.
    pub title: Option<String>,
    /// Best-effort resolved meta description.
    pub description: Option<String>,
    /// Resolved author or byline.
    pub author: Option<String>,
    /// Resolved publication date/time string.
    pub published_date: Option<String>,
    /// Resolved last modification date/time string.
    pub modified_date: Option<String>,
    /// Document natural language (e.g. "en-US").
    pub language: Option<String>,
    /// Site or publisher name.
    pub site_name: Option<String>,
    /// List of keyword tags.
    pub keywords: Vec<String>,
    /// License or copyright specification if present.
    pub license: Option<String>,
    /// Canonical document URL declared via `<link rel="canonical">`.
    pub canonical_link: Option<String>,
    /// Key-value map of OpenGraph tags (`og:*`).
    pub opengraph: HashMap<String, String>,
    /// Key-value map of Twitter Card tags (`twitter:*`).
    pub twitter: HashMap<String, String>,
    /// Raw Schema.org JSON-LD blocks extracted from `<script type="application/ld+json">`.
    pub json_ld: Vec<String>,
}

/// Generated citations and attribution references for downstream AI context ingestion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceCitations {
    /// Single-line concise source citation (e.g. `Source: https://example.com/docs (2026-11-17)`).
    pub concise: String,
    /// Markdown callout block format suitable for AI context document headers.
    pub markdown_callout: String,
    /// Plaintext formatted citation block.
    pub plaintext: String,
}

/// Complete provenance, source attribution, cryptographic hashes, and metadata for an extracted webpage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceAttribution {
    /// The terminal canonical URL resolved for this webpage.
    pub canonical_url: String,
    /// Detailed canonical URL resolution signal and confidence metrics.
    pub canonical_resolution: CanonicalResolution,
    /// The original requested URL provided by the caller.
    pub initial_url: String,
    /// UTC timestamp when content acquisition completed.
    pub retrieved_at: DateTime<Utc>,
    /// Total elapsed duration in milliseconds for acquisition, rendering, and parsing.
    pub load_duration_ms: u64,
    /// The terminal extraction method used (Static vs DynamicBrowser).
    pub extraction_method: ExtractionMethod,
    /// HTTP transport provenance if static acquisition occurred.
    pub http_provenance: Option<HttpProvenance>,
    /// Dual SHA-256 and BLAKE3 cryptographic hashes.
    pub content_hashes: ContentHashes,
    /// Comprehensive metadata extracted from HTML markup and Schema.org.
    pub metadata: CapturedMetadata,
    /// Browser diagnostics if dynamic browser rendering occurred.
    pub browser_diagnostics: Option<BrowserDiagnostics>,
    /// Formatted citation blocks.
    pub citations: ProvenanceCitations,
    /// Diagnostic warnings and anomaly reports.
    pub warnings: Vec<ProvenanceWarning>,
}

impl SourceAttribution {
    /// Returns true if any diagnostic warnings are present.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Returns a list of all warning codes.
    #[must_use]
    pub fn warning_codes(&self) -> Vec<&str> {
        self.warnings.iter().map(|w| w.code.as_str()).collect()
    }

    /// Returns a reference to the markdown citation callout block.
    #[must_use]
    pub fn markdown_citation(&self) -> &str {
        &self.citations.markdown_callout
    }
}
