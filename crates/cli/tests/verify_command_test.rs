//! # Artifact and Checksum Verification Integration Tests
//!
//! Validates `web-repo-to-ai verify`:
//! - Single-file SHA-256 and BLAKE3 verification
//! - `checksums.sha256` coreutils format manifest parsing & validation
//! - `manifest.json` package inventory verification
//! - ZIP package integrity and embedded manifest verification
//! - Strict CDM schema validation for JSON documents
//! - Structured JSON summary envelopes (`VerifyJsonEnvelope`)

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};
use web_repo_to_ai_cdm::builder::CdmDocumentBuilder;
use web_repo_to_ai_cdm::model::{SourceIdentity, SourceType};
use web_repo_to_ai_cli::{run_cli_with_args, ExitCode};
use web_repo_to_ai_exporters::manifest::{PackageFileEntry, PackageManifest};
use web_repo_to_ai_exporters::{export_zip_package, ZipPackageOptions};

static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(300);

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(prefix: &str) -> Self {
        let seq = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "web_repo_to_ai_test_verify_{prefix}_{}_{seq}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create test temp dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn blake3_hex(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

#[tokio::test]
async fn test_verify_cli_help() {
    let res = run_cli_with_args(["web-repo-to-ai", "verify", "--help"]).await;
    assert_eq!(res.unwrap(), 0);
}

#[tokio::test]
async fn test_verify_missing_target_error() {
    let res = run_cli_with_args(["web-repo-to-ai", "verify"]).await;
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.exit_code(), ExitCode::Validation as i32);
}

#[tokio::test]
async fn test_verify_single_file_sha256_and_blake3() {
    let temp = TestDir::new("single");
    let file = temp.path().join("content.txt");
    let text = b"Deterministic payload for verification test.";
    fs::write(&file, text).unwrap();

    let expected_sha256 = sha256_hex(text);
    let expected_blake3 = blake3_hex(text);

    // 1. Matching SHA-256
    let res = run_cli_with_args([
        "web-repo-to-ai",
        "verify",
        file.to_str().unwrap(),
        "--sha256",
        &expected_sha256,
    ])
    .await;
    assert_eq!(res.unwrap(), ExitCode::Success as i32);

    // 2. Mismatching SHA-256 returns validation exit code 1
    let res_mismatch = run_cli_with_args([
        "web-repo-to-ai",
        "verify",
        file.to_str().unwrap(),
        "--sha256",
        "0000000000000000000000000000000000000000000000000000000000000000",
    ])
    .await;
    assert_eq!(res_mismatch.unwrap(), ExitCode::Validation as i32);

    // 3. Matching BLAKE3
    let res_b3 = run_cli_with_args([
        "web-repo-to-ai",
        "verify",
        file.to_str().unwrap(),
        "--blake3",
        &expected_blake3,
    ])
    .await;
    assert_eq!(res_b3.unwrap(), ExitCode::Success as i32);

    // 4. Mismatching BLAKE3 returns 1
    let res_b3_err = run_cli_with_args([
        "web-repo-to-ai",
        "verify",
        file.to_str().unwrap(),
        "--blake3",
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    ])
    .await;
    assert_eq!(res_b3_err.unwrap(), ExitCode::Validation as i32);
}

#[tokio::test]
async fn test_verify_checksums_manifest_file() {
    let temp = TestDir::new("checksums");
    let f1 = temp.path().join("doc1.md");
    let f2 = temp.path().join("doc2.md");
    let f3 = temp.path().join("doc3.json");

    let b1 = b"# Doc 1\nContent 1";
    let b2 = b"# Doc 2\nContent 2";
    let b3 = b"{\"key\": \"val\"}";

    fs::write(&f1, b1).unwrap();
    fs::write(&f2, b2).unwrap();
    fs::write(&f3, b3).unwrap();

    let checksums_file = temp.path().join("checksums.sha256");
    let manifest_body = format!(
        "{}  doc1.md\n{} *doc2.md\n{}  doc3.json\n",
        sha256_hex(b1),
        sha256_hex(b2),
        sha256_hex(b3)
    );
    fs::write(&checksums_file, manifest_body).unwrap();

    // 1. Verification succeeds
    let res =
        run_cli_with_args(["web-repo-to-ai", "verify", checksums_file.to_str().unwrap()]).await;
    assert_eq!(res.unwrap(), ExitCode::Success as i32);

    // 2. Corrupted file detected
    fs::write(&f1, b"# Doc 1 - Tampered!").unwrap();
    let res_tampered =
        run_cli_with_args(["web-repo-to-ai", "verify", checksums_file.to_str().unwrap()]).await;
    assert_eq!(res_tampered.unwrap(), ExitCode::Validation as i32);

    // 3. Missing file detected
    let _ = fs::remove_file(&f2);
    let res_missing =
        run_cli_with_args(["web-repo-to-ai", "verify", checksums_file.to_str().unwrap()]).await;
    assert_eq!(res_missing.unwrap(), ExitCode::Validation as i32);
}

#[tokio::test]
async fn test_verify_package_manifest_json() {
    let temp = TestDir::new("manifest");
    let f1 = temp.path().join("context.md");
    let b1 = b"# Unified Context Document\nExtracted content.";
    fs::write(&f1, b1).unwrap();

    let manifest = PackageManifest {
        manifest_version: "1.0.0".to_string(),
        package_generator: "web-repo-to-ai-test/0.1.0".to_string(),
        document_id: "018f0a00-0000-7000-8000-000000000001".to_string(),
        schema_version: "1.0.0".to_string(),
        created_at: "2026-11-19T00:00:00Z".to_string(),
        source_url: "https://example.com".to_string(),
        source_type: "webpage".to_string(),
        file_count: 1,
        total_uncompressed_bytes: b1.len() as u64,
        files: vec![PackageFileEntry {
            path: "context.md".to_string(),
            size_bytes: b1.len() as u64,
            sha256: sha256_hex(b1),
            blake3: Some(blake3_hex(b1)),
            format: "markdown".to_string(),
            mime_type: "text/markdown; charset=utf-8".to_string(),
            line_count: Some(2),
            token_estimate: Some(10),
        }],
    };

    let manifest_file = temp.path().join("manifest.json");
    fs::write(
        &manifest_file,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();

    let res =
        run_cli_with_args(["web-repo-to-ai", "verify", manifest_file.to_str().unwrap()]).await;
    assert_eq!(res.unwrap(), ExitCode::Success as i32);
}

#[tokio::test]
async fn test_verify_zip_package_integrity() {
    let temp = TestDir::new("zip");
    let zip_path = temp.path().join("archive.zip");

    let source_identity = SourceIdentity::new("https://example.com/zip", "https://example.com/zip");
    let cdm_doc = CdmDocumentBuilder::new(
        source_identity,
        SourceType::WebpageStatic,
        "https://example.com/zip",
    )
    .title("ZIP Verified Title")
    .build();

    let options = ZipPackageOptions::default();
    let (zip_bytes, _metrics) = export_zip_package(&cdm_doc, &options).expect("export zip package");
    fs::write(&zip_path, zip_bytes).unwrap();

    let res = run_cli_with_args(["web-repo-to-ai", "verify", zip_path.to_str().unwrap()]).await;
    assert_eq!(res.unwrap(), ExitCode::Success as i32);
}

#[tokio::test]
async fn test_verify_cdm_document_schema() {
    let temp = TestDir::new("cdm");

    // 1. Valid CDM document
    let valid_file = temp.path().join("valid_doc.json");
    let source_identity = SourceIdentity::new(
        "https://example.com/verified-doc",
        "https://example.com/verified-doc",
    );
    let cdm_doc = CdmDocumentBuilder::new(
        source_identity,
        SourceType::WebpageStatic,
        "https://example.com/verified-doc",
    )
    .title("Verified Title")
    .build();

    let json_bytes = serde_json::to_vec_pretty(&cdm_doc).unwrap();
    fs::write(&valid_file, &json_bytes).unwrap();

    let res_valid = run_cli_with_args([
        "web-repo-to-ai",
        "verify",
        valid_file.to_str().unwrap(),
        "--strict",
    ])
    .await;
    assert_eq!(res_valid.unwrap(), ExitCode::Success as i32);

    // 2. Invalid CDM document
    let invalid_file = temp.path().join("invalid_doc.json");
    fs::write(&invalid_file, b"{\"not_a\": \"cdm_document\"}").unwrap();

    let res_invalid = run_cli_with_args([
        "web-repo-to-ai",
        "verify",
        invalid_file.to_str().unwrap(),
        "--strict",
    ])
    .await;
    assert_eq!(res_invalid.unwrap(), ExitCode::Validation as i32);
}

#[tokio::test]
async fn test_verify_json_envelope() {
    let temp = TestDir::new("envelope");
    let file = temp.path().join("item.txt");
    fs::write(&file, b"Hello Envelope").unwrap();
    let expected = sha256_hex(b"Hello Envelope");

    let res = run_cli_with_args([
        "web-repo-to-ai",
        "verify",
        file.to_str().unwrap(),
        "--sha256",
        &expected,
        "--json",
    ])
    .await;
    assert_eq!(res.unwrap(), ExitCode::Success as i32);
}
