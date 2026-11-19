//! # CLI Configuration File Discovery, Parsing, and Precedence Merging
//!
//! Provides support for persistent configuration files in TOML or JSON format.
//! Manages hierarchical settings, discovery across standard filesystem locations,
//! credential masking, template generation, and precedence resolution:
//! `CLI Flag > Environment Variable > Config File > Built-in Default`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::args::{BatchArgs, OutputFormat, RepoArgs, UrlArgs};
use crate::error::CliError;

/// Name of the standard configuration file.
pub const CONFIG_FILE_NAME_TOML: &str = "web-repo-to-ai.toml";
/// Name of the hidden configuration file.
pub const CONFIG_FILE_NAME_DOT_TOML: &str = ".web-repo-to-ai.toml";
/// Name of the JSON configuration file.
pub const CONFIG_FILE_NAME_JSON: &str = "web-repo-to-ai.json";
/// Name of the hidden JSON configuration file.
pub const CONFIG_FILE_NAME_DOT_JSON: &str = ".web-repo-to-ai.json";

/// Structured configuration file schema.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CliConfigFile {
    /// Default execution settings for conversions.
    #[serde(default)]
    pub defaults: DefaultsConfig,

    /// Remote API connection parameters.
    #[serde(default)]
    pub api: ApiConfig,

    /// GitHub access and integration options.
    #[serde(default)]
    pub github: GitHubConfig,

    /// Batch conversion parameters.
    #[serde(default)]
    pub batch: BatchConfig,
}

/// Default execution settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefaultsConfig {
    /// Default output artifact format (e.g. `markdown`, `json`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<OutputFormat>,

    /// Target context token budget ceiling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget: Option<usize>,

    /// Target model identifier for token counting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Disable context budget optimization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_optimize: Option<bool>,

    /// HTTP request timeout in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,

    /// Custom User-Agent header string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    /// Suppress progress indicators and diagnostic messages on stderr.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quiet: Option<bool>,

    /// Verbose diagnostic output to stderr.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verbose: Option<bool>,

    /// Output execution status as a structured JSON envelope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub json: Option<bool>,
}

/// Remote API configuration parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiConfig {
    /// Base URL of remote `web-repo-to-ai` API server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_url: Option<String>,

    /// API authentication key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

/// GitHub integration configuration parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitHubConfig {
    /// GitHub personal access token or installation token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// Default git branch, tag, or commit reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_ref: Option<String>,
}

/// Batch execution configuration parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BatchConfig {
    /// Concurrency limit for parallel batch processing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<usize>,

    /// Continue batch execution if an individual item conversion fails.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continue_on_error: Option<bool>,

    /// Default output directory for batch converted artifacts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_dir: Option<PathBuf>,
}

impl CliConfigFile {
    /// Applies configuration defaults to `UrlArgs` when command-line options are unconfigured.
    pub fn apply_to_url_args(&self, args: &mut UrlArgs) {
        if let Some(fmt) = self.defaults.format {
            if args.format == OutputFormat::Markdown {
                args.format = fmt;
            }
        }
        if let Some(b) = self.defaults.budget {
            if args.budget == 160_000 {
                args.budget = b;
            }
        }
        if let Some(ref m) = self.defaults.model {
            if args.model == "cl100k_base" {
                args.model = m.clone();
            }
        }
        if let Some(no_opt) = self.defaults.no_optimize {
            if !args.no_optimize {
                args.no_optimize = no_opt;
            }
        }
        if let Some(t) = self.defaults.timeout {
            if args.timeout == 30 {
                args.timeout = t;
            }
        }
        if args.user_agent.is_none() {
            args.user_agent = self.defaults.user_agent.clone();
        }
        if args.api_url.is_none() {
            args.api_url = self.api.api_url.clone();
        }
        if args.api_key.is_none() {
            args.api_key = self.api.api_key.clone();
        }
    }

    /// Applies configuration defaults to `RepoArgs` when command-line options are unconfigured.
    pub fn apply_to_repo_args(&self, args: &mut RepoArgs) {
        if let Some(fmt) = self.defaults.format {
            if args.format == OutputFormat::Markdown {
                args.format = fmt;
            }
        }
        if let Some(b) = self.defaults.budget {
            if args.budget == 160_000 {
                args.budget = b;
            }
        }
        if let Some(ref m) = self.defaults.model {
            if args.model == "cl100k_base" {
                args.model = m.clone();
            }
        }
        if let Some(no_opt) = self.defaults.no_optimize {
            if !args.no_optimize {
                args.no_optimize = no_opt;
            }
        }
        if let Some(t) = self.defaults.timeout {
            if args.timeout == 60 {
                args.timeout = t;
            }
        }
        if args.r#ref.is_none() {
            args.r#ref = self.github.default_ref.clone();
        }
        if args.github_token.is_none() {
            args.github_token = self.github.token.clone();
        }
        if args.api_url.is_none() {
            args.api_url = self.api.api_url.clone();
        }
        if args.api_key.is_none() {
            args.api_key = self.api.api_key.clone();
        }
    }

    /// Applies configuration defaults to `BatchArgs` when command-line options are unconfigured.
    pub fn apply_to_batch_args(&self, args: &mut BatchArgs) {
        if let Some(fmt) = self.defaults.format {
            if args.format == OutputFormat::Markdown {
                args.format = fmt;
            }
        }
        if let Some(b) = self.defaults.budget {
            if args.budget == 160_000 {
                args.budget = b;
            }
        }
        if let Some(ref m) = self.defaults.model {
            if args.model == "cl100k_base" {
                args.model = m.clone();
            }
        }
        if let Some(no_opt) = self.defaults.no_optimize {
            if !args.no_optimize {
                args.no_optimize = no_opt;
            }
        }
        if let Some(c) = self.batch.concurrency {
            if args.concurrency == 4 {
                args.concurrency = c;
            }
        }
        if let Some(cont) = self.batch.continue_on_error {
            args.continue_on_error = cont;
        }
        if let Some(ref out_dir) = self.batch.output_dir {
            if args.output_dir == Path::new("./output") {
                args.output_dir = out_dir.clone();
            }
        }
        if let Some(t) = self.defaults.timeout {
            if args.timeout == 60 {
                args.timeout = t;
            }
        }
        if args.github_token.is_none() {
            args.github_token = self.github.token.clone();
        }
        if args.api_url.is_none() {
            args.api_url = self.api.api_url.clone();
        }
        if args.api_key.is_none() {
            args.api_key = self.api.api_key.clone();
        }
    }

    /// Formats this configuration file as a TOML string representation.
    #[must_use]
    pub fn to_toml_string(&self, mask_credentials: bool) -> String {
        let mut out = String::new();
        out.push_str("# web-repo-to-ai configuration file\n\n");

        out.push_str("[defaults]\n");
        if let Some(fmt) = self.defaults.format {
            out.push_str(&format!("format = \"{}\"\n", fmt.as_str()));
        }
        if let Some(b) = self.defaults.budget {
            out.push_str(&format!("budget = {b}\n"));
        }
        if let Some(ref m) = self.defaults.model {
            out.push_str(&format!("model = \"{m}\"\n"));
        }
        if let Some(no_opt) = self.defaults.no_optimize {
            out.push_str(&format!("no_optimize = {no_opt}\n"));
        }
        if let Some(t) = self.defaults.timeout {
            out.push_str(&format!("timeout = {t}\n"));
        }
        if let Some(ref ua) = self.defaults.user_agent {
            out.push_str(&format!("user_agent = \"{ua}\"\n"));
        }
        if let Some(q) = self.defaults.quiet {
            out.push_str(&format!("quiet = {q}\n"));
        }
        if let Some(v) = self.defaults.verbose {
            out.push_str(&format!("verbose = {v}\n"));
        }
        if let Some(j) = self.defaults.json {
            out.push_str(&format!("json = {j}\n"));
        }

        out.push_str("\n[api]\n");
        if let Some(ref url) = self.api.api_url {
            out.push_str(&format!("api_url = \"{url}\"\n"));
        }
        if let Some(ref key) = self.api.api_key {
            let val = if mask_credentials {
                mask_secret(key)
            } else {
                key.clone()
            };
            out.push_str(&format!("api_key = \"{val}\"\n"));
        }

        out.push_str("\n[github]\n");
        if let Some(ref tok) = self.github.token {
            let val = if mask_credentials {
                mask_secret(tok)
            } else {
                tok.clone()
            };
            out.push_str(&format!("token = \"{val}\"\n"));
        }
        if let Some(ref r) = self.github.default_ref {
            out.push_str(&format!("default_ref = \"{r}\"\n"));
        }

        out.push_str("\n[batch]\n");
        if let Some(c) = self.batch.concurrency {
            out.push_str(&format!("concurrency = {c}\n"));
        }
        if let Some(cont) = self.batch.continue_on_error {
            out.push_str(&format!("continue_on_error = {cont}\n"));
        }
        if let Some(ref dir) = self.batch.output_dir {
            out.push_str(&format!("output_dir = \"{}\"\n", dir.display()));
        }

        out
    }
}

/// Discovers the path to the active configuration file.
///
/// Priority:
/// 1. Explicit path passed via `--config <PATH>`
/// 2. Current working directory: `web-repo-to-ai.toml`, `.web-repo-to-ai.toml`, `web-repo-to-ai.json`, `.web-repo-to-ai.json`
/// 3. User configuration directory: `~/.config/web-repo-to-ai/config.toml`, `~/.config/web-repo-to-ai/config.json`
#[must_use]
pub fn find_config_file(explicit_path: Option<&Path>) -> Option<PathBuf> {
    if let Some(explicit) = explicit_path {
        if explicit.exists() {
            return Some(explicit.to_path_buf());
        }
    }

    let cwd_candidates = [
        CONFIG_FILE_NAME_TOML,
        CONFIG_FILE_NAME_DOT_TOML,
        CONFIG_FILE_NAME_JSON,
        CONFIG_FILE_NAME_DOT_JSON,
    ];

    for name in &cwd_candidates {
        let path = PathBuf::from(name);
        if path.is_file() {
            return Some(path);
        }
    }

    if let Some(home) = dirs_home() {
        let home_candidates = [
            home.join(".config")
                .join("web-repo-to-ai")
                .join("config.toml"),
            home.join(".config")
                .join("web-repo-to-ai")
                .join("config.json"),
            home.join(CONFIG_FILE_NAME_DOT_TOML),
            home.join(CONFIG_FILE_NAME_DOT_JSON),
        ];
        for candidate in &home_candidates {
            if candidate.is_file() {
                return Some(candidate.clone());
            }
        }
    }

    None
}

/// Resolves user home directory using environment variables across platforms.
fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

/// Loads and parses a configuration file from disk.
///
/// # Errors
///
/// Returns [`CliError::Validation`] if the file cannot be read or contains invalid syntax.
pub fn load_config(explicit_path: Option<&Path>) -> Result<Option<CliConfigFile>, CliError> {
    let file_path = match (explicit_path, find_config_file(explicit_path)) {
        (Some(explicit), None) => {
            return Err(CliError::Validation(format!(
                "Specified configuration file not found: {}",
                explicit.display()
            )));
        }
        (_, Some(path)) => path,
        (None, None) => return Ok(None),
    };

    let content = fs::read_to_string(&file_path).map_err(|e| {
        CliError::Validation(format!(
            "Failed to read configuration file '{}': {e}",
            file_path.display()
        ))
    })?;

    let is_json = file_path.extension().and_then(|e| e.to_str()) == Some("json");
    let config = parse_config_str(&content, is_json).map_err(|e| {
        CliError::Validation(format!(
            "Failed to parse configuration file '{}': {e}",
            file_path.display()
        ))
    })?;

    Ok(Some(config))
}

/// Parses raw configuration content from string (either TOML or JSON).
///
/// # Errors
///
/// Returns descriptive error string on syntax error or schema mismatch.
pub fn parse_config_str(content: &str, is_json: bool) -> Result<CliConfigFile, String> {
    if is_json || content.trim_start().starts_with('{') {
        serde_json::from_str(content).map_err(|e| format!("JSON syntax error: {e}"))
    } else {
        let value = parse_toml_to_json_value(content)?;
        serde_json::from_value(value).map_err(|e| format!("Config schema error: {e}"))
    }
}

/// Parses lightweight TOML into a structured `serde_json::Value` object.
///
/// # Errors
///
/// Returns descriptive error string on invalid line syntax or malformed section headers.
pub fn parse_toml_to_json_value(content: &str) -> Result<serde_json::Value, String> {
    let mut root = serde_json::Map::new();
    let mut current_section: Option<String> = None;

    for (line_idx, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            let section_name = line[1..line.len() - 1].trim();
            if section_name.is_empty() {
                return Err(format!("Empty section header on line {}", line_idx + 1));
            }
            current_section = Some(section_name.to_string());
            if !root.contains_key(section_name) {
                root.insert(
                    section_name.to_string(),
                    serde_json::Value::Object(serde_json::Map::new()),
                );
            }
            continue;
        }

        if let Some((raw_key, raw_val)) = line.split_once('=') {
            let key = raw_key.trim();
            let mut val_str = raw_val.trim();

            if !val_str.starts_with('"') && !val_str.starts_with('\'') {
                if let Some((before_comment, _)) = val_str.split_once('#') {
                    val_str = before_comment.trim();
                }
            }

            let parsed_val = parse_toml_scalar(val_str)
                .map_err(|e| format!("Error on line {}: {e}", line_idx + 1))?;

            if let Some(ref sec) = current_section {
                if let Some(serde_json::Value::Object(sec_map)) = root.get_mut(sec) {
                    sec_map.insert(key.to_string(), parsed_val);
                }
            } else {
                root.insert(key.to_string(), parsed_val);
            }
        } else {
            return Err(format!(
                "Invalid TOML line syntax on line {}: expected 'key = value'",
                line_idx + 1
            ));
        }
    }

    Ok(serde_json::Value::Object(root))
}

fn parse_toml_scalar(s: &str) -> Result<serde_json::Value, String> {
    let trimmed = s.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2)
        || (trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2)
    {
        let inner = &trimmed[1..trimmed.len() - 1];
        Ok(serde_json::Value::String(inner.to_string()))
    } else if trimmed == "true" {
        Ok(serde_json::Value::Bool(true))
    } else if trimmed == "false" {
        Ok(serde_json::Value::Bool(false))
    } else if let Ok(n) = trimmed.parse::<i64>() {
        Ok(serde_json::Value::Number(n.into()))
    } else if let Ok(f) = trimmed.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(f) {
            Ok(serde_json::Value::Number(num))
        } else {
            Err(format!("Invalid float number: {trimmed}"))
        }
    } else {
        Ok(serde_json::Value::String(trimmed.to_string()))
    }
}

/// Masks sensitive credentials for terminal output (e.g. `ghp_12345678` -> `ghp_...5678`).
#[must_use]
pub fn mask_secret(secret: &str) -> String {
    let len = secret.len();
    if len <= 8 {
        "********".to_string()
    } else {
        let prefix = &secret[..4.min(len)];
        let suffix = &secret[len.saturating_sub(4)..];
        format!("{prefix}...{suffix}")
    }
}

/// Returns a commented template TOML configuration string.
#[must_use]
pub fn generate_template_toml() -> &'static str {
    r#"# web-repo-to-ai configuration file
# Place in current directory as 'web-repo-to-ai.toml' or in ~/.config/web-repo-to-ai/config.toml

[defaults]
# Default output format: markdown, txt, json, html, jsonl, zip, manifest, checksums
format = "markdown"

# Target context token budget ceiling per converted source
budget = 160000

# Target model identifier for token counting (cl100k_base, gpt-4-turbo, claude-3-5-sonnet, gemini-1.5-pro)
model = "cl100k_base"

# Disable context optimization (deduplication & compression)
no_optimize = false

# HTTP request timeout in seconds
timeout = 60

# Custom User-Agent string for HTTP fetches
# user_agent = "web-repo-to-ai/0.1.0"

# Quiet mode (suppress stderr diagnostics)
quiet = false

# Verbose diagnostic mode
verbose = false

# Structured JSON execution envelope output
json = false

[api]
# Remote web-repo-to-ai API base URL (e.g. http://localhost:8080)
# api_url = "http://localhost:8080"

# Remote API authentication key (wr_live_...)
# api_key = "wr_live_your_api_key_here"

[github]
# GitHub personal access token for private repositories or increased rate limits
# token = "ghp_your_github_token_here"

# Default git branch, tag, or commit reference
default_ref = "main"

[batch]
# Number of concurrent workers for batch processing
concurrency = 4

# Continue batch execution if an individual item conversion fails
continue_on_error = true

# Destination directory for batch converted artifacts
output_dir = "./output"
"#
}

/// Returns a commented template JSON configuration string.
#[must_use]
pub fn generate_template_json() -> &'static str {
    r#"{
  "defaults": {
    "format": "markdown",
    "budget": 160000,
    "model": "cl100k_base",
    "no_optimize": false,
    "timeout": 60,
    "quiet": false,
    "verbose": false,
    "json": false
  },
  "api": {
    "api_url": "http://localhost:8080"
  },
  "github": {
    "default_ref": "main"
  },
  "batch": {
    "concurrency": 4,
    "continue_on_error": true,
    "output_dir": "./output"
  }
}
"#
}
