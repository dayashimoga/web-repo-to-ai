//! # Multi-Format Exporters (`web-repo-to-ai-exporters`)
//!
//! Generates deterministic AI-ready artifact formats from CDM documents:
//! - Concatenated Markdown (`.md`)
//! - Canonical JSON CDM schema v1.0 (`.json`)
//! - Claude XML document hierarchy (`.xml`)
//! - Plaintext context (`.txt`)
//! - RAG / Fine-tuning JSON Lines (`.jsonl`)
//! - Streaming ZIP bundle with coreutils SHA-256 manifests

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod checksum;
pub mod error;
pub mod html;
pub mod json;
pub mod jsonl;
pub mod manifest;
pub mod markdown;
pub mod options;
pub mod plain_text;
pub(crate) mod util;
pub mod zip;

// Public re-exports
pub use checksum::{
    compute_blake3_hex, compute_sha256_hex, export_checksums_to_writer,
    generate_checksums_manifest, parse_checksums_manifest, verify_checksums, ChecksumAlgorithm,
    ChecksumEntry, ChecksumMismatch, ChecksumVerificationReport,
};
pub use error::{ExporterError, Result};
pub use html::{export_html, export_html_to_writer, HtmlExporter, DEFAULT_HTML_CSS};
pub use json::{
    export_json, export_json_to_writer, JsonExporter, StructuredCitation, StructuredCompleteness,
    StructuredContextEnvelope, StructuredExclusion, StructuredHierarchyNode, StructuredMetrics,
    StructuredSection,
};
pub use jsonl::{
    export_jsonl, export_jsonl_to_writer, ChatMessage, FineTuningChatRecord,
    FineTuningPromptCompletionRecord, JsonlExporter, RagChunkRecord,
};
pub use manifest::{
    build_manifest, build_provenance, build_quality_report, detect_mime_type, export_manifest,
    export_manifest_to_writer, export_provenance, export_provenance_to_writer,
    export_quality_report, export_quality_report_to_writer, ManifestExporter, PackageFileEntry,
    PackageManifest, PackageProvenance, PackageQualityReport,
};
pub use markdown::{export_markdown, export_markdown_to_writer, MarkdownExporter};
pub use options::{
    ChecksumOptions, CodeFenceStyle, ExportMetrics, HierarchyStyle, HtmlOptions, JsonExportOptions,
    JsonFormatFlavor, JsonlFlavor, JsonlOptions, ManifestOptions, MarkdownOptions,
    PlainTextOptions, ZipPackageMetrics, ZipPackageOptions,
};
pub use plain_text::{export_plain_text, export_plain_text_to_writer, PlainTextExporter};
pub use zip::{export_zip_package, export_zip_package_to_writer, ZipPackageExporter};

/// Returns the unique crate identifier name.
#[must_use]
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exporters_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-exporters");
    }
}
