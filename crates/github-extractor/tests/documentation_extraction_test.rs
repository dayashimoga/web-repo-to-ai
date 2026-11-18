//! Integration tests for repository documentation extraction, format detection,
//! title/frontmatter parsing, and CDM prose section generation in `crates/github-extractor`.

use std::sync::Arc;

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use web_repo_to_ai_cdm::SectionKind;
use web_repo_to_ai_github_extractor::{
    compute_git_blob_sha, ClassifiedFile, DocFormat, DocType, ExtractedDocFile, FileCategory,
    GitFileMode, GitHubApiClient, GitHubUrlParser, SourceExtractionConfig,
};

#[tokio::test]
async fn test_live_http_readme_extraction_and_cdm_mapping() {
    let readme_content =
        "# Awesome Web-Repo Platform\n\nUniversal content converter for AI applications.\n";
    let bytes = readme_content.as_bytes().to_vec();
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

    let target = GitHubUrlParser::parse_url("https://github.com/test-org/test-repo").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let classified = ClassifiedFile {
        path: "README.md".to_string(),
        category: FileCategory::Documentation,
        mode: GitFileMode::RegularFile,
        sha: expected_sha.clone(),
        size: Some(bytes.len() as u64),
        language: Some("markdown".to_string()),
        is_eligible: true,
        exclusion_reason: None,
    };

    let config = SourceExtractionConfig::default();
    let doc = client
        .fetch_doc_file(&target, &classified, &config)
        .await
        .unwrap();

    assert_eq!(doc.path, "README.md");
    assert_eq!(doc.git_sha, expected_sha);
    assert_eq!(doc.doc_type, DocType::Readme);
    assert_eq!(doc.format, DocFormat::Markdown);
    assert_eq!(doc.title, "Awesome Web-Repo Platform");
    assert!(!doc.has_frontmatter);

    let section = doc.to_cdm_section(
        "sec-doc-1",
        "https://github.com/test-org/test-repo/blob/main/README.md",
    );
    assert_eq!(section.section_id, "sec-doc-1");
    assert_eq!(section.kind, SectionKind::Prose);
    assert_eq!(section.title.as_deref(), Some("Awesome Web-Repo Platform"));
    assert_eq!(section.language.as_deref(), Some("markdown"));
    assert_eq!(
        section.attributes.get("doc_type"),
        Some(&"readme".to_string())
    );
}

#[tokio::test]
async fn test_live_http_license_extraction() {
    let license_content =
        "MIT License\n\nCopyright (c) 2026 Test Org\n\nPermission is hereby granted...\n";
    let bytes = license_content.as_bytes().to_vec();
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

    let target = GitHubUrlParser::parse_url("https://github.com/test-org/test-repo").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let classified = ClassifiedFile {
        path: "LICENSE".to_string(),
        category: FileCategory::Documentation,
        mode: GitFileMode::RegularFile,
        sha: expected_sha.clone(),
        size: Some(bytes.len() as u64),
        language: None,
        is_eligible: true,
        exclusion_reason: None,
    };

    let config = SourceExtractionConfig::default();
    let doc = client
        .fetch_doc_file(&target, &classified, &config)
        .await
        .unwrap();

    assert_eq!(doc.path, "LICENSE");
    assert_eq!(doc.doc_type, DocType::License);
    assert_eq!(doc.format, DocFormat::PlainText);
    assert_eq!(doc.title, "LICENSE");
}

#[tokio::test]
async fn test_live_http_doc_with_yaml_frontmatter() {
    let frontmatter_content = "---\ntitle: \"Distributed Storage Architecture\"\ncategory: \"specs\"\n---\n\n# System Layout\n\nDetails here.\n";
    let bytes = frontmatter_content.as_bytes().to_vec();
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

    let target = GitHubUrlParser::parse_url("https://github.com/test-org/test-repo").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let classified = ClassifiedFile {
        path: "docs/ARCHITECTURE.md".to_string(),
        category: FileCategory::Documentation,
        mode: GitFileMode::RegularFile,
        sha: expected_sha.clone(),
        size: Some(bytes.len() as u64),
        language: Some("markdown".to_string()),
        is_eligible: true,
        exclusion_reason: None,
    };

    let config = SourceExtractionConfig::default();
    let doc = client
        .fetch_doc_file(&target, &classified, &config)
        .await
        .unwrap();

    assert_eq!(doc.path, "docs/ARCHITECTURE.md");
    assert_eq!(doc.doc_type, DocType::Architecture);
    assert!(doc.has_frontmatter);
    assert_eq!(doc.title, "Distributed Storage Architecture");
}

#[tokio::test]
async fn test_concurrent_batch_doc_extraction_with_readme_and_license_lookup() {
    let docs = vec![
        ("README.md", "# Main Readme\n"),
        ("LICENSE", "MIT License\n"),
        ("docs/ARCHITECTURE.md", "# Architecture Spec\n"),
        ("CONTRIBUTING.md", "# How to Contribute\n"),
        ("CHANGELOG.md", "# Change Log\n"),
    ];

    let mut mock_map = std::collections::HashMap::new();
    let mut classified_files = Vec::new();

    for (path, content) in docs {
        let bytes = content.as_bytes().to_vec();
        let sha = compute_git_blob_sha(&bytes);
        mock_map.insert(sha.clone(), bytes.clone());

        classified_files.push(ClassifiedFile {
            path: path.to_string(),
            category: FileCategory::Documentation,
            mode: GitFileMode::RegularFile,
            sha,
            size: Some(bytes.len() as u64),
            language: Some("markdown".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        });
    }

    let map_arc = Arc::new(mock_map);
    let app = Router::new().route(
        "/repos/:owner/:repo/git/blobs/:sha",
        get(
            move |Path((_owner, _repo, sha)): Path<(String, String, String)>| {
                let map = map_arc.clone();
                async move {
                    if let Some(bytes) = map.get(&sha) {
                        (StatusCode::OK, bytes.clone()).into_response()
                    } else {
                        (StatusCode::NOT_FOUND, "Not Found").into_response()
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

    let config = SourceExtractionConfig::default().with_concurrency(3);
    let report = client
        .extract_doc_files(&target, &classified_files, &config)
        .await;

    assert_eq!(report.extracted.len(), 5);
    assert_eq!(report.failed.len(), 0);

    // Verify specialized accessors
    let readme = report.readme().expect("Root README must be found");
    assert_eq!(readme.path, "README.md");
    assert_eq!(readme.title, "Main Readme");

    let license = report.license().expect("LICENSE must be found");
    assert_eq!(license.path, "LICENSE");

    let arch_docs = report.filter_by_type(DocType::Architecture);
    assert_eq!(arch_docs.len(), 1);
    assert_eq!(arch_docs[0].path, "docs/ARCHITECTURE.md");
}

#[tokio::test]
async fn test_partial_failure_and_completeness_report_formula() {
    let app = Router::new().route(
        "/repos/:owner/:repo/git/blobs/:sha",
        get(
            |Path((_owner, _repo, sha)): Path<(String, String, String)>| async move {
                if sha == "doc_sha_1" {
                    (StatusCode::OK, "# Doc 1\n").into_response()
                } else if sha == "doc_sha_2" {
                    (StatusCode::OK, "# Doc 2\n").into_response()
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Server Error").into_response()
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
            path: "docs/one.md".to_string(),
            category: FileCategory::Documentation,
            mode: GitFileMode::RegularFile,
            sha: "doc_sha_1".to_string(),
            size: Some(10),
            language: Some("markdown".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "docs/two.md".to_string(),
            category: FileCategory::Documentation,
            mode: GitFileMode::RegularFile,
            sha: "doc_sha_2".to_string(),
            size: Some(10),
            language: Some("markdown".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "docs/missing.md".to_string(),
            category: FileCategory::Documentation,
            mode: GitFileMode::RegularFile,
            sha: "doc_fail_sha".to_string(),
            size: Some(10),
            language: Some("markdown".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
    ];

    let config = SourceExtractionConfig::default().with_verify_git_sha(false);
    let report = client
        .extract_doc_files(&target, &classified_files, &config)
        .await;

    assert_eq!(report.extracted.len(), 2);
    assert_eq!(report.failed.len(), 1);

    // Discovered = 6: 2 extracted + 1 failed + 3 excluded
    let cdm = report.to_completeness_report(6, 3);
    assert!(cdm.formula_valid());
    assert_eq!(cdm.discovered_count, 6);
    assert_eq!(cdm.extracted_count, 2);
    assert_eq!(cdm.failed_count, 1);
    assert_eq!(cdm.excluded_count, 3);
}

#[tokio::test]
async fn test_cdm_sections_generation_for_docs() {
    let text = "# Developer Manual\n\nSetup steps.\n";
    let bytes = text.as_bytes().to_vec();
    let blob_sha = compute_git_blob_sha(&bytes);
    let blob = web_repo_to_ai_github_extractor::GitBlob::new(blob_sha, bytes, "raw");

    let config = SourceExtractionConfig::default();
    let doc = ExtractedDocFile::from_blob("docs/MANUAL.md", &blob, &config).unwrap();

    let mut report = web_repo_to_ai_github_extractor::DocExtractionReport::new();
    report.add_extracted(doc);

    let sections = report.to_cdm_sections("https://github.com/org/repo/blob/main");
    assert_eq!(sections.len(), 1);

    let sec = &sections[0];
    assert_eq!(sec.section_id, "sec-doc-0001");
    assert_eq!(sec.kind, SectionKind::Prose);
    assert_eq!(sec.original_path.as_deref(), Some("docs/MANUAL.md"));
    assert_eq!(sec.title.as_deref(), Some("Developer Manual"));
    assert_eq!(sec.attributes.get("doc_type"), Some(&"guide".to_string()));
}
