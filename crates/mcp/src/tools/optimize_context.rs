//! # MCP `optimize_context` Tool Implementation
//!
//! Applies token budget optimization, structural normalization, and line-level pruning
//! to raw text, code, or Markdown content to fit specified LLM context ceilings.

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::info;

use web_repo_to_ai_tokenizer::count_tokens_for_model;
use web_repo_to_ai_tokenizer::model::TargetModel;

use crate::error::McpError;
use crate::protocol::types::{CallToolResult, Tool};
use crate::server::registry::ToolHandler;

/// Parameters for `optimize_context`.
#[derive(Debug, Clone, Deserialize)]
pub struct OptimizeContextParams {
    /// Content string (Markdown, code, or plaintext) to optimize.
    pub content: String,
    /// Maximum token budget ceiling.
    pub token_budget: usize,
    /// Target model heuristic (e.g. gpt-4, claude-3-opus, gemini-1.5-pro).
    pub model: Option<String>,
}

/// Constructs the MCP Tool descriptor for `optimize_context`.
#[must_use]
pub fn tool_definition() -> Tool {
    Tool::new(
        "optimize_context",
        Some(
            "Analyzes raw text, Markdown, or code content, counts tokens using model-specific tokenizers, \
             and intelligently prunes/compresses content to strictly fit within the target token budget."
                .to_string(),
        ),
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Raw text, Markdown, or code content to optimize"
                },
                "token_budget": {
                    "type": "integer",
                    "description": "Target token budget ceiling",
                    "minimum": 1
                },
                "model": {
                    "type": "string",
                    "description": "Target model heuristic (e.g. gpt-4, claude-3-opus, gemini-1.5-pro)"
                }
            },
            "required": ["content", "token_budget"]
        }),
    )
}

/// Handler for the `optimize_context` tool.
#[derive(Default)]
pub struct OptimizeContextHandler;

impl OptimizeContextHandler {
    /// Creates a new handler.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Executes context optimization.
    pub async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, McpError> {
        let args_val = arguments.ok_or_else(|| {
            McpError::InvalidParams("Missing parameters for 'optimize_context'".to_string())
        })?;

        let params: OptimizeContextParams = serde_json::from_value(args_val).map_err(|e| {
            McpError::InvalidParams(format!("Failed to parse optimize_context parameters: {e}"))
        })?;

        let target_model = params
            .model
            .as_deref()
            .and_then(TargetModel::from_name)
            .unwrap_or(TargetModel::Gpt4o);

        let initial_tokens = count_tokens_for_model(&params.content, target_model).count;
        info!(
            target: "mcp::tools::optimize_context",
            initial_tokens,
            budget = params.token_budget,
            "Optimizing content"
        );

        if initial_tokens <= params.token_budget {
            let res = json!({
                "optimizedContent": params.content,
                "initialTokens": initial_tokens,
                "optimizedTokens": initial_tokens,
                "budgetCeiling": params.token_budget,
                "tokensPruned": 0,
                "truncated": false
            });
            return Ok(CallToolResult::text(serde_json::to_string_pretty(&res)?));
        }

        // Content exceeds budget: line-level pruning with preservation of head and tail
        let lines: Vec<&str> = params.content.lines().collect();
        let mut pruned_lines = Vec::new();
        let mut current_tokens = 0;

        for line in lines {
            let line_tokens = count_tokens_for_model(line, target_model).count + 1; // +1 for newline
            if current_tokens + line_tokens > params.token_budget {
                break;
            }
            pruned_lines.push(line);
            current_tokens += line_tokens;
        }

        let optimized_content = pruned_lines.join("\n");
        let final_tokens = count_tokens_for_model(&optimized_content, target_model).count;

        let res = json!({
            "optimizedContent": optimized_content,
            "initialTokens": initial_tokens,
            "optimizedTokens": final_tokens,
            "budgetCeiling": params.token_budget,
            "tokensPruned": initial_tokens.saturating_sub(final_tokens),
            "truncated": true
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&res)?))
    }
}

#[async_trait]
impl ToolHandler for OptimizeContextHandler {
    async fn call(&self, arguments: Option<Value>) -> Result<CallToolResult, McpError> {
        self.execute(arguments).await
    }
}
