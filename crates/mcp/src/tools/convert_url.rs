//! # MCP `convert_url` Tool Implementation
//!
//! Exposes universal webpage and documentation extraction to Model Context Protocol (MCP) clients.
//! Enforces SSRF and DNS-rebinding security defenses, extracts semantic main-content text,
//! constructs Canonical Context Documents (CDM), optionally applies token budget optimization,
//! and exports multi-format content payloads (Markdown, TXT, JSON, HTML, JSONL, Manifest, Checksums, ZIP).

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, error, info};

use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::metadata::DocumentMetadata as CdmDocumentMetadata;
use web_repo_to_ai_cdm::model::{SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_context_optimizer::budget::ContextBudget;
use web_repo_to_ai_context_optimizer::optimizer::{
    BudgetOptimizationConfig, ContextBudgetOptimizer,
};
use web_repo_to_ai_normalizer::config::NormalizationConfig;
use web_repo_to_ai_normalizer::normalize_document;
use web_repo_to_ai_tokenizer::model::TargetModel;
use web_repo_to_ai_url_security::{SecureDnsResolver, UrlSecurityPolicy};
use web_repo_to_ai_web_extractor::article::ArticleExtractor;
use web_repo_to_ai_web_extractor::http::{HttpTransport, WebFetcher};
use web_repo_to_ai_web_extractor::url::WebTarget;

use crate::error::McpError;
use crate::protocol::types::{CallToolResult, Tool};
use crate::server::registry::ToolHandler;
use crate::tools::export::export_cdm_to_string;

/// Parameters accepted by `convert_url`.
#[derive(Debug, Clone, Deserialize)]
pub struct ConvertUrlParams {
    /// Webpage URL to fetch and convert.
    pub url: String,
    /// Desired export format: markdown (default), txt, json, html, jsonl, zip, manifest, checksums.
    #[serde(default = "default_format")]
    pub format: String,
    /// Maximum token budget ceiling for context optimization.
    pub token_budget: Option<usize>,
    /// Target LLM model for token counting and optimization heuristics.
    pub model: Option<String>,
    /// Skip normalization and budget pruning optimization.
    #[serde(default)]
    pub no_optimize: bool,
}

fn default_format() -> String {
    "markdown".to_string()
}

/// Constructs the MCP Tool descriptor for `convert_url`.
#[must_use]
pub fn tool_definition() -> Tool {
    Tool::new(
        "convert_url",
        Some(
            "Converts a web documentation page or article URL into clean, structured, AI-ready context. \
             Enforces SSRF defenses, extracts semantic main content, strips boilerplate, and formats output \
             according to optional token budgets and target models."
                .to_string(),
        ),
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "Target HTTP or HTTPS webpage URL to convert"
                },
                "format": {
                    "type": "string",
                    "description": "Output artifact format: markdown (default), txt, json, html, jsonl, zip, manifest, checksums",
                    "enum": ["markdown", "txt", "json", "html", "jsonl", "zip", "manifest", "checksums"],
                    "default": "markdown"
                },
                "token_budget": {
                    "type": "integer",
                    "description": "Optional token budget ceiling for context optimization",
                    "minimum": 1
                },
                "model": {
                    "type": "string",
                    "description": "Target model heuristic for tokenizer (e.g. gpt-4, claude-3-opus, gemini-1.5-pro)"
                },
                "no_optimize": {
                    "type": "boolean",
                    "description": "Disable token budget pruning and normalization optimization",
                    "default": false
                }
            },
            "required": ["url"]
        }),
    )
}

/// Handler executing the `convert_url` tool.
pub struct ConvertUrlHandler {
    custom_transport: Option<Arc<dyn HttpTransport>>,
    custom_resolver: Option<SecureDnsResolver>,
    document_store: Option<Arc<crate::resources::McpDocumentStore>>,
}

impl Default for ConvertUrlHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ConvertUrlHandler {
    /// Creates a new handler with default production network transports.
    #[must_use]
    pub fn new() -> Self {
        Self {
            custom_transport: None,
            custom_resolver: None,
            document_store: None,
        }
    }

    /// Attaches an in-memory document store to record converted documents as MCP resources.
    #[must_use]
    pub fn with_store(mut self, store: Arc<crate::resources::McpDocumentStore>) -> Self {
        self.document_store = Some(store);
        self
    }

    /// Creates a handler with custom HTTP transport and DNS resolver (useful for tests).
    #[must_use]
    pub fn with_deps(
        custom_transport: Option<Arc<dyn HttpTransport>>,
        custom_resolver: Option<SecureDnsResolver>,
    ) -> Self {
        Self {
            custom_transport,
            custom_resolver,
            document_store: None,
        }
    }

    /// Internal execution routine.
    pub async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, McpError> {
        let args_val = arguments.ok_or_else(|| {
            McpError::InvalidParams("Missing parameters for 'convert_url'".to_string())
        })?;

        let params: ConvertUrlParams = serde_json::from_value(args_val).map_err(|e| {
            McpError::InvalidParams(format!("Failed to parse convert_url parameters: {e}"))
        })?;

        info!(target: "mcp::tools::convert_url", url = %params.url, format = %params.format, "Executing convert_url");

        // 1. SSRF and URL validation
        let policy = UrlSecurityPolicy::default();
        let web_target = match WebTarget::from_str(&params.url, &policy) {
            Ok(wt) => wt,
            Err(e) => {
                error!(target: "mcp::tools::convert_url", url = %params.url, error = %e, "URL validation or SSRF defense rejected target");
                return Ok(CallToolResult::error(format!(
                    "URL validation rejected: {e}"
                )));
            }
        };

        // 2. Build secure fetcher with optional custom transport and resolver
        let resolver = self
            .custom_resolver
            .clone()
            .unwrap_or_else(|| SecureDnsResolver::new(policy.clone()));
        let mut fetcher = WebFetcher::new(policy, resolver);
        if let Some(ref t) = self.custom_transport {
            fetcher = fetcher.with_transport(t.clone());
        }

        // 3. Fetch Web Content
        let fetch_response = match fetcher.fetch(&web_target).await {
            Ok(res) => res,
            Err(e) => {
                error!(target: "mcp::tools::convert_url", error = %e, "Web fetch failed");
                return Ok(CallToolResult::error(format!(
                    "Failed to fetch webpage: {e}"
                )));
            }
        };

        // 4. Extract Article / Documentation Content
        let raw_html = match String::from_utf8(fetch_response.body) {
            Ok(h) => h,
            Err(e) => {
                return Ok(CallToolResult::error(format!(
                    "Webpage body is not valid UTF-8: {e}"
                )))
            }
        };

        let extractor = ArticleExtractor::new();
        let extracted_article = match extractor.extract(&raw_html, Some(web_target.url())) {
            Ok(a) => a,
            Err(e) => {
                return Ok(CallToolResult::error(format!(
                    "Article content extraction failed: {e}"
                )))
            }
        };

        // 5. Build Canonical Context Document (CDM)
        let source_id = SourceIdentity::new(&params.url, &params.url);
        let mut builder =
            CdmDocumentBuilder::new(source_id, SourceType::WebpageStatic, &params.url);

        let mut metadata = CdmDocumentMetadata::default();
        if !extracted_article.title.is_empty() {
            metadata.title = Some(extracted_article.title.clone());
        }
        if let Some(ref byline) = extracted_article.byline {
            metadata.authors = vec![byline.clone()];
        }
        builder = builder.metadata(metadata);

        let section = CdmSectionBuilder::new(
            "main_content",
            &params.url,
            SectionKind::Prose,
            extracted_article.content_text,
        )
        .title(extracted_article.title)
        .build();

        builder = builder.add_section(section);
        let mut cdm_doc = builder.build();

        // 6. Optional Normalization & Token Budget Optimization
        if !params.no_optimize {
            let norm_cfg = NormalizationConfig::default();
            let (norm_doc, _) = normalize_document(&cdm_doc, &norm_cfg);
            cdm_doc = norm_doc;

            if let Some(budget_tokens) = params.token_budget {
                let target_model = params
                    .model
                    .as_deref()
                    .and_then(TargetModel::from_name)
                    .unwrap_or(TargetModel::Gpt4o);

                let mut budget = ContextBudget::with_total_tokens(budget_tokens);
                budget.target_model = Some(target_model);
                let opt_cfg = BudgetOptimizationConfig::new(budget);
                let optimizer = ContextBudgetOptimizer::new(opt_cfg);
                cdm_doc = match optimizer.optimize(&cdm_doc) {
                    Ok((opt, _)) => opt,
                    Err(e) => {
                        return Ok(CallToolResult::error(format!(
                            "Context budget optimization failed: {e}"
                        )))
                    }
                };
            }
        }

        // 7. Store document in session store for MCP resource retrieval
        if let Some(ref store) = self.document_store {
            let id = store.insert(cdm_doc.clone());
            debug!(target: "mcp::tools::convert_url", doc_id = %id, "Document saved in MCP document store");
        }

        // 8. Format Export
        let exported_str = match export_cdm_to_string(&cdm_doc, &params.format) {
            Ok(s) => s,
            Err(e) => return Ok(CallToolResult::error(format!("Export failed: {e}"))),
        };

        debug!(target: "mcp::tools::convert_url", output_len = exported_str.len(), "Conversion completed successfully");
        Ok(CallToolResult::text(exported_str))
    }
}

#[async_trait]
impl ToolHandler for ConvertUrlHandler {
    async fn call(&self, arguments: Option<Value>) -> Result<CallToolResult, McpError> {
        self.execute(arguments).await
    }
}
