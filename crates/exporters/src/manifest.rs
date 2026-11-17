//! # Package Manifest and Quality Report Generator (`crates/exporters/src/manifest.rs`)
//!
//! Provides standalone generation, serialization, and schema validation for:
//! - Package inventory manifest (`manifest.json`)
//! - Provenance metadata (`provenance.json`)
//! - Completeness & quality report (`quality-report.json`)
//! - MIME type detection and file entry statistics.

use std::io::Write;

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{CdmDocument, Citation, ExclusionRecord, RevisionInfo, SourceIdentity};

use crate::checksum::{compute_blake3_hex, compute_sha256_hex};
use crate::error::Result;
use crate::options::ManifestOptions;

/// Detects the standard IANA MIME type for a file path based on its extension.
#[must_use]
pub fn detect_mime_type(path: &str) -> &'static str {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".md") || lower.ends_with(".markdown") {
        "text/markdown; charset=utf-8"
    } else if lower.ends_with(".txt") || lower.ends_with(".text") {
        "text/plain; charset=utf-8"
    } else if lower.ends_with(".json") {
        "application/json"
    } else if lower.ends_with(".jsonl") || lower.ends_with(".ndjson") {
        "application/x-ndjson"
    } else if lower.ends_with(".html") || lower.ends_with(".htm") {
        "text/html; charset=utf-8"
    } else if lower.ends_with(".zip") {
        "application/zip"
    } else if lower.ends_with(".rs") {
        "text/x-rust; charset=utf-8"
    } else if lower.ends_with(".ts") {
        "application/typescript; charset=utf-8"
    } else if lower.ends_with(".js") || lower.ends_with(".mjs") || lower.ends_with(".cjs") {
        "application/javascript; charset=utf-8"
    } else if lower.ends_with(".py") {
        "text/x-python; charset=utf-8"
    } else if lower.ends_with(".go") {
        "text/x-go; charset=utf-8"
    } else if lower.ends_with(".c") || lower.ends_with(".h") {
        "text/x-c; charset=utf-8"
    } else if lower.ends_with(".cpp") || lower.ends_with(".cc") || lower.ends_with(".cxx") {
        "text/x-c++; charset=utf-8"
    } else if lower.ends_with(".java") {
        "text/x-java; charset=utf-8"
    } else if lower.ends_with(".yaml") || lower.ends_with(".yml") {
        "text/yaml; charset=utf-8"
    } else if lower.ends_with(".toml") {
        "application/toml"
    } else if lower.ends_with(".xml") {
        "application/xml; charset=utf-8"
    } else if lower.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if lower.ends_with(".csv") {
        "text/csv; charset=utf-8"
    } else {
        "text/plain; charset=utf-8"
    }
}

/// Detailed inventory record for an individual file entry in `manifest.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageFileEntry {
    /// Relative archive or export path (e.g. `"context.md"`, `"sources/src/main.rs"`).
    pub path: String,
    /// Uncompressed file size in bytes.
    pub size_bytes: u64,
    /// Cryptographic SHA-256 hex digest.
    pub sha256: String,
    /// Cryptographic BLAKE3 hex digest (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blake3: Option<String>,
    /// Format descriptor (e.g. `"markdown"`, `"json"`, `"source_code"`).
    pub format: String,
    /// Canonical IANA MIME media type.
    pub mime_type: String,
    /// Number of lines in the file (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_count: Option<usize>,
    /// Estimated LLM token count for the file contents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_estimate: Option<usize>,
}

impl PackageFileEntry {
    /// Constructs a file entry from raw bytes, calculating dual digests and statistics.
    #[must_use]
    pub fn from_bytes(
        path: impl Into<String>,
        bytes: &[u8],
        format: impl Into<String>,
        include_blake3: bool,
    ) -> Self {
        let path_str = path.into();
        let mime = detect_mime_type(&path_str).to_string();
        let sha256 = compute_sha256_hex(bytes);
        let blake3 = if include_blake3 {
            Some(compute_blake3_hex(bytes))
        } else {
            None
        };

        // Textual metrics
        let line_count = if mime.starts_with("text/")
            || mime.starts_with("application/json")
            || mime.starts_with("application/x-ndjson")
            || mime.starts_with("application/javascript")
            || mime.starts_with("application/typescript")
        {
            Some(bytecount_lines(bytes))
        } else {
            None
        };

        let token_estimate = if line_count.is_some() {
            Some(bytes.len().div_ceil(4))
        } else {
            None
        };

        Self {
            path: path_str,
            size_bytes: bytes.len() as u64,
            sha256,
            blake3,
            format: format.into(),
            mime_type: mime,
            line_count,
            token_estimate,
        }
    }
}

fn bytecount_lines(bytes: &[u8]) -> usize {
    if bytes.is_empty() {
        return 0;
    }
    let newlines = bytes.iter().filter(|&&b| b == b'\n').count();
    if bytes.last() == Some(&b'\n') {
        newlines
    } else {
        newlines + 1
    }
}

/// Root schema for `manifest.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManifest {
    /// Manifest schema specification version.
    pub manifest_version: String,
    /// Name and version of the package generator engine.
    pub package_generator: String,
    /// Parent document UUID identifier.
    pub document_id: String,
    /// CDM schema version.
    pub schema_version: String,
    /// UTC ISO-8601 creation timestamp.
    pub created_at: String,
    /// Authoritative canonical source URL.
    pub source_url: String,
    /// Source classification type.
    pub source_type: String,
    /// Total count of files contained in this package.
    pub file_count: usize,
    /// Total aggregate uncompressed size in bytes across all files.
    pub total_uncompressed_bytes: u64,
    /// Detailed listing of all files in the package.
    pub files: Vec<PackageFileEntry>,
}

/// Root schema for `provenance.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageProvenance {
    /// Document UUID identifier.
    pub document_id: String,
    /// Canonical source URL.
    pub canonical_url: String,
    /// Detailed source origin identity.
    pub source_identity: SourceIdentity,
    /// Source classification type.
    pub source_type: String,
    /// UTC timestamp of acquisition.
    pub retrieved_at: String,
    /// Git revision and cache validation hashes.
    pub revision: RevisionInfo,
    /// Total count of authoritative citations.
    pub citations_count: usize,
    /// List of provenance citations.
    pub citations: Vec<Citation>,
}

/// Root schema for `quality-report.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageQualityReport {
    /// Document UUID identifier.
    pub document_id: String,
    /// Numerical extraction yield ratio (0.0 to 1.0).
    pub completeness_ratio: f64,
    /// Formatted percentage string (e.g. `"100.0%"`).
    pub completeness_percentage: String,
    /// Items discovered during crawl or tree traversal.
    pub discovered_count: u64,
    /// Items successfully converted into CDM sections.
    pub extracted_count: u64,
    /// Items skipped due to rules, size, or binary filters.
    pub excluded_count: u64,
    /// Items that failed during acquisition or extraction.
    pub failed_count: u64,
    /// Items skipped due to unsupported media types.
    pub unsupported_count: u64,
    /// Total count of exclusion records.
    pub exclusions_count: usize,
    /// Detailed auditable exclusion logs.
    pub exclusions: Vec<ExclusionRecord>,
}

/// Builds a [`PackageManifest`] from a [`CdmDocument`] and an inventory of files.
#[must_use]
pub fn build_manifest(doc: &CdmDocument, files: Vec<PackageFileEntry>) -> PackageManifest {
    let total_uncompressed_bytes = files.iter().map(|f| f.size_bytes).sum();
    PackageManifest {
        manifest_version: "1.0.0".to_string(),
        package_generator: "web-repo-to-ai v0.1.0".to_string(),
        document_id: doc.document_id.to_string(),
        schema_version: doc.schema_version.clone(),
        created_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        source_url: doc.canonical_url.clone(),
        source_type: doc.source_type.as_str().to_string(),
        file_count: files.len(),
        total_uncompressed_bytes,
        files,
    }
}

/// Builds a [`PackageProvenance`] report from a [`CdmDocument`].
#[must_use]
pub fn build_provenance(doc: &CdmDocument) -> PackageProvenance {
    PackageProvenance {
        document_id: doc.document_id.to_string(),
        canonical_url: doc.canonical_url.clone(),
        source_identity: doc.source_identity.clone(),
        source_type: doc.source_type.as_str().to_string(),
        retrieved_at: doc.retrieved_at.to_rfc3339_opts(SecondsFormat::Secs, true),
        revision: doc.revision.clone(),
        citations_count: doc.citations.len(),
        citations: doc.citations.clone(),
    }
}

/// Builds a [`PackageQualityReport`] from a [`CdmDocument`].
#[must_use]
pub fn build_quality_report(doc: &CdmDocument) -> PackageQualityReport {
    let comp = &doc.completeness;
    let ratio = comp.extraction_ratio();
    PackageQualityReport {
        document_id: doc.document_id.to_string(),
        completeness_ratio: ratio,
        completeness_percentage: format!("{:.1}%", ratio * 100.0),
        discovered_count: comp.discovered_count,
        extracted_count: comp.extracted_count,
        excluded_count: comp.excluded_count,
        failed_count: comp.failed_count,
        unsupported_count: comp.unsupported_count,
        exclusions_count: doc.exclusions.len(),
        exclusions: doc.exclusions.clone(),
    }
}

/// Serializes a [`PackageManifest`] to a JSON string.
///
/// # Errors
/// Returns [`crate::error::ExporterError::Json`] on serialization failure.
pub fn export_manifest(manifest: &PackageManifest, pretty: bool) -> Result<String> {
    if pretty {
        serde_json::to_string_pretty(manifest).map_err(Into::into)
    } else {
        serde_json::to_string(manifest).map_err(Into::into)
    }
}

/// Streams a [`PackageManifest`] directly to a writer.
///
/// # Errors
/// Returns [`crate::error::ExporterError::Json`] or [`crate::error::ExporterError::Io`] on failure.
pub fn export_manifest_to_writer<W: Write>(
    manifest: &PackageManifest,
    mut writer: W,
    pretty: bool,
) -> Result<usize> {
    let bytes = if pretty {
        serde_json::to_vec_pretty(manifest)?
    } else {
        serde_json::to_vec(manifest)?
    };
    writer.write_all(&bytes)?;
    Ok(bytes.len())
}

/// Serializes a [`PackageProvenance`] report to a JSON string.
///
/// # Errors
/// Returns [`crate::error::ExporterError::Json`] on serialization failure.
pub fn export_provenance(prov: &PackageProvenance, pretty: bool) -> Result<String> {
    if pretty {
        serde_json::to_string_pretty(prov).map_err(Into::into)
    } else {
        serde_json::to_string(prov).map_err(Into::into)
    }
}

/// Streams a [`PackageProvenance`] report directly to a writer.
///
/// # Errors
/// Returns [`crate::error::ExporterError::Json`] or [`crate::error::ExporterError::Io`] on failure.
pub fn export_provenance_to_writer<W: Write>(
    prov: &PackageProvenance,
    mut writer: W,
    pretty: bool,
) -> Result<usize> {
    let bytes = if pretty {
        serde_json::to_vec_pretty(prov)?
    } else {
        serde_json::to_vec(prov)?
    };
    writer.write_all(&bytes)?;
    Ok(bytes.len())
}

/// Serializes a [`PackageQualityReport`] to a JSON string.
///
/// # Errors
/// Returns [`crate::error::ExporterError::Json`] on serialization failure.
pub fn export_quality_report(report: &PackageQualityReport, pretty: bool) -> Result<String> {
    if pretty {
        serde_json::to_string_pretty(report).map_err(Into::into)
    } else {
        serde_json::to_string(report).map_err(Into::into)
    }
}

/// Streams a [`PackageQualityReport`] directly to a writer.
///
/// # Errors
/// Returns [`crate::error::ExporterError::Json`] or [`crate::error::ExporterError::Io`] on failure.
pub fn export_quality_report_to_writer<W: Write>(
    report: &PackageQualityReport,
    mut writer: W,
    pretty: bool,
) -> Result<usize> {
    let bytes = if pretty {
        serde_json::to_vec_pretty(report)?
    } else {
        serde_json::to_vec(report)?
    };
    writer.write_all(&bytes)?;
    Ok(bytes.len())
}

/// Fluent builder and exporter for package manifests.
#[derive(Debug, Clone, Default)]
pub struct ManifestExporter {
    options: ManifestOptions,
}

impl ManifestExporter {
    /// Creates a new manifest exporter with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a manifest exporter with specific options.
    #[must_use]
    pub const fn with_options(options: ManifestOptions) -> Self {
        Self { options }
    }

    /// Exports the manifest to a JSON string from a CDM document and files.
    ///
    /// # Errors
    /// Returns [`crate::error::ExporterError`] on serialization failure.
    pub fn export_to_string(
        &self,
        doc: &CdmDocument,
        files: Vec<PackageFileEntry>,
    ) -> Result<String> {
        let manifest = build_manifest(doc, files);
        export_manifest(&manifest, self.options.pretty_print)
    }

    /// Streams the manifest to a writer from a CDM document and files.
    ///
    /// # Errors
    /// Returns [`crate::error::ExporterError`] on failure.
    pub fn export_to_writer<W: Write>(
        &self,
        doc: &CdmDocument,
        files: Vec<PackageFileEntry>,
        writer: W,
    ) -> Result<usize> {
        let manifest = build_manifest(doc, files);
        export_manifest_to_writer(&manifest, writer, self.options.pretty_print)
    }
}
