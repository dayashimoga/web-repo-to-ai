//! # URL Conversion Command Execution Engine

use std::sync::Arc;
use std::time::{Duration, Instant};

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
use web_repo_to_ai_tokenizer::count_tokens_for_model;
use web_repo_to_ai_tokenizer::model::TargetModel;
use web_repo_to_ai_url_security::{SecureDnsResolver, UrlSecurityPolicy, ValidatedUrl};
use web_repo_to_ai_web_extractor::article::ArticleExtractor;
use web_repo_to_ai_web_extractor::dom::DomParser;
use web_repo_to_ai_web_extractor::http::{HttpTransport, WebFetcher};
use web_repo_to_ai_web_extractor::url::WebTarget;

use crate::args::UrlArgs;
use crate::error::CliError;
use crate::output::{export_artifact, CliJsonEnvelope, OutputWriter};

/// Executes the `url` command with standard transports.
///
/// # Errors
///
/// Returns [`CliError`] on invalid input, network failure, extraction error, or budget violation.
pub async fn execute_url(
    args: &UrlArgs,
    writer: &OutputWriter,
) -> std::result::Result<i32, CliError> {
    execute_url_with_options(args, writer, None, None).await
}

/// Executes the `url` command with an optional custom transport (useful for tests).
///
/// # Errors
///
/// Returns [`CliError`] on invalid input, network failure, extraction error, or budget violation.
pub async fn execute_url_with_transport(
    args: &UrlArgs,
    writer: &OutputWriter,
    custom_transport: Option<Arc<dyn HttpTransport>>,
) -> std::result::Result<i32, CliError> {
    execute_url_with_options(args, writer, custom_transport, None).await
}

/// Executes the `url` command with optional custom transport and DNS resolver (useful for hermetic tests).
///
/// # Errors
///
/// Returns [`CliError`] on invalid input, network failure, extraction error, or budget violation.
pub async fn execute_url_with_options(
    args: &UrlArgs,
    writer: &OutputWriter,
    custom_transport: Option<Arc<dyn HttpTransport>>,
    custom_resolver: Option<SecureDnsResolver>,
) -> std::result::Result<i32, CliError> {
    let start_time = Instant::now();
    let policy = UrlSecurityPolicy::default();

    let validated_url = match ValidatedUrl::parse(&args.url, &policy) {
        Ok(u) => u,
        Err(e) => {
            let err = CliError::from(e);
            if writer.json {
                let _ = writer.write_json_envelope(&CliJsonEnvelope {
                    success: false,
                    url: args.url.clone(),
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
            return Err(err);
        }
    };

    if args.api_url.is_some() {
        execute_remote(args, &validated_url, writer, start_time).await
    } else {
        execute_local(
            args,
            &validated_url,
            writer,
            start_time,
            custom_transport,
            custom_resolver,
        )
        .await
    }
}

async fn execute_remote(
    args: &UrlArgs,
    validated_url: &ValidatedUrl,
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
        validated_url.as_str(),
        api_base
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
    let payload = serde_json::json!({
        "sources": [validated_url.as_str()],
        "export_formats": [args.format.as_str()],
        "token_budget": args.budget,
    });

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

    emit_result(args, &artifact_bytes, 0, start_time.elapsed(), writer)
}

async fn execute_local(
    args: &UrlArgs,
    validated_url: &ValidatedUrl,
    writer: &OutputWriter,
    start_time: Instant,
    custom_transport: Option<Arc<dyn HttpTransport>>,
    custom_resolver: Option<SecureDnsResolver>,
) -> std::result::Result<i32, CliError> {
    writer.status(&format!("Fetching {}...", validated_url.as_str()));

    let policy = UrlSecurityPolicy::default();
    let resolver = custom_resolver.unwrap_or_else(|| SecureDnsResolver::new(policy.clone()));
    let mut fetcher =
        WebFetcher::new(policy, resolver).with_total_timeout(Duration::from_secs(args.timeout));
    if let Some(ref ua) = args.user_agent {
        fetcher = fetcher.with_user_agent(ua.clone());
    }
    if let Some(transport) = custom_transport {
        fetcher = fetcher.with_transport(transport);
    }

    let target = WebTarget::new(validated_url.clone());
    let response = fetcher.fetch(&target).await.map_err(|e| match e {
        web_repo_to_ai_web_extractor::WebExtractorError::UrlSecurity(sec) => CliError::from(sec),
        other => CliError::from(other),
    })?;

    if !response.is_success() {
        return Err(CliError::Extraction(format!(
            "HTTP fetch failed with status {}: {}",
            response.status, response.status_text
        )));
    }

    writer.debug(&format!(
        "Fetched {} bytes (HTTP {}) from {}",
        response.body.len(),
        response.status,
        response.pinned_ip
    ));

    // Article extraction
    let html_text = response.text_lossy();
    let extractor = ArticleExtractor::new();
    let article_opt = extractor.extract(&html_text, Some(validated_url)).ok();

    let (content_title, content_body, byline) = if let Some(article) = article_opt {
        if !article.content_text.trim().is_empty() {
            let title = if article.title.trim().is_empty() {
                validated_url.as_str().to_string()
            } else {
                article.title
            };
            (title, article.content_text, article.byline)
        } else {
            fallback_dom_extract(&html_text, validated_url)?
        }
    } else {
        fallback_dom_extract(&html_text, validated_url)?
    };

    // Construct CDM Document
    let mut doc_builder = CdmDocumentBuilder::new(
        SourceIdentity::new(validated_url.as_str(), validated_url.as_str()),
        SourceType::WebpageStatic,
        validated_url.as_str(),
    );
    doc_builder = doc_builder.title(&content_title);
    if let Some(author) = byline {
        doc_builder = doc_builder.metadata(CdmDocumentMetadata {
            title: Some(content_title.clone()),
            authors: vec![author],
            ..Default::default()
        });
    }

    let section = CdmSectionBuilder::new(
        "main",
        validated_url.as_str(),
        SectionKind::Prose,
        content_body,
    )
    .title(&content_title)
    .build();

    doc_builder = doc_builder.add_section(section);
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
            "Content size ({} tokens) exceeds budget ({}); optimizing...",
            estimated_tokens, args.budget
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

    emit_result(
        args,
        &artifact_bytes,
        final_tokens,
        start_time.elapsed(),
        writer,
    )
}

fn fallback_dom_extract(
    html_text: &str,
    validated_url: &ValidatedUrl,
) -> std::result::Result<(String, String, Option<String>), CliError> {
    let dom_parser = DomParser::default();
    let cleaned_opt = dom_parser.parse(html_text, Some(validated_url)).ok();

    if let Some(cleaned) = cleaned_opt {
        if !cleaned.cleaned_text.trim().is_empty() {
            let title = cleaned
                .metadata
                .title
                .unwrap_or_else(|| validated_url.as_str().to_string());
            let author = cleaned.metadata.author;
            return Ok((title, cleaned.cleaned_text, author));
        }
    }

    if !html_text.trim().is_empty() {
        Ok((
            validated_url.as_str().to_string(),
            html_text.to_string(),
            None,
        ))
    } else {
        Err(CliError::Extraction(
            "No extractable content found in web page".to_string(),
        ))
    }
}

fn emit_result(
    args: &UrlArgs,
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
            url: args.url.clone(),
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
                args.url, args.output, tokens, bytes_count, elapsed
            ));
        }
    }
    Ok(0)
}
