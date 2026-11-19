//! # Multi-URL Batch Export Generator (`crates/exporters/src/batch.rs`)
//!
//! Generates consolidated and itemized distribution artifacts for multi-URL batch conversion jobs:
//! - Consolidated Markdown (`batch.md` / `consolidated.md`) with navigation Table of Contents
//! - Consolidated Plaintext (`batch.txt`)
//! - Line-delimited JSON Lines (`batch.jsonl`) for batch RAG chunk ingestion and fine-tuning
//! - Consolidated JSON envelope (`batch.json`)
//! - Comprehensive Batch ZIP package (`batch.zip`) with root manifests, summaries, item subfolders, and checksums.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Seek, Write};
use web_repo_to_ai_cdm::CdmDocument;
use web_repo_to_ai_core::id::JobId;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::checksum::compute_sha256_hex;
use crate::error::{ExporterError, Result};
use crate::json::export_json;
use crate::markdown::export_markdown;
use crate::options::{JsonExportOptions, MarkdownOptions, PlainTextOptions};
use crate::plain_text::export_plain_text;

/// Input descriptor for an individual item to be included in a batch export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItemExportInput {
    /// Zero-based sequence index within the batch submission.
    pub item_index: usize,
    /// Strongly-typed child job identifier.
    pub job_id: JobId,
    /// Target resource URL.
    pub url: String,
    /// Optional caller-supplied correlation identifier.
    pub custom_id: Option<String>,
    /// Extracted and optimized CDM document, if conversion succeeded.
    pub document: Option<CdmDocument>,
    /// Detailed error message, if conversion failed.
    pub error_message: Option<String>,
}

impl BatchItemExportInput {
    /// Creates a successful item input with a CDM document.
    #[must_use]
    pub fn success(
        item_index: usize,
        job_id: JobId,
        url: String,
        custom_id: Option<String>,
        document: CdmDocument,
    ) -> Self {
        Self {
            item_index,
            job_id,
            url,
            custom_id,
            document: Some(document),
            error_message: None,
        }
    }

    /// Creates a failed item input with an error message.
    #[must_use]
    pub fn failure(
        item_index: usize,
        job_id: JobId,
        url: String,
        custom_id: Option<String>,
        error_message: String,
    ) -> Self {
        Self {
            item_index,
            job_id,
            url,
            custom_id,
            document: None,
            error_message: Some(error_message),
        }
    }

    /// Returns `true` if this item succeeded and has a CDM document.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.document.is_some()
    }
}

/// Options configuring multi-URL batch artifact generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchExportOptions {
    /// Optional human-readable batch title.
    pub title: Option<String>,
    /// Whether to generate a root Table of Contents in consolidated text formats.
    pub include_toc: bool,
    /// Whether to include a human-readable summary file (`batch-summary.md`) in the ZIP bundle.
    pub include_summary: bool,
    /// Whether to include a machine-readable manifest (`batch-manifest.json`) in the ZIP bundle.
    pub include_manifest: bool,
    /// Whether to include a `checksums.sha256` manifest covering all package files.
    pub include_checksums: bool,
}

impl Default for BatchExportOptions {
    fn default() -> Self {
        Self {
            title: None,
            include_toc: true,
            include_summary: true,
            include_manifest: true,
            include_checksums: true,
        }
    }
}

/// Metadata summary entry for an individual item inside a batch export manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchItemManifestEntry {
    /// Zero-based item sequence index.
    pub item_index: usize,
    /// Child conversion job identifier.
    pub job_id: String,
    /// Target resource URL.
    pub url: String,
    /// Caller-supplied correlation identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_id: Option<String>,
    /// Item status: "completed" or "failed".
    pub status: String,
    /// Document title, if successfully extracted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Token count in optimized document.
    pub token_count: usize,
    /// Number of sections/files in document.
    pub section_count: usize,
    /// Error message if conversion failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Structured manifest metadata for an exported batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchExportManifest {
    /// Parent batch identifier.
    pub batch_id: String,
    /// Optional batch title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// UTC timestamp of manifest generation.
    pub generated_at: DateTime<Utc>,
    /// Total items in batch.
    pub total_items: usize,
    /// Successfully converted items count.
    pub succeeded_items: usize,
    /// Failed items count.
    pub failed_items: usize,
    /// Sum of all tokens across successfully converted documents.
    pub total_tokens: usize,
    /// Sum of all sections/files across successfully converted documents.
    pub total_sections: usize,
    /// Itemized breakdown entries.
    pub items: Vec<BatchItemManifestEntry>,
}

/// Execution metrics for a generated batch ZIP archive package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BatchZipMetrics {
    /// Total count of discrete files written into the ZIP package.
    pub total_files_written: usize,
    /// Total byte size of the compressed ZIP archive.
    pub total_bytes_written: usize,
    /// Total uncompressed byte size of all member files.
    pub total_uncompressed_bytes: usize,
    /// SHA-256 hexadecimal digest of the complete ZIP archive.
    pub sha256_checksum: String,
}

/// Generates a sanitized directory/file name slug from a URL string.
#[must_use]
pub fn slugify_url(url: &str) -> String {
    let mut slug = String::with_capacity(url.len());
    let mut prev_underscore = false;
    for c in url.chars() {
        if c.is_ascii_alphanumeric() || c == '-' {
            slug.push(c.to_ascii_lowercase());
            prev_underscore = false;
        } else if !prev_underscore {
            slug.push('_');
            prev_underscore = true;
        }
    }
    let trimmed = slug.trim_matches('_');
    if trimmed.is_empty() {
        "item".to_string()
    } else if trimmed.len() > 48 {
        trimmed[..48].trim_matches('_').to_string()
    } else {
        trimmed.to_string()
    }
}

/// Computes the total estimated tokens in a CDM document.
fn document_tokens(doc: &CdmDocument) -> usize {
    doc.sections
        .iter()
        .map(|s| {
            s.token_metrics.as_ref().map_or_else(
                || s.body.chars().count().div_ceil(4),
                |m| m.estimated_tokens,
            )
        })
        .sum()
}

/// Returns the display title for a CDM document.
fn document_title<'a>(doc: &'a CdmDocument, fallback: &'a str) -> &'a str {
    doc.metadata.title.as_deref().unwrap_or(fallback)
}

/// Constructs a [`BatchExportManifest`] from a slice of [`BatchItemExportInput`] items.
#[must_use]
pub fn build_batch_manifest(
    batch_id: &str,
    items: &[BatchItemExportInput],
    title: Option<String>,
) -> BatchExportManifest {
    let mut succeeded_items = 0;
    let mut failed_items = 0;
    let mut total_tokens = 0;
    let mut total_sections = 0;
    let mut manifest_items = Vec::with_capacity(items.len());

    for item in items {
        if let Some(ref doc) = item.document {
            succeeded_items += 1;
            let tokens = document_tokens(doc);
            let sections = doc.sections.len();
            total_tokens += tokens;
            total_sections += sections;

            let title_str = document_title(doc, &item.url).to_string();
            manifest_items.push(BatchItemManifestEntry {
                item_index: item.item_index,
                job_id: item.job_id.to_string(),
                url: item.url.clone(),
                custom_id: item.custom_id.clone(),
                status: "completed".to_string(),
                title: Some(title_str),
                token_count: tokens,
                section_count: sections,
                error: None,
            });
        } else {
            failed_items += 1;
            manifest_items.push(BatchItemManifestEntry {
                item_index: item.item_index,
                job_id: item.job_id.to_string(),
                url: item.url.clone(),
                custom_id: item.custom_id.clone(),
                status: "failed".to_string(),
                title: None,
                token_count: 0,
                section_count: 0,
                error: item.error_message.clone(),
            });
        }
    }

    BatchExportManifest {
        batch_id: batch_id.to_string(),
        title,
        generated_at: Utc::now(),
        total_items: items.len(),
        succeeded_items,
        failed_items,
        total_tokens,
        total_sections,
        items: manifest_items,
    }
}

/// Exports all items in a batch into a single consolidated Markdown document.
///
/// # Errors
/// Returns [`ExporterError`] if rendering any member document fails.
pub fn export_batch_consolidated_markdown(
    batch_id: &str,
    items: &[BatchItemExportInput],
    options: &BatchExportOptions,
) -> Result<String> {
    let mut out = String::with_capacity(64 * 1024);
    let title = options
        .title
        .as_deref()
        .unwrap_or("Batch Conversion Context");
    let manifest = build_batch_manifest(batch_id, items, options.title.clone());

    // 1. Root Header & Metadata
    out.push_str(&format!("# {title}\n\n"));
    out.push_str(&format!("- **Batch ID:** `{batch_id}`\n"));
    out.push_str(&format!(
        "- **Generated At:** {}\n",
        manifest.generated_at.to_rfc3339()
    ));
    out.push_str(&format!(
        "- **Total Items:** {} ({} succeeded, {} failed)\n",
        manifest.total_items, manifest.succeeded_items, manifest.failed_items
    ));
    out.push_str(&format!(
        "- **Total Context Tokens:** {}\n\n",
        manifest.total_tokens
    ));

    // 2. Table of Contents
    if options.include_toc && !items.is_empty() {
        out.push_str("## Table of Contents\n\n");
        for item in items {
            let slug = slugify_url(&item.url);
            let item_label = item
                .document
                .as_ref()
                .map_or_else(|| item.url.as_str(), |d| document_title(d, &item.url));

            if let Some(ref doc) = item.document {
                let tokens = document_tokens(doc);
                out.push_str(&format!(
                    "- [Item {}: {}](#item-{}-{}) — *Completed ({} tokens)*\n",
                    item.item_index, item_label, item.item_index, slug, tokens
                ));
            } else {
                out.push_str(&format!(
                    "- [Item {}: {}](#item-{}-{}) — *Failed*\n",
                    item.item_index, item_label, item.item_index, slug
                ));
            }
        }
        out.push_str("\n---\n\n");
    }

    // 3. Member Item Sections
    for item in items {
        let slug = slugify_url(&item.url);
        let item_title = item
            .document
            .as_ref()
            .map_or_else(|| item.url.as_str(), |d| document_title(d, &item.url));

        out.push_str(&format!(
            "<a id=\"item-{}-{}\"></a>\n\n## Item {}: {}\n\n",
            item.item_index, slug, item.item_index, item_title
        ));
        out.push_str(&format!("- **Source URL:** {}\n", item.url));
        out.push_str(&format!("- **Job ID:** `{}`\n", item.job_id));
        if let Some(ref cid) = item.custom_id {
            out.push_str(&format!("- **Custom ID:** `{cid}`\n"));
        }

        if let Some(ref doc) = item.document {
            let tokens = document_tokens(doc);
            out.push_str(&format!("- **Token Count:** {tokens}\n\n"));

            let rendered_md = export_markdown(doc, &MarkdownOptions::default())?;
            out.push_str(&rendered_md);
        } else {
            let err = item.error_message.as_deref().unwrap_or("Unknown error");
            out.push_str("\n> [!WARNING]\n");
            out.push_str(&format!("> **Item Conversion Failed:** {err}\n\n"));
        }

        out.push_str("\n\n---\n\n");
    }

    Ok(out)
}

/// Exports all items in a batch into a single consolidated Plaintext document.
///
/// # Errors
/// Returns [`ExporterError`] on formatting failure.
pub fn export_batch_consolidated_plain_text(
    batch_id: &str,
    items: &[BatchItemExportInput],
    options: &BatchExportOptions,
) -> Result<String> {
    let mut out = String::with_capacity(32 * 1024);
    let title = options
        .title
        .as_deref()
        .unwrap_or("BATCH CONVERSION CONTEXT");
    let manifest = build_batch_manifest(batch_id, items, options.title.clone());

    out.push_str(&format!("================================================================================\n{title}\n================================================================================\n"));
    out.push_str(&format!("Batch ID:     {batch_id}\n"));
    out.push_str(&format!(
        "Generated At: {}\n",
        manifest.generated_at.to_rfc3339()
    ));
    out.push_str(&format!(
        "Total Items:  {} ({} succeeded, {} failed)\n",
        manifest.total_items, manifest.succeeded_items, manifest.failed_items
    ));
    out.push_str(&format!("Total Tokens: {}\n\n", manifest.total_tokens));

    for item in items {
        out.push_str(
            "--------------------------------------------------------------------------------\n",
        );
        out.push_str(&format!("ITEM {}: {}\n", item.item_index, item.url));
        out.push_str(&format!("Job ID: {}\n", item.job_id));
        if let Some(ref cid) = item.custom_id {
            out.push_str(&format!("Custom ID: {cid}\n"));
        }

        if let Some(ref doc) = item.document {
            let rendered_txt = export_plain_text(doc, &PlainTextOptions::default())?;
            out.push('\n');
            out.push_str(&rendered_txt);
        } else {
            let err = item.error_message.as_deref().unwrap_or("Unknown error");
            out.push_str(&format!("\nSTATUS: FAILED - {err}\n"));
        }
        out.push('\n');
    }

    Ok(out)
}

/// Record format for line-delimited JSONL batch exports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJsonlRecord {
    /// Batch identifier.
    pub batch_id: String,
    /// Item sequence index.
    pub item_index: usize,
    /// Child job identifier.
    pub job_id: String,
    /// Target resource URL.
    pub url: String,
    /// Optional caller correlation ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_id: Option<String>,
    /// Status: "completed" or "failed".
    pub status: String,
    /// Relative file path within document, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Extracted text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Error description if failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Exports all items in a batch as line-delimited JSON Lines (JSONL).
///
/// # Errors
/// Returns [`ExporterError`] on serialization failure.
pub fn export_batch_consolidated_jsonl(
    batch_id: &str,
    items: &[BatchItemExportInput],
    _options: &BatchExportOptions,
) -> Result<String> {
    let mut out = String::with_capacity(32 * 1024);

    for item in items {
        if let Some(ref doc) = item.document {
            for section in &doc.sections {
                let rec = BatchJsonlRecord {
                    batch_id: batch_id.to_string(),
                    item_index: item.item_index,
                    job_id: item.job_id.to_string(),
                    url: item.url.clone(),
                    custom_id: item.custom_id.clone(),
                    status: "completed".to_string(),
                    path: section.original_path.clone(),
                    content: Some(section.body.clone()),
                    error: None,
                };
                let line = serde_json::to_string(&rec)?;
                out.push_str(&line);
                out.push('\n');
            }
        } else {
            let rec = BatchJsonlRecord {
                batch_id: batch_id.to_string(),
                item_index: item.item_index,
                job_id: item.job_id.to_string(),
                url: item.url.clone(),
                custom_id: item.custom_id.clone(),
                status: "failed".to_string(),
                path: None,
                content: None,
                error: item.error_message.clone(),
            };
            let line = serde_json::to_string(&rec)?;
            out.push_str(&line);
            out.push('\n');
        }
    }

    Ok(out)
}

/// Structured consolidated batch JSON envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJsonEnvelope {
    /// Batch manifest summary.
    pub manifest: BatchExportManifest,
    /// Itemized inputs and documents.
    pub items: Vec<BatchItemExportInput>,
}

/// Exports a batch as a unified JSON envelope.
///
/// # Errors
/// Returns [`ExporterError`] on JSON serialization failure.
pub fn export_batch_consolidated_json(
    batch_id: &str,
    items: &[BatchItemExportInput],
    options: &BatchExportOptions,
) -> Result<String> {
    let manifest = build_batch_manifest(batch_id, items, options.title.clone());
    let envelope = BatchJsonEnvelope {
        manifest,
        items: items.to_vec(),
    };
    Ok(serde_json::to_string_pretty(&envelope)?)
}

/// Packages a complete multi-URL batch into a structured ZIP archive in memory.
///
/// # Errors
/// Returns [`ExporterError`] on packaging failure.
pub fn export_batch_zip_package(
    batch_id: &str,
    items: &[BatchItemExportInput],
    options: &BatchExportOptions,
) -> Result<(Vec<u8>, BatchZipMetrics)> {
    let mut buffer = Cursor::new(Vec::with_capacity(64 * 1024));
    let mut metrics = export_batch_zip_to_writer(batch_id, items, options, &mut buffer)?;
    let bytes = buffer.into_inner();
    metrics.total_bytes_written = bytes.len();
    metrics.sha256_checksum = compute_sha256_hex(&bytes);
    Ok((bytes, metrics))
}

fn map_zip_err(e: impl std::error::Error + Send + Sync + 'static) -> ExporterError {
    ExporterError::Io(std::io::Error::other(e))
}

/// Streams a multi-URL batch ZIP package directly to any `Write + Seek` destination.
///
/// # Errors
/// Returns [`ExporterError`] on writing or compression failure.
pub fn export_batch_zip_to_writer<W: Write + Seek>(
    batch_id: &str,
    items: &[BatchItemExportInput],
    options: &BatchExportOptions,
    writer: W,
) -> Result<BatchZipMetrics> {
    let mut zip = ZipWriter::new(writer);
    let mut files_written = 0;
    let mut uncompressed_bytes = 0;
    let mut checksum_lines = Vec::new();

    let file_opts = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let manifest = build_batch_manifest(batch_id, items, options.title.clone());

    // 1. Root batch-manifest.json
    if options.include_manifest {
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        let bytes = manifest_json.into_bytes();
        let path = "batch-manifest.json";

        let hash = compute_sha256_hex(&bytes);
        checksum_lines.push(format!("{hash}  {path}"));

        zip.start_file(path, file_opts).map_err(map_zip_err)?;
        zip.write_all(&bytes)?;
        uncompressed_bytes += bytes.len();
        files_written += 1;
    }

    // 2. Root batch-summary.md
    if options.include_summary {
        let summary_md = export_batch_consolidated_markdown(batch_id, items, options)?;
        let bytes = summary_md.into_bytes();
        let path = "batch-summary.md";

        let hash = compute_sha256_hex(&bytes);
        checksum_lines.push(format!("{hash}  {path}"));

        zip.start_file(path, file_opts).map_err(map_zip_err)?;
        zip.write_all(&bytes)?;
        uncompressed_bytes += bytes.len();
        files_written += 1;
    }

    // 3. Consolidated Markdown (ready for one-click LLM context ingestion)
    {
        let consolidated_md = export_batch_consolidated_markdown(batch_id, items, options)?;
        let bytes = consolidated_md.into_bytes();
        let path = "consolidated.md";

        let hash = compute_sha256_hex(&bytes);
        checksum_lines.push(format!("{hash}  {path}"));

        zip.start_file(path, file_opts).map_err(map_zip_err)?;
        zip.write_all(&bytes)?;
        uncompressed_bytes += bytes.len();
        files_written += 1;
    }

    // 4. Per-Item Directories
    for item in items {
        let slug = slugify_url(&item.url);
        let folder = format!("items/{:03}_{slug}", item.item_index);

        if let Some(ref doc) = item.document {
            // context.md
            let md_str = export_markdown(doc, &MarkdownOptions::default())?;
            let md_bytes = md_str.into_bytes();
            let md_path = format!("{folder}/context.md");
            checksum_lines.push(format!("{}  {}", compute_sha256_hex(&md_bytes), md_path));
            zip.start_file(&md_path, file_opts).map_err(map_zip_err)?;
            zip.write_all(&md_bytes)?;
            uncompressed_bytes += md_bytes.len();
            files_written += 1;

            // context.txt
            let txt_str = export_plain_text(doc, &PlainTextOptions::default())?;
            let txt_bytes = txt_str.into_bytes();
            let txt_path = format!("{folder}/context.txt");
            checksum_lines.push(format!("{}  {}", compute_sha256_hex(&txt_bytes), txt_path));
            zip.start_file(&txt_path, file_opts).map_err(map_zip_err)?;
            zip.write_all(&txt_bytes)?;
            uncompressed_bytes += txt_bytes.len();
            files_written += 1;

            // context.json
            let json_str = export_json(doc, &JsonExportOptions::canonical())?;
            let json_bytes = json_str.into_bytes();
            let json_path = format!("{folder}/context.json");
            checksum_lines.push(format!(
                "{}  {}",
                compute_sha256_hex(&json_bytes),
                json_path
            ));
            zip.start_file(&json_path, file_opts).map_err(map_zip_err)?;
            zip.write_all(&json_bytes)?;
            uncompressed_bytes += json_bytes.len();
            files_written += 1;
        } else {
            // error.txt for failed items
            let err_msg = item
                .error_message
                .as_deref()
                .unwrap_or("Item conversion failed with unclassified error");
            let err_bytes = err_msg.as_bytes();
            let err_path = format!("{folder}/error.txt");
            checksum_lines.push(format!("{}  {}", compute_sha256_hex(err_bytes), err_path));
            zip.start_file(&err_path, file_opts).map_err(map_zip_err)?;
            zip.write_all(err_bytes)?;
            uncompressed_bytes += err_bytes.len();
            files_written += 1;
        }
    }

    // 5. Root checksums.sha256
    if options.include_checksums {
        checksum_lines.sort();
        let checksum_content = checksum_lines.join("\n") + "\n";
        let bytes = checksum_content.into_bytes();
        let path = "checksums.sha256";

        zip.start_file(path, file_opts).map_err(map_zip_err)?;
        zip.write_all(&bytes)?;
        uncompressed_bytes += bytes.len();
        files_written += 1;
    }

    zip.finish().map_err(map_zip_err)?;

    Ok(BatchZipMetrics {
        total_files_written: files_written,
        total_bytes_written: 0,
        total_uncompressed_bytes: uncompressed_bytes,
        sha256_checksum: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
    use web_repo_to_ai_cdm::metadata::DocumentMetadata;
    use web_repo_to_ai_cdm::model::{SourceIdentity, SourceType};
    use web_repo_to_ai_cdm::section::SectionKind;

    fn make_test_document(title: &str, url: &str, content: &str) -> CdmDocument {
        let identity = SourceIdentity {
            raw_input: url.to_string(),
            canonical_url: url.to_string(),
            domain: Some("example.com".to_string()),
            repo_owner: None,
            repo_name: None,
            git_ref: None,
            subpath: None,
        };

        let section = CdmSectionBuilder::new(
            "sec-01",
            format!("{url}/src/main.rs"),
            SectionKind::CodeFile,
            content,
        )
        .title("src/main.rs")
        .original_path("src/main.rs")
        .language("rust")
        .build();

        CdmDocumentBuilder::new(identity, SourceType::WebpageStatic, url)
            .metadata(DocumentMetadata {
                title: Some(title.to_string()),
                ..Default::default()
            })
            .add_section(section)
            .build()
    }

    #[test]
    fn test_batch_consolidated_markdown() {
        let items = vec![
            BatchItemExportInput::success(
                0,
                JobId::new(),
                "https://example.com/one".to_string(),
                Some("item-1".to_string()),
                make_test_document("Doc One", "https://example.com/one", "fn main() {}"),
            ),
            BatchItemExportInput::failure(
                1,
                JobId::new(),
                "https://example.com/two".to_string(),
                None,
                "Timeout connecting to host".to_string(),
            ),
        ];

        let options = BatchExportOptions {
            title: Some("My Test Batch".to_string()),
            ..Default::default()
        };

        let md = export_batch_consolidated_markdown("batch-123", &items, &options).unwrap();
        assert!(md.contains("# My Test Batch"));
        assert!(md.contains("- **Batch ID:** `batch-123`"));
        assert!(md.contains("## Table of Contents"));
        assert!(md.contains("Item 0: Doc One"));
        assert!(md.contains("Item 1: https://example.com/two"));
        assert!(md.contains("**Item Conversion Failed:** Timeout connecting to host"));
        assert!(md.contains("fn main() {}"));
    }

    #[test]
    fn test_batch_consolidated_jsonl() {
        let items = vec![
            BatchItemExportInput::success(
                0,
                JobId::new(),
                "https://example.com/item".to_string(),
                None,
                make_test_document("Title", "https://example.com/item", "let x = 1;"),
            ),
            BatchItemExportInput::failure(
                1,
                JobId::new(),
                "https://example.com/err".to_string(),
                None,
                "404 Not Found".to_string(),
            ),
        ];

        let jsonl =
            export_batch_consolidated_jsonl("batch-xyz", &items, &BatchExportOptions::default())
                .unwrap();
        let lines: Vec<&str> = jsonl.trim().lines().collect();
        assert_eq!(lines.len(), 2);

        let rec0: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(rec0["status"], "completed");
        assert_eq!(rec0["content"], "let x = 1;");

        let rec1: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(rec1["status"], "failed");
        assert_eq!(rec1["error"], "404 Not Found");
    }

    #[test]
    fn test_batch_zip_packaging() {
        let items = vec![
            BatchItemExportInput::success(
                0,
                JobId::new(),
                "https://github.com/rust-lang/rust".to_string(),
                Some("item-rust".to_string()),
                make_test_document(
                    "Rust Repo",
                    "https://github.com/rust-lang/rust",
                    "println!();",
                ),
            ),
            BatchItemExportInput::failure(
                1,
                JobId::new(),
                "https://example.com/fail".to_string(),
                None,
                "DNS resolution failed".to_string(),
            ),
        ];

        let (zip_bytes, metrics) =
            export_batch_zip_package("batch-zip-test", &items, &BatchExportOptions::default())
                .unwrap();
        assert!(!zip_bytes.is_empty());
        assert!(metrics.total_files_written >= 6);
        assert!(!metrics.sha256_checksum.is_empty());

        let cursor = Cursor::new(zip_bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        assert!(archive.by_name("batch-manifest.json").is_ok());
        assert!(archive.by_name("batch-summary.md").is_ok());
        assert!(archive.by_name("consolidated.md").is_ok());
        assert!(archive.by_name("checksums.sha256").is_ok());
        assert!(archive
            .by_name("items/000_https_github_com_rust-lang_rust/context.md")
            .is_ok());
        assert!(archive
            .by_name("items/001_https_example_com_fail/error.txt")
            .is_ok());
    }
}
