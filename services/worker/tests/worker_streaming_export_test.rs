//! Worker streaming export integration test.

use std::io::Cursor;
use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::model::{RevisionInfo, SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_exporters::streaming::StreamingExportConfig;
use web_repo_to_ai_exporters::ZipArchive;
use web_repo_to_ai_worker::config::WorkerServiceConfig;
use web_repo_to_ai_worker::engine::WorkerEngine;

#[test]
fn test_worker_streaming_export_lifecycle() {
    let config = WorkerServiceConfig::for_worker("test-worker-stream-suite", 4, 2).unwrap();
    let engine = WorkerEngine::new(config).unwrap();

    let identity1 = SourceIdentity {
        raw_input: "https://github.com/example/repo1".to_string(),
        canonical_url: "https://github.com/example/repo1".to_string(),
        domain: Some("github.com".to_string()),
        repo_owner: Some("example".to_string()),
        repo_name: Some("repo1".to_string()),
        git_ref: Some("main".to_string()),
        subpath: None,
    };

    let s1 = CdmSectionBuilder::new(
        "sec-1",
        "https://github.com/example/repo1/blob/main/src/lib.rs",
        SectionKind::CodeFile,
        "pub fn core_computation() -> usize { 1024 }\n",
    )
    .title("src/lib.rs")
    .original_path("src/lib.rs")
    .language("rust")
    .build();

    let doc1 = CdmDocumentBuilder::new(
        identity1,
        SourceType::GitHubPublic,
        "https://github.com/example/repo1",
    )
    .revision(RevisionInfo {
        git_branch: Some("main".to_string()),
        ..Default::default()
    })
    .add_section(s1)
    .build();

    let mut zip_buffer = Cursor::new(Vec::new());
    let export_config = StreamingExportConfig::new(8192)
        .unwrap()
        .with_manifests(true, true);

    let summary = engine
        .execute_streaming_export(&mut zip_buffer, export_config, &[doc1])
        .expect("Streaming export must succeed");

    assert!(summary.total_items_written >= 4);
    assert!(summary.total_uncompressed_bytes > 0);
    assert!(!summary.sha256_checksum.is_empty());
    assert!(!summary.blake3_checksum.is_empty());

    let metrics = engine.metrics_snapshot();
    assert_eq!(metrics.streaming_exports_completed, 1);
    assert_eq!(
        metrics.streaming_bytes_exported,
        summary.total_uncompressed_bytes as usize
    );

    // Validate archive contents
    let mut reader = Cursor::new(zip_buffer.into_inner());
    let mut zip = ZipArchive::new(&mut reader).expect("Valid ZIP archive");
    assert!(zip.by_name("doc_0/context.md").is_ok());
    assert!(zip.by_name("doc_0/context.txt").is_ok());
    assert!(zip.by_name("doc_0/document.json").is_ok());
    assert!(zip.by_name("doc_0/sources/src/lib.rs").is_ok());
    assert!(zip.by_name("manifest.json").is_ok());
    assert!(zip.by_name("checksums.sha256").is_ok());
}
