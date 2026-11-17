//! # ZIP Archive Package Generator (`crates/exporters/src/zip.rs`)
//!
//! Generates standardized, compressed `.zip` distribution packages containing:
//! - Multi-format context exports (`context.md`, `context.txt`, `context.json`, `document.json`, `context.html`, `context.jsonl`)
//! - Individual source files extracted into directory hierarchies (`sources/{path}`)
//! - Package metadata manifests (`manifest.json`)
//! - Provenance reports (`provenance.json`)
//! - Completeness and quality reports (`quality-report.json`)
//! - Coreutils-compatible SHA-256 checksums (`checksums.sha256`)

use std::io::{Cursor, Seek, Write};

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use web_repo_to_ai_cdm::{CdmDocument, Citation, ExclusionRecord, RevisionInfo, SourceIdentity};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::error::{ExporterError, Result};
use crate::html::export_html;
use crate::json::export_json;
use crate::jsonl::export_jsonl;
use crate::markdown::export_markdown;
use crate::options::{JsonExportOptions, ZipPackageMetrics, ZipPackageOptions};
use crate::plain_text::export_plain_text;

/// Entry record for a file within `manifest.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageFileEntry {
    /// Relative archive path (e.g. `"context.md"` or `"sources/src/lib.rs"`).
    pub path: String,
    /// Uncompressed file size in bytes.
    pub size_bytes: u64,
    /// Cryptographic SHA-256 hex digest of file contents.
    pub sha256: String,
    /// Format descriptor (e.g. `"markdown"`, `"json"`, `"source_code"`).
    pub format: String,
}

/// Root schema for `manifest.json` embedded in the package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManifest {
    /// Manifest schema specification version.
    pub manifest_version: String,
    /// Name and version of the package generating engine.
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
    /// Total count of files contained in this archive.
    pub file_count: usize,
    /// Detailed listing of archived files.
    pub files: Vec<PackageFileEntry>,
}

/// Root schema for `provenance.json` embedded in the package.
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

/// Root schema for `quality-report.json` embedded in the package.
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

/// High-performance deterministic ZIP package exporter.
#[derive(Debug, Clone, Default)]
pub struct ZipPackageExporter {
    options: ZipPackageOptions,
}

impl ZipPackageExporter {
    /// Creates a new ZIP exporter with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a ZIP exporter with custom options.
    #[must_use]
    pub const fn with_options(options: ZipPackageOptions) -> Self {
        Self { options }
    }

    /// Exports the CDM document into an in-memory ZIP byte vector.
    ///
    /// # Errors
    /// Returns `ExporterError` on generation failure.
    pub fn export_to_bytes(&self, doc: &CdmDocument) -> Result<(Vec<u8>, ZipPackageMetrics)> {
        export_zip_package(doc, &self.options)
    }

    /// Streams the ZIP archive directly to any `Write + Seek` destination.
    ///
    /// # Errors
    /// Returns `ExporterError` on write failure.
    pub fn export_to_writer<W: Write + Seek>(
        &self,
        doc: &CdmDocument,
        writer: W,
    ) -> Result<ZipPackageMetrics> {
        export_zip_package_to_writer(doc, &self.options, writer)
    }
}

/// Computes the SHA-256 hex digest of a byte slice.
fn compute_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Sanitizes a file path to prevent directory traversal attacks in ZIP archives.
fn sanitize_zip_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let segments: Vec<&str> = normalized
        .split('/')
        .filter(|seg| !seg.is_empty() && *seg != "." && *seg != "..")
        .collect();
    if segments.is_empty() {
        "unnamed_file".to_string()
    } else {
        segments.join("/")
    }
}

fn map_zip_err(e: impl std::error::Error + Send + Sync + 'static) -> ExporterError {
    ExporterError::Io(std::io::Error::other(e))
}

/// Exports a CDM document into an in-memory ZIP package.
///
/// # Errors
/// Returns `ExporterError` on packaging failure.
pub fn export_zip_package(
    doc: &CdmDocument,
    options: &ZipPackageOptions,
) -> Result<(Vec<u8>, ZipPackageMetrics)> {
    let mut buffer = Cursor::new(Vec::with_capacity(64 * 1024));
    let mut metrics = export_zip_package_to_writer(doc, options, &mut buffer)?;
    let bytes = buffer.into_inner();
    metrics.total_bytes_written = bytes.len();
    metrics.sha256_checksum = compute_sha256(&bytes);
    Ok((bytes, metrics))
}

/// Streams a CDM document into a ZIP package writer.
///
/// # Errors
/// Returns `ExporterError` if archive writing fails.
pub fn export_zip_package_to_writer<W: Write + Seek>(
    doc: &CdmDocument,
    options: &ZipPackageOptions,
    writer: W,
) -> Result<ZipPackageMetrics> {
    let mut zip = ZipWriter::new(writer);
    let mut metrics = ZipPackageMetrics::default();
    let mut file_entries = Vec::new();

    // Deterministic timestamp: 2026-01-01 00:00:00 for reproducible zip builds
    let file_opts = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // 1. Markdown Export
    if options.include_markdown {
        let md_str = export_markdown(doc, &options.markdown_options)?;
        let md_bytes = md_str.into_bytes();
        let sha = compute_sha256(&md_bytes);
        file_entries.push(PackageFileEntry {
            path: options.markdown_path.clone(),
            size_bytes: md_bytes.len() as u64,
            sha256: sha,
            format: "markdown".to_string(),
        });
        zip.start_file(&options.markdown_path, file_opts)
            .map_err(map_zip_err)?;
        zip.write_all(&md_bytes)?;
        metrics.uncompressed_bytes += md_bytes.len() as u64;
    }

    // 2. Plain-Text Export
    if options.include_plain_text {
        let txt_str = export_plain_text(doc, &options.plain_text_options)?;
        let txt_bytes = txt_str.into_bytes();
        let sha = compute_sha256(&txt_bytes);
        file_entries.push(PackageFileEntry {
            path: options.plain_text_path.clone(),
            size_bytes: txt_bytes.len() as u64,
            sha256: sha,
            format: "plain_text".to_string(),
        });
        zip.start_file(&options.plain_text_path, file_opts)
            .map_err(map_zip_err)?;
        zip.write_all(&txt_bytes)?;
        metrics.uncompressed_bytes += txt_bytes.len() as u64;
    }

    // 3. Canonical CDM JSON Export
    if options.include_json_canonical {
        let json_opts = JsonExportOptions::canonical();
        let json_str = export_json(doc, &json_opts)?;
        let json_bytes = json_str.into_bytes();
        let sha = compute_sha256(&json_bytes);
        file_entries.push(PackageFileEntry {
            path: options.json_canonical_path.clone(),
            size_bytes: json_bytes.len() as u64,
            sha256: sha,
            format: "cdm_json_schema_v1".to_string(),
        });
        zip.start_file(&options.json_canonical_path, file_opts)
            .map_err(map_zip_err)?;
        zip.write_all(&json_bytes)?;
        metrics.uncompressed_bytes += json_bytes.len() as u64;
    }

    // 4. Structured Context JSON Export
    if options.include_json_structured {
        let json_opts = JsonExportOptions::structured();
        let json_str = export_json(doc, &json_opts)?;
        let json_bytes = json_str.into_bytes();
        let sha = compute_sha256(&json_bytes);
        file_entries.push(PackageFileEntry {
            path: options.json_structured_path.clone(),
            size_bytes: json_bytes.len() as u64,
            sha256: sha,
            format: "structured_context_json".to_string(),
        });
        zip.start_file(&options.json_structured_path, file_opts)
            .map_err(map_zip_err)?;
        zip.write_all(&json_bytes)?;
        metrics.uncompressed_bytes += json_bytes.len() as u64;
    }

    // 5. HTML Export
    if options.include_html {
        let (html_str, _) = export_html(doc, &options.html_options)?;
        let html_bytes = html_str.into_bytes();
        let sha = compute_sha256(&html_bytes);
        file_entries.push(PackageFileEntry {
            path: options.html_path.clone(),
            size_bytes: html_bytes.len() as u64,
            sha256: sha,
            format: "html".to_string(),
        });
        zip.start_file(&options.html_path, file_opts)
            .map_err(map_zip_err)?;
        zip.write_all(&html_bytes)?;
        metrics.uncompressed_bytes += html_bytes.len() as u64;
    }

    // 6. JSONL Export
    if options.include_jsonl {
        let (jsonl_str, _) = export_jsonl(doc, &options.jsonl_options)?;
        let jsonl_bytes = jsonl_str.into_bytes();
        let sha = compute_sha256(&jsonl_bytes);
        file_entries.push(PackageFileEntry {
            path: options.jsonl_path.clone(),
            size_bytes: jsonl_bytes.len() as u64,
            sha256: sha,
            format: "jsonl_rag_chunks".to_string(),
        });
        zip.start_file(&options.jsonl_path, file_opts)
            .map_err(map_zip_err)?;
        zip.write_all(&jsonl_bytes)?;
        metrics.uncompressed_bytes += jsonl_bytes.len() as u64;
    }

    // 7. Individual Extracted Source Files
    if options.include_individual_files {
        for (sec_idx, sec) in doc.sections.iter().enumerate() {
            let relative_path = sec
                .original_path
                .as_deref()
                .unwrap_or_else(|| sec.title.as_deref().unwrap_or("section"));
            let sanitized = sanitize_zip_path(relative_path);
            let archive_path = format!("{}{sanitized}", options.individual_files_dir);

            // De-duplicate if multiple sections have the same path
            let final_path = if file_entries.iter().any(|e| e.path == archive_path) {
                format!("{archive_path}.{sec_idx}")
            } else {
                archive_path
            };

            let body_bytes = sec.body.as_bytes();
            let sha = compute_sha256(body_bytes);
            file_entries.push(PackageFileEntry {
                path: final_path.clone(),
                size_bytes: body_bytes.len() as u64,
                sha256: sha,
                format: sec.language.clone().unwrap_or_else(|| "text".to_string()),
            });

            zip.start_file(&final_path, file_opts)
                .map_err(map_zip_err)?;
            zip.write_all(body_bytes)?;
            metrics.uncompressed_bytes += body_bytes.len() as u64;
        }
    }

    // 8. Provenance Report (`provenance.json`)
    if options.include_provenance {
        let prov = PackageProvenance {
            document_id: doc.document_id.to_string(),
            canonical_url: doc.canonical_url.clone(),
            source_identity: doc.source_identity.clone(),
            source_type: doc.source_type.as_str().to_string(),
            retrieved_at: doc.retrieved_at.to_rfc3339_opts(SecondsFormat::Secs, true),
            revision: doc.revision.clone(),
            citations_count: doc.citations.len(),
            citations: doc.citations.clone(),
        };
        let prov_bytes = serde_json::to_vec_pretty(&prov)?;
        let sha = compute_sha256(&prov_bytes);
        file_entries.push(PackageFileEntry {
            path: "provenance.json".to_string(),
            size_bytes: prov_bytes.len() as u64,
            sha256: sha,
            format: "json_provenance".to_string(),
        });
        zip.start_file("provenance.json", file_opts)
            .map_err(map_zip_err)?;
        zip.write_all(&prov_bytes)?;
        metrics.uncompressed_bytes += prov_bytes.len() as u64;
    }

    // 9. Completeness & Quality Report (`quality-report.json`)
    if options.include_quality_report {
        let comp = &doc.completeness;
        let qr = PackageQualityReport {
            document_id: doc.document_id.to_string(),
            completeness_ratio: comp.extraction_ratio(),
            completeness_percentage: format!("{:.1}%", comp.extraction_ratio() * 100.0),
            discovered_count: comp.discovered_count,
            extracted_count: comp.extracted_count,
            excluded_count: comp.excluded_count,
            failed_count: comp.failed_count,
            unsupported_count: comp.unsupported_count,
            exclusions_count: doc.exclusions.len(),
            exclusions: doc.exclusions.clone(),
        };
        let qr_bytes = serde_json::to_vec_pretty(&qr)?;
        let sha = compute_sha256(&qr_bytes);
        file_entries.push(PackageFileEntry {
            path: "quality-report.json".to_string(),
            size_bytes: qr_bytes.len() as u64,
            sha256: sha,
            format: "json_quality_report".to_string(),
        });
        zip.start_file("quality-report.json", file_opts)
            .map_err(map_zip_err)?;
        zip.write_all(&qr_bytes)?;
        metrics.uncompressed_bytes += qr_bytes.len() as u64;
    }

    // 10. Manifest (`manifest.json`)
    if options.include_manifest {
        let manifest = PackageManifest {
            manifest_version: "1.0.0".to_string(),
            package_generator: "web-repo-to-ai v0.1.0".to_string(),
            document_id: doc.document_id.to_string(),
            schema_version: doc.schema_version.clone(),
            created_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
            source_url: doc.canonical_url.clone(),
            source_type: doc.source_type.as_str().to_string(),
            file_count: file_entries.len() + if options.include_checksums { 2 } else { 1 },
            files: file_entries.clone(),
        };
        let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
        let sha = compute_sha256(&manifest_bytes);
        file_entries.push(PackageFileEntry {
            path: "manifest.json".to_string(),
            size_bytes: manifest_bytes.len() as u64,
            sha256: sha,
            format: "json_manifest".to_string(),
        });
        zip.start_file("manifest.json", file_opts)
            .map_err(map_zip_err)?;
        zip.write_all(&manifest_bytes)?;
        metrics.uncompressed_bytes += manifest_bytes.len() as u64;
    }

    // 11. Coreutils Checksums Manifest (`checksums.sha256`)
    if options.include_checksums {
        let mut checksums_content = String::new();
        for entry in &file_entries {
            checksums_content.push_str(&format!("{}  {}\n", entry.sha256, entry.path));
        }
        let cs_bytes = checksums_content.into_bytes();
        zip.start_file("checksums.sha256", file_opts)
            .map_err(map_zip_err)?;
        zip.write_all(&cs_bytes)?;
        metrics.uncompressed_bytes += cs_bytes.len() as u64;
        metrics.files_archived = file_entries.len() + 1;
    } else {
        metrics.files_archived = file_entries.len();
    }

    let finish_res = zip.finish().map_err(map_zip_err)?;
    drop(finish_res);

    Ok(metrics)
}
