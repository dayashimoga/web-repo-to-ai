//! # CDM Multi-Format Export Helpers for MCP Tools

use web_repo_to_ai_cdm::model::CdmDocument;
use web_repo_to_ai_exporters::{
    export_html, export_json, export_jsonl, export_markdown, export_plain_text, export_zip_package,
    generate_checksums_manifest, ChecksumEntry, HtmlOptions, JsonExportOptions, JsonlOptions,
    ManifestExporter, MarkdownOptions, PlainTextOptions, ZipPackageOptions,
};

use crate::error::McpError;

/// Serializes a Canonical Context Document (`CdmDocument`) into requested format string.
pub fn export_cdm_to_string(doc: &CdmDocument, format_str: &str) -> Result<String, McpError> {
    match format_str.to_lowercase().as_str() {
        "markdown" | "md" => export_markdown(doc, &MarkdownOptions::default())
            .map_err(|e| McpError::ToolError(format!("Markdown export failed: {e}"))),
        "txt" | "text" => export_plain_text(doc, &PlainTextOptions::default())
            .map_err(|e| McpError::ToolError(format!("Plaintext export failed: {e}"))),
        "json" => export_json(doc, &JsonExportOptions::canonical())
            .map_err(|e| McpError::ToolError(format!("JSON export failed: {e}"))),
        "html" => export_html(doc, &HtmlOptions::default())
            .map(|(s, _)| s)
            .map_err(|e| McpError::ToolError(format!("HTML export failed: {e}"))),
        "jsonl" => export_jsonl(doc, &JsonlOptions::default())
            .map(|(s, _)| s)
            .map_err(|e| McpError::ToolError(format!("JSONL export failed: {e}"))),
        "manifest" => ManifestExporter::new()
            .export_to_string(doc, Vec::new())
            .map_err(|e| McpError::ToolError(format!("Manifest export failed: {e}"))),
        "checksums" => {
            let md = export_markdown(doc, &MarkdownOptions::default())
                .map_err(|e| McpError::ToolError(format!("Checksums export failed: {e}")))?;
            let entry = ChecksumEntry::from_sha256_bytes("content.md", md.as_bytes());
            Ok(generate_checksums_manifest(&[entry]))
        }
        "zip" => {
            let (zip_bytes, _) = export_zip_package(doc, &ZipPackageOptions::default())
                .map_err(|e| McpError::ToolError(format!("ZIP package export failed: {e}")))?;
            Ok(format!("[ZIP Package: {} bytes]", zip_bytes.len()))
        }
        other => Err(McpError::InvalidParams(format!(
            "Unsupported format '{other}'. Valid options: markdown, txt, json, html, jsonl, zip, manifest, checksums"
        ))),
    }
}
