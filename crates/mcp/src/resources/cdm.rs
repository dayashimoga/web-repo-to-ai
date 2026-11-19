//! # CDM Document Resource Provider
//!
//! Exposes converted Canonical Context Documents (`CdmDocument`) stored in `McpDocumentStore`
//! as Model Context Protocol (MCP) resources under the `cdm://` URI scheme.
//! Supports multiple format exports (Markdown, JSON, Plaintext, HTML, JSONL, Manifest, Checksums),
//! completeness audit reports, Base64-encoded ZIP archives, and individual section retrieval.

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info};

use web_repo_to_ai_cdm::model::CdmDocument;
use web_repo_to_ai_exporters::{
    build_quality_report, export_html, export_json, export_jsonl, export_markdown,
    export_plain_text, export_quality_report, export_zip_package, generate_checksums_manifest,
    ChecksumEntry, HtmlOptions, JsonExportOptions, JsonlOptions, ManifestExporter, MarkdownOptions,
    PlainTextOptions, ZipPackageOptions,
};

use crate::error::McpError;
use crate::protocol::types::{ReadResourceResult, Resource, ResourceContents, ResourceTemplate};
use crate::resources::base64::encode_base64;
use crate::resources::store::McpDocumentStore;
use crate::server::registry::DynamicResourceProvider;

/// Dynamic resource provider resolving `cdm://` URIs against `McpDocumentStore`.
#[derive(Clone)]
pub struct CdmResourceProvider {
    store: Arc<McpDocumentStore>,
}

impl CdmResourceProvider {
    /// Constructs a provider backed by the given document store.
    #[must_use]
    pub fn new(store: Arc<McpDocumentStore>) -> Self {
        Self { store }
    }

    /// Helper parsing `cdm://<doc_id>[/<subpath>]`.
    fn parse_uri(uri: &str) -> Result<(String, String), McpError> {
        let stripped = uri
            .strip_prefix("cdm://")
            .ok_or_else(|| McpError::ResourceError(format!("Invalid CDM URI scheme in '{uri}'")))?;

        let mut parts = stripped.splitn(2, '/');
        let doc_id = parts.next().unwrap_or("").trim().to_string();
        let subpath = parts.next().unwrap_or("").trim().to_string();

        if doc_id.is_empty() {
            return Err(McpError::ResourceError(format!(
                "Missing document identifier in URI '{uri}'"
            )));
        }

        Ok((doc_id, subpath))
    }

    /// Renders a specific format or aspect of a stored document.
    fn render_subpath(
        doc: &CdmDocument,
        subpath: &str,
        uri: &str,
    ) -> Result<ResourceContents, McpError> {
        match subpath.to_lowercase().as_str() {
            "" | "markdown" | "md" => {
                let md = export_markdown(doc, &MarkdownOptions::default())
                    .map_err(|e| McpError::ResourceError(format!("Markdown export failed: {e}")))?;
                Ok(ResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("text/markdown".to_string()),
                    text: Some(md),
                    blob: None,
                })
            }
            "txt" | "text" => {
                let txt = export_plain_text(doc, &PlainTextOptions::default())
                    .map_err(|e| McpError::ResourceError(format!("Plaintext export failed: {e}")))?;
                Ok(ResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("text/plain".to_string()),
                    text: Some(txt),
                    blob: None,
                })
            }
            "json" => {
                let json_str = export_json(doc, &JsonExportOptions::canonical())
                    .map_err(|e| McpError::ResourceError(format!("JSON export failed: {e}")))?;
                Ok(ResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(json_str),
                    blob: None,
                })
            }
            "html" => {
                let (html_str, _) = export_html(doc, &HtmlOptions::default())
                    .map_err(|e| McpError::ResourceError(format!("HTML export failed: {e}")))?;
                Ok(ResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("text/html".to_string()),
                    text: Some(html_str),
                    blob: None,
                })
            }
            "jsonl" => {
                let (jsonl_str, _) = export_jsonl(doc, &JsonlOptions::default())
                    .map_err(|e| McpError::ResourceError(format!("JSONL export failed: {e}")))?;
                Ok(ResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("application/x-ndjson".to_string()),
                    text: Some(jsonl_str),
                    blob: None,
                })
            }
            "manifest" => {
                let manifest_json = ManifestExporter::new()
                    .export_to_string(doc, Vec::new())
                    .map_err(|e| McpError::ResourceError(format!("Manifest export failed: {e}")))?;
                Ok(ResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(manifest_json),
                    blob: None,
                })
            }
            "checksums" => {
                let md = export_markdown(doc, &MarkdownOptions::default())
                    .map_err(|e| McpError::ResourceError(format!("Checksums generation failed: {e}")))?;
                let entry = ChecksumEntry::from_sha256_bytes("content.md", md.as_bytes());
                let manifest = generate_checksums_manifest(&[entry]);
                Ok(ResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("text/plain".to_string()),
                    text: Some(manifest),
                    blob: None,
                })
            }
            "report" | "completeness" => {
                let report = build_quality_report(doc);
                let report_json = export_quality_report(&report, true)
                    .map_err(|e| McpError::ResourceError(format!("Quality report generation failed: {e}")))?;
                Ok(ResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(report_json),
                    blob: None,
                })
            }
            "zip" => {
                let (zip_bytes, _) = export_zip_package(doc, &ZipPackageOptions::default())
                    .map_err(|e| McpError::ResourceError(format!("ZIP archive export failed: {e}")))?;
                let b64_blob = encode_base64(&zip_bytes);
                Ok(ResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("application/zip".to_string()),
                    text: None,
                    blob: Some(b64_blob),
                })
            }
            sub if sub.starts_with("section/") => {
                let sec_id = &sub["section/".len()..];
                let section = doc
                    .sections
                    .iter()
                    .find(|s| s.section_id == sec_id || s.original_path.as_deref() == Some(sec_id))
                    .ok_or_else(|| {
                        McpError::ResourceError(format!(
                            "Section '{sec_id}' not found in document '{}'",
                            doc.document_id
                        ))
                    })?;

                Ok(ResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("text/plain".to_string()),
                    text: Some(section.body.clone()),
                    blob: None,
                })
            }
            other => Err(McpError::ResourceError(format!(
                "Unsupported resource subpath '{other}' for document. Valid options: markdown, txt, json, html, jsonl, manifest, checksums, report, zip, section/<id>"
            ))),
        }
    }
}

#[async_trait]
impl DynamicResourceProvider for CdmResourceProvider {
    fn can_handle(&self, uri: &str) -> bool {
        uri.starts_with("cdm://")
    }

    async fn read(&self, uri: &str) -> Result<ReadResourceResult, McpError> {
        let (doc_id, subpath) = Self::parse_uri(uri)?;

        debug!(target: "mcp::resources::cdm", doc_id = %doc_id, subpath = %subpath, "Reading CDM resource");

        let doc = self.store.get(&doc_id).ok_or_else(|| {
            McpError::ResourceError(format!("Document '{doc_id}' not found in MCP store"))
        })?;

        let contents = Self::render_subpath(&doc, &subpath, uri)?;
        info!(target: "mcp::resources::cdm", uri = %uri, "CDM resource successfully retrieved");

        Ok(ReadResourceResult {
            contents: vec![contents],
        })
    }

    fn list_resources(&self) -> Vec<Resource> {
        let summaries = self.store.list();
        let mut out = Vec::new();
        for summary in summaries {
            let id = &summary.document_id;
            let title = summary.title.as_deref().unwrap_or(&summary.canonical_url);

            out.push(Resource {
                uri: format!("cdm://{id}/markdown"),
                name: format!("{title} (Markdown)"),
                description: Some(format!(
                    "Clean AI-ready Markdown translation of {}",
                    summary.canonical_url
                )),
                mime_type: Some("text/markdown".to_string()),
            });
            out.push(Resource {
                uri: format!("cdm://{id}/json"),
                name: format!("{title} (CDM JSON)"),
                description: Some(format!(
                    "Canonical JSON Context Document for {}",
                    summary.canonical_url
                )),
                mime_type: Some("application/json".to_string()),
            });
            out.push(Resource {
                uri: format!("cdm://{id}/manifest"),
                name: format!("{title} (Manifest)"),
                description: Some(format!(
                    "Package inventory manifest for {}",
                    summary.canonical_url
                )),
                mime_type: Some("application/json".to_string()),
            });
            out.push(Resource {
                uri: format!("cdm://{id}/checksums"),
                name: format!("{title} (Checksums)"),
                description: Some(format!(
                    "Cryptographic digests for {}",
                    summary.canonical_url
                )),
                mime_type: Some("text/plain".to_string()),
            });
            out.push(Resource {
                uri: format!("cdm://{id}/report"),
                name: format!("{title} (Completeness Report)"),
                description: Some(format!(
                    "Extraction completeness audit report for {}",
                    summary.canonical_url
                )),
                mime_type: Some("application/json".to_string()),
            });
            out.push(Resource {
                uri: format!("cdm://{id}/zip"),
                name: format!("{title} (ZIP Archive)"),
                description: Some(format!(
                    "Self-contained archive package for {}",
                    summary.canonical_url
                )),
                mime_type: Some("application/zip".to_string()),
            });
        }
        out
    }

    fn list_templates(&self) -> Vec<ResourceTemplate> {
        vec![
            ResourceTemplate::new("cdm://{document_id}/markdown", "Document Markdown")
                .with_description("Renders full document as clean AI-ready Markdown text")
                .with_mime_type("text/markdown"),
            ResourceTemplate::new("cdm://{document_id}/json", "Document Canonical JSON")
                .with_description("Canonical Context Document schema structure in JSON")
                .with_mime_type("application/json"),
            ResourceTemplate::new("cdm://{document_id}/text", "Document Plaintext")
                .with_description("Raw plaintext extraction of document body")
                .with_mime_type("text/plain"),
            ResourceTemplate::new("cdm://{document_id}/txt", "Document Plaintext (Alias)")
                .with_description("Raw plaintext extraction of document body")
                .with_mime_type("text/plain"),
            ResourceTemplate::new("cdm://{document_id}/html", "Document HTML")
                .with_description("Normalized HTML document")
                .with_mime_type("text/html"),
            ResourceTemplate::new("cdm://{document_id}/jsonl", "Document JSONL")
                .with_description("Line-delimited JSON document chunks")
                .with_mime_type("application/x-ndjson"),
            ResourceTemplate::new("cdm://{document_id}/manifest", "Document Manifest")
                .with_description("Package inventory and provenance manifest")
                .with_mime_type("application/json"),
            ResourceTemplate::new("cdm://{document_id}/checksums", "Document Checksums")
                .with_description("BSD/GNU coreutils cryptographic digest manifest")
                .with_mime_type("text/plain"),
            ResourceTemplate::new("cdm://{document_id}/report", "Completeness Audit Report")
                .with_description(
                    "Extraction completeness, section inventory, and quality ratio report",
                )
                .with_mime_type("application/json"),
            ResourceTemplate::new("cdm://{document_id}/zip", "Document ZIP Package")
                .with_description("Complete self-contained export package encoded in Base64")
                .with_mime_type("application/zip"),
            ResourceTemplate::new(
                "cdm://{document_id}/section/{section_id}",
                "Document Section",
            )
            .with_description(
                "Extracts a specific individual section by identifier or relative path",
            )
            .with_mime_type("text/plain"),
        ]
    }
}
