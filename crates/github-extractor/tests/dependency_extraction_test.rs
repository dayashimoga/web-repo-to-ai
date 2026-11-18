//! Integration tests for package manifest acquisition, dependency parsing,
//! lockfile extraction, and CDM manifest section generation in `crates/github-extractor`.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use web_repo_to_ai_cdm::SectionKind;
use web_repo_to_ai_github_extractor::{
    compute_git_blob_sha, ClassifiedFile, DependencyType, FileCategory, GitFileMode,
    GitHubApiClient, GitHubUrlParser, ManifestRole, PackageEcosystem, SourceExtractionConfig,
};

#[tokio::test]
async fn test_cargo_toml_extraction_and_parsing() {
    let cargo_content = r#"
[package]
name = "hyper-drive"
version = "2.4.1"
description = "Fast async warp engine"
license = "MIT"

[dependencies]
tokio = "1.40"
serde = { version = "1.0", features = ["derive"], optional = true }

[dev-dependencies]
criterion = "0.5"
"#;
    let bytes = cargo_content.as_bytes().to_vec();
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

    let target = GitHubUrlParser::parse_url("https://github.com/engineers/hyper-drive").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let classified = ClassifiedFile {
        path: "Cargo.toml".to_string(),
        category: FileCategory::DependencyManifest,
        mode: GitFileMode::RegularFile,
        sha: expected_sha.clone(),
        size: Some(bytes.len() as u64),
        language: Some("toml".to_string()),
        is_eligible: true,
        exclusion_reason: None,
    };

    let config = SourceExtractionConfig::default();
    let extracted = client
        .fetch_dependency_file(&target, &classified, &config)
        .await
        .unwrap();

    assert_eq!(extracted.path, "Cargo.toml");
    assert_eq!(extracted.git_sha, expected_sha);
    assert_eq!(extracted.ecosystem, PackageEcosystem::Cargo);
    assert_eq!(extracted.role, ManifestRole::Manifest);

    let parsed = extracted.parsed.as_ref().expect("Cargo.toml must parse");
    assert_eq!(parsed.package_name.as_deref(), Some("hyper-drive"));
    assert_eq!(parsed.package_version.as_deref(), Some("2.4.1"));
    assert_eq!(
        parsed.description.as_deref(),
        Some("Fast async warp engine")
    );
    assert_eq!(parsed.license.as_deref(), Some("MIT"));
    assert_eq!(parsed.dependencies.len(), 3);

    let tokio_dep = parsed.find_dependency("tokio").expect("tokio found");
    assert_eq!(tokio_dep.version_req.as_deref(), Some("1.40"));
    assert_eq!(tokio_dep.dependency_type, DependencyType::Direct);

    let serde_dep = parsed.find_dependency("serde").expect("serde found");
    assert!(serde_dep.optional);
    assert_eq!(serde_dep.features, vec!["derive".to_string()]);

    let criterion_dep = parsed
        .find_dependency("criterion")
        .expect("criterion found");
    assert_eq!(criterion_dep.dependency_type, DependencyType::Dev);

    let section = extracted.to_cdm_section(
        "sec-man-01",
        "https://github.com/engineers/hyper-drive/blob/main/Cargo.toml",
    );
    assert_eq!(section.kind, SectionKind::Manifest);
    assert_eq!(section.language.as_deref(), Some("toml"));
    assert_eq!(
        section.attributes.get("ecosystem"),
        Some(&"cargo".to_string())
    );
    assert_eq!(
        section.attributes.get("manifest_role"),
        Some(&"manifest".to_string())
    );
    assert_eq!(
        section.attributes.get("package_name"),
        Some(&"hyper-drive".to_string())
    );
    assert_eq!(
        section.attributes.get("package_version"),
        Some(&"2.4.1".to_string())
    );
    assert_eq!(
        section.attributes.get("dependency_count"),
        Some(&"3".to_string())
    );
}

#[tokio::test]
async fn test_package_json_extraction_and_parsing() {
    let pkg_json = r#"{
  "name": "@acme/design-system",
  "version": "3.1.0",
  "description": "Component design system",
  "license": "Apache-2.0",
  "dependencies": {
    "react": "^18.3.0",
    "clsx": "^2.1.0"
  },
  "devDependencies": {
    "typescript": "^5.5.0",
    "vite": "^5.4.0"
  },
  "peerDependencies": {
    "react-dom": "^18.3.0"
  }
}"#;
    let bytes = pkg_json.as_bytes().to_vec();
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

    let target = GitHubUrlParser::parse_url("https://github.com/acme/design-system").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let classified = ClassifiedFile {
        path: "packages/ui/package.json".to_string(),
        category: FileCategory::DependencyManifest,
        mode: GitFileMode::RegularFile,
        sha: expected_sha.clone(),
        size: Some(bytes.len() as u64),
        language: Some("json".to_string()),
        is_eligible: true,
        exclusion_reason: None,
    };

    let config = SourceExtractionConfig::default();
    let extracted = client
        .fetch_dependency_file(&target, &classified, &config)
        .await
        .unwrap();

    assert_eq!(extracted.ecosystem, PackageEcosystem::Npm);
    assert_eq!(extracted.role, ManifestRole::Manifest);

    let parsed = extracted.parsed.as_ref().expect("package.json parsed");
    assert_eq!(parsed.package_name.as_deref(), Some("@acme/design-system"));
    assert_eq!(parsed.package_version.as_deref(), Some("3.1.0"));
    assert_eq!(parsed.dependencies.len(), 5);

    let react = parsed.find_dependency("react").expect("react found");
    assert_eq!(react.dependency_type, DependencyType::Direct);
    assert_eq!(react.version_req.as_deref(), Some("^18.3.0"));

    let ts = parsed.find_dependency("typescript").expect("ts found");
    assert_eq!(ts.dependency_type, DependencyType::Dev);

    let react_dom = parsed
        .find_dependency("react-dom")
        .expect("react-dom found");
    assert_eq!(react_dom.dependency_type, DependencyType::Peer);
}

#[tokio::test]
async fn test_requirements_txt_extraction_and_parsing() {
    let reqs = r#"
# Production dependencies
torch>=2.2.0
transformers==4.40.1
numpy~=1.26.0
accelerate
"#;
    let bytes = reqs.as_bytes().to_vec();
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

    let target = GitHubUrlParser::parse_url("https://github.com/ai-lab/models").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let classified = ClassifiedFile {
        path: "requirements.txt".to_string(),
        category: FileCategory::DependencyManifest,
        mode: GitFileMode::RegularFile,
        sha: expected_sha.clone(),
        size: Some(bytes.len() as u64),
        language: Some("python".to_string()),
        is_eligible: true,
        exclusion_reason: None,
    };

    let config = SourceExtractionConfig::default();
    let extracted = client
        .fetch_dependency_file(&target, &classified, &config)
        .await
        .unwrap();

    assert_eq!(extracted.ecosystem, PackageEcosystem::Pip);
    assert_eq!(extracted.role, ManifestRole::Manifest);

    let parsed = extracted.parsed.as_ref().expect("requirements.txt parsed");
    assert_eq!(parsed.dependencies.len(), 4);

    let torch = parsed.find_dependency("torch").expect("torch found");
    assert_eq!(torch.version_req.as_deref(), Some(">=2.2.0"));

    let accelerate = parsed
        .find_dependency("accelerate")
        .expect("accelerate found");
    assert_eq!(accelerate.version_req, None);
}

#[tokio::test]
async fn test_go_mod_extraction_and_parsing() {
    let go_mod_content = r#"
module github.com/protocol/node

go 1.22.4

require (
	github.com/ethereum/go-ethereum v1.14.8
	github.com/libp2p/go-libp2p v0.35.1
	github.com/stretchr/testify v1.9.0 // indirect
)
"#;
    let bytes = go_mod_content.as_bytes().to_vec();
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

    let target = GitHubUrlParser::parse_url("https://github.com/protocol/node").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let classified = ClassifiedFile {
        path: "go.mod".to_string(),
        category: FileCategory::DependencyManifest,
        mode: GitFileMode::RegularFile,
        sha: expected_sha.clone(),
        size: Some(bytes.len() as u64),
        language: Some("go".to_string()),
        is_eligible: true,
        exclusion_reason: None,
    };

    let config = SourceExtractionConfig::default();
    let extracted = client
        .fetch_dependency_file(&target, &classified, &config)
        .await
        .unwrap();

    assert_eq!(extracted.ecosystem, PackageEcosystem::GoMod);
    assert_eq!(extracted.role, ManifestRole::Manifest);

    let parsed = extracted.parsed.as_ref().expect("go.mod parsed");
    assert_eq!(
        parsed.package_name.as_deref(),
        Some("github.com/protocol/node")
    );
    assert_eq!(parsed.package_version.as_deref(), Some("1.22.4"));
    assert_eq!(parsed.dependencies.len(), 3);

    let geth = parsed
        .find_dependency("github.com/ethereum/go-ethereum")
        .expect("geth found");
    assert_eq!(geth.version_req.as_deref(), Some("v1.14.8"));
    assert_eq!(geth.dependency_type, DependencyType::Direct);

    let testify = parsed
        .find_dependency("github.com/stretchr/testify")
        .expect("testify found");
    assert_eq!(testify.dependency_type, DependencyType::Dev);
}

#[tokio::test]
async fn test_lockfile_acquisition_and_categorization() {
    let lock_content = r#"# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 3

[[package]]
name = "hyper-drive"
version = "2.4.1"
dependencies = [
 "tokio",
]

[[package]]
name = "tokio"
version = "1.40.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "525248fe6571393771f8afd1f79dc5b82a5de44888e32579696885f74577aa6c"
"#;
    let bytes = lock_content.as_bytes().to_vec();
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

    let target = GitHubUrlParser::parse_url("https://github.com/engineers/hyper-drive").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let classified = ClassifiedFile {
        path: "Cargo.lock".to_string(),
        category: FileCategory::GeneratedOrLockfile,
        mode: GitFileMode::RegularFile,
        sha: expected_sha.clone(),
        size: Some(bytes.len() as u64),
        language: Some("toml".to_string()),
        is_eligible: true,
        exclusion_reason: None,
    };

    let config = SourceExtractionConfig::default();
    let extracted = client
        .fetch_dependency_file(&target, &classified, &config)
        .await
        .unwrap();

    assert_eq!(extracted.path, "Cargo.lock");
    assert_eq!(extracted.ecosystem, PackageEcosystem::Cargo);
    assert_eq!(extracted.role, ManifestRole::Lockfile);
    assert!(extracted.parsed.is_none()); // Lockfiles are not parsed into DeclaredDependency
    assert!(extracted.content.contains("checksum ="));
}

#[tokio::test]
async fn test_concurrent_batch_dependency_extraction_and_completeness_report() {
    let mut blob_store: HashMap<String, Vec<u8>> = HashMap::new();

    let cargo_bytes =
        b"[package]\nname = \"core\"\nversion = \"0.1.0\"\n[dependencies]\ntokio = \"1.0\"\n"
            .to_vec();
    let cargo_sha = compute_git_blob_sha(&cargo_bytes);
    blob_store.insert(cargo_sha.clone(), cargo_bytes.clone());

    let pkg_bytes =
        b"{\"name\": \"web\", \"version\": \"1.0.0\", \"dependencies\": {\"svelte\": \"^5\"}}"
            .to_vec();
    let pkg_sha = compute_git_blob_sha(&pkg_bytes);
    blob_store.insert(pkg_sha.clone(), pkg_bytes.clone());

    let lock_bytes = b"# Cargo lockfile\nversion = 3\n".to_vec();
    let lock_sha = compute_git_blob_sha(&lock_bytes);
    blob_store.insert(lock_sha.clone(), lock_bytes.clone());

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

    let target = GitHubUrlParser::parse_url("https://github.com/monorepo/service").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let files = vec![
        ClassifiedFile {
            path: "Cargo.toml".to_string(),
            category: FileCategory::DependencyManifest,
            mode: GitFileMode::RegularFile,
            sha: cargo_sha,
            size: Some(cargo_bytes.len() as u64),
            language: Some("toml".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "apps/web/package.json".to_string(),
            category: FileCategory::DependencyManifest,
            mode: GitFileMode::RegularFile,
            sha: pkg_sha,
            size: Some(pkg_bytes.len() as u64),
            language: Some("json".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "Cargo.lock".to_string(),
            category: FileCategory::GeneratedOrLockfile,
            mode: GitFileMode::RegularFile,
            sha: lock_sha,
            size: Some(lock_bytes.len() as u64),
            language: Some("toml".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        // Ineligible binary
        ClassifiedFile {
            path: "assets/logo.png".to_string(),
            category: FileCategory::Binary,
            mode: GitFileMode::RegularFile,
            sha: "1111111111111111111111111111111111111111".to_string(),
            size: Some(1024),
            language: None,
            is_eligible: false,
            exclusion_reason: Some("Binary format".to_string()),
        },
        // Missing blob -> will fail
        ClassifiedFile {
            path: "broken/package.json".to_string(),
            category: FileCategory::DependencyManifest,
            mode: GitFileMode::RegularFile,
            sha: "9999999999999999999999999999999999999999".to_string(),
            size: Some(100),
            language: Some("json".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
    ];

    let config = SourceExtractionConfig::default().with_concurrency(4);
    let report = client
        .extract_dependency_files(&target, &files, &config)
        .await;

    assert_eq!(report.extracted.len(), 3);
    assert_eq!(report.manifests().len(), 2);
    assert_eq!(report.lockfiles().len(), 1);
    assert_eq!(report.unsupported.len(), 1);
    assert_eq!(report.failed.len(), 1);

    // Direct dependencies aggregated across manifests
    let direct_deps = report.all_direct_dependencies();
    assert_eq!(direct_deps.len(), 2);
    let names: Vec<&str> = direct_deps.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"tokio"));
    assert!(names.contains(&"svelte"));

    // Filter by ecosystem
    let cargo_files = report.filter_by_ecosystem(PackageEcosystem::Cargo);
    assert_eq!(cargo_files.len(), 2); // Cargo.toml + Cargo.lock

    let npm_files = report.filter_by_ecosystem(PackageEcosystem::Npm);
    assert_eq!(npm_files.len(), 1); // package.json

    // CDM sections
    let sections = report.to_cdm_sections("https://github.com/monorepo/service/blob/main");
    assert_eq!(sections.len(), 3);
    for sec in &sections {
        assert_eq!(sec.kind, SectionKind::Manifest);
    }

    // Completeness Report
    // Total discovered = 5 files, 0 excluded, 3 extracted, 1 unsupported, 1 failed
    let completeness = report.to_completeness_report(5, 0);
    assert!(completeness.formula_valid());
    assert_eq!(completeness.discovered_count, 5);
    assert_eq!(completeness.extracted_count, 3);
    assert_eq!(completeness.failed_count, 1);
    assert_eq!(completeness.unsupported_count, 1);
    assert_eq!(completeness.excluded_count, 0);
    assert_eq!(completeness.eligible_count, 4); // 3 extracted + 1 failed
}
