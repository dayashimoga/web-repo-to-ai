//! # Manifest & Checksums Exporter Integration Tests (`crates/exporters/tests/manifest_checksums.rs`)

use std::collections::{BTreeMap, HashMap};

use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::completeness::CompletenessReport;
use web_repo_to_ai_cdm::hierarchy::HierarchyNode;
use web_repo_to_ai_cdm::metadata::{Citation, DocumentMetadata, ExclusionReason, ExclusionRecord};
use web_repo_to_ai_cdm::model::{CdmDocument, RevisionInfo, SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_exporters::{
    build_manifest, build_provenance, build_quality_report, detect_mime_type,
    export_checksums_to_writer, export_manifest, export_manifest_to_writer, export_provenance,
    export_provenance_to_writer, export_quality_report, export_quality_report_to_writer,
    generate_checksums_manifest, parse_checksums_manifest, verify_checksums, ChecksumAlgorithm,
    ChecksumEntry, ChecksumOptions, ExporterError, ManifestExporter, ManifestOptions,
    PackageFileEntry, PackageManifest, PackageProvenance, PackageQualityReport,
};

fn create_sample_cdm_document() -> CdmDocument {
    let identity = SourceIdentity {
        raw_input: "https://github.com/example/manifest-demo".to_string(),
        canonical_url: "https://github.com/example/manifest-demo".to_string(),
        domain: Some("github.com".to_string()),
        repo_owner: Some("example".to_string()),
        repo_name: Some("manifest-demo".to_string()),
        git_ref: Some("main".to_string()),
        subpath: None,
    };

    let s1 = CdmSectionBuilder::new(
        "sec-1",
        "https://github.com/example/manifest-demo/blob/main/src/lib.rs",
        SectionKind::CodeFile,
        "pub fn hello() -> &'static str {\n    \"Hello, World!\"\n}\n",
    )
    .title("src/lib.rs")
    .original_path("src/lib.rs")
    .language("rust")
    .line_range(1, 3)
    .byte_range(0, 52)
    .build();

    let s2 = CdmSectionBuilder::new(
        "sec-2",
        "https://github.com/example/manifest-demo/blob/main/README.md",
        SectionKind::Prose,
        "# Manifest Demo\n\nDemonstration of manifests.\n",
    )
    .title("README.md")
    .original_path("README.md")
    .language("markdown")
    .line_range(1, 3)
    .byte_range(0, 44)
    .build();

    let mut root = HierarchyNode::root("manifest-demo");
    let mut src_dir = HierarchyNode::directory("src-dir", "src", "src");
    let lib_file = HierarchyNode::file(
        "lib-file",
        "lib.rs",
        "src/lib.rs",
        Some("sec-1".to_string()),
        Some(52),
        Some(3),
    );
    src_dir.add_child(lib_file);
    root.add_child(src_dir);

    let citation = Citation::new(
        "cite-1",
        "https://github.com/example/manifest-demo/blob/main/src/lib.rs",
        Some("Source code citation".to_string()),
        Some("Apache-2.0".to_string()),
    );

    let exclusion = ExclusionRecord::new(
        "target/debug/build.log",
        ExclusionReason::RulePattern,
        Some("target/**".to_string()),
        Some("Build artifact omitted".to_string()),
        Some(1024),
    );

    let completeness = CompletenessReport::new(5, 4, 4, 1, 0, 0);

    CdmDocumentBuilder::new(
        identity,
        SourceType::GitHubPublic,
        "https://github.com/example/manifest-demo",
    )
    .document_id(DocumentId::new())
    .revision(RevisionInfo {
        git_commit: Some("9a8b7c6d5e4f3a2b1c0d".to_string()),
        git_branch: Some("main".to_string()),
        content_hash_sha256: Some(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
        ),
        content_hash_blake3: Some(
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262".to_string(),
        ),
        ..Default::default()
    })
    .metadata(DocumentMetadata {
        title: Some("Manifest Demo Repository".to_string()),
        description: Some("Demonstration of standalone manifests and checksums".to_string()),
        license: Some("Apache-2.0".to_string()),
        primary_language: Some("Rust".to_string()),
        authors: vec!["Alice Dev".to_string()],
        language_breakdown: BTreeMap::from([("Rust".to_string(), 52)]),
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

#[test]
fn test_manifest_options_and_checksum_options_serde() {
    let m_opts = ManifestOptions::default();
    let m_json = serde_json::to_string(&m_opts).unwrap();
    let m_deserialized: ManifestOptions = serde_json::from_str(&m_json).unwrap();
    assert_eq!(m_opts, m_deserialized);

    let m_compact = ManifestOptions::compact();
    assert!(!m_compact.pretty_print);
    assert!(m_compact.include_blake3);

    let c_opts = ChecksumOptions::default();
    let c_json = serde_json::to_string(&c_opts).unwrap();
    let c_deserialized: ChecksumOptions = serde_json::from_str(&c_json).unwrap();
    assert_eq!(c_opts, c_deserialized);

    let c_blake = ChecksumOptions::blake3();
    assert_eq!(c_blake.algorithm, ChecksumAlgorithm::Blake3);
}

#[test]
fn test_mime_type_detection() {
    assert_eq!(
        detect_mime_type("context.md"),
        "text/markdown; charset=utf-8"
    );
    assert_eq!(detect_mime_type("context.txt"), "text/plain; charset=utf-8");
    assert_eq!(detect_mime_type("manifest.json"), "application/json");
    assert_eq!(detect_mime_type("context.jsonl"), "application/x-ndjson");
    assert_eq!(detect_mime_type("index.html"), "text/html; charset=utf-8");
    assert_eq!(detect_mime_type("archive.zip"), "application/zip");
    assert_eq!(detect_mime_type("main.rs"), "text/x-rust; charset=utf-8");
    assert_eq!(
        detect_mime_type("app.ts"),
        "application/typescript; charset=utf-8"
    );
    assert_eq!(
        detect_mime_type("index.js"),
        "application/javascript; charset=utf-8"
    );
    assert_eq!(
        detect_mime_type("script.py"),
        "text/x-python; charset=utf-8"
    );
    assert_eq!(detect_mime_type("main.go"), "text/x-go; charset=utf-8");
    assert_eq!(detect_mime_type("styles.css"), "text/css; charset=utf-8");
    assert_eq!(detect_mime_type("Cargo.toml"), "application/toml");
    assert_eq!(
        detect_mime_type("checksums.sha256"),
        "text/plain; charset=utf-8"
    );
}

#[test]
fn test_package_file_entry_from_bytes() {
    let content = b"line 1\nline 2\nline 3\n";
    let entry = PackageFileEntry::from_bytes("test.rs", content, "rust", true);

    assert_eq!(entry.path, "test.rs");
    assert_eq!(entry.size_bytes, content.len() as u64);
    assert_eq!(entry.format, "rust");
    assert_eq!(entry.mime_type, "text/x-rust; charset=utf-8");
    assert_eq!(entry.line_count, Some(3));
    assert!(entry.token_estimate.is_some());
    assert_eq!(entry.sha256.len(), 64);
    assert!(entry.blake3.is_some());
    assert_eq!(entry.blake3.unwrap().len(), 64);

    // Without BLAKE3
    let entry_no_blake = PackageFileEntry::from_bytes("data.bin", b"\x00\x01\x02", "binary", false);
    assert!(entry_no_blake.blake3.is_none());
}

#[test]
fn test_manifest_generation_and_serialization() {
    let doc = create_sample_cdm_document();

    let file1 =
        PackageFileEntry::from_bytes("context.md", b"# Header\n\nContent", "markdown", true);
    let file2 = PackageFileEntry::from_bytes("context.txt", b"Header\nContent", "plain_text", true);
    let files = vec![file1, file2];

    let manifest = build_manifest(&doc, files);
    assert_eq!(manifest.manifest_version, "1.0.0");
    assert_eq!(manifest.package_generator, "web-repo-to-ai v0.1.0");
    assert_eq!(manifest.document_id, doc.document_id.to_string());
    assert_eq!(manifest.schema_version, doc.schema_version);
    assert_eq!(
        manifest.source_url,
        "https://github.com/example/manifest-demo"
    );
    assert_eq!(manifest.file_count, 2);
    assert!(manifest.total_uncompressed_bytes > 0);

    // Pretty serialization
    let json_pretty = export_manifest(&manifest, true).unwrap();
    assert!(json_pretty.contains('\n'));
    assert!(json_pretty.contains("\"manifest_version\": \"1.0.0\""));

    // Deserialization roundtrip
    let parsed: PackageManifest = serde_json::from_str(&json_pretty).unwrap();
    assert_eq!(parsed, manifest);

    // Compact serialization
    let json_compact = export_manifest(&manifest, false).unwrap();
    assert!(!json_compact.contains('\n'));

    // Streaming writer equivalence
    let mut writer_bytes = Vec::new();
    let bytes_written = export_manifest_to_writer(&manifest, &mut writer_bytes, true).unwrap();
    assert_eq!(bytes_written, writer_bytes.len());
    assert_eq!(String::from_utf8(writer_bytes).unwrap(), json_pretty);
}

#[test]
fn test_provenance_and_quality_report_exports() {
    let doc = create_sample_cdm_document();

    // 1. Provenance
    let prov = build_provenance(&doc);
    assert_eq!(prov.document_id, doc.document_id.to_string());
    assert_eq!(prov.canonical_url, doc.canonical_url);
    assert_eq!(prov.citations_count, 1);
    assert_eq!(prov.citations.len(), 1);

    let prov_json = export_provenance(&prov, true).unwrap();
    let parsed_prov: PackageProvenance = serde_json::from_str(&prov_json).unwrap();
    assert_eq!(parsed_prov, prov);

    // Streaming writer
    let mut prov_buf = Vec::new();
    export_provenance_to_writer(&prov, &mut prov_buf, true).unwrap();
    assert_eq!(String::from_utf8(prov_buf).unwrap(), prov_json);

    // 2. Quality Report
    let qr = build_quality_report(&doc);
    assert_eq!(qr.document_id, doc.document_id.to_string());
    assert_eq!(qr.discovered_count, 5);
    assert_eq!(qr.extracted_count, 4);
    assert_eq!(qr.excluded_count, 1);
    assert_eq!(qr.completeness_percentage, "80.0%");
    assert_eq!(qr.exclusions_count, 1);

    let qr_json = export_quality_report(&qr, true).unwrap();
    let parsed_qr: PackageQualityReport = serde_json::from_str(&qr_json).unwrap();
    assert_eq!(parsed_qr.document_id, qr.document_id);
    assert_eq!(
        parsed_qr.completeness_percentage,
        qr.completeness_percentage
    );

    // Streaming writer
    let mut qr_buf = Vec::new();
    export_quality_report_to_writer(&qr, &mut qr_buf, true).unwrap();
    assert_eq!(String::from_utf8(qr_buf).unwrap(), qr_json);
}

#[test]
fn test_manifest_exporter_builder_api() {
    let doc = create_sample_cdm_document();
    let file = PackageFileEntry::from_bytes("data.json", b"{\"key\":\"val\"}", "json", true);

    let exporter = ManifestExporter::new();
    let json_str = exporter.export_to_string(&doc, vec![file.clone()]).unwrap();
    assert!(json_str.contains("\"data.json\""));

    let mut buf = Vec::new();
    exporter
        .export_to_writer(&doc, vec![file], &mut buf)
        .unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), json_str);
}

#[test]
fn test_checksums_generation_and_coreutils_format() {
    let bytes1 = b"First file content\n";
    let bytes2 = b"Second file content\n";

    let e1 = ChecksumEntry::from_sha256_bytes("file1.txt", bytes1);
    let e2 = ChecksumEntry::from_sha256_bytes("sub/file2.md", bytes2);
    let entries = vec![e1.clone(), e2.clone()];

    let manifest_str = generate_checksums_manifest(&entries);

    // Coreutils requires exact: "<hash>  <path>\n"
    let lines: Vec<&str> = manifest_str.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], format!("{}  file1.txt", e1.hash));
    assert_eq!(lines[1], format!("{}  sub/file2.md", e2.hash));

    // Streaming writer equivalence
    let mut buf = Vec::new();
    let written = export_checksums_to_writer(&entries, &mut buf).unwrap();
    assert_eq!(written, buf.len());
    assert_eq!(String::from_utf8(buf).unwrap(), manifest_str);
}

#[test]
fn test_checksums_parsing_valid_and_commented() {
    let raw = r#"
# Standard SHA-256 Checksums
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  empty.txt
# Binary flag variant
ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad *abc.txt

# Trailing whitespace
cb8379ac2098aa165029e3938a51da0bcecfc008fd97974306403a65ac130c52   spaced/path.rs
"#;

    let parsed = parse_checksums_manifest(raw, ChecksumAlgorithm::Sha256).unwrap();
    assert_eq!(parsed.len(), 3);

    assert_eq!(parsed[0].path, "empty.txt");
    assert_eq!(
        parsed[0].hash,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );

    assert_eq!(parsed[1].path, "abc.txt");
    assert_eq!(
        parsed[1].hash,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );

    assert_eq!(parsed[2].path, "spaced/path.rs");
    assert_eq!(
        parsed[2].hash,
        "cb8379ac2098aa165029e3938a51da0bcecfc008fd97974306403a65ac130c52"
    );
}

#[test]
fn test_checksums_parsing_invalid_rejections() {
    // Missing filename
    let bad_line1 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    assert!(parse_checksums_manifest(bad_line1, ChecksumAlgorithm::Sha256).is_err());

    // Invalid hash length (short)
    let bad_line2 = "e3b0c442  short.txt";
    let err2 = parse_checksums_manifest(bad_line2, ChecksumAlgorithm::Sha256).unwrap_err();
    assert!(matches!(err2, ExporterError::ChecksumValidation(_)));

    // Non-hex characters in hash
    let bad_line3 = "g3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  file.txt";
    let err3 = parse_checksums_manifest(bad_line3, ChecksumAlgorithm::Sha256).unwrap_err();
    assert!(matches!(err3, ExporterError::ChecksumValidation(_)));
}

#[test]
fn test_checksum_verification_matching_tampered_and_missing() {
    let file1_bytes = b"Hello from file 1\n";
    let file2_bytes = b"Hello from file 2\n";
    let file3_bytes = b"Hello from file 3\n";

    let e1 = ChecksumEntry::from_sha256_bytes("file1.txt", file1_bytes);
    let e2 = ChecksumEntry::from_sha256_bytes("file2.txt", file2_bytes);
    let e3 = ChecksumEntry::from_sha256_bytes("file3.txt", file3_bytes);
    let entries = vec![e1, e2, e3];

    let mut virtual_fs: HashMap<&str, &[u8]> = HashMap::new();
    virtual_fs.insert("file1.txt", file1_bytes);
    virtual_fs.insert("file2.txt", file2_bytes);
    virtual_fs.insert("file3.txt", file3_bytes);

    // Scenario 1: All files match perfectly
    let report1 = verify_checksums(&entries, |p| virtual_fs.get(p).copied());
    assert!(report1.is_valid());
    assert_eq!(report1.matched_files.len(), 3);
    assert!(report1.mismatched_files.is_empty());
    assert!(report1.missing_files.is_empty());

    // Scenario 2: File 2 is tampered/corrupted
    let tampered_bytes = b"Tampered malicious content!\n";
    virtual_fs.insert("file2.txt", tampered_bytes);
    let report2 = verify_checksums(&entries, |p| virtual_fs.get(p).copied());
    assert!(!report2.is_valid());
    assert_eq!(report2.matched_files.len(), 2);
    assert_eq!(report2.mismatched_files.len(), 1);
    assert_eq!(report2.mismatched_files[0].path, "file2.txt");
    assert_ne!(
        report2.mismatched_files[0].actual,
        report2.mismatched_files[0].expected
    );

    // Scenario 3: File 3 is missing
    virtual_fs.remove("file3.txt");
    let report3 = verify_checksums(&entries, |p| virtual_fs.get(p).copied());
    assert!(!report3.is_valid());
    assert_eq!(report3.missing_files, vec!["file3.txt"]);
}

#[test]
fn test_blake3_checksum_generation_and_verification() {
    let bytes = b"BLAKE3 test data\n";
    let entry = ChecksumEntry::from_blake3_bytes("data.bin", bytes);
    assert_eq!(entry.algorithm, ChecksumAlgorithm::Blake3);

    let manifest = generate_checksums_manifest(std::slice::from_ref(&entry));
    let parsed = parse_checksums_manifest(&manifest, ChecksumAlgorithm::Blake3).unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].algorithm, ChecksumAlgorithm::Blake3);

    let report = verify_checksums(
        &parsed,
        |p| {
            if p == "data.bin" {
                Some(bytes)
            } else {
                None
            }
        },
    );
    assert!(report.is_valid());
}
