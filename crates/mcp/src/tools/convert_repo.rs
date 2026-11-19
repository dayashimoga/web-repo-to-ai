//! # MCP `convert_repo` Tool Implementation
//!
//! Exposes GitHub repository and local filesystem directory conversion to Model Context Protocol (MCP) clients.
//! Traverses repository trees, filters unwanted binary/vendor assets, performs subpath filtering
//! with directory traversal defenses, assembles Canonical Context Documents (CDM), optionally applies
//! token budgeting, and exports clean multi-format context.

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::model::{RevisionInfo, SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_context_optimizer::budget::ContextBudget;
use web_repo_to_ai_context_optimizer::optimizer::{
    BudgetOptimizationConfig, ContextBudgetOptimizer,
};
use web_repo_to_ai_github_extractor::classification::{
    detect_language, FileClassificationConfig, FileClassifier,
};
use web_repo_to_ai_github_extractor::client::GitHubApiClient;
use web_repo_to_ai_github_extractor::source::is_binary_content;
use web_repo_to_ai_github_extractor::target::{validate_and_sanitize_subpath, GitHubUrlParser};
use web_repo_to_ai_github_extractor::tree::{GitFileMode, GitTreeEntry, GitTreeEntryType};
use web_repo_to_ai_normalizer::config::NormalizationConfig;
use web_repo_to_ai_normalizer::normalize_document;
use web_repo_to_ai_tokenizer::model::TargetModel;

use crate::error::McpError;
use crate::protocol::types::{CallToolResult, Tool};
use crate::server::registry::ToolHandler;
use crate::tools::export::export_cdm_to_string;

/// Parameters accepted by `convert_repo`.
#[derive(Debug, Clone, Deserialize)]
pub struct ConvertRepoParams {
    /// Target GitHub repository URL, shorthand (owner/repo or owner/repo@ref), or local directory path.
    pub target: String,
    /// Optional directory subpath within the repository to restrict extraction to.
    pub subpath: Option<String>,
    /// Desired export format: markdown (default), txt, json, html, jsonl, zip, manifest, checksums.
    #[serde(default = "default_format")]
    pub format: String,
    /// Maximum token budget ceiling for context optimization.
    pub token_budget: Option<usize>,
    /// Target model heuristic for tokenizer (e.g. gpt-4, claude-3-opus, gemini-1.5-pro).
    pub model: Option<String>,
    /// Optional GitHub personal access token for higher rate limits or private repos.
    pub github_token: Option<String>,
    /// Skip normalization and budget pruning optimization.
    #[serde(default)]
    pub no_optimize: bool,
}

fn default_format() -> String {
    "markdown".to_string()
}

/// Constructs the MCP Tool descriptor for `convert_repo`.
#[must_use]
pub fn tool_definition() -> Tool {
    Tool::new(
        "convert_repo",
        Some(
            "Converts a GitHub repository (by URL or owner/repo shorthand) or a local filesystem directory \
             into structured, AI-ready context. Scans source files, skips binary and vendored directories, \
             enforces strict path traversal defenses, and serializes into chosen format with token budgeting."
                .to_string(),
        ),
        json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Target GitHub repository URL (https://github.com/owner/repo), shorthand (owner/repo or owner/repo@ref), or local directory path"
                },
                "subpath": {
                    "type": "string",
                    "description": "Optional subpath within the repository to restrict extraction to"
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
                "github_token": {
                    "type": "string",
                    "description": "Optional GitHub personal access token for authenticated access"
                },
                "no_optimize": {
                    "type": "boolean",
                    "description": "Disable token budget pruning and normalization optimization",
                    "default": false
                }
            },
            "required": ["target"]
        }),
    )
}

/// Handler executing the `convert_repo` tool.
pub struct ConvertRepoHandler {
    custom_client: Option<GitHubApiClient>,
}

impl Default for ConvertRepoHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ConvertRepoHandler {
    /// Creates a new handler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            custom_client: None,
        }
    }

    /// Creates a handler with a custom GitHub API client (useful for testing with mock servers).
    #[must_use]
    pub fn with_client(custom_client: Option<GitHubApiClient>) -> Self {
        Self { custom_client }
    }

    /// Internal execution routine.
    pub async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, McpError> {
        let args_val = arguments.ok_or_else(|| {
            McpError::InvalidParams("Missing parameters for 'convert_repo'".to_string())
        })?;

        let params: ConvertRepoParams = serde_json::from_value(args_val).map_err(|e| {
            McpError::InvalidParams(format!("Failed to parse convert_repo parameters: {e}"))
        })?;

        info!(target: "mcp::tools::convert_repo", target = %params.target, format = %params.format, "Executing convert_repo");

        // 1. Sanitize subpath if provided
        let sanitized_subpath = if let Some(ref sp) = params.subpath {
            match validate_and_sanitize_subpath(sp) {
                Ok(clean_opt) => clean_opt,
                Err(e) => {
                    return Ok(CallToolResult::error(format!("Invalid subpath: {e}")));
                }
            }
        } else {
            None
        };

        // 2. Dispatch to local filesystem or remote GitHub
        let local_path = Path::new(&params.target);
        let mut cdm_doc = if local_path.is_dir() {
            match convert_local_directory(local_path, sanitized_subpath.as_deref()) {
                Ok(doc) => doc,
                Err(e) => {
                    return Ok(CallToolResult::error(format!(
                        "Local directory extraction failed: {e}"
                    )))
                }
            }
        } else {
            match convert_github_repo(
                &params,
                sanitized_subpath.as_deref(),
                self.custom_client.clone(),
            )
            .await
            {
                Ok(doc) => doc,
                Err(e) => {
                    return Ok(CallToolResult::error(format!(
                        "GitHub repository extraction failed: {e}"
                    )))
                }
            }
        };

        // 3. Normalization and Context Budget Optimization
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

        // 4. Format Export
        let exported_str = match export_cdm_to_string(&cdm_doc, &params.format) {
            Ok(s) => s,
            Err(e) => return Ok(CallToolResult::error(format!("Export failed: {e}"))),
        };

        debug!(target: "mcp::tools::convert_repo", output_len = exported_str.len(), "Repo conversion completed");
        Ok(CallToolResult::text(exported_str))
    }
}

#[async_trait]
impl ToolHandler for ConvertRepoHandler {
    async fn call(&self, arguments: Option<Value>) -> Result<CallToolResult, McpError> {
        self.execute(arguments).await
    }
}

fn convert_local_directory(
    root: &Path,
    subpath: Option<&str>,
) -> Result<web_repo_to_ai_cdm::model::CdmDocument, String> {
    let target_dir = match subpath {
        Some(sp) => root.join(sp),
        None => root.to_path_buf(),
    };

    if !target_dir.is_dir() {
        return Err(format!(
            "Target directory does not exist: {}",
            target_dir.display()
        ));
    }

    let source_id = SourceIdentity::new(root.display().to_string(), root.display().to_string());
    let mut builder = CdmDocumentBuilder::new(
        source_id,
        SourceType::GitHubPublic,
        root.display().to_string(),
    );

    let classification_cfg = FileClassificationConfig::default();
    let mut file_paths = Vec::new();
    collect_directory_files(&target_dir, root, &mut file_paths)?;

    for (rel_path, abs_path) in file_paths {
        let content_bytes = match fs::read(&abs_path) {
            Ok(b) => b,
            Err(_) => continue,
        };

        if is_binary_content(&content_bytes) {
            continue;
        }

        let entry = GitTreeEntry {
            path: rel_path.clone(),
            mode: GitFileMode::RegularFile,
            entry_type: GitTreeEntryType::Blob,
            sha: "0".repeat(40),
            size: Some(content_bytes.len() as u64),
            url: None,
        };
        let classified = FileClassifier::classify_entry(&entry, &classification_cfg);
        if !classified.is_eligible {
            continue;
        }

        let content_str = match String::from_utf8(content_bytes) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let path_obj = Path::new(&rel_path);
        let file_name = path_obj.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let ext = path_obj.extension().and_then(|e| e.to_str());
        let language = detect_language(file_name, ext);

        let mut section_builder = CdmSectionBuilder::new(
            &rel_path,
            format!("file://{rel_path}"),
            SectionKind::CodeFile,
            content_str,
        )
        .title(&rel_path)
        .original_path(&rel_path);

        if let Some(lang) = language {
            section_builder = section_builder.language(lang);
        }

        builder = builder.add_section(section_builder.build());
    }

    Ok(builder.build())
}

fn collect_directory_files(
    current: &Path,
    root: &Path,
    results: &mut Vec<(String, PathBuf)>,
) -> Result<(), String> {
    let entries = fs::read_dir(current).map_err(|e| e.to_string())?;

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Standard ignore directories
        if file_name.starts_with('.')
            || file_name == "target"
            || file_name == "node_modules"
            || file_name == "dist"
            || file_name == "build"
        {
            continue;
        }

        if path.is_dir() {
            collect_directory_files(&path, root, results)?;
        } else if path.is_file() {
            if let Ok(rel) = path.strip_prefix(root) {
                let rel_str = rel.to_slash_lossy();
                results.push((rel_str, path));
            }
        }
    }

    Ok(())
}

trait PathExt {
    fn to_slash_lossy(&self) -> String;
}

impl PathExt for Path {
    fn to_slash_lossy(&self) -> String {
        self.to_string_lossy().replace('\\', "/")
    }
}

async fn convert_github_repo(
    params: &ConvertRepoParams,
    subpath: Option<&str>,
    custom_client: Option<GitHubApiClient>,
) -> Result<web_repo_to_ai_cdm::model::CdmDocument, String> {
    let parsed_target = GitHubUrlParser::parse_url(&params.target)
        .map_err(|e| format!("Failed to parse GitHub target '{}': {e}", params.target))?;

    let client = match custom_client {
        Some(c) => c,
        None => {
            let mut c = GitHubApiClient::new();
            if let Some(ref token) = params.github_token {
                c = c.with_auth_token(token.clone());
            }
            c
        }
    };

    let owner = parsed_target.owner();
    let repo = parsed_target.repo();
    let git_ref = parsed_target.ref_spec().unwrap_or("HEAD");

    let (repo_meta, _) = client
        .get_repo_metadata(&parsed_target)
        .await
        .map_err(|e| format!("Failed to fetch repository metadata for {owner}/{repo}: {e}"))?;

    let default_branch = repo_meta.default_branch.as_str();
    let target_ref = if git_ref == "HEAD" {
        default_branch
    } else {
        git_ref
    };

    let (commit_sha, _, _) = client
        .resolve_commit_sha(&parsed_target, Some(target_ref))
        .await
        .map_err(|e| format!("Failed to resolve commit for ref '{target_ref}': {e}"))?;

    let (tree_response, _) = client
        .get_tree(&parsed_target, &commit_sha, true)
        .await
        .map_err(|e| format!("Failed to fetch tree for commit '{commit_sha}': {e}"))?;

    let canonical_url = parsed_target.canonical_repo_url();
    let source_id = SourceIdentity::new(&canonical_url, &canonical_url);
    let source_type = if params.github_token.is_some() {
        SourceType::GitHubPrivate
    } else {
        SourceType::GitHubPublic
    };

    let mut builder = CdmDocumentBuilder::new(source_id, source_type, &canonical_url);

    let revision_info = RevisionInfo {
        git_commit: Some(commit_sha.clone()),
        git_branch: Some(target_ref.to_string()),
        ..Default::default()
    };
    builder = builder.revision(revision_info);

    let classification_cfg = FileClassificationConfig::default();
    let mut files_to_fetch = Vec::new();

    for entry in tree_response.tree {
        if entry.entry_type != GitTreeEntryType::Blob {
            continue;
        }

        let path = entry.path.clone();
        if let Some(sp) = subpath {
            if !path.starts_with(sp) {
                continue;
            }
        }

        let classified = FileClassifier::classify_entry(&entry, &classification_cfg);
        if !classified.is_eligible {
            continue;
        }

        files_to_fetch.push((path, entry.sha));
    }

    // Limit initial direct fetch to reasonable ceiling per tool call (e.g. 50 files)
    let fetch_limit = files_to_fetch.len().min(50);
    for (path, sha) in files_to_fetch.into_iter().take(fetch_limit) {
        let blob_res = client.get_blob(&parsed_target, &sha).await;
        if let Ok(blob) = blob_res {
            if is_binary_content(&blob.raw_bytes) {
                continue;
            }
            if let Ok(content_str) = String::from_utf8(blob.raw_bytes) {
                let path_obj = Path::new(&path);
                let file_name = path_obj.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let ext = path_obj.extension().and_then(|e| e.to_str());
                let language = detect_language(file_name, ext);

                let mut section_builder = CdmSectionBuilder::new(
                    &path,
                    format!("https://github.com/{owner}/{repo}/blob/{commit_sha}/{path}"),
                    SectionKind::CodeFile,
                    content_str,
                )
                .title(&path)
                .original_path(&path);

                if let Some(lang) = language {
                    section_builder = section_builder.language(lang);
                }

                builder = builder.add_section(section_builder.build());
            }
        }
    }

    Ok(builder.build())
}
