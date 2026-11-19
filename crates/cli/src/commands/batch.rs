//! # CLI Batch Conversion Execution Engine
//!
//! Provides parallel, bounded batch processing of multiple URLs, GitHub repositories,
//! and local filesystem directories specified via input file or CLI positional arguments.
//! Handles input file parsing (newline-delimited and JSON array), comments stripping,
//! concurrency bounds, individual error tolerance (`--continue-on-error`), and structured
//! JSON batch summary reporting.

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use web_repo_to_ai_github_extractor::client::GitHubApiClient;
use web_repo_to_ai_url_security::SecureDnsResolver;
use web_repo_to_ai_web_extractor::http::HttpTransport;

use crate::args::{BatchArgs, OutputFormat, RepoArgs, UrlArgs};
use crate::commands::repo::execute_repo_with_client;
use crate::commands::url::execute_url_with_options;
use crate::error::{CliError, ExitCode};
use crate::output::OutputWriter;

/// Granular conversion outcome for a single target item in a batch operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BatchItemResult {
    /// Zero-based ordinal index of the target in the batch input.
    pub index: usize,
    /// Target URL, GitHub shorthand, or local filesystem directory path.
    pub target: String,
    /// Classified category of the target (`url`, `github`, or `local_directory`).
    pub kind: String,
    /// Whether this specific item converted successfully.
    pub success: bool,
    /// Absolute or relative path to the generated output artifact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
    /// Byte size of the generated artifact on disk.
    pub bytes: u64,
    /// Conversion execution time in milliseconds.
    pub duration_ms: u128,
    /// Error message if conversion failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Numeric process exit code associated with this item.
    pub exit_code: i32,
}

/// Structured JSON execution summary envelope for a completed batch conversion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BatchJsonEnvelope {
    /// Whether all targets in the batch converted without error.
    pub success: bool,
    /// Total count of targets submitted in the batch.
    pub total: usize,
    /// Count of successfully converted targets.
    pub succeeded: usize,
    /// Count of failed targets.
    pub failed: usize,
    /// Overall batch processing duration in milliseconds.
    pub duration_ms: u128,
    /// Output destination directory where artifacts were written.
    pub output_dir: String,
    /// Detailed individual item results in input order.
    pub results: Vec<BatchItemResult>,
}

/// Classified target category for dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchTargetKind {
    /// Web article or documentation URL.
    Url,
    /// Remote GitHub repository (shorthand or full HTTPS URL).
    GitHub,
    /// Local filesystem directory.
    LocalDirectory,
}

impl BatchTargetKind {
    /// Returns the string label for the target kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Url => "url",
            Self::GitHub => "github",
            Self::LocalDirectory => "local_directory",
        }
    }
}

/// Executes the `batch` command with standard transports and resolvers.
///
/// # Errors
///
/// Returns [`CliError`] on invalid input file, unresolvable targets, or execution failure.
pub async fn execute_batch(args: &BatchArgs, writer: &OutputWriter) -> Result<i32, CliError> {
    execute_batch_with_options(args, writer, None, None, None).await
}

/// Executes the `batch` command with optional custom transports and clients (useful for tests).
///
/// # Errors
///
/// Returns [`CliError`] on invalid input file, unresolvable targets, or execution failure.
pub async fn execute_batch_with_options(
    args: &BatchArgs,
    writer: &OutputWriter,
    custom_transport: Option<Arc<dyn HttpTransport>>,
    custom_resolver: Option<SecureDnsResolver>,
    custom_github_client: Option<GitHubApiClient>,
) -> Result<i32, CliError> {
    let start_time = Instant::now();
    let targets = load_targets(args)?;

    if targets.is_empty() {
        return Err(CliError::Validation(
            "No targets found to convert. Provide --input-file or positional arguments."
                .to_string(),
        ));
    }

    fs::create_dir_all(&args.output_dir).map_err(|e| {
        CliError::Validation(format!(
            "Failed to create batch output directory '{}': {e}",
            args.output_dir.display()
        ))
    })?;

    writer.status(&format!(
        "Starting batch processing of {} target(s) with concurrency {} into '{}'",
        targets.len(),
        args.concurrency,
        args.output_dir.display()
    ));

    let concurrency = args.concurrency.max(1);
    let semaphore = Arc::new(Semaphore::new(concurrency));

    let is_verbose = writer.verbose;
    let futures: Vec<_> = targets
        .iter()
        .enumerate()
        .map(|(idx, target)| {
            let target = target.clone();
            let sem = semaphore.clone();
            let args_clone = args.clone();
            let transport = custom_transport.clone();
            let resolver = custom_resolver.clone();
            let gh_client = custom_github_client.clone();

            async move {
                let _permit = sem.acquire().await.map_err(|e| {
                    CliError::Validation(format!("Semaphore acquisition failed: {e}"))
                })?;

                convert_single_batch_item(
                    idx,
                    &target,
                    &args_clone,
                    is_verbose,
                    transport,
                    resolver,
                    gh_client,
                )
                .await
            }
        })
        .collect();

    let item_results: Vec<BatchItemResult> = join_all(futures)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, CliError>>()?;

    let total = item_results.len();
    let succeeded = item_results.iter().filter(|r| r.success).count();
    let failed = total - succeeded;
    let overall_duration_ms = start_time.elapsed().as_millis();

    if writer.json {
        let envelope = BatchJsonEnvelope {
            success: failed == 0,
            total,
            succeeded,
            failed,
            duration_ms: overall_duration_ms,
            output_dir: args.output_dir.display().to_string(),
            results: item_results,
        };
        let formatted = serde_json::to_string_pretty(&envelope).map_err(|e| {
            CliError::Validation(format!("Failed to format batch JSON envelope: {e}"))
        })?;
        println!("{formatted}");
    } else {
        writer.status(&format!(
            "Batch conversion finished: {succeeded}/{total} succeeded ({failed} failed) in {overall_duration_ms}ms"
        ));
        if writer.verbose {
            for res in &item_results {
                let status_label = if res.success { "OK" } else { "FAIL" };
                eprintln!(
                    "  [{status_label}] #{:03} [{}] {} -> {} ({} bytes, {}ms)",
                    res.index + 1,
                    res.kind,
                    res.target,
                    res.output_path.as_deref().unwrap_or("none"),
                    res.bytes,
                    res.duration_ms
                );
                if let Some(ref err) = res.error {
                    eprintln!("        Error: {err}");
                }
            }
        }
    }

    if failed == 0 {
        Ok(ExitCode::Success as i32)
    } else if !args.continue_on_error {
        Err(CliError::Validation(format!(
            "Batch execution halted due to item failure ({failed} failed)"
        )))
    } else {
        Ok(ExitCode::Validation as i32)
    }
}

async fn convert_single_batch_item(
    idx: usize,
    target: &str,
    args: &BatchArgs,
    verbose: bool,
    custom_transport: Option<Arc<dyn HttpTransport>>,
    custom_resolver: Option<SecureDnsResolver>,
    custom_github_client: Option<GitHubApiClient>,
) -> Result<BatchItemResult, CliError> {
    let item_start = Instant::now();
    let kind = classify_target(target);
    let filename = sanitize_filename(idx, target, args.format);
    let output_path = args.output_dir.join(&filename);
    let output_path_str = output_path.to_string_lossy().to_string();

    let sub_writer = OutputWriter::new(verbose, true, false);

    let res = match kind {
        BatchTargetKind::Url => {
            let url_args = UrlArgs {
                url: target.to_string(),
                output: output_path_str.clone(),
                format: args.format,
                budget: args.budget,
                model: args.model.clone(),
                no_optimize: args.no_optimize,
                timeout: args.timeout,
                user_agent: None,
                api_url: args.api_url.clone(),
                api_key: args.api_key.clone(),
            };
            execute_url_with_options(&url_args, &sub_writer, custom_transport, custom_resolver)
                .await
        }
        BatchTargetKind::GitHub | BatchTargetKind::LocalDirectory => {
            let repo_args = RepoArgs {
                repo: target.to_string(),
                output: output_path_str.clone(),
                format: args.format,
                budget: args.budget,
                model: args.model.clone(),
                r#ref: None,
                subpath: None,
                no_optimize: args.no_optimize,
                timeout: args.timeout,
                github_token: args.github_token.clone(),
                api_url: args.api_url.clone(),
                api_key: args.api_key.clone(),
            };
            execute_repo_with_client(&repo_args, &sub_writer, custom_github_client).await
        }
    };

    let duration_ms = item_start.elapsed().as_millis();
    let bytes = fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);

    match res {
        Ok(code) => Ok(BatchItemResult {
            index: idx,
            target: target.to_string(),
            kind: kind.as_str().to_string(),
            success: true,
            output_path: Some(output_path_str),
            bytes,
            duration_ms,
            error: None,
            exit_code: code,
        }),
        Err(err) => {
            if !args.continue_on_error {
                Err(err)
            } else {
                Ok(BatchItemResult {
                    index: idx,
                    target: target.to_string(),
                    kind: kind.as_str().to_string(),
                    success: false,
                    output_path: None,
                    bytes: 0,
                    duration_ms,
                    error: Some(err.to_string()),
                    exit_code: err.exit_code(),
                })
            }
        }
    }
}

/// Parses target identifiers from input file or positional arguments.
///
/// Supports newline-delimited entries (ignoring `#` comments and empty lines)
/// and JSON array of strings (`["http...", "owner/repo"]`).
pub fn load_targets(args: &BatchArgs) -> Result<Vec<String>, CliError> {
    let mut targets = Vec::new();

    if let Some(ref path) = args.input_file {
        let content = if path.to_str() == Some("-") {
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf).map_err(|e| {
                CliError::Validation(format!("Failed to read batch input from stdin: {e}"))
            })?;
            buf
        } else {
            fs::read_to_string(path).map_err(|e| {
                CliError::Validation(format!(
                    "Failed to read batch input file '{}': {e}",
                    path.display()
                ))
            })?
        };

        let trimmed = content.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let json_targets: Vec<String> = serde_json::from_str(trimmed).map_err(|e| {
                CliError::Validation(format!("Failed to parse batch JSON input: {e}"))
            })?;
            for t in json_targets {
                let s = t.trim().to_string();
                if !s.is_empty() {
                    targets.push(s);
                }
            }
        } else {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
                    continue;
                }
                targets.push(trimmed.to_string());
            }
        }
    }

    for pos in &args.targets {
        let trimmed = pos.trim();
        if !trimmed.is_empty() {
            targets.push(trimmed.to_string());
        }
    }

    Ok(targets)
}

/// Classifies a target string into URL, GitHub, or LocalDirectory.
#[must_use]
pub fn classify_target(target: &str) -> BatchTargetKind {
    if target.starts_with("https://github.com/") || target.starts_with("http://github.com/") {
        BatchTargetKind::GitHub
    } else if target.starts_with("http://") || target.starts_with("https://") {
        BatchTargetKind::Url
    } else if Path::new(target).is_dir() {
        BatchTargetKind::LocalDirectory
    } else if target.contains('/') && !target.starts_with('.') && !target.contains('\\') {
        BatchTargetKind::GitHub
    } else {
        BatchTargetKind::LocalDirectory
    }
}

/// Generates a sanitized, deterministic filename for a batch item.
#[must_use]
pub fn sanitize_filename(idx: usize, target: &str, format: OutputFormat) -> String {
    let clean = target
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("github.com/")
        .replace(['/', '\\', ':', '?', '&', '=', '.', '-', '@'], "_");

    let mut slug: String = clean
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect();

    while slug.contains("__") {
        slug = slug.replace("__", "_");
    }

    if slug.len() > 50 {
        let start = slug.len() - 50;
        slug = slug[start..].to_string();
    }
    let slug = slug.trim_matches('_');
    let ext = match format {
        OutputFormat::Markdown => "md",
        OutputFormat::Txt => "txt",
        OutputFormat::Json => "json",
        OutputFormat::Html => "html",
        OutputFormat::Jsonl => "jsonl",
        OutputFormat::Zip => "zip",
        OutputFormat::Manifest => "manifest",
        OutputFormat::Checksums => "checksums",
    };

    if slug.is_empty() {
        format!("{:03}_item.{ext}", idx + 1)
    } else {
        format!("{:03}_{slug}.{ext}", idx + 1)
    }
}
