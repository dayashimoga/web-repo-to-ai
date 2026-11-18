//! Integration tests for automated repository architecture synthesis, workspace topology detection,
//! component cataloging, entry point discovery, and CDM architecture overview section construction.

use axum::routing::get;
use axum::Json;
use axum::Router;
use serde_json::json;
use std::time::Duration;
use web_repo_to_ai_cdm::SectionKind;
use web_repo_to_ai_github_extractor::{
    ClassifiedFile, ComponentRole, ConfigCategory, ConfigTestExtractionReport, DeclaredDependency,
    DependencyExtractionReport, DependencyType, DocExtractionReport, DocFormat, DocType,
    EntryPointRole, ExtractedConfigFile, ExtractedDependencyFile, ExtractedDocFile, FileCategory,
    GitFileMode, GitHubApiClient, GitTreeEntry, GitTreeEntryType, GitTreeResponse, ManifestRole,
    PackageEcosystem, ParsedManifest, RepoArchitectureContext, WorkspaceType,
};

fn sample_monorepo_files() -> Vec<ClassifiedFile> {
    vec![
        ClassifiedFile {
            path: "Cargo.toml".to_string(),
            category: FileCategory::DependencyManifest,
            mode: GitFileMode::RegularFile,
            sha: "1000000000000000000000000000000000000001".to_string(),
            size: Some(500),
            language: Some("toml".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "crates/core/Cargo.toml".to_string(),
            category: FileCategory::DependencyManifest,
            mode: GitFileMode::RegularFile,
            sha: "1000000000000000000000000000000000000002".to_string(),
            size: Some(300),
            language: Some("toml".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "crates/core/src/lib.rs".to_string(),
            category: FileCategory::SourceCode,
            mode: GitFileMode::RegularFile,
            sha: "1000000000000000000000000000000000000003".to_string(),
            size: Some(4200),
            language: Some("rust".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "services/api/Cargo.toml".to_string(),
            category: FileCategory::DependencyManifest,
            mode: GitFileMode::RegularFile,
            sha: "1000000000000000000000000000000000000004".to_string(),
            size: Some(400),
            language: Some("toml".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "services/api/src/main.rs".to_string(),
            category: FileCategory::SourceCode,
            mode: GitFileMode::RegularFile,
            sha: "1000000000000000000000000000000000000005".to_string(),
            size: Some(2500),
            language: Some("rust".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "services/worker/src/main.rs".to_string(),
            category: FileCategory::SourceCode,
            mode: GitFileMode::RegularFile,
            sha: "1000000000000000000000000000000000000006".to_string(),
            size: Some(1800),
            language: Some("rust".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "apps/web/package.json".to_string(),
            category: FileCategory::DependencyManifest,
            mode: GitFileMode::RegularFile,
            sha: "1000000000000000000000000000000000000007".to_string(),
            size: Some(800),
            language: Some("json".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "apps/web/src/routes/+page.svelte".to_string(),
            category: FileCategory::SourceCode,
            mode: GitFileMode::RegularFile,
            sha: "1000000000000000000000000000000000000008".to_string(),
            size: Some(3100),
            language: Some("svelte".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "deploy/compose.yml".to_string(),
            category: FileCategory::Configuration,
            mode: GitFileMode::RegularFile,
            sha: "1000000000000000000000000000000000000009".to_string(),
            size: Some(1200),
            language: Some("yaml".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "docs/ARCHITECTURE.md".to_string(),
            category: FileCategory::Documentation,
            mode: GitFileMode::RegularFile,
            sha: "1000000000000000000000000000000000000010".to_string(),
            size: Some(5000),
            language: Some("markdown".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "docs/DATA_MODEL.md".to_string(),
            category: FileCategory::Documentation,
            mode: GitFileMode::RegularFile,
            sha: "1000000000000000000000000000000000000011".to_string(),
            size: Some(4000),
            language: Some("markdown".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "scripts/check-all.ps1".to_string(),
            category: FileCategory::Configuration,
            mode: GitFileMode::RegularFile,
            sha: "1000000000000000000000000000000000000012".to_string(),
            size: Some(1500),
            language: Some("powershell".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: ".github/workflows/ci.yml".to_string(),
            category: FileCategory::Configuration,
            mode: GitFileMode::RegularFile,
            sha: "1000000000000000000000000000000000000013".to_string(),
            size: Some(2000),
            language: Some("yaml".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
    ]
}

#[test]
fn test_workspace_topology_detection_monorepo() {
    let files = sample_monorepo_files();
    let entries: Vec<GitTreeEntry> = files
        .iter()
        .map(|f| {
            GitTreeEntry::new(
                &f.path,
                f.mode.clone(),
                GitTreeEntryType::Blob,
                &f.sha,
                f.size,
            )
        })
        .collect();
    let tree_response = GitTreeResponse::new("sha-root", entries, false);
    let classification = tree_response.classify(&Default::default());

    let arch = RepoArchitectureContext::from_repository_data(
        &classification,
        None,
        None,
        None,
        None,
        None,
    );

    assert_eq!(arch.workspace_type, WorkspaceType::Monorepo);
    assert!(arch.workspace_type.is_monorepo());
    assert_eq!(arch.workspace_type.as_str(), "monorepo");
}

#[test]
fn test_workspace_topology_detection_single_package() {
    let files = vec![
        GitTreeEntry::new(
            "Cargo.toml",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1111111111111111111111111111111111111111",
            Some(300),
        ),
        GitTreeEntry::new(
            "src/main.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "2222222222222222222222222222222222222222",
            Some(1200),
        ),
    ];
    let tree_response = GitTreeResponse::new("sha-root", files, false);
    let classification = tree_response.classify(&Default::default());

    let arch = RepoArchitectureContext::from_repository_data(
        &classification,
        None,
        None,
        None,
        None,
        None,
    );

    assert_eq!(arch.workspace_type, WorkspaceType::SinglePackage);
    assert!(!arch.workspace_type.is_monorepo());
}

#[test]
fn test_workspace_topology_detection_library() {
    let files = vec![
        GitTreeEntry::new(
            "Cargo.toml",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1111111111111111111111111111111111111111",
            Some(300),
        ),
        GitTreeEntry::new(
            "src/lib.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "2222222222222222222222222222222222222222",
            Some(3500),
        ),
    ];
    let tree_response = GitTreeResponse::new("sha-root", files, false);
    let classification = tree_response.classify(&Default::default());

    let arch = RepoArchitectureContext::from_repository_data(
        &classification,
        None,
        None,
        None,
        None,
        None,
    );

    assert_eq!(arch.workspace_type, WorkspaceType::Library);
}

#[test]
fn test_entry_points_discovery() {
    let files = sample_monorepo_files();
    let entries: Vec<GitTreeEntry> = files
        .iter()
        .map(|f| {
            GitTreeEntry::new(
                &f.path,
                f.mode.clone(),
                GitTreeEntryType::Blob,
                &f.sha,
                f.size,
            )
        })
        .collect();
    let tree_response = GitTreeResponse::new("sha-root", entries, false);
    let classification = tree_response.classify(&Default::default());

    let arch = RepoArchitectureContext::from_repository_data(
        &classification,
        None,
        None,
        None,
        None,
        None,
    );

    let ep_paths: Vec<&str> = arch.entry_points.iter().map(|e| e.path.as_str()).collect();

    assert!(ep_paths.contains(&"crates/core/src/lib.rs"));
    assert!(ep_paths.contains(&"services/api/src/main.rs"));
    assert!(ep_paths.contains(&"services/worker/src/main.rs"));
    assert!(ep_paths.contains(&"apps/web/src/routes/+page.svelte"));
    assert!(ep_paths.contains(&"scripts/check-all.ps1"));

    let api_ep = arch
        .entry_points
        .iter()
        .find(|e| e.path == "services/api/src/main.rs")
        .expect("api entry point");
    assert_eq!(api_ep.role, EntryPointRole::WebApi);

    let worker_ep = arch
        .entry_points
        .iter()
        .find(|e| e.path == "services/worker/src/main.rs")
        .expect("worker entry point");
    assert_eq!(worker_ep.role, EntryPointRole::WorkerDaemon);

    let ui_ep = arch
        .entry_points
        .iter()
        .find(|e| e.path == "apps/web/src/routes/+page.svelte")
        .expect("ui entry point");
    assert_eq!(ui_ep.role, EntryPointRole::UiRoute);

    let lib_ep = arch
        .entry_points
        .iter()
        .find(|e| e.path == "crates/core/src/lib.rs")
        .expect("lib entry point");
    assert_eq!(lib_ep.role, EntryPointRole::LibraryRoot);
}

#[test]
fn test_component_cataloging_and_metrics() {
    let files = sample_monorepo_files();
    let entries: Vec<GitTreeEntry> = files
        .iter()
        .map(|f| {
            GitTreeEntry::new(
                &f.path,
                f.mode.clone(),
                GitTreeEntryType::Blob,
                &f.sha,
                f.size,
            )
        })
        .collect();
    let tree_response = GitTreeResponse::new("sha-root", entries, false);
    let classification = tree_response.classify(&Default::default());

    let arch = RepoArchitectureContext::from_repository_data(
        &classification,
        None,
        None,
        None,
        None,
        None,
    );

    let comp_roots: Vec<&str> = arch
        .components
        .iter()
        .map(|c| c.root_path.as_str())
        .collect();

    assert!(comp_roots.contains(&"crates/core"));
    assert!(comp_roots.contains(&"services/api"));
    assert!(comp_roots.contains(&"services/worker"));
    assert!(comp_roots.contains(&"apps/web"));
    assert!(comp_roots.contains(&"deploy"));
    assert!(comp_roots.contains(&"docs"));
    assert!(comp_roots.contains(&"scripts"));

    let core_comp = arch
        .components
        .iter()
        .find(|c| c.root_path == "crates/core")
        .expect("core component");
    assert_eq!(core_comp.role, ComponentRole::LibraryCrate);
    assert_eq!(core_comp.file_count, 2); // Cargo.toml, src/lib.rs

    let api_comp = arch
        .components
        .iter()
        .find(|c| c.root_path == "services/api")
        .expect("api component");
    assert_eq!(api_comp.role, ComponentRole::Service);

    let web_comp = arch
        .components
        .iter()
        .find(|c| c.root_path == "apps/web")
        .expect("web component");
    assert_eq!(web_comp.role, ComponentRole::WebFrontend);
}

#[test]
fn test_technology_and_framework_inference() {
    let files = sample_monorepo_files();
    let entries: Vec<GitTreeEntry> = files
        .iter()
        .map(|f| {
            GitTreeEntry::new(
                &f.path,
                f.mode.clone(),
                GitTreeEntryType::Blob,
                &f.sha,
                f.size,
            )
        })
        .collect();
    let tree_response = GitTreeResponse::new("sha-root", entries, false);
    let classification = tree_response.classify(&Default::default());

    let mut dependencies = DependencyExtractionReport::default();
    let mut parsed_cargo = ParsedManifest::new();
    parsed_cargo.dependencies.push(DeclaredDependency::new(
        "tokio",
        Some("1.0".to_string()),
        DependencyType::Direct,
    ));
    parsed_cargo.dependencies.push(DeclaredDependency::new(
        "axum",
        Some("0.7".to_string()),
        DependencyType::Direct,
    ));
    parsed_cargo.dependencies.push(DeclaredDependency::new(
        "sqlx",
        Some("0.8".to_string()),
        DependencyType::Direct,
    ));
    dependencies.add_extracted(ExtractedDependencyFile {
        path: "services/api/Cargo.toml".to_string(),
        git_sha: "1000000000000000000000000000000000000004".to_string(),
        ecosystem: PackageEcosystem::Cargo,
        role: ManifestRole::Manifest,
        content: String::new(),
        parsed: Some(parsed_cargo),
        byte_size: 400,
        line_count: 15,
        blake3_hash: "h1".to_string(),
        sha256_hash: "h2".to_string(),
    });

    let mut configs = ConfigTestExtractionReport::new();
    configs.add_config(ExtractedConfigFile {
        path: ".prettierrc".to_string(),
        git_sha: "123".to_string(),
        config_category: ConfigCategory::LinterFormatter,
        format: "json".to_string(),
        content: "{}".to_string(),
        byte_size: 2,
        line_count: 1,
        blake3_hash: "h3".to_string(),
        sha256_hash: "h4".to_string(),
    });

    let arch = RepoArchitectureContext::from_repository_data(
        &classification,
        None,
        None,
        Some(&dependencies),
        Some(&configs),
        None,
    );

    assert!(arch.key_technologies.iter().any(|t| t.contains("Tokio")));
    assert!(arch.key_technologies.iter().any(|t| t.contains("Axum")));
    assert!(arch.key_technologies.iter().any(|t| t.contains("SQLx")));
    assert!(arch.key_technologies.iter().any(|t| t.contains("Prettier")));
    assert!(arch
        .key_technologies
        .iter()
        .any(|t| t.contains("GitHub Actions")));
}

#[test]
fn test_architecture_docs_discovery() {
    let files = sample_monorepo_files();
    let entries: Vec<GitTreeEntry> = files
        .iter()
        .map(|f| {
            GitTreeEntry::new(
                &f.path,
                f.mode.clone(),
                GitTreeEntryType::Blob,
                &f.sha,
                f.size,
            )
        })
        .collect();
    let tree_response = GitTreeResponse::new("sha-root", entries, false);
    let classification = tree_response.classify(&Default::default());

    let mut docs = DocExtractionReport::new();
    docs.add_extracted(ExtractedDocFile {
        path: "DECISIONS.md".to_string(),
        git_sha: "1000000000000000000000000000000000000099".to_string(),
        doc_type: DocType::Architecture,
        format: DocFormat::Markdown,
        title: "ADRs".to_string(),
        has_frontmatter: false,
        content: "# ADRs".to_string(),
        byte_size: 6,
        line_count: 1,
        blake3_hash: "b3".to_string(),
        sha256_hash: "s2".to_string(),
    });

    let arch = RepoArchitectureContext::from_repository_data(
        &classification,
        None,
        Some(&docs),
        None,
        None,
        None,
    );

    assert!(arch
        .architecture_docs
        .contains(&"docs/ARCHITECTURE.md".to_string()));
    assert!(arch
        .architecture_docs
        .contains(&"docs/DATA_MODEL.md".to_string()));
    assert!(arch.architecture_docs.contains(&"DECISIONS.md".to_string()));
}

#[test]
fn test_markdown_and_json_serialization_roundtrip() {
    let files = sample_monorepo_files();
    let entries: Vec<GitTreeEntry> = files
        .iter()
        .map(|f| {
            GitTreeEntry::new(
                &f.path,
                f.mode.clone(),
                GitTreeEntryType::Blob,
                &f.sha,
                f.size,
            )
        })
        .collect();
    let tree_response = GitTreeResponse::new("sha-root", entries, false);
    let classification = tree_response.classify(&Default::default());

    let arch = RepoArchitectureContext::from_repository_data(
        &classification,
        None,
        None,
        None,
        None,
        None,
    );

    let md = arch.to_markdown();
    assert!(md.contains("# Repository Architecture & System Overview"));
    assert!(md.contains("## Executive Summary"));
    assert!(md.contains("## System Profile"));
    assert!(md.contains("| Workspace Topology | `monorepo` |"));
    assert!(md.contains("## Subsystems & Component Catalog"));
    assert!(md.contains("## Application & Service Entry Points"));

    let json_str = arch.to_json().expect("pretty json serialized");
    assert!(json_str.contains("\"workspace_type\": \"monorepo\""));

    let compact = arch.to_json_compact().expect("compact json serialized");
    assert!(compact.contains("\"workspace_type\":\"monorepo\""));

    let deserialized: RepoArchitectureContext =
        serde_json::from_str(&json_str).expect("deserialized successfully");
    assert_eq!(deserialized.workspace_type, arch.workspace_type);
    assert_eq!(deserialized.components.len(), arch.components.len());
    assert_eq!(deserialized.entry_points.len(), arch.entry_points.len());
}

#[test]
fn test_to_cdm_section_mapping() {
    let files = sample_monorepo_files();
    let entries: Vec<GitTreeEntry> = files
        .iter()
        .map(|f| {
            GitTreeEntry::new(
                &f.path,
                f.mode.clone(),
                GitTreeEntryType::Blob,
                &f.sha,
                f.size,
            )
        })
        .collect();
    let tree_response = GitTreeResponse::new("sha-root", entries, false);
    let classification = tree_response.classify(&Default::default());

    let arch = RepoArchitectureContext::from_repository_data(
        &classification,
        None,
        None,
        None,
        None,
        None,
    );

    let section = arch.to_cdm_section("sec-arch-001", "https://github.com/owner/repo");

    assert_eq!(section.section_id, "sec-arch-001");
    assert_eq!(section.source_url, "https://github.com/owner/repo");
    assert_eq!(section.kind, SectionKind::Prose);
    assert_eq!(
        section.title.as_deref(),
        Some("Repository Architecture & System Overview")
    );
    assert_eq!(
        section.original_path.as_deref(),
        Some("ARCHITECTURE_OVERVIEW.md")
    );
    assert_eq!(section.language.as_deref(), Some("markdown"));
    assert_eq!(
        section.attributes.get("manifest_role").map(String::as_str),
        Some("architecture_context")
    );
    assert_eq!(
        section.attributes.get("workspace_type").map(String::as_str),
        Some("monorepo")
    );
    assert!(!section.body.is_empty());
}

#[tokio::test]
async fn test_client_generate_architecture_context() {
    let app = Router::new().route(
        "/rate_limit",
        get(|| async {
            Json(json!({
                "resources": {
                    "core": { "limit": 5000, "remaining": 4999, "reset": 1700000000, "used": 1 }
                }
            }))
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bound to ephemeral port");
    let addr = listener.local_addr().expect("local addr");

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server serves");
    });

    let client = GitHubApiClient::new()
        .with_base_url_override(format!("http://{addr}"))
        .with_timeout(Duration::from_millis(500));

    let files = sample_monorepo_files();
    let entries: Vec<GitTreeEntry> = files
        .iter()
        .map(|f| {
            GitTreeEntry::new(
                &f.path,
                f.mode.clone(),
                GitTreeEntryType::Blob,
                &f.sha,
                f.size,
            )
        })
        .collect();
    let tree_response = GitTreeResponse::new("sha-root", entries, false);
    let classification = tree_response.classify(&Default::default());

    let arch = client.generate_architecture_context(&classification, None, None, None, None, None);

    assert_eq!(arch.workspace_type, WorkspaceType::Monorepo);
    assert!(!arch.components.is_empty());
}
