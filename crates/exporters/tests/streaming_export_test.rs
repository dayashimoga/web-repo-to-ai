//! Streaming exporters integration test suite.

use std::io::Cursor;
use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::model::{CdmDocument, RevisionInfo, SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_exporters::error::ExporterError;
use web_repo_to_ai_exporters::streaming::{
    StreamingExportConfig, StreamingJsonlWriter, StreamingTextWriter, StreamingZipPackageWriter,
};
use zip::ZipArchive;

fn sample_cdm_document() -> CdmDocument {
    let identity = SourceIdentity {
        raw_input: "https://github.com/example/streaming-repo".to_string(),
        canonical_url: "https://github.com/example/streaming-repo".to_string(),
        domain: Some("github.com".to_string()),
        repo_owner: Some("example".to_string()),
        repo_name: Some("streaming-repo".to_string()),
        git_ref: Some("main".to_string()),
        subpath: None,
    };

    let s1 = CdmSectionBuilder::new(
        "sec-1",
        "https://github.com/example/streaming-repo/blob/main/src/main.rs",
        SectionKind::CodeFile,
        "fn main() { println!(\"Hello Streaming\"); }\n",
    )
    .title("src/main.rs")
    .original_path("src/main.rs")
    .language("rust")
    .build();

    let s2 = CdmSectionBuilder::new(
        "sec-2",
        "https://github.com/example/streaming-repo/blob/main/docs/readme.md",
        SectionKind::Prose,
        "# Readme\nStreaming large export test.\n",
    )
    .title("docs/readme.md")
    .original_path("docs/readme.md")
    .language("markdown")
    .build();

    CdmDocumentBuilder::new(
        identity,
        SourceType::GitHubPublic,
        "https://github.com/example/streaming-repo",
    )
    .revision(RevisionInfo {
        git_branch: Some("main".to_string()),
        ..Default::default()
    })
    .add_section(s1)
    .add_section(s2)
    .build()
}

#[test]
fn test_streaming_config_validation() {
    let valid = StreamingExportConfig::new(8192);
    assert!(valid.is_ok());

    let invalid_buffer = StreamingExportConfig::new(512);
    assert!(invalid_buffer.is_err());

    let invalid_max_bytes = StreamingExportConfig::new(4096)
        .unwrap()
        .with_max_package_bytes(0);
    assert!(invalid_max_bytes.validate().is_err());
}

#[test]
fn test_streaming_jsonl_writer() {
    let mut buffer = Cursor::new(Vec::new());
    let config = StreamingExportConfig::new(4096).unwrap();
    let mut writer = StreamingJsonlWriter::new(&mut buffer, config).unwrap();

    #[derive(serde::Serialize)]
    struct Record {
        id: usize,
        title: String,
        active: bool,
    }

    for i in 1..=5 {
        writer
            .write_record(&Record {
                id: i,
                title: format!("Document {i}"),
                active: i % 2 == 0,
            })
            .unwrap();
    }

    let summary = writer.finish().unwrap();
    assert_eq!(summary.total_items_written, 5);
    assert!(summary.total_uncompressed_bytes > 0);
    assert!(!summary.sha256_checksum.is_empty());
    assert!(!summary.blake3_checksum.is_empty());

    let output_str = String::from_utf8(buffer.into_inner()).unwrap();
    let lines: Vec<&str> = output_str.trim().split('\n').collect();
    assert_eq!(lines.len(), 5);

    // Verify valid JSON on each line
    for line in lines {
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        assert!(v.get("id").is_some());
    }
}

#[test]
fn test_streaming_text_writer() {
    let mut buffer = Cursor::new(Vec::new());
    let config = StreamingExportConfig::new(4096).unwrap();
    let mut writer = StreamingTextWriter::new(&mut buffer, config).unwrap();

    writer
        .append_chunk(b"# Consolidated Documentation\n")
        .unwrap();
    writer
        .append_section("Section 1", "Body content 1")
        .unwrap();
    writer
        .append_section("Section 2", "Body content 2")
        .unwrap();

    let summary = writer.finish().unwrap();
    assert_eq!(summary.total_items_written, 3);
    assert!(summary.total_uncompressed_bytes > 0);

    let output_str = String::from_utf8(buffer.into_inner()).unwrap();
    assert!(output_str.contains("# Consolidated Documentation"));
    assert!(output_str.contains("## Section 1"));
    assert!(output_str.contains("## Section 2"));
}

#[test]
fn test_streaming_zip_package_writer() {
    let mut buffer = Cursor::new(Vec::new());
    let config = StreamingExportConfig::new(4096)
        .unwrap()
        .with_manifests(true, true);

    let mut writer = StreamingZipPackageWriter::new(&mut buffer, config).unwrap();

    // 1. Append raw files
    writer
        .append_file_bytes("metadata/notes.txt", b"Notes on streaming package")
        .unwrap();

    // 2. Append CDM document
    let doc = sample_cdm_document();
    let entries = writer.append_cdm_document("doc1", &doc).unwrap();
    assert!(entries.len() >= 4);

    let summary = writer.finish().unwrap();
    assert!(summary.total_items_written >= 5);
    assert!(summary.total_uncompressed_bytes > 0);
    assert!(!summary.sha256_checksum.is_empty());

    // 3. Inspect generated ZIP archive
    let mut reader = Cursor::new(buffer.into_inner());
    let mut zip = ZipArchive::new(&mut reader).unwrap();

    assert!(zip.by_name("metadata/notes.txt").is_ok());
    assert!(zip.by_name("doc1/context.md").is_ok());
    assert!(zip.by_name("doc1/document.json").is_ok());
    assert!(zip.by_name("doc1/sources/src/main.rs").is_ok());
    assert!(zip.by_name("checksums.sha256").is_ok());
    assert!(zip.by_name("manifest.json").is_ok());
}

#[test]
fn test_streaming_size_ceiling_enforcement() {
    let mut buffer = Cursor::new(Vec::new());
    let config = StreamingExportConfig::new(4096)
        .unwrap()
        .with_max_package_bytes(100);

    let mut writer = StreamingTextWriter::new(&mut buffer, config).unwrap();

    // First write fits within 100 bytes
    writer.append_chunk(b"Short prefix").unwrap();

    // Large write breaches 100 bytes limit
    let big_payload = vec![b'A'; 200];
    let result = writer.append_chunk(&big_payload);

    match result {
        Err(ExporterError::PackageSizeExceeded {
            max_bytes,
            actual_bytes,
        }) => {
            assert_eq!(max_bytes, 100);
            assert!(actual_bytes > 100);
        }
        _ => panic!("Expected PackageSizeExceeded, got {:?}", result),
    }
}
