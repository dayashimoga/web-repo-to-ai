//! Integration tests for repository directory tree generation, formatting,
//! and CDM manifest section construction in `crates/github-extractor`.

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Json;
use axum::Router;
use serde_json::json;
use web_repo_to_ai_cdm::SectionKind;
use web_repo_to_ai_github_extractor::{
    ClassifiedFile, FileCategory, GitFileMode, GitHubApiClient, GitHubUrlParser, GitTreeEntry,
    GitTreeEntryType, GitTreeResponse, RepoTree, TreeClassificationReport, TreeFormatConfig,
    TreeStyle,
};

fn sample_monorepo_entries() -> Vec<GitTreeEntry> {
    vec![
        GitTreeEntry::new(
            "Cargo.toml",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000001",
            Some(350),
        ),
        GitTreeEntry::new(
            "Cargo.lock",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000002",
            Some(45000),
        ),
        GitTreeEntry::new(
            "README.md",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000003",
            Some(2100),
        ),
        GitTreeEntry::new(
            ".github/workflows/ci.yml",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000004",
            Some(1200),
        ),
        GitTreeEntry::new(
            "crates/core/Cargo.toml",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000005",
            Some(200),
        ),
        GitTreeEntry::new(
            "crates/core/src/lib.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000006",
            Some(5600),
        ),
        GitTreeEntry::new(
            "crates/core/src/parser.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000007",
            Some(8900),
        ),
        GitTreeEntry::new(
            "crates/core/tests/parser_test.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000008",
            Some(3400),
        ),
        GitTreeEntry::new(
            "apps/web/package.json",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000009",
            Some(800),
        ),
        GitTreeEntry::new(
            "apps/web/src/routes/+page.svelte",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000010",
            Some(2500),
        ),
    ]
}

#[test]
fn test_repo_tree_generation_from_git_tree_entries() {
    let entries = sample_monorepo_entries();
    let response = GitTreeResponse::new("root_tree_sha", entries.clone(), false);

    let tree = response.to_repo_tree();
    assert_eq!(tree.total_files, 10);
    // Directories: .github, .github/workflows, crates, crates/core, crates/core/src, crates/core/tests, apps, apps/web, apps/web/src, apps/web/src/routes
    assert_eq!(tree.total_directories, 10);
    assert_eq!(tree.max_depth, 4); // apps/web/src/routes/+page.svelte (depth 4)

    let total_bytes: u64 = entries.iter().filter_map(|e| e.size).sum();
    assert_eq!(tree.total_bytes, total_bytes);

    let rendered = tree.render_default();
    assert!(rendered.starts_with(".\n"));
    assert!(rendered.contains("crates/"));
    assert!(rendered.contains("core/"));
    assert!(rendered.contains("src/"));
    assert!(rendered.contains("lib.rs"));
    assert!(rendered.contains("apps/"));
    assert!(rendered.contains("10 directories, 10 files"));
}

#[test]
fn test_repo_tree_generation_with_classification_annotations() {
    let files = vec![
        ClassifiedFile {
            path: "README.md".to_string(),
            category: FileCategory::Documentation,
            mode: GitFileMode::RegularFile,
            sha: "1111111111111111111111111111111111111111".to_string(),
            size: Some(1500),
            language: Some("markdown".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "Cargo.toml".to_string(),
            category: FileCategory::DependencyManifest,
            mode: GitFileMode::RegularFile,
            sha: "2222222222222222222222222222222222222222".to_string(),
            size: Some(400),
            language: Some("toml".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: ".github/workflows/ci.yml".to_string(),
            category: FileCategory::Configuration,
            mode: GitFileMode::RegularFile,
            sha: "3333333333333333333333333333333333333333".to_string(),
            size: Some(1200),
            language: Some("yaml".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "src/main.rs".to_string(),
            category: FileCategory::SourceCode,
            mode: GitFileMode::RegularFile,
            sha: "4444444444444444444444444444444444444444".to_string(),
            size: Some(3000),
            language: Some("rust".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "tests/integration_test.rs".to_string(),
            category: FileCategory::TestFixture,
            mode: GitFileMode::RegularFile,
            sha: "5555555555555555555555555555555555555555".to_string(),
            size: Some(2000),
            language: Some("rust".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
    ];

    let tree = RepoTree::from_classified_files(&files);
    assert_eq!(tree.category_counts.get("documentation"), Some(&1));
    assert_eq!(tree.category_counts.get("dependency_manifest"), Some(&1));
    assert_eq!(tree.category_counts.get("configuration"), Some(&1));
    assert_eq!(tree.category_counts.get("source_code"), Some(&1));
    assert_eq!(tree.category_counts.get("test_fixture"), Some(&1));

    let rendered = tree.render_default();
    assert!(rendered.contains("[doc]"));
    assert!(rendered.contains("[manifest]"));
    assert!(rendered.contains("[config]"));
    assert!(rendered.contains("[source]"));
    assert!(rendered.contains("[test]"));

    // Also verify TreeClassificationReport::to_repo_tree
    let mut category_sizes = std::collections::BTreeMap::new();
    category_sizes.insert("source_code".to_string(), 3000);
    let report = TreeClassificationReport {
        files,
        total_files: 5,
        category_counts: tree.category_counts.clone(),
        category_sizes,
        eligible_count: 5,
        excluded_count: 0,
        binary_count: 0,
        oversized_count: 0,
    };
    let tree_from_report = report.to_repo_tree();
    assert_eq!(tree_from_report.total_files, 5);
}

#[test]
fn test_repo_tree_ascii_style_and_options() {
    let entries = sample_monorepo_entries();
    let tree = RepoTree::from_entries(&entries);

    let config = TreeFormatConfig::new()
        .with_style(TreeStyle::Ascii)
        .with_include_sizes(false)
        .with_include_categories(false);

    let rendered = tree.render(&config);
    assert!(rendered.contains("|-- ") || rendered.contains("\\-- "));
    assert!(!rendered.contains("├── "));
    assert!(!rendered.contains("[source]"));
    assert!(!rendered.contains("KB)"));
}

#[test]
fn test_repo_tree_depth_limiting() {
    let entries = sample_monorepo_entries();
    let tree = RepoTree::from_entries(&entries);

    // Limit to depth 1 (root and direct subdirectories)
    let config = TreeFormatConfig::new().with_max_depth(1);
    let rendered = tree.render(&config);

    // At depth 0: Cargo.toml, README.md, .github/, crates/, apps/
    // At depth 1: .github/workflows/, crates/core/, apps/web/
    assert!(rendered.contains("crates/"));
    assert!(rendered.contains("core/"));
    // Depth 2: crates/core/src/ should be omitted by depth limit
    assert!(!rendered.contains("parser.rs"));
}

#[test]
fn test_repo_tree_max_entries_ceiling() {
    let entries = sample_monorepo_entries(); // 10 files + directories
    let tree = RepoTree::from_entries(&entries);

    // Cap output at 4 entries
    let config = TreeFormatConfig::new().with_max_entries(4);
    let rendered = tree.render(&config);

    assert!(rendered.contains("... [Additional entries omitted by max_entries ceiling]"));
}

#[test]
fn test_repo_tree_to_cdm_section_mapping() {
    let entries = sample_monorepo_entries();
    let tree = RepoTree::from_entries(&entries);

    let config = TreeFormatConfig::default();
    let section = tree.to_cdm_section(
        "sec-repo-tree",
        "https://github.com/my-org/my-project/tree/main",
        &config,
    );

    assert_eq!(section.section_id, "sec-repo-tree");
    assert_eq!(section.kind, SectionKind::Manifest);
    assert_eq!(section.title.as_deref(), Some("Repository Directory Tree"));
    assert_eq!(section.original_path.as_deref(), Some("TREE.md"));
    assert_eq!(section.language.as_deref(), Some("text"));

    assert_eq!(
        section.attributes.get("manifest_role"),
        Some(&"repository_tree".to_string())
    );
    assert_eq!(
        section.attributes.get("total_files"),
        Some(&"10".to_string())
    );
    assert_eq!(
        section.attributes.get("total_directories"),
        Some(&"10".to_string())
    );
    assert!(section.attributes.contains_key("total_bytes"));
    assert_eq!(section.attributes.get("max_depth"), Some(&"4".to_string()));

    // Verify token metrics and line spans
    let metrics = section
        .token_metrics
        .as_ref()
        .expect("Token metrics must be present");
    assert!(metrics.line_count > 10);
    assert!(metrics.word_count > 10);
    assert!(!section.content_hash.is_empty());
}

#[tokio::test]
async fn test_api_client_generate_repo_tree_integration() {
    let tree_payload = json!({
        "sha": "tree_sha_xyz",
        "truncated": false,
        "tree": [
            {
                "path": "Cargo.toml",
                "mode": "100644",
                "type": "blob",
                "sha": "1111111111111111111111111111111111111111",
                "size": 500
            },
            {
                "path": "src",
                "mode": "040000",
                "type": "tree",
                "sha": "2222222222222222222222222222222222222222"
            },
            {
                "path": "src/lib.rs",
                "mode": "100644",
                "type": "blob",
                "sha": "3333333333333333333333333333333333333333",
                "size": 2048
            }
        ]
    });

    let app = Router::new().route(
        "/repos/:owner/:repo/git/trees/:tree_sha",
        get(move |_path: Path<(String, String, String)>| {
            let p = tree_payload.clone();
            async move { (StatusCode::OK, Json(p)).into_response() }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let target = GitHubUrlParser::parse_url("https://github.com/web-repo/live-tree").unwrap();
    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));

    let (tree_response, _rate) = client
        .get_tree(&target, "tree_sha_xyz", true)
        .await
        .unwrap();
    let repo_tree = client.generate_repo_tree(&tree_response);

    assert_eq!(repo_tree.total_files, 2);
    assert_eq!(repo_tree.total_directories, 1);
    assert_eq!(repo_tree.total_bytes, 500 + 2048);

    let rendered = repo_tree.render_default();
    assert!(rendered.contains("Cargo.toml"));
    assert!(rendered.contains("src/"));
    assert!(rendered.contains("lib.rs"));
    assert!(rendered.contains("1 directories, 2 files (2.5 KB total)"));
}
