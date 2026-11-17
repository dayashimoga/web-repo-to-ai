//! Integration tests for ZIP package generator (`crates/exporters/tests/zip_export.rs`).

use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use sha2::{Digest, Sha256};
use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::completeness::CompletenessReport;
use web_repo_to_ai_cdm::hierarchy::HierarchyNode;
use web_repo_to_ai_cdm::metadata::{Citation, DocumentMetadata, ExclusionReason, ExclusionRecord};
use web_repo_to_ai_cdm::model::{CdmDocument, RevisionInfo, SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_exporters::{
    export_zip_package, export_zip_package_to_writer, PackageManifest, PackageProvenance,
    PackageQualityReport, ZipPackageExporter, ZipPackageOptions,
};
use zip::ZipArchive;

fn create_sample_document() -> CdmDocument {
    let identity = SourceIdentity {
        raw_input: "https://github.com/example/sample-repo".to_string(),
        canonical_url: "https://github.com/example/sample-repo".to_string(),
        domain: Some("github.com".to_string()),
        repo_owner: Some("example".to_string()),
        repo_name: Some("sample-repo".to_string()),
        git_ref: Some("main".to_string()),
        subpath: None,
    };

    let s1 = CdmSectionBuilder::new(
        "sec-1",
        "https://github.com/example/sample-repo/blob/main/src/lib.rs",
        SectionKind::CodeFile,
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    )
    .title("src/lib.rs")
    .original_path("src/lib.rs")
    .language("rust")
    .line_range(1, 3)
    .byte_range(0, 42)
    .build();

    let s2 = CdmSectionBuilder::new(
        "sec-2",
        "https://github.com/example/sample-repo/blob/main/README.md",
        SectionKind::Prose,
        "# Sample Repo\n\nA simple calculator library in Rust.\n",
    )
    .title("README.md")
    .original_path("README.md")
    .language("markdown")
    .line_range(1, 3)
    .byte_range(0, 52)
    .build();

    let mut root = HierarchyNode::root("sample-repo");
    let mut src_dir = HierarchyNode::directory("src-dir", "src", "src");
    let lib_file = HierarchyNode::file(
        "lib-file",
        "lib.rs",
        "src/lib.rs",
        Some("sec-1".to_string()),
        Some(42),
        Some(3),
    );
    src_dir.add_child(lib_file);
    root.add_child(src_dir);

    let citation = Citation::new(
        "cite-1",
        "https://example.com/docs",
        Some("Official Documentation".to_string()),
        None,
    );

    let exclusion = ExclusionRecord::new(
        "target/debug/build.log",
        ExclusionReason::BinaryFile,
        Some("target/**".to_string()),
        Some("Build artifact".to_string()),
        Some(1024),
    );

    let completeness = CompletenessReport::new(3, 2, 2, 1, 0, 0);

    CdmDocumentBuilder::new(
        identity,
        SourceType::GitHubPublic,
        "https://github.com/example/sample-repo",
    )
    .document_id(DocumentId::new())
    .revision(RevisionInfo {
        git_commit: Some("abcdef1234567890abcdef1234567890abcdef12".to_string()),
        git_branch: Some("main".to_string()),
        ..Default::default()
    })
    .metadata(DocumentMetadata {
        title: Some("Sample Repository".to_string()),
        description: Some("Test repository for ZIP exporter validation".to_string()),
        primary_language: Some("Rust".to_string()),
        license: Some("MIT".to_string()),
        authors: vec!["Alice Dev".to_string()],
        language_breakdown: BTreeMap::from([("Rust".to_string(), 42)]),
        ..Default::default()
    })
    .hierarchy(root)
    .add_section(s1)
    .add_section(s2)
    .add_citation(citation)
    .add_exclusion(exclusion)
    .completeness(completeness)
    .build()
}

fn compute_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[test]
fn test_zip_package_export_complete_default() {
    let doc = create_sample_document();
    let options = ZipPackageOptions::default();
    let (zip_bytes, metrics) = export_zip_package(&doc, &options).unwrap();

    assert!(!zip_bytes.is_empty());
    assert!(metrics.total_bytes_written > 0);
    assert_eq!(metrics.total_bytes_written, zip_bytes.len());
    assert!(!metrics.sha256_checksum.is_empty());

    // Inspect archive with ZipArchive
    let mut archive = ZipArchive::new(Cursor::new(&zip_bytes)).unwrap();
    let file_names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().unwrap().to_string())
        .collect();

    // Verify presence of all expected files
    assert!(file_names.contains(&"context.md".to_string()));
    assert!(file_names.contains(&"context.txt".to_string()));
    assert!(file_names.contains(&"document.json".to_string()));
    assert!(file_names.contains(&"context.json".to_string()));
    assert!(file_names.contains(&"context.html".to_string()));
    assert!(file_names.contains(&"context.jsonl".to_string()));
    assert!(file_names.contains(&"sources/src/lib.rs".to_string()));
    assert!(file_names.contains(&"sources/README.md".to_string()));
    assert!(file_names.contains(&"manifest.json".to_string()));
    assert!(file_names.contains(&"provenance.json".to_string()));
    assert!(file_names.contains(&"quality-report.json".to_string()));
    assert!(file_names.contains(&"checksums.sha256".to_string()));

    // Verify SHA-256 checksums file integrity
    let mut checksums_file = archive.by_name("checksums.sha256").unwrap();
    let mut checksums_content = String::new();
    checksums_file
        .read_to_string(&mut checksums_content)
        .unwrap();
    drop(checksums_file);

    for line in checksums_content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split("  ").collect();
        assert_eq!(
            parts.len(),
            2,
            "Line must be in '<hash>  <filename>' format: {line}"
        );
        let declared_sha = parts[0];
        let file_path = parts[1];

        let mut target_file = archive.by_name(file_path).unwrap();
        let mut target_bytes = Vec::new();
        target_file.read_to_end(&mut target_bytes).unwrap();
        let computed_sha = compute_sha256(&target_bytes);

        assert_eq!(
            declared_sha, computed_sha,
            "Checksum mismatch for file {file_path}"
        );
    }

    // Verify manifest.json deserializes cleanly
    let mut manifest_str = String::new();
    {
        let mut manifest_file = archive.by_name("manifest.json").unwrap();
        manifest_file.read_to_string(&mut manifest_str).unwrap();
    }
    let manifest: PackageManifest = serde_json::from_str(&manifest_str).unwrap();
    assert_eq!(manifest.manifest_version, "1.0.0");
    assert_eq!(
        manifest.source_url,
        "https://github.com/example/sample-repo"
    );
    assert!(manifest.file_count > 0);

    // Verify quality-report.json deserializes cleanly
    let mut qr_str = String::new();
    {
        let mut qr_file = archive.by_name("quality-report.json").unwrap();
        qr_file.read_to_string(&mut qr_str).unwrap();
    }
    let qr: PackageQualityReport = serde_json::from_str(&qr_str).unwrap();
    assert_eq!(qr.discovered_count, 3);
    assert_eq!(qr.extracted_count, 2);
    assert_eq!(qr.exclusions_count, 1);

    // Verify provenance.json deserializes cleanly
    let mut prov_str = String::new();
    {
        let mut prov_file = archive.by_name("provenance.json").unwrap();
        prov_file.read_to_string(&mut prov_str).unwrap();
    }
    let prov: PackageProvenance = serde_json::from_str(&prov_str).unwrap();
    assert_eq!(prov.citations_count, 1);
    assert_eq!(prov.canonical_url, "https://github.com/example/sample-repo");
}

#[test]
fn test_zip_package_minimal_preset() {
    let doc = create_sample_document();
    let options = ZipPackageOptions::minimal();
    let (zip_bytes, metrics) = export_zip_package(&doc, &options).unwrap();

    let mut archive = ZipArchive::new(Cursor::new(&zip_bytes)).unwrap();
    let file_names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().unwrap().to_string())
        .collect();

    assert_eq!(file_names.len(), 3);
    assert!(file_names.contains(&"context.md".to_string()));
    assert!(file_names.contains(&"manifest.json".to_string()));
    assert!(file_names.contains(&"checksums.sha256".to_string()));
    assert!(!file_names.contains(&"context.txt".to_string()));
    assert!(!file_names.contains(&"context.html".to_string()));
    assert_eq!(metrics.files_archived, 3);
}

#[test]
fn test_zip_package_formats_only_preset() {
    let doc = create_sample_document();
    let options = ZipPackageOptions::formats_only();
    let (zip_bytes, _) = export_zip_package(&doc, &options).unwrap();

    let mut archive = ZipArchive::new(Cursor::new(&zip_bytes)).unwrap();
    let file_names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().unwrap().to_string())
        .collect();

    assert!(file_names.contains(&"context.md".to_string()));
    assert!(file_names.contains(&"context.txt".to_string()));
    assert!(file_names.contains(&"context.html".to_string()));
    // Assert no sources directory entries exist
    assert!(!file_names.iter().any(|f| f.starts_with("sources/")));
}

#[test]
fn test_zip_package_sources_only_preset() {
    let doc = create_sample_document();
    let options = ZipPackageOptions::sources_only();
    let (zip_bytes, _) = export_zip_package(&doc, &options).unwrap();

    let mut archive = ZipArchive::new(Cursor::new(&zip_bytes)).unwrap();
    let file_names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().unwrap().to_string())
        .collect();

    assert!(file_names.contains(&"sources/src/lib.rs".to_string()));
    assert!(file_names.contains(&"sources/README.md".to_string()));
    assert!(!file_names.contains(&"context.md".to_string()));
    assert!(!file_names.contains(&"context.txt".to_string()));
    assert!(!file_names.contains(&"context.html".to_string()));
}

#[test]
fn test_zip_streaming_writer_equivalence() {
    let doc = create_sample_document();
    let options = ZipPackageOptions::minimal();

    let (in_memory_bytes, m1) = export_zip_package(&doc, &options).unwrap();

    let mut cursor = Cursor::new(Vec::new());
    let m2 = export_zip_package_to_writer(&doc, &options, &mut cursor).unwrap();
    let stream_bytes = cursor.into_inner();

    assert_eq!(m1.files_archived, m2.files_archived);
    assert_eq!(m1.uncompressed_bytes, m2.uncompressed_bytes);
    assert_eq!(in_memory_bytes.len(), stream_bytes.len());

    // Both archives must open cleanly and have identical file entries
    let mut a1 = ZipArchive::new(Cursor::new(&in_memory_bytes)).unwrap();
    let mut a2 = ZipArchive::new(Cursor::new(&stream_bytes)).unwrap();
    assert_eq!(a1.len(), a2.len());

    for i in 0..a1.len() {
        assert_eq!(
            a1.by_index(i).unwrap().name().unwrap(),
            a2.by_index(i).unwrap().name().unwrap()
        );
    }
}

#[test]
fn test_zip_empty_document_handling() {
    let identity = SourceIdentity::new("https://example.com/empty", "https://example.com/empty");
    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::WebpageStatic,
        "https://example.com/empty",
    )
    .build();

    let options = ZipPackageOptions::minimal();
    let (zip_bytes, metrics) = export_zip_package(&doc, &options).unwrap();

    let archive = ZipArchive::new(Cursor::new(&zip_bytes)).unwrap();
    assert_eq!(archive.len(), 3);
    assert_eq!(metrics.files_archived, 3);
}

#[test]
fn test_zip_options_serde_roundtrip() {
    let options = ZipPackageOptions::default()
        .with_html(false)
        .with_individual_files(false)
        .with_max_token_budget(Some(2048));

    let json = serde_json::to_string(&options).unwrap();
    let deserialized: ZipPackageOptions = serde_json::from_str(&json).unwrap();

    assert_eq!(options, deserialized);
}

#[test]
fn test_zip_path_sanitization_defense() {
    let identity = SourceIdentity::new("https://example.com/test", "https://example.com/test");
    let sec1 = CdmSectionBuilder::new(
        "traversal-1",
        "https://example.com/test#1",
        SectionKind::CodeFile,
        "malicious content 1",
    )
    .original_path("../../etc/passwd")
    .build();

    let sec2 = CdmSectionBuilder::new(
        "traversal-2",
        "https://example.com/test#2",
        SectionKind::CodeFile,
        "malicious content 2",
    )
    .original_path("/absolute/root/secret.key")
    .build();

    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::WebpageStatic,
        "https://example.com/test",
    )
    .add_section(sec1)
    .add_section(sec2)
    .build();

    let options = ZipPackageOptions::sources_only();
    let (zip_bytes, _) = export_zip_package(&doc, &options).unwrap();

    let mut archive = ZipArchive::new(Cursor::new(&zip_bytes)).unwrap();
    let file_names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().unwrap().to_string())
        .collect();

    // Verify paths never contain `../` or leading `/`
    for name in &file_names {
        assert!(
            !name.contains(".."),
            "Archive path must not contain '..': {name}"
        );
        assert!(
            !name.starts_with('/'),
            "Archive path must not start with '/': {name}"
        );
        assert!(
            !name.contains('\\'),
            "Archive path must use forward slashes: {name}"
        );
    }
    assert!(file_names.contains(&"sources/etc/passwd".to_string()));
    assert!(file_names.contains(&"sources/absolute/root/secret.key".to_string()));
}

#[test]
fn test_zip_exporter_builder_api() {
    let doc = create_sample_document();
    let exporter = ZipPackageExporter::new();
    let (bytes1, m1) = exporter.export_to_bytes(&doc).unwrap();

    let mut cursor = Cursor::new(Vec::new());
    let exporter_writer = ZipPackageExporter::with_options(ZipPackageOptions::default());
    let m2 = exporter_writer.export_to_writer(&doc, &mut cursor).unwrap();

    assert_eq!(m1.files_archived, m2.files_archived);
    assert_eq!(bytes1.len(), cursor.into_inner().len());
}
