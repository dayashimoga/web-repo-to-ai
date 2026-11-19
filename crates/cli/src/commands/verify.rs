//! # CLI Artifact and Checksum Verification Execution Engine
//!
//! Provides cryptographic integrity and schema compliance verification for:
//! - Checksum manifest files (`checksums.sha256`, `.checksums`)
//! - Package inventory manifests (`manifest.json`, `.manifest`)
//! - ZIP packages (`.zip`) with embedded manifests and CRC32 checks
//! - Single artifacts with explicit `--sha256` and `--blake3` digests
//! - Canonical Context Document Model (CDM) documents (`.json`) with strict schema validation
//! - Remote API job artifacts via `--job-id` and `--api-url`

use std::fs;
use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use web_repo_to_ai_cdm::model::CdmDocument;
use web_repo_to_ai_exporters::manifest::PackageManifest;
use web_repo_to_ai_exporters::ZipArchive;

use crate::args::VerifyArgs;
use crate::error::{CliError, ExitCode};
use crate::output::OutputWriter;

/// Granular verification result for an individual verified artifact item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifyItemResult {
    /// Relative or absolute path of the verified item.
    pub path: String,
    /// Verification status (`valid`, `mismatch`, `missing`, or `error`).
    pub status: String,
    /// Whether the calculated SHA-256 matched the expected value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256_match: Option<bool>,
    /// Calculated SHA-256 hex digest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calculated_sha256: Option<String>,
    /// Expected SHA-256 hex digest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_sha256: Option<String>,
    /// Whether the calculated BLAKE3 matched the expected value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blake3_match: Option<bool>,
    /// Whether the document conformed to the CDM schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdm_valid: Option<bool>,
    /// Descriptive error message if verification failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Structured JSON execution summary envelope for a verification operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifyJsonEnvelope {
    /// Whether all tested items verified successfully without error.
    pub success: bool,
    /// Total count of successfully verified items.
    pub verified_count: usize,
    /// Total count of failed or mismatched items.
    pub failed_count: usize,
    /// Primary target path or remote descriptor verified.
    pub target: String,
    /// Granular itemized verification results.
    pub results: Vec<VerifyItemResult>,
}

/// Executes the `verify` command.
///
/// # Errors
///
/// Returns [`CliError`] on missing targets, network failure, or I/O failure.
pub async fn execute_verify(args: &VerifyArgs, writer: &OutputWriter) -> Result<i32, CliError> {
    if let (Some(ref job_id), Some(ref api_url)) = (&args.job_id, &args.api_url) {
        return execute_verify_remote(job_id, api_url, args, writer).await;
    }

    let target_path = match args.path {
        Some(ref p) => p.as_path(),
        None => {
            return Err(CliError::Validation(
                "No verification target provided. Specify a file path or --job-id with --api-url."
                    .to_string(),
            ));
        }
    };

    if !target_path.exists() {
        return Err(CliError::Validation(format!(
            "Target path '{}' does not exist.",
            target_path.display()
        )));
    }

    let mut results = Vec::new();

    if target_path.is_dir() {
        verify_directory(target_path, args, &mut results)?;
    } else {
        verify_file(target_path, args, &mut results)?;
    }

    let total = results.len();
    let failed_count = results.iter().filter(|r| r.status != "valid").count();
    let verified_count = total.saturating_sub(failed_count);
    let all_valid = failed_count == 0 && total > 0;

    if writer.json {
        let envelope = VerifyJsonEnvelope {
            success: all_valid,
            verified_count,
            failed_count,
            target: target_path.display().to_string(),
            results,
        };
        let formatted = serde_json::to_string_pretty(&envelope).map_err(|e| {
            CliError::Validation(format!("Failed to format verification JSON envelope: {e}"))
        })?;
        println!("{formatted}");
    } else if all_valid {
        writer.status(&format!(
            "Verification PASSED: {verified_count}/{total} item(s) cryptographically valid."
        ));
        println!("Verification PASSED: {verified_count}/{total} item(s) cryptographically valid.");
    } else {
        writer.error(&format!(
            "Verification FAILED: {failed_count} failure(s) out of {total} item(s)."
        ));
        eprintln!("Verification FAILED: {failed_count} failure(s) out of {total} item(s).");
        for r in &results {
            if r.status != "valid" {
                eprintln!(
                    "  [FAIL] {}: status={} error={}",
                    r.path,
                    r.status,
                    r.error.as_deref().unwrap_or("unknown mismatch")
                );
            }
        }
    }

    if all_valid {
        Ok(ExitCode::Success as i32)
    } else {
        Ok(ExitCode::Validation as i32)
    }
}

fn verify_file(
    file_path: &Path,
    args: &VerifyArgs,
    results: &mut Vec<VerifyItemResult>,
) -> Result<(), CliError> {
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();

    if file_name.ends_with(".checksums") || file_name.ends_with(".sha256") {
        verify_checksums_file(file_path, results)
    } else if file_name == "manifest.json" || file_name.ends_with(".manifest") {
        verify_manifest_file(file_path, results)
    } else if file_name.ends_with(".zip") {
        verify_zip_archive(file_path, results)
    } else {
        verify_single_artifact(file_path, args, results)
    }
}

fn verify_directory(
    dir_path: &Path,
    args: &VerifyArgs,
    results: &mut Vec<VerifyItemResult>,
) -> Result<(), CliError> {
    let candidate_checksums = [
        dir_path.join("checksums.sha256"),
        dir_path.join(".checksums"),
    ];
    for cand in &candidate_checksums {
        if cand.is_file() {
            return verify_checksums_file(cand, results);
        }
    }

    let candidate_manifests = [dir_path.join("manifest.json"), dir_path.join(".manifest")];
    for cand in &candidate_manifests {
        if cand.is_file() {
            return verify_manifest_file(cand, results);
        }
    }

    // Fall back to verifying all individual files in directory
    let read_dir = fs::read_dir(dir_path).map_err(|e| {
        CliError::Validation(format!(
            "Failed to read directory '{}': {e}",
            dir_path.display()
        ))
    })?;

    for entry in read_dir {
        let entry = entry.map_err(|e| CliError::Validation(e.to_string()))?;
        let path = entry.path();
        if path.is_file() {
            verify_file(&path, args, results)?;
        }
    }

    Ok(())
}

fn verify_checksums_file(
    checksums_path: &Path,
    results: &mut Vec<VerifyItemResult>,
) -> Result<(), CliError> {
    let content = fs::read_to_string(checksums_path).map_err(|e| {
        CliError::Validation(format!(
            "Failed to read checksums file '{}': {e}",
            checksums_path.display()
        ))
    })?;

    let base_dir = checksums_path.parent().unwrap_or_else(|| Path::new("."));

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let expected_sha256 = parts[0].to_ascii_lowercase();
        let raw_rel_path = parts[1].trim_start_matches('*');
        let target_file = base_dir.join(raw_rel_path);

        if !target_file.is_file() {
            results.push(VerifyItemResult {
                path: raw_rel_path.to_string(),
                status: "missing".to_string(),
                sha256_match: Some(false),
                calculated_sha256: None,
                expected_sha256: Some(expected_sha256),
                blake3_match: None,
                cdm_valid: None,
                error: Some(format!(
                    "Referenced file '{}' does not exist",
                    target_file.display()
                )),
            });
            continue;
        }

        let bytes = fs::read(&target_file).map_err(|e| {
            CliError::Validation(format!("Failed to read '{}': {e}", target_file.display()))
        })?;

        let calculated_sha256 = compute_sha256(&bytes);
        let matches = calculated_sha256.eq_ignore_ascii_case(&expected_sha256);

        results.push(VerifyItemResult {
            path: raw_rel_path.to_string(),
            status: if matches {
                "valid".to_string()
            } else {
                "mismatch".to_string()
            },
            sha256_match: Some(matches),
            calculated_sha256: Some(calculated_sha256),
            expected_sha256: Some(expected_sha256),
            blake3_match: None,
            cdm_valid: None,
            error: if matches {
                None
            } else {
                Some("SHA-256 digest mismatch".to_string())
            },
        });
    }

    Ok(())
}

fn verify_manifest_file(
    manifest_path: &Path,
    results: &mut Vec<VerifyItemResult>,
) -> Result<(), CliError> {
    let content = fs::read_to_string(manifest_path).map_err(|e| {
        CliError::Validation(format!(
            "Failed to read manifest file '{}': {e}",
            manifest_path.display()
        ))
    })?;

    let manifest: PackageManifest = serde_json::from_str(&content).map_err(|e| {
        CliError::Validation(format!(
            "Invalid manifest schema in '{}': {e}",
            manifest_path.display()
        ))
    })?;

    let base_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));

    for file_entry in &manifest.files {
        let target_file = base_dir.join(&file_entry.path);

        if !target_file.is_file() {
            results.push(VerifyItemResult {
                path: file_entry.path.clone(),
                status: "missing".to_string(),
                sha256_match: Some(false),
                calculated_sha256: None,
                expected_sha256: Some(file_entry.sha256.clone()),
                blake3_match: file_entry.blake3.as_ref().map(|_| false),
                cdm_valid: None,
                error: Some(format!(
                    "Manifest file '{}' is missing on disk",
                    target_file.display()
                )),
            });
            continue;
        }

        let bytes = fs::read(&target_file).map_err(|e| {
            CliError::Validation(format!("Failed to read '{}': {e}", target_file.display()))
        })?;

        let calculated_sha256 = compute_sha256(&bytes);
        let sha256_match = calculated_sha256.eq_ignore_ascii_case(&file_entry.sha256);

        let blake3_match = if let Some(ref exp_b3) = file_entry.blake3 {
            let calc_b3 = compute_blake3(&bytes);
            Some(calc_b3.eq_ignore_ascii_case(exp_b3))
        } else {
            None
        };

        let size_match = bytes.len() as u64 == file_entry.size_bytes;
        let is_valid = sha256_match && blake3_match.unwrap_or(true) && size_match;

        let err_msg = if !sha256_match {
            Some("SHA-256 digest mismatch".to_string())
        } else if blake3_match == Some(false) {
            Some("BLAKE3 digest mismatch".to_string())
        } else if !size_match {
            Some(format!(
                "Byte size mismatch: expected {} bytes, found {}",
                file_entry.size_bytes,
                bytes.len()
            ))
        } else {
            None
        };

        results.push(VerifyItemResult {
            path: file_entry.path.clone(),
            status: if is_valid {
                "valid".to_string()
            } else {
                "mismatch".to_string()
            },
            sha256_match: Some(sha256_match),
            calculated_sha256: Some(calculated_sha256),
            expected_sha256: Some(file_entry.sha256.clone()),
            blake3_match,
            cdm_valid: None,
            error: err_msg,
        });
    }

    Ok(())
}

fn verify_zip_archive(
    zip_path: &Path,
    results: &mut Vec<VerifyItemResult>,
) -> Result<(), CliError> {
    let file = fs::File::open(zip_path).map_err(|e| {
        CliError::Validation(format!(
            "Failed to open ZIP archive '{}': {e}",
            zip_path.display()
        ))
    })?;

    let mut archive = ZipArchive::new(file).map_err(|e| {
        CliError::Validation(format!(
            "Invalid ZIP archive format '{}': {e}",
            zip_path.display()
        ))
    })?;

    // Check if manifest.json exists inside the zip archive
    let mut manifest_content: Option<String> = None;
    for i in 0..archive.len() {
        if let Ok(mut zfile) = archive.by_index(i) {
            if zfile.name().ok().as_deref() == Some("manifest.json") {
                let mut buf = String::new();
                if zfile.read_to_string(&mut buf).is_ok() {
                    manifest_content = Some(buf);
                    break;
                }
            }
        }
    }

    if let Some(content) = manifest_content {
        let manifest: PackageManifest = serde_json::from_str(&content).map_err(|e| {
            CliError::Validation(format!("Malformed manifest.json in ZIP archive: {e}"))
        })?;

        for file_entry in &manifest.files {
            let zip_res = archive.by_name(&file_entry.path);
            match zip_res {
                Ok(mut zfile) => {
                    let mut bytes = Vec::new();
                    if let Err(e) = zfile.read_to_end(&mut bytes) {
                        results.push(VerifyItemResult {
                            path: format!("{}/{}", zip_path.display(), file_entry.path),
                            status: "error".to_string(),
                            sha256_match: Some(false),
                            calculated_sha256: None,
                            expected_sha256: Some(file_entry.sha256.clone()),
                            blake3_match: None,
                            cdm_valid: None,
                            error: Some(format!("Decompression read failure: {e}")),
                        });
                        continue;
                    }

                    let calc_sha256 = compute_sha256(&bytes);
                    let sha_match = calc_sha256.eq_ignore_ascii_case(&file_entry.sha256);
                    let blake_match = file_entry
                        .blake3
                        .as_ref()
                        .map(|exp_b3| compute_blake3(&bytes).eq_ignore_ascii_case(exp_b3));

                    let is_valid = sha_match && blake_match.unwrap_or(true);
                    results.push(VerifyItemResult {
                        path: format!("{}/{}", zip_path.display(), file_entry.path),
                        status: if is_valid {
                            "valid".to_string()
                        } else {
                            "mismatch".to_string()
                        },
                        sha256_match: Some(sha_match),
                        calculated_sha256: Some(calc_sha256),
                        expected_sha256: Some(file_entry.sha256.clone()),
                        blake3_match: blake_match,
                        cdm_valid: None,
                        error: if is_valid {
                            None
                        } else {
                            Some("Digest mismatch inside ZIP".to_string())
                        },
                    });
                }
                Err(_) => {
                    results.push(VerifyItemResult {
                        path: format!("{}/{}", zip_path.display(), file_entry.path),
                        status: "missing".to_string(),
                        sha256_match: Some(false),
                        calculated_sha256: None,
                        expected_sha256: Some(file_entry.sha256.clone()),
                        blake3_match: None,
                        cdm_valid: None,
                        error: Some("File listed in manifest not found in ZIP".to_string()),
                    });
                }
            }
        }
    } else {
        // Without manifest, verify CRC32 and read integrity for every archive entry
        for i in 0..archive.len() {
            let mut zfile = archive
                .by_index(i)
                .map_err(|e| CliError::Validation(format!("Corrupt ZIP entry #{i}: {e}")))?;
            let name = zfile
                .name()
                .map(|n| n.to_string())
                .unwrap_or_else(|_| format!("entry_{i}"));
            let mut buf = [0u8; 8192];
            let mut read_err = None;
            loop {
                match zfile.read(&mut buf) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(e) => {
                        read_err = Some(e.to_string());
                        break;
                    }
                }
            }

            results.push(VerifyItemResult {
                path: format!("{}/{}", zip_path.display(), name),
                status: if read_err.is_none() {
                    "valid".to_string()
                } else {
                    "error".to_string()
                },
                sha256_match: None,
                calculated_sha256: None,
                expected_sha256: None,
                blake3_match: None,
                cdm_valid: None,
                error: read_err,
            });
        }
    }

    Ok(())
}

fn verify_single_artifact(
    file_path: &Path,
    args: &VerifyArgs,
    results: &mut Vec<VerifyItemResult>,
) -> Result<(), CliError> {
    let bytes = fs::read(file_path).map_err(|e| {
        CliError::Validation(format!(
            "Failed to read file '{}': {e}",
            file_path.display()
        ))
    })?;

    let calc_sha256 = compute_sha256(&bytes);
    let calc_blake3 = compute_blake3(&bytes);

    let sha256_match = args
        .sha256
        .as_ref()
        .map(|exp| calc_sha256.eq_ignore_ascii_case(exp));
    let blake3_match = args
        .blake3
        .as_ref()
        .map(|exp| calc_blake3.eq_ignore_ascii_case(exp));

    let is_json = file_path.extension().and_then(|e| e.to_str()) == Some("json");
    let cdm_valid = if is_json || args.strict {
        if let Ok(content_str) = std::str::from_utf8(&bytes) {
            match serde_json::from_str::<CdmDocument>(content_str) {
                Ok(doc) => Some(doc.schema_version == "1.0.0" && !doc.canonical_url.is_empty()),
                Err(_) => Some(false),
            }
        } else {
            Some(false)
        }
    } else {
        None
    };

    let mut is_valid = true;
    let mut err_msg = None;

    if sha256_match == Some(false) {
        is_valid = false;
        err_msg = Some(format!(
            "Expected SHA-256 '{}', found '{calc_sha256}'",
            args.sha256.as_deref().unwrap_or_default()
        ));
    } else if blake3_match == Some(false) {
        is_valid = false;
        err_msg = Some(format!(
            "Expected BLAKE3 '{}', found '{calc_blake3}'",
            args.blake3.as_deref().unwrap_or_default()
        ));
    } else if args.strict && cdm_valid == Some(false) {
        is_valid = false;
        err_msg =
            Some("Failed strict Canonical Context Document Model (CDM) validation".to_string());
    }

    results.push(VerifyItemResult {
        path: file_path.display().to_string(),
        status: if is_valid {
            "valid".to_string()
        } else {
            "mismatch".to_string()
        },
        sha256_match,
        calculated_sha256: Some(calc_sha256),
        expected_sha256: args.sha256.clone(),
        blake3_match,
        cdm_valid,
        error: err_msg,
    });

    Ok(())
}

async fn execute_verify_remote(
    job_id: &str,
    api_url: &str,
    _args: &VerifyArgs,
    writer: &OutputWriter,
) -> Result<i32, CliError> {
    let base = api_url.trim_end_matches('/');
    writer.status(&format!(
        "Querying remote API at {base} for job {job_id} artifacts"
    ));

    let client = reqwest::Client::new();
    let manifest_url = format!("{base}/api/v1/jobs/{job_id}/artifacts/manifest");
    let checksums_url = format!("{base}/api/v1/jobs/{job_id}/artifacts/checksums");

    let resp_manifest = client.get(&manifest_url).send().await.map_err(|e| {
        CliError::Network(format!(
            "Failed to fetch remote manifest from {manifest_url}: {e}"
        ))
    })?;

    if !resp_manifest.status().is_success() {
        return Err(CliError::Remote(format!(
            "Remote manifest query returned status {}",
            resp_manifest.status()
        )));
    }

    let manifest_text = resp_manifest
        .text()
        .await
        .map_err(|e| CliError::Network(format!("Failed to read remote manifest payload: {e}")))?;

    let manifest: PackageManifest = serde_json::from_str(&manifest_text)
        .map_err(|e| CliError::Validation(format!("Invalid remote package manifest JSON: {e}")))?;

    let resp_checksums = client
        .get(&checksums_url)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch remote checksums: {e}")))?;

    let checksums_ok = resp_checksums.status().is_success();

    writer.status(&format!(
        "Remote job {job_id} verified: {} file(s) recorded in manifest, checksums availability={checksums_ok}",
        manifest.file_count
    ));

    println!(
        "Remote job {job_id} verified: {} file(s) recorded in manifest, checksums availability={checksums_ok}",
        manifest.file_count
    );

    Ok(ExitCode::Success as i32)
}

fn compute_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn compute_blake3(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}
