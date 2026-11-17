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

use web_repo_to_ai_cdm::CdmDocument;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

pub use crate::checksum::{compute_sha256_hex, ChecksumEntry};
use crate::error::{ExporterError, Result};
use crate::html::export_html;
use crate::json::export_json;
use crate::jsonl::export_jsonl;
pub use crate::manifest::{
    build_manifest, build_provenance, build_quality_report, PackageFileEntry, PackageManifest,
    PackageProvenance, PackageQualityReport,
};
use crate::markdown::export_markdown;
use crate::options::{JsonExportOptions, ZipPackageMetrics, ZipPackageOptions};
use crate::plain_text::export_plain_text;

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
    metrics.sha256_checksum = compute_sha256_hex(&bytes);
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
        file_entries.push(PackageFileEntry::from_bytes(
            &options.markdown_path,
            &md_bytes,
            "markdown",
            true,
        ));
        zip.start_file(&options.markdown_path, file_opts)
            .map_err(map_zip_err)?;
        zip.write_all(&md_bytes)?;
        metrics.uncompressed_bytes += md_bytes.len() as u64;
    }

    // 2. Plain-Text Export
    if options.include_plain_text {
        let txt_str = export_plain_text(doc, &options.plain_text_options)?;
        let txt_bytes = txt_str.into_bytes();
        file_entries.push(PackageFileEntry::from_bytes(
            &options.plain_text_path,
            &txt_bytes,
            "plain_text",
            true,
        ));
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
        file_entries.push(PackageFileEntry::from_bytes(
            &options.json_canonical_path,
            &json_bytes,
            "cdm_json_schema_v1",
            true,
        ));
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
        file_entries.push(PackageFileEntry::from_bytes(
            &options.json_structured_path,
            &json_bytes,
            "structured_context_json",
            true,
        ));
        zip.start_file(&options.json_structured_path, file_opts)
            .map_err(map_zip_err)?;
        zip.write_all(&json_bytes)?;
        metrics.uncompressed_bytes += json_bytes.len() as u64;
    }

    // 5. HTML Export
    if options.include_html {
        let (html_str, _) = export_html(doc, &options.html_options)?;
        let html_bytes = html_str.into_bytes();
        file_entries.push(PackageFileEntry::from_bytes(
            &options.html_path,
            &html_bytes,
            "html",
            true,
        ));
        zip.start_file(&options.html_path, file_opts)
            .map_err(map_zip_err)?;
        zip.write_all(&html_bytes)?;
        metrics.uncompressed_bytes += html_bytes.len() as u64;
    }

    // 6. JSONL Export
    if options.include_jsonl {
        let (jsonl_str, _) = export_jsonl(doc, &options.jsonl_options)?;
        let jsonl_bytes = jsonl_str.into_bytes();
        file_entries.push(PackageFileEntry::from_bytes(
            &options.jsonl_path,
            &jsonl_bytes,
            "jsonl_rag_chunks",
            true,
        ));
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
            file_entries.push(PackageFileEntry::from_bytes(
                &final_path,
                body_bytes,
                sec.language.clone().unwrap_or_else(|| "text".to_string()),
                true,
            ));

            zip.start_file(&final_path, file_opts)
                .map_err(map_zip_err)?;
            zip.write_all(body_bytes)?;
            metrics.uncompressed_bytes += body_bytes.len() as u64;
        }
    }

    // 8. Provenance Report (`provenance.json`)
    if options.include_provenance {
        let prov = build_provenance(doc);
        let prov_bytes = serde_json::to_vec_pretty(&prov)?;
        file_entries.push(PackageFileEntry::from_bytes(
            "provenance.json",
            &prov_bytes,
            "json_provenance",
            true,
        ));
        zip.start_file("provenance.json", file_opts)
            .map_err(map_zip_err)?;
        zip.write_all(&prov_bytes)?;
        metrics.uncompressed_bytes += prov_bytes.len() as u64;
    }

    // 9. Completeness & Quality Report (`quality-report.json`)
    if options.include_quality_report {
        let qr = build_quality_report(doc);
        let qr_bytes = serde_json::to_vec_pretty(&qr)?;
        file_entries.push(PackageFileEntry::from_bytes(
            "quality-report.json",
            &qr_bytes,
            "json_quality_report",
            true,
        ));
        zip.start_file("quality-report.json", file_opts)
            .map_err(map_zip_err)?;
        zip.write_all(&qr_bytes)?;
        metrics.uncompressed_bytes += qr_bytes.len() as u64;
    }

    // 10. Manifest (`manifest.json`)
    if options.include_manifest {
        let mut manifest = build_manifest(doc, file_entries.clone());
        manifest.file_count = file_entries.len() + if options.include_checksums { 2 } else { 1 };
        let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
        file_entries.push(PackageFileEntry::from_bytes(
            "manifest.json",
            &manifest_bytes,
            "json_manifest",
            true,
        ));
        zip.start_file("manifest.json", file_opts)
            .map_err(map_zip_err)?;
        zip.write_all(&manifest_bytes)?;
        metrics.uncompressed_bytes += manifest_bytes.len() as u64;
    }

    // 11. Coreutils Checksums Manifest (`checksums.sha256`)
    if options.include_checksums {
        let checksum_entries: Vec<ChecksumEntry> = file_entries
            .iter()
            .map(|e| ChecksumEntry::sha256(&e.path, &e.sha256))
            .collect();
        let checksums_content = crate::checksum::generate_checksums_manifest(&checksum_entries);
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
