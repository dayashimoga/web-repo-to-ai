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

pub mod error;
pub mod markdown;
pub mod options;
pub mod plain_text;
pub(crate) mod util;

// Public re-exports
pub use error::{ExporterError, Result};
pub use markdown::{export_markdown, export_markdown_to_writer, MarkdownExporter};
pub use options::{
    CodeFenceStyle, ExportMetrics, HierarchyStyle, MarkdownOptions, PlainTextOptions,
};
pub use plain_text::{export_plain_text, export_plain_text_to_writer, PlainTextExporter};

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
