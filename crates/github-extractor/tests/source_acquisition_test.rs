//! Integration tests for source code acquisition, blob fetching, Git SHA verification,
//! and CDM section generation in `crates/github-extractor`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use web_repo_to_ai_cdm::SectionKind;
use web_repo_to_ai_github_extractor::{
    compute_blake3_hash, compute_git_blob_sha, compute_sha256_hash, ClassifiedFile,
    ExtractedSourceFile, FileCategory, GitFileMode, GitHubApiClient, GitHubExtractorError,
    GitHubUrlParser, SourceExtractionConfig, SourceExtractionReport,
};

#[tokio::test]
async fn test_live_http_raw_blob_fetch_and_sha_verification() {
    let rust_code = "pub fn add(x: i32, y: i32) -> i32 {\n    x + y\n}\n";
    let blob_bytes = rust_code.as_bytes().to_vec();
    let expected_sha = compute_git_blob_sha(&blob_bytes);

    let route_sha = expected_sha.clone();
    let route_bytes = blob_bytes.clone();

    let app = Router::new().route(
        "/repos/:owner/:repo/git/blobs/:sha",
        get(
            move |Path((_owner, _repo, sha)): Path<(String, String, String)>| {
                let r_sha = route_sha.clone();
                let r_bytes = route_bytes.clone();
                async move {
                    if sha == r_sha {
                        let mut headers = HeaderMap::new();
                        headers.insert(
                            axum::http::header::CONTENT_TYPE,
                            "application/vnd.github.raw".parse().unwrap(),
                        );
                        (StatusCode::OK, headers, r_bytes).into_response()
                    } else {
                        (StatusCode::NOT_FOUND, "Blob not found").into_response()
                    }
                }
            },
        ),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let target = GitHubUrlParser::parse_url("https://github.com/test-org/test-repo").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let blob = client.get_blob(&target, &expected_sha).await.unwrap();
    assert_eq!(blob.sha, expected_sha);
    assert_eq!(blob.raw_bytes, blob_bytes);
    assert!(blob.verify_git_sha());

    let classified = ClassifiedFile {
        path: "src/math.rs".to_string(),
        category: FileCategory::SourceCode,
        mode: GitFileMode::RegularFile,
        sha: expected_sha.clone(),
        size: Some(blob_bytes.len() as u64),
        language: Some("rust".to_string()),
        is_eligible: true,
        exclusion_reason: None,
    };

    let config = SourceExtractionConfig::default();
    let extracted = client
        .fetch_source_file(&target, &classified, &config)
        .await
        .unwrap();

    assert_eq!(extracted.path, "src/math.rs");
    assert_eq!(extracted.git_sha, expected_sha);
    assert_eq!(extracted.content, rust_code);
    assert_eq!(extracted.line_count, 3);
    assert_eq!(extracted.language.as_deref(), Some("rust"));
    assert_eq!(extracted.blake3_hash, compute_blake3_hash(&blob_bytes));
    assert_eq!(extracted.sha256_hash, compute_sha256_hash(&blob_bytes));
}

#[tokio::test]
async fn test_live_http_base64_blob_fetch_and_sha_verification() {
    let ts_code = "export function greet(name: string): string {\n  return `Hello, ${name}!`;\n}\n";
    let blob_bytes = ts_code.as_bytes().to_vec();
    let expected_sha = compute_git_blob_sha(&blob_bytes);

    // Simple base64 encoding
    let mut base64_payload = String::new();
    let clean = blob_bytes.clone();
    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    for chunk in clean.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);

        let triple = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        base64_payload.push(chars.as_bytes()[((triple >> 18) & 0x3F) as usize] as char);
        base64_payload.push(chars.as_bytes()[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            base64_payload.push(chars.as_bytes()[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            base64_payload.push('=');
        }
        if chunk.len() > 2 {
            base64_payload.push(chars.as_bytes()[(triple & 0x3F) as usize] as char);
        } else {
            base64_payload.push('=');
        }
    }

    let json_body = serde_json::json!({
        "sha": expected_sha,
        "size": blob_bytes.len(),
        "encoding": "base64",
        "content": base64_payload
    })
    .to_string();

    let app = Router::new().route(
        "/repos/:owner/:repo/git/blobs/:sha",
        get(move |_path: Path<(String, String, String)>| {
            let body = json_body.clone();
            async move {
                let mut headers = HeaderMap::new();
                headers.insert(
                    axum::http::header::CONTENT_TYPE,
                    "application/json".parse().unwrap(),
                );
                (StatusCode::OK, headers, body).into_response()
            }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let target = GitHubUrlParser::parse_url("https://github.com/test-org/test-repo").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let blob = client.get_blob(&target, &expected_sha).await.unwrap();
    assert_eq!(blob.sha, expected_sha);
    assert_eq!(blob.raw_bytes, blob_bytes);
    assert!(blob.verify_git_sha());
    assert_eq!(blob.content_str().unwrap(), ts_code);
}

#[tokio::test]
async fn test_live_http_blob_sha_mismatch_error() {
    let app = Router::new().route(
        "/repos/:owner/:repo/git/blobs/:sha",
        get(|_path: Path<(String, String, String)>| async {
            // Return corrupt bytes not matching requested SHA
            (StatusCode::OK, "corrupted content not matching sha").into_response()
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let target = GitHubUrlParser::parse_url("https://github.com/test-org/test-repo").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let classified = ClassifiedFile {
        path: "src/tampered.rs".to_string(),
        category: FileCategory::SourceCode,
        mode: GitFileMode::RegularFile,
        sha: "expected_sha_123456789012345678901234567890".to_string(),
        size: Some(100),
        language: Some("rust".to_string()),
        is_eligible: true,
        exclusion_reason: None,
    };

    let config = SourceExtractionConfig::default().with_verify_git_sha(true);
    let result = client
        .fetch_source_file(&target, &classified, &config)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        GitHubExtractorError::BlobError { sha: _, message } => {
            assert!(message.contains("Git blob SHA-1 mismatch"));
        }
        other => panic!("Expected BlobError, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_binary_detection_and_unsupported_categorization() {
    let binary_bytes = b"header\0binary\0data".to_vec();
    let expected_sha = compute_git_blob_sha(&binary_bytes);

    let route_bytes = binary_bytes.clone();
    let app = Router::new().route(
        "/repos/:owner/:repo/git/blobs/:sha",
        get(move |_path: Path<(String, String, String)>| {
            let r_bytes = route_bytes.clone();
            async move { (StatusCode::OK, r_bytes).into_response() }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let target = GitHubUrlParser::parse_url("https://github.com/test-org/test-repo").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let classified = ClassifiedFile {
        path: "data/file.dat".to_string(),
        category: FileCategory::SourceCode, // Initially marked source code, but content is binary!
        mode: GitFileMode::RegularFile,
        sha: expected_sha,
        size: Some(binary_bytes.len() as u64),
        language: None,
        is_eligible: true,
        exclusion_reason: None,
    };

    let config = SourceExtractionConfig::default();
    let report = client
        .extract_source_files(&target, &[classified], &config)
        .await;

    assert_eq!(report.extracted.len(), 0);
    assert_eq!(report.unsupported.len(), 1);
    assert_eq!(report.unsupported[0].path, "data/file.dat");
}

#[tokio::test]
async fn test_concurrent_batch_extraction_with_semaphore() {
    let files_data = vec![
        ("src/lib.rs", "pub mod app;\n"),
        ("src/app.rs", "pub fn run() {}\n"),
        ("src/utils.rs", "pub fn helper() -> bool { true }\n"),
        ("src/config.rs", "pub struct Config;\n"),
    ];

    let mut mock_map = std::collections::HashMap::new();
    let mut classified_files = Vec::new();

    for (path, content) in files_data {
        let bytes = content.as_bytes().to_vec();
        let sha = compute_git_blob_sha(&bytes);
        mock_map.insert(sha.clone(), bytes.clone());

        classified_files.push(ClassifiedFile {
            path: path.to_string(),
            category: FileCategory::SourceCode,
            mode: GitFileMode::RegularFile,
            sha,
            size: Some(bytes.len() as u64),
            language: Some("rust".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        });
    }

    let map_arc = Arc::new(mock_map);
    let active_concurrency = Arc::new(AtomicUsize::new(0));
    let max_concurrency_seen = Arc::new(AtomicUsize::new(0));

    let app = Router::new().route(
        "/repos/:owner/:repo/git/blobs/:sha",
        get({
            let map = map_arc.clone();
            let active = active_concurrency.clone();
            let max_seen = max_concurrency_seen.clone();
            move |Path((_owner, _repo, sha)): Path<(String, String, String)>| {
                let map = map.clone();
                let active = active.clone();
                let max_seen = max_seen.clone();
                async move {
                    let cur = active.fetch_add(1, Ordering::SeqCst) + 1;
                    max_seen.fetch_max(cur, Ordering::SeqCst);
                    tokio::time::sleep(std::time::Duration::from_millis(15)).await;
                    active.fetch_sub(1, Ordering::SeqCst);

                    if let Some(bytes) = map.get(&sha) {
                        (StatusCode::OK, bytes.clone()).into_response()
                    } else {
                        (StatusCode::NOT_FOUND, "Not Found").into_response()
                    }
                }
            }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let target = GitHubUrlParser::parse_url("https://github.com/test-org/test-repo").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    // Bounded concurrency = 2
    let config = SourceExtractionConfig::default().with_concurrency(2);
    let report = client
        .extract_source_files(&target, &classified_files, &config)
        .await;

    assert_eq!(report.extracted.len(), 4);
    assert_eq!(report.failed.len(), 0);
    assert!(report.is_all_successful());
    assert!(report.total_bytes > 0);
    assert!(report.total_lines > 0);

    // Concurrency ceiling respected
    let peak = max_concurrency_seen.load(Ordering::SeqCst);
    assert!(
        peak <= 2,
        "Peak concurrency ({peak}) must not exceed limit (2)"
    );
}

#[tokio::test]
async fn test_partial_failure_handling_and_cdm_completeness_report() {
    let app = Router::new().route(
        "/repos/:owner/:repo/git/blobs/:sha",
        get(
            |Path((_owner, _repo, sha)): Path<(String, String, String)>| async move {
                if sha == "good_sha_1" {
                    let content = b"fn first() {}\n";
                    (StatusCode::OK, content.to_vec()).into_response()
                } else if sha == "good_sha_2" {
                    let content = b"fn second() {}\n";
                    (StatusCode::OK, content.to_vec()).into_response()
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Upstream GitHub Error").into_response()
                }
            },
        ),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let target = GitHubUrlParser::parse_url("https://github.com/test-org/test-repo").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let classified_files = vec![
        ClassifiedFile {
            path: "src/one.rs".to_string(),
            category: FileCategory::SourceCode,
            mode: GitFileMode::RegularFile,
            sha: "good_sha_1".to_string(),
            size: Some(15),
            language: Some("rust".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "src/two.rs".to_string(),
            category: FileCategory::SourceCode,
            mode: GitFileMode::RegularFile,
            sha: "good_sha_2".to_string(),
            size: Some(16),
            language: Some("rust".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "src/broken.rs".to_string(),
            category: FileCategory::SourceCode,
            mode: GitFileMode::RegularFile,
            sha: "broken_sha".to_string(),
            size: Some(20),
            language: Some("rust".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
    ];

    let config = SourceExtractionConfig::default().with_verify_git_sha(false);
    let report = client
        .extract_source_files(&target, &classified_files, &config)
        .await;

    assert_eq!(report.extracted.len(), 2);
    assert_eq!(report.failed.len(), 1);
    assert!(!report.is_all_successful());
    assert_eq!(report.failed[0].path, "src/broken.rs");

    // Total discovered = 5: 2 extracted + 1 failed + 2 excluded
    let cdm = report.to_completeness_report(5, 2);
    assert!(cdm.formula_valid());
    assert_eq!(cdm.discovered_count, 5);
    assert_eq!(cdm.extracted_count, 2);
    assert_eq!(cdm.failed_count, 1);
    assert_eq!(cdm.excluded_count, 2);
    assert_eq!(cdm.unsupported_count, 0);
}

#[tokio::test]
async fn test_cdm_sections_generation() {
    let mut report = SourceExtractionReport::new();

    let f1 = ExtractedSourceFile {
        path: "src/main.rs".to_string(),
        git_sha: "sha_main".to_string(),
        content: "fn main() {}\n".to_string(),
        byte_size: 13,
        line_count: 1,
        language: Some("rust".to_string()),
        blake3_hash: compute_blake3_hash(b"fn main() {}\n"),
        sha256_hash: compute_sha256_hash(b"fn main() {}\n"),
        is_truncated: false,
    };
    report.add_extracted(f1);

    let sections = report.to_cdm_sections("https://github.com/org/repo/blob/main");
    assert_eq!(sections.len(), 1);

    let sec = &sections[0];
    assert_eq!(sec.section_id, "sec-source-0001");
    assert_eq!(sec.kind, SectionKind::CodeFile);
    assert_eq!(sec.original_path.as_deref(), Some("src/main.rs"));
    assert_eq!(
        sec.source_url,
        "https://github.com/org/repo/blob/main/src/main.rs"
    );
    assert_eq!(sec.language.as_deref(), Some("rust"));
    assert_eq!(sec.body, "fn main() {}\n");
    assert_eq!(
        sec.attributes.get("git_blob_sha"),
        Some(&"sha_main".to_string())
    );
}
