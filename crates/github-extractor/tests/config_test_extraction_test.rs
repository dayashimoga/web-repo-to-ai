//! Integration tests for repository configuration and test file extraction in `crates/github-extractor`.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use web_repo_to_ai_cdm::SectionKind;
use web_repo_to_ai_github_extractor::{
    compute_git_blob_sha, ClassifiedFile, ConfigCategory, FileCategory, GitFileMode,
    GitHubApiClient, GitHubUrlParser, SourceExtractionConfig, TestCategory,
};

#[tokio::test]
async fn test_ci_cd_workflow_extraction() {
    let yaml_content = r#"name: CI
on:
  push:
    branches: [main]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo test
"#;
    let bytes = yaml_content.as_bytes().to_vec();
    let expected_sha = compute_git_blob_sha(&bytes);

    let route_bytes = bytes.clone();
    let app = Router::new().route(
        "/repos/:owner/:repo/git/blobs/:sha",
        get(move |_path: Path<(String, String, String)>| {
            let b = route_bytes.clone();
            async move { (StatusCode::OK, b).into_response() }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let target = GitHubUrlParser::parse_url("https://github.com/org/pipeline-repo").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let classified = ClassifiedFile {
        path: ".github/workflows/ci.yml".to_string(),
        category: FileCategory::Configuration,
        mode: GitFileMode::RegularFile,
        sha: expected_sha.clone(),
        size: Some(bytes.len() as u64),
        language: Some("yaml".to_string()),
        is_eligible: true,
        exclusion_reason: None,
    };

    let config = SourceExtractionConfig::default();
    let extracted = client
        .fetch_config_file(&target, &classified, &config)
        .await
        .unwrap();

    assert_eq!(extracted.path, ".github/workflows/ci.yml");
    assert_eq!(extracted.git_sha, expected_sha);
    assert_eq!(extracted.config_category, ConfigCategory::CiCd);
    assert_eq!(extracted.format, "yaml");
    assert_eq!(extracted.line_count, 10);
    assert!(extracted.content.contains("runs-on: ubuntu-latest"));

    let section = extracted.to_cdm_section(
        "sec-cfg-01",
        "https://github.com/org/pipeline-repo/blob/main/.github/workflows/ci.yml",
    );
    assert_eq!(section.kind, SectionKind::CodeFile);
    assert_eq!(section.language.as_deref(), Some("yaml"));
    assert_eq!(
        section.attributes.get("file_category"),
        Some(&"configuration".to_string())
    );
    assert_eq!(
        section.attributes.get("config_category"),
        Some(&"ci_cd".to_string())
    );
    assert_eq!(section.attributes.get("git_blob_sha"), Some(&expected_sha));
}

#[tokio::test]
async fn test_build_and_container_config_extraction() {
    let docker_content = "FROM rust:1.80-slim\nWORKDIR /app\nCOPY . .\nRUN cargo build --release\n";
    let bytes = docker_content.as_bytes().to_vec();
    let expected_sha = compute_git_blob_sha(&bytes);

    let route_bytes = bytes.clone();
    let app = Router::new().route(
        "/repos/:owner/:repo/git/blobs/:sha",
        get(move |_path: Path<(String, String, String)>| {
            let b = route_bytes.clone();
            async move { (StatusCode::OK, b).into_response() }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let target = GitHubUrlParser::parse_url("https://github.com/org/container-repo").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let classified = ClassifiedFile {
        path: "Dockerfile".to_string(),
        category: FileCategory::Configuration,
        mode: GitFileMode::RegularFile,
        sha: expected_sha.clone(),
        size: Some(bytes.len() as u64),
        language: Some("dockerfile".to_string()),
        is_eligible: true,
        exclusion_reason: None,
    };

    let config = SourceExtractionConfig::default();
    let extracted = client
        .fetch_config_file(&target, &classified, &config)
        .await
        .unwrap();

    assert_eq!(extracted.config_category, ConfigCategory::Build);
    assert_eq!(extracted.format, "dockerfile");
    assert!(extracted.content.contains("FROM rust:1.80-slim"));
}

#[tokio::test]
async fn test_linter_and_environment_extraction() {
    let fmt_content = "max_width = 100\nedition = \"2021\"\n";
    let bytes = fmt_content.as_bytes().to_vec();
    let expected_sha = compute_git_blob_sha(&bytes);

    let route_bytes = bytes.clone();
    let app = Router::new().route(
        "/repos/:owner/:repo/git/blobs/:sha",
        get(move |_path: Path<(String, String, String)>| {
            let b = route_bytes.clone();
            async move { (StatusCode::OK, b).into_response() }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let target = GitHubUrlParser::parse_url("https://github.com/org/lint-repo").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let classified = ClassifiedFile {
        path: "rustfmt.toml".to_string(),
        category: FileCategory::Configuration,
        mode: GitFileMode::RegularFile,
        sha: expected_sha.clone(),
        size: Some(bytes.len() as u64),
        language: Some("toml".to_string()),
        is_eligible: true,
        exclusion_reason: None,
    };

    let config = SourceExtractionConfig::default();
    let extracted = client
        .fetch_config_file(&target, &classified, &config)
        .await
        .unwrap();

    assert_eq!(extracted.config_category, ConfigCategory::LinterFormatter);
    assert_eq!(extracted.format, "toml");
    assert!(extracted.content.contains("max_width = 100"));
}

#[tokio::test]
async fn test_unit_and_integration_test_extraction() {
    let test_code = r#"#[tokio::test]
async fn test_execute_pipeline() {
    assert_eq!(2 + 2, 4);
}
"#;
    let bytes = test_code.as_bytes().to_vec();
    let expected_sha = compute_git_blob_sha(&bytes);

    let route_bytes = bytes.clone();
    let app = Router::new().route(
        "/repos/:owner/:repo/git/blobs/:sha",
        get(move |_path: Path<(String, String, String)>| {
            let b = route_bytes.clone();
            async move { (StatusCode::OK, b).into_response() }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let target = GitHubUrlParser::parse_url("https://github.com/org/test-repo").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let classified = ClassifiedFile {
        path: "crates/core/src/parser_test.rs".to_string(),
        category: FileCategory::TestFixture,
        mode: GitFileMode::RegularFile,
        sha: expected_sha.clone(),
        size: Some(bytes.len() as u64),
        language: Some("rust".to_string()),
        is_eligible: true,
        exclusion_reason: None,
    };

    let config = SourceExtractionConfig::default();
    let extracted = client
        .fetch_test_file(&target, &classified, &config)
        .await
        .unwrap();

    assert_eq!(extracted.path, "crates/core/src/parser_test.rs");
    assert_eq!(extracted.test_category, TestCategory::Unit);
    assert_eq!(extracted.language.as_deref(), Some("rust"));

    let section = extracted.to_cdm_section(
        "sec-test-01",
        "https://github.com/org/test-repo/blob/main/crates/core/src/parser_test.rs",
    );
    assert_eq!(section.kind, SectionKind::CodeFile);
    assert_eq!(
        section.attributes.get("file_category"),
        Some(&"test_fixture".to_string())
    );
    assert_eq!(
        section.attributes.get("test_category"),
        Some(&"unit".to_string())
    );
}

#[tokio::test]
async fn test_static_fixture_extraction() {
    let fixture_content = r#"{"id": 42, "status": "verified", "tags": ["alpha", "beta"]}"#;
    let bytes = fixture_content.as_bytes().to_vec();
    let expected_sha = compute_git_blob_sha(&bytes);

    let route_bytes = bytes.clone();
    let app = Router::new().route(
        "/repos/:owner/:repo/git/blobs/:sha",
        get(move |_path: Path<(String, String, String)>| {
            let b = route_bytes.clone();
            async move { (StatusCode::OK, b).into_response() }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let target = GitHubUrlParser::parse_url("https://github.com/org/fixture-repo").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let classified = ClassifiedFile {
        path: "tests/fixtures/response.json".to_string(),
        category: FileCategory::TestFixture,
        mode: GitFileMode::RegularFile,
        sha: expected_sha.clone(),
        size: Some(bytes.len() as u64),
        language: Some("json".to_string()),
        is_eligible: true,
        exclusion_reason: None,
    };

    let config = SourceExtractionConfig::default();
    let extracted = client
        .fetch_test_file(&target, &classified, &config)
        .await
        .unwrap();

    assert_eq!(extracted.test_category, TestCategory::Fixture);
    assert_eq!(extracted.language.as_deref(), Some("json"));
    assert!(extracted.content.contains("\"verified\""));
}

#[tokio::test]
async fn test_concurrent_batch_config_and_test_extraction() {
    let mut blob_store: HashMap<String, Vec<u8>> = HashMap::new();

    let ci_bytes = b"name: CI\non: [push]\n".to_vec();
    let ci_sha = compute_git_blob_sha(&ci_bytes);
    blob_store.insert(ci_sha.clone(), ci_bytes.clone());

    let ts_bytes = b"{\n  \"compilerOptions\": {\"target\": \"ES2022\"}\n}\n".to_vec();
    let ts_sha = compute_git_blob_sha(&ts_bytes);
    blob_store.insert(ts_sha.clone(), ts_bytes.clone());

    let docker_bytes = b"FROM alpine:3.20\n".to_vec();
    let docker_sha = compute_git_blob_sha(&docker_bytes);
    blob_store.insert(docker_sha.clone(), docker_bytes.clone());

    let unit_test_bytes = b"#[test]\nfn test_add() { assert_eq!(1+1, 2); }\n".to_vec();
    let unit_sha = compute_git_blob_sha(&unit_test_bytes);
    blob_store.insert(unit_sha.clone(), unit_test_bytes.clone());

    let fixture_bytes = b"{\"mock\": true}\n".to_vec();
    let fixture_sha = compute_git_blob_sha(&fixture_bytes);
    blob_store.insert(fixture_sha.clone(), fixture_bytes.clone());

    let shared_store = Arc::new(blob_store);
    let app = Router::new().route(
        "/repos/:owner/:repo/git/blobs/:sha",
        get(
            move |Path((_owner, _repo, sha)): Path<(String, String, String)>| {
                let store = shared_store.clone();
                async move {
                    if let Some(content) = store.get(&sha) {
                        (StatusCode::OK, content.clone()).into_response()
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

    let target = GitHubUrlParser::parse_url("https://github.com/monorepo/batch-test").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let files = vec![
        ClassifiedFile {
            path: ".github/workflows/ci.yml".to_string(),
            category: FileCategory::Configuration,
            mode: GitFileMode::RegularFile,
            sha: ci_sha,
            size: Some(ci_bytes.len() as u64),
            language: Some("yaml".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "tsconfig.json".to_string(),
            category: FileCategory::Configuration,
            mode: GitFileMode::RegularFile,
            sha: ts_sha,
            size: Some(ts_bytes.len() as u64),
            language: Some("json".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "Dockerfile".to_string(),
            category: FileCategory::Configuration,
            mode: GitFileMode::RegularFile,
            sha: docker_sha,
            size: Some(docker_bytes.len() as u64),
            language: Some("dockerfile".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "src/calc_test.rs".to_string(),
            category: FileCategory::TestFixture,
            mode: GitFileMode::RegularFile,
            sha: unit_sha,
            size: Some(unit_test_bytes.len() as u64),
            language: Some("rust".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "fixtures/data.json".to_string(),
            category: FileCategory::TestFixture,
            mode: GitFileMode::RegularFile,
            sha: fixture_sha,
            size: Some(fixture_bytes.len() as u64),
            language: Some("json".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        // Ineligible binary
        ClassifiedFile {
            path: "tests/fixtures/image.png".to_string(),
            category: FileCategory::Binary,
            mode: GitFileMode::RegularFile,
            sha: "1111111111111111111111111111111111111111".to_string(),
            size: Some(2048),
            language: None,
            is_eligible: false,
            exclusion_reason: Some("Binary image format".to_string()),
        },
        // Missing blob -> will fail
        ClassifiedFile {
            path: "missing/broken.yml".to_string(),
            category: FileCategory::Configuration,
            mode: GitFileMode::RegularFile,
            sha: "9999999999999999999999999999999999999999".to_string(),
            size: Some(50),
            language: Some("yaml".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
    ];

    let config = SourceExtractionConfig::default().with_concurrency(4);
    let report = client
        .extract_config_and_test_files(&target, &files, &config)
        .await;

    assert_eq!(report.extracted_configs.len(), 3);
    assert_eq!(report.extracted_tests.len(), 2);
    assert_eq!(report.total_extracted(), 5);
    assert_eq!(report.ci_workflows().len(), 1);
    assert_eq!(report.fixtures().len(), 1);
    assert_eq!(report.unsupported.len(), 1);
    assert_eq!(report.failed.len(), 1);

    // CDM sections
    let sections = report.to_cdm_sections("https://github.com/monorepo/batch-test/blob/main");
    assert_eq!(sections.len(), 5);
    for sec in &sections {
        assert_eq!(sec.kind, SectionKind::CodeFile);
    }

    // Completeness Report
    // Total discovered = 7, excluded = 0, extracted = 5 (3 configs + 2 tests), failed = 1, unsupported = 1
    let completeness = report.to_completeness_report(7, 0);
    assert!(completeness.formula_valid());
    assert_eq!(completeness.discovered_count, 7);
    assert_eq!(completeness.extracted_count, 5);
    assert_eq!(completeness.failed_count, 1);
    assert_eq!(completeness.unsupported_count, 1);
    assert_eq!(completeness.excluded_count, 0);
    assert_eq!(completeness.eligible_count, 6); // 5 extracted + 1 failed
}
