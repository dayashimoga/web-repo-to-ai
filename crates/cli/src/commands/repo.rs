//! # Repository Conversion Command Execution Engine

use std::path::Path;
use std::time::{Duration, Instant};

use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::model::{RevisionInfo, SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::{CdmSection, SectionKind};
use web_repo_to_ai_context_optimizer::budget::ContextBudget;
use web_repo_to_ai_context_optimizer::optimizer::{
    BudgetOptimizationConfig, ContextBudgetOptimizer,
};
use web_repo_to_ai_github_extractor::classification::{
    detect_language, ClassifiedFile, FileClassificationConfig, FileClassifier,
};
use web_repo_to_ai_github_extractor::client::GitHubApiClient;
use web_repo_to_ai_github_extractor::source::{is_binary_content, SourceExtractionConfig};
use web_repo_to_ai_github_extractor::target::{validate_and_sanitize_subpath, GitHubUrlParser};
use web_repo_to_ai_normalizer::config::NormalizationConfig;
use web_repo_to_ai_normalizer::normalize_document;
use web_repo_to_ai_tokenizer::count_tokens_for_model;
use web_repo_to_ai_tokenizer::model::TargetModel;

use crate::args::RepoArgs;
use crate::error::CliError;
use crate::output::{export_artifact, CliJsonEnvelope, OutputWriter};

/// Executes the `repo` command with default GitHub API client.
///
/// # Errors
///
/// Returns [`CliError`] on invalid input, network failure, extraction error, or budget violation.
pub async fn execute_repo(
    args: &RepoArgs,
    writer: &OutputWriter,
) -> std::result::Result<i32, CliError> {
    execute_repo_with_client(args, writer, None).await
}

/// Executes the `repo` command with an optional custom GitHub API client (useful for tests).
///
/// # Errors
///
/// Returns [`CliError`] on invalid input, network failure, extraction error, or budget violation.
pub async fn execute_repo_with_client(
    args: &RepoArgs,
    writer: &OutputWriter,
    custom_client: Option<GitHubApiClient>,
) -> std::result::Result<i32, CliError> {
    let start_time = Instant::now();

    let res = if args.api_url.is_some() {
        execute_remote(args, writer, start_time).await
    } else if Path::new(&args.repo).is_dir() {
        execute_local_directory(Path::new(&args.repo), args, writer, start_time).await
    } else {
        execute_github(args, writer, start_time, custom_client).await
    };

    match res {
        Ok(code) => Ok(code),
        Err(err) => {
            if writer.json {
                let _ = writer.write_json_envelope(&CliJsonEnvelope {
                    success: false,
                    url: args.repo.clone(),
                    format: args.format.as_str().to_string(),
                    tokens: 0,
                    bytes: 0,
                    duration_ms: start_time.elapsed().as_millis(),
                    output_path: args.output.clone(),
                    content: None,
                    error: Some(err.to_string()),
                    exit_code: Some(err.exit_code()),
                });
            } else {
                writer.error(&err.to_string());
            }
            Err(err)
        }
    }
}

async fn execute_local_directory(
    path: &Path,
    args: &RepoArgs,
    writer: &OutputWriter,
    start_time: Instant,
) -> std::result::Result<i32, CliError> {
    writer.status(&format!("Scanning local directory {}...", path.display()));

    let clean_subpath = if let Some(sub) = &args.subpath {
        validate_and_sanitize_subpath(sub)?
    } else {
        None
    };

    let canonical_root = path
        .canonicalize()
        .map_err(|e| CliError::Extraction(format!("Failed to resolve path: {e}")))?;

    let mut sections = Vec::new();
    walk_dir_recursive(
        &canonical_root,
        &canonical_root,
        &mut sections,
        clean_subpath.as_deref(),
    )?;

    if sections.is_empty() {
        return Err(CliError::Extraction(
            "No eligible source or documentation files found in repository".to_string(),
        ));
    }

    // Sort sections deterministically by relative path
    sections.sort_by(|a, b| a.section_id.cmp(&b.section_id));

    writer.debug(&format!(
        "Discovered and extracted {} local files",
        sections.len()
    ));

    let folder_name = canonical_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("repository");
    let canonical_url = format!("file:///{}", canonical_root.display()).replace('\\', "/");

    let mut doc_builder = CdmDocumentBuilder::new(
        SourceIdentity::new(&args.repo, &canonical_url),
        SourceType::GitHubPublic,
        &canonical_url,
    );
    doc_builder = doc_builder.title(folder_name);
    doc_builder = doc_builder.extend_sections(sections);
    let cdm_doc = doc_builder.build();

    // Normalize document
    let (norm_doc, _norm_report) = normalize_document(&cdm_doc, &NormalizationConfig::default());

    // Token counting & context budget optimization
    let target_model = TargetModel::from_name(&args.model).unwrap_or(TargetModel::Gpt4Turbo);
    let estimated_tokens: usize = norm_doc
        .sections
        .iter()
        .map(|s| count_tokens_for_model(&s.body, target_model).count)
        .sum();

    let final_doc = if !args.no_optimize && estimated_tokens > args.budget {
        writer.status(&format!(
            "Content size ({estimated_tokens} tokens) exceeds budget ({}); optimizing...",
            args.budget
        ));
        let budget = ContextBudget::with_total_tokens(args.budget).with_target_model(target_model);
        let optimizer = ContextBudgetOptimizer::new(BudgetOptimizationConfig::new(budget));
        let (optimized, _plan) = optimizer.optimize(&norm_doc)?;
        optimized
    } else {
        norm_doc
    };

    // Exporter execution
    let artifact_bytes = export_artifact(&final_doc, args.format)?;
    let final_tokens: usize = final_doc
        .sections
        .iter()
        .map(|s| count_tokens_for_model(&s.body, target_model).count)
        .sum();

    emit_repo_result(
        args,
        &artifact_bytes,
        final_tokens,
        start_time.elapsed(),
        writer,
    )
}

fn walk_dir_recursive(
    current_dir: &Path,
    root_dir: &Path,
    sections: &mut Vec<CdmSection>,
    subpath_filter: Option<&str>,
) -> Result<(), CliError> {
    let entries =
        std::fs::read_dir(current_dir).map_err(|e| CliError::Extraction(e.to_string()))?;

    for entry in entries {
        let entry = entry.map_err(|e| CliError::Extraction(e.to_string()))?;
        let entry_path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| CliError::Extraction(e.to_string()))?;

        if file_type.is_dir() {
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if is_ignored_directory(&dir_name) {
                continue;
            }
            walk_dir_recursive(&entry_path, root_dir, sections, subpath_filter)?;
        } else if file_type.is_file() {
            let rel_path = entry_path
                .strip_prefix(root_dir)
                .map_err(|e| CliError::Extraction(e.to_string()))?;
            let rel_str = rel_path.to_string_lossy().replace('\\', "/");
            let clean_rel = rel_str.trim_start_matches('/');

            if let Some(filter) = subpath_filter {
                if clean_rel != filter && !clean_rel.starts_with(&format!("{filter}/")) {
                    continue;
                }
            }

            // Check file size (limit to 1 MiB)
            let metadata = entry
                .metadata()
                .map_err(|e| CliError::Extraction(e.to_string()))?;
            if metadata.len() > 1024 * 1024 {
                continue;
            }

            let bytes =
                std::fs::read(&entry_path).map_err(|e| CliError::Extraction(e.to_string()))?;
            if is_binary_content(&bytes) {
                continue;
            }

            let content = match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(e) => String::from_utf8_lossy(&e.into_bytes()).to_string(),
            };

            let file_name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            let extension = entry_path.extension().and_then(|e| e.to_str());
            let language = detect_language(file_name, extension);

            let is_doc = clean_rel.ends_with(".md")
                || clean_rel.ends_with(".markdown")
                || clean_rel.ends_with(".txt")
                || clean_rel.ends_with(".rst")
                || clean_rel.starts_with("docs/");
            let kind = if is_doc {
                SectionKind::Prose
            } else {
                SectionKind::CodeFile
            };

            let mut sec_builder =
                CdmSectionBuilder::new(clean_rel, format!("file:///{clean_rel}"), kind, content)
                    .title(clean_rel);

            if let Some(lang) = language {
                sec_builder = sec_builder.language(lang);
            }

            sections.push(sec_builder.build());
        }
    }
    Ok(())
}

fn is_ignored_directory(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | "target"
            | "node_modules"
            | "dist"
            | ".DS_Store"
            | ".idea"
            | ".vscode"
            | "venv"
            | ".env"
            | "coverage"
            | "build"
            | "__pycache__"
    )
}

async fn execute_github(
    args: &RepoArgs,
    writer: &OutputWriter,
    start_time: Instant,
    custom_client: Option<GitHubApiClient>,
) -> std::result::Result<i32, CliError> {
    let target = GitHubUrlParser::new().parse(&args.repo)?;

    let clean_subpath = if let Some(sub) = &args.subpath {
        validate_and_sanitize_subpath(sub)?
    } else {
        None
    };

    let effective_subpath = clean_subpath.or_else(|| target.subpath().map(ToString::to_string));

    writer.status(&format!("Querying GitHub repository {}...", target.nwo()));

    let mut client = custom_client.unwrap_or_default();
    if let Some(ref token) = args.github_token {
        client = client.with_auth_token(token);
    }
    client = client.with_timeout(Duration::from_secs(args.timeout));

    // 1. Fetch metadata
    let (metadata, _rate_limit) = client.get_repo_metadata(&target).await?;
    writer.debug(&format!(
        "Repository metadata: {} (stars: {}, default_branch: {})",
        metadata.full_name, metadata.stargazers_count, metadata.default_branch
    ));

    // 2. Resolve target branch / commit ref
    let requested_ref = args.r#ref.as_deref().or_else(|| target.ref_spec());
    let ref_to_resolve = requested_ref.unwrap_or(&metadata.default_branch);
    let (commit_sha, _commit_meta, _rate_limit) = client
        .resolve_commit_sha(&target, Some(ref_to_resolve))
        .await?;
    writer.debug(&format!("Resolved ref '{ref_to_resolve}' -> {commit_sha}"));

    // 3. Fetch tree
    let (tree, _rate_limit) = client.get_tree(&target, &commit_sha, true).await?;
    writer.debug(&format!("Discovered {} tree entries", tree.tree.len()));

    // 4. Classify tree
    let classification_config = FileClassificationConfig::default();
    let report = FileClassifier::classify_tree(&tree, &classification_config);
    writer.debug(&format!(
        "Classified files: {} eligible, {} excluded, {} binary, {} oversized",
        report.eligible_count, report.excluded_count, report.binary_count, report.oversized_count
    ));

    // 5. Filter files by subpath if specified
    let eligible_files: Vec<ClassifiedFile> = report
        .files
        .into_iter()
        .filter(|f| {
            if !f.is_eligible {
                return false;
            }
            if let Some(ref sub) = effective_subpath {
                let clean_sub = sub.trim_matches('/');
                if f.path != clean_sub && !f.path.starts_with(&format!("{clean_sub}/")) {
                    return false;
                }
            }
            true
        })
        .collect();

    if eligible_files.is_empty() {
        return Err(CliError::Extraction(
            "No eligible source or documentation files found for extraction".to_string(),
        ));
    }

    writer.status(&format!(
        "Extracting {} files from {}@{}...",
        eligible_files.len(),
        target.nwo(),
        &commit_sha[..7.min(commit_sha.len())]
    ));

    // 6. Extract source files
    let source_config = SourceExtractionConfig::default();
    let extraction_report = client
        .extract_source_files(&target, &eligible_files, &source_config)
        .await;

    if extraction_report.extracted.is_empty() {
        return Err(CliError::Extraction(
            "Failed to extract any file contents from GitHub repository".to_string(),
        ));
    }

    // 7. Build CDM Document
    let mut doc_builder = CdmDocumentBuilder::new(
        target.to_source_identity(&args.repo),
        SourceType::GitHubPublic,
        target.canonical_target_url(),
    );
    doc_builder = doc_builder.title(&metadata.name);
    doc_builder = doc_builder.revision(RevisionInfo {
        git_commit: Some(commit_sha.clone()),
        git_branch: Some(ref_to_resolve.to_string()),
        ..Default::default()
    });

    for file in &extraction_report.extracted {
        let is_doc = file.path.ends_with(".md")
            || file.path.ends_with(".markdown")
            || file.path.ends_with(".txt")
            || file.path.ends_with(".rst")
            || file.path.starts_with("docs/");
        let kind = if is_doc {
            SectionKind::Prose
        } else {
            SectionKind::CodeFile
        };

        let mut sec_builder = CdmSectionBuilder::new(
            &file.path,
            target.raw_blob_url(ref_to_resolve, &file.path),
            kind,
            &file.content,
        )
        .original_path(&file.path)
        .title(&file.path);

        if let Some(ref lang) = file.language {
            sec_builder = sec_builder.language(lang);
        }

        doc_builder = doc_builder.add_section(sec_builder.build());
    }

    let cdm_doc = doc_builder.build();

    // 8. Normalize
    let (norm_doc, _norm_report) = normalize_document(&cdm_doc, &NormalizationConfig::default());

    // 9. Optimize budget
    let target_model = TargetModel::from_name(&args.model).unwrap_or(TargetModel::Gpt4Turbo);
    let estimated_tokens: usize = norm_doc
        .sections
        .iter()
        .map(|s| count_tokens_for_model(&s.body, target_model).count)
        .sum();

    let final_doc = if !args.no_optimize && estimated_tokens > args.budget {
        writer.status(&format!(
            "Content size ({estimated_tokens} tokens) exceeds budget ({}); optimizing...",
            args.budget
        ));
        let budget = ContextBudget::with_total_tokens(args.budget).with_target_model(target_model);
        let optimizer = ContextBudgetOptimizer::new(BudgetOptimizationConfig::new(budget));
        let (optimized, _plan) = optimizer.optimize(&norm_doc)?;
        optimized
    } else {
        norm_doc
    };

    // 10. Export artifact
    let artifact_bytes = export_artifact(&final_doc, args.format)?;
    let final_tokens: usize = final_doc
        .sections
        .iter()
        .map(|s| count_tokens_for_model(&s.body, target_model).count)
        .sum();

    emit_repo_result(
        args,
        &artifact_bytes,
        final_tokens,
        start_time.elapsed(),
        writer,
    )
}

async fn execute_remote(
    args: &RepoArgs,
    writer: &OutputWriter,
    start_time: Instant,
) -> std::result::Result<i32, CliError> {
    let api_base = args
        .api_url
        .as_deref()
        .unwrap_or("http://localhost:8080")
        .trim_end_matches('/');

    writer.status(&format!(
        "Dispatching conversion of {} to remote API: {}",
        args.repo, api_base
    ));

    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(ref key) = args.api_key {
        if let Ok(hv) = format!("Bearer {}", key.trim()).parse() {
            headers.insert(reqwest::header::AUTHORIZATION, hv);
        }
    }
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        "application/json".parse().unwrap(),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(args.timeout))
        .build()
        .map_err(|e| CliError::Network(e.to_string()))?;

    // 1. Submit job
    let submit_url = format!("{api_base}/api/v1/jobs");
    let mut payload = serde_json::json!({
        "sources": [args.repo],
        "export_formats": [args.format.as_str()],
        "token_budget": args.budget,
    });
    if let Some(ref r) = args.r#ref {
        payload["git_ref"] = serde_json::json!(r);
    }
    if let Some(ref s) = args.subpath {
        payload["subpath"] = serde_json::json!(s);
    }

    let res = client
        .post(&submit_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to submit job to remote API: {e}")))?;

    let status = res.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(CliError::Security(
            "API key required or unauthorized".to_string(),
        ));
    }
    if status == reqwest::StatusCode::FORBIDDEN {
        return Err(CliError::Security("Forbidden by remote API".to_string()));
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(CliError::Network(
            "Rate limit exceeded on remote API".to_string(),
        ));
    }
    if !status.is_success() {
        let err_body = res.text().await.unwrap_or_default();
        return Err(CliError::Remote(format!(
            "Remote API rejected job ({status}): {err_body}"
        )));
    }

    let job_res: serde_json::Value = res
        .json()
        .await
        .map_err(|e| CliError::Remote(format!("Malformed response from remote API: {e}")))?;
    let job_id = job_res["job_id"]
        .as_str()
        .or_else(|| job_res["id"].as_str())
        .ok_or_else(|| CliError::Remote("Missing job_id in API response".to_string()))?;

    writer.status(&format!(
        "Remote job submitted: {job_id}. Waiting for completion..."
    ));

    // 2. Poll job status
    let poll_url = format!("{api_base}/api/v1/jobs/{job_id}");
    let poll_deadline = Instant::now() + Duration::from_secs(args.timeout);
    loop {
        if Instant::now() > poll_deadline {
            return Err(CliError::Network(
                "Timed out waiting for remote job completion".to_string(),
            ));
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
        let poll_res = client
            .get(&poll_url)
            .send()
            .await
            .map_err(|e| CliError::Network(format!("Polling failed: {e}")))?;
        if !poll_res.status().is_success() {
            continue;
        }
        let status_json: serde_json::Value = poll_res
            .json()
            .await
            .map_err(|e| CliError::Remote(format!("Malformed poll response: {e}")))?;
        let job_status = status_json["status"].as_str().unwrap_or("Pending");
        match job_status {
            "Completed" => break,
            "Failed" => {
                let err_msg = status_json["error"]
                    .as_str()
                    .unwrap_or("Remote job execution failed");
                return Err(CliError::Extraction(err_msg.to_string()));
            }
            "Cancelled" => {
                return Err(CliError::Extraction(
                    "Job was cancelled on remote server".to_string(),
                ))
            }
            _ => {}
        }
    }

    // 3. Download artifact
    let artifact_url = format!(
        "{api_base}/api/v1/jobs/{job_id}/artifacts/{}",
        args.format.as_str()
    );
    let art_res = client
        .get(&artifact_url)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to download artifact: {e}")))?;
    if !art_res.status().is_success() {
        return Err(CliError::Extraction(format!(
            "Failed to retrieve artifact ({})",
            art_res.status()
        )));
    }
    let artifact_bytes = art_res
        .bytes()
        .await
        .map_err(|e| CliError::Network(format!("Failed reading artifact body: {e}")))?
        .to_vec();

    emit_repo_result(args, &artifact_bytes, 0, start_time.elapsed(), writer)
}

fn emit_repo_result(
    args: &RepoArgs,
    artifact_bytes: &[u8],
    tokens: usize,
    elapsed: Duration,
    writer: &OutputWriter,
) -> std::result::Result<i32, CliError> {
    let bytes_count = artifact_bytes.len();
    if writer.json {
        let content = if args.output == "-" {
            if args.format.is_binary() {
                Some(format!("<binary payload: {bytes_count} bytes>"))
            } else {
                Some(String::from_utf8_lossy(artifact_bytes).to_string())
            }
        } else {
            writer.write_payload(&args.output, artifact_bytes)?;
            None
        };
        let envelope = CliJsonEnvelope {
            success: true,
            url: args.repo.clone(),
            format: args.format.as_str().to_string(),
            tokens,
            bytes: bytes_count,
            duration_ms: elapsed.as_millis(),
            output_path: args.output.clone(),
            content,
            error: None,
            exit_code: Some(0),
        };
        writer.write_json_envelope(&envelope)?;
    } else {
        writer.write_payload(&args.output, artifact_bytes)?;
        if args.output != "-" {
            writer.status(&format!(
                "Successfully converted {} -> {} ({} tokens, {} bytes in {:.2?})",
                args.repo, args.output, tokens, bytes_count, elapsed
            ));
        }
    }
    Ok(0)
}
