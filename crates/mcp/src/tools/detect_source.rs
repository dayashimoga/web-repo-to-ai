//! # MCP `detect_source` Tool Implementation
//!
//! Analyzes input URLs or filesystem paths to classify source targets
//! (GitHub repository, web documentation, web article, local directory).

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::Path;
use tracing::info;

use web_repo_to_ai_github_extractor::target::GitHubUrlParser;
use web_repo_to_ai_web_extractor::url::{WebPageType, WebTarget};

use crate::error::McpError;
use crate::protocol::types::{CallToolResult, Tool};
use crate::server::registry::ToolHandler;

/// Parameters for `detect_source`.
#[derive(Debug, Clone, Deserialize)]
pub struct DetectSourceParams {
    /// Target URL or directory path to inspect.
    pub target: String,
}

/// Constructs the MCP Tool descriptor for `detect_source`.
#[must_use]
pub fn tool_definition() -> Tool {
    Tool::new(
        "detect_source",
        Some(
            "Inspects a target URL or local filesystem path, detects its source archetype \
             (GitHub repository, documentation site, web article, or local directory), and returns \
             structured classification metadata."
                .to_string(),
        ),
        json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "URL or path to inspect and classify"
                }
            },
            "required": ["target"]
        }),
    )
}

/// Handler for the `detect_source` tool.
#[derive(Default)]
pub struct DetectSourceHandler;

impl DetectSourceHandler {
    /// Creates a new handler.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Executes source classification.
    pub async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, McpError> {
        let args_val = arguments.ok_or_else(|| {
            McpError::InvalidParams("Missing parameters for 'detect_source'".to_string())
        })?;

        let params: DetectSourceParams = serde_json::from_value(args_val).map_err(|e| {
            McpError::InvalidParams(format!("Failed to parse detect_source parameters: {e}"))
        })?;

        info!(target: "mcp::tools::detect_source", target = %params.target, "Classifying target");

        let target_str = params.target.trim();

        // 1. Check local directory
        let local_path = Path::new(target_str);
        if local_path.is_dir() {
            let res = json!({
                "target": target_str,
                "sourceType": "local_directory",
                "exists": true,
                "isAbsolute": local_path.is_absolute(),
                "canonicalPath": local_path.canonicalize().ok().map(|p| p.display().to_string())
            });
            return Ok(CallToolResult::text(serde_json::to_string_pretty(&res)?));
        }

        // 2. Check GitHub repository URL or shorthand
        if let Ok(gh) = GitHubUrlParser::parse_url(target_str) {
            let res = json!({
                "target": target_str,
                "sourceType": "github_repository",
                "owner": gh.owner(),
                "repo": gh.repo(),
                "gitRef": gh.ref_spec(),
                "subpath": gh.subpath(),
                "canonicalUrl": gh.canonical_repo_url()
            });
            return Ok(CallToolResult::text(serde_json::to_string_pretty(&res)?));
        }

        // 3. Check WebTarget (article vs docs)
        if let Ok(wt) = WebTarget::from_str_default(target_str) {
            let archetype = match wt.page_type() {
                WebPageType::Documentation => "web_documentation",
                WebPageType::Article => "web_article",
                WebPageType::FileResource => "file_resource",
                WebPageType::GenericWeb => "generic_web",
            };

            let res = json!({
                "target": target_str,
                "sourceType": archetype,
                "domain": wt.url().host_str(),
                "canonicalUrl": wt.as_str(),
                "scheme": wt.url().scheme()
            });
            return Ok(CallToolResult::text(serde_json::to_string_pretty(&res)?));
        }

        let res = json!({
            "target": target_str,
            "sourceType": "unknown",
            "message": "Target could not be classified as a valid URL, GitHub repository, or directory"
        });
        Ok(CallToolResult::text(serde_json::to_string_pretty(&res)?))
    }
}

#[async_trait]
impl ToolHandler for DetectSourceHandler {
    async fn call(&self, arguments: Option<Value>) -> Result<CallToolResult, McpError> {
        self.execute(arguments).await
    }
}
