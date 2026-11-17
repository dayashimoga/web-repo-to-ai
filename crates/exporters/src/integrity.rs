//! # Package and Export Integrity Verification Engine (`crates/exporters/src/integrity.rs`)
//!
//! Provides diagnostic auditing, cryptographic verification, format validation, and security
//! analysis of generated distribution packages, manifests, and multi-format exports.

use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read, Seek};

use serde::{Deserialize, Serialize};

use crate::checksum::{
    compute_blake3_hex, compute_sha256_hex, parse_checksums_manifest, verify_checksums,
    ChecksumAlgorithm, ChecksumMismatch,
};
use crate::error::{ExporterError, Result};
use crate::manifest::{PackageManifest, PackageProvenance, PackageQualityReport};

/// Comprehensive diagnostic report detailing the integrity of an exported distribution package.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PackageIntegrityReport {
    /// Overall validity status. `true` only if zero errors, mismatches, or violations are found.
    pub is_valid: bool,
    /// List of file paths whose cryptographic hashes and sizes were verified successfully.
    pub verified_files: Vec<String>,
    /// Checksum mismatches indicating corrupted or tampered files.
    pub mismatched_checksums: Vec<ChecksumMismatch>,
    /// Files declared in `manifest.json` or `checksums.sha256` that are absent from the archive.
    pub missing_files: Vec<String>,
    /// Files present in the archive that were not declared in `manifest.json`.
    pub unlisted_files: Vec<String>,
    /// Files violating path sanitization rules (e.g. zip-slip directory traversal, absolute paths).
    pub path_traversal_violations: Vec<String>,
    /// Deserialized package manifest metadata, if present and valid.
    pub manifest: Option<PackageManifest>,
    /// Deserialized package provenance metadata, if present and valid.
    pub provenance: Option<PackageProvenance>,
    /// Deserialized package quality and completeness report, if present and valid.
    pub quality_report: Option<PackageQualityReport>,
}

impl PackageIntegrityReport {
    /// Returns `true` if any integrity issues, mismatches, missing files, or security violations exist.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.mismatched_checksums.is_empty()
            || !self.missing_files.is_empty()
            || !self.unlisted_files.is_empty()
            || !self.path_traversal_violations.is_empty()
            || self.verified_files.is_empty()
    }

    /// Total count of successfully verified entries.
    #[must_use]
    pub fn total_verified(&self) -> usize {
        self.verified_files.len()
    }
}

/// Verifies the cryptographic and structural integrity of an in-memory ZIP package.
///
/// # Errors
/// Returns [`ExporterError`] if the ZIP structure is unreadable or malformed.
pub fn verify_zip_package_bytes(zip_bytes: &[u8]) -> Result<PackageIntegrityReport> {
    verify_zip_package_integrity(Cursor::new(zip_bytes))
}

/// Verifies the cryptographic and structural integrity of a streaming or seekable ZIP package.
///
/// Performs deep verification:
/// 1. Verifies that all entry paths conform to strict zip-slip security rules (no `..`, leading `/`, or `\`).
/// 2. Reads and parses `manifest.json`, `provenance.json`, and `quality-report.json`.
/// 3. Cross-checks every entry in `checksums.sha256` against extracted file contents.
/// 4. Validates that every file declared in `manifest.json` exists with exact size and SHA-256 / BLAKE3 digests.
/// 5. Ensures zero unlisted rogue files exist inside the package.
///
/// # Errors
/// Returns [`ExporterError`] if the archive structure is corrupted.
pub fn verify_zip_package_integrity<R: Read + Seek>(reader: R) -> Result<PackageIntegrityReport> {
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| {
        ExporterError::InvalidDocument(format!("Failed to open ZIP package archive: {e}"))
    })?;

    let mut report = PackageIntegrityReport::default();
    let mut archive_files: HashMap<String, Vec<u8>> = HashMap::new();

    // 1. Scan and read all archive entries
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            ExporterError::InvalidDocument(format!("Failed to read ZIP entry at index {i}: {e}"))
        })?;

        let name = file
            .name()
            .map_err(|e| ExporterError::InvalidDocument(format!("Invalid entry name in ZIP: {e}")))?
            .to_string();

        // Check path security
        if name.contains("..")
            || name.starts_with('/')
            || name.contains('\\')
            || name.contains(':')
            || name.contains('\0')
        {
            report.path_traversal_violations.push(name.clone());
        }

        if file.is_dir() || name.ends_with('/') {
            continue;
        }

        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        archive_files.insert(name, content);
    }

    // 2. Parse `manifest.json`
    if let Some(manifest_bytes) = archive_files.get("manifest.json") {
        match serde_json::from_slice::<PackageManifest>(manifest_bytes) {
            Ok(m) => report.manifest = Some(m),
            Err(e) => {
                report.mismatched_checksums.push(ChecksumMismatch {
                    path: "manifest.json".to_string(),
                    expected: "valid JSON".to_string(),
                    actual: format!("syntax error: {e}"),
                    algorithm: ChecksumAlgorithm::Sha256,
                });
            }
        }
    } else {
        report.missing_files.push("manifest.json".to_string());
    }

    // 3. Parse `provenance.json` (if present)
    if let Some(prov_bytes) = archive_files.get("provenance.json") {
        match serde_json::from_slice::<PackageProvenance>(prov_bytes) {
            Ok(p) => report.provenance = Some(p),
            Err(e) => {
                report.mismatched_checksums.push(ChecksumMismatch {
                    path: "provenance.json".to_string(),
                    expected: "valid JSON".to_string(),
                    actual: format!("syntax error: {e}"),
                    algorithm: ChecksumAlgorithm::Sha256,
                });
            }
        }
    }

    // 4. Parse `quality-report.json` (if present)
    if let Some(qr_bytes) = archive_files.get("quality-report.json") {
        match serde_json::from_slice::<PackageQualityReport>(qr_bytes) {
            Ok(q) => report.quality_report = Some(q),
            Err(e) => {
                report.mismatched_checksums.push(ChecksumMismatch {
                    path: "quality-report.json".to_string(),
                    expected: "valid JSON".to_string(),
                    actual: format!("syntax error: {e}"),
                    algorithm: ChecksumAlgorithm::Sha256,
                });
            }
        }
    }

    // 5. Verify `checksums.sha256` (if present)
    if let Some(cs_bytes) = archive_files.get("checksums.sha256") {
        let cs_str = String::from_utf8_lossy(cs_bytes);
        match parse_checksums_manifest(&cs_str, ChecksumAlgorithm::Sha256) {
            Ok(entries) => {
                let cs_report =
                    verify_checksums(&entries, |p| archive_files.get(p).map(|v| v.as_slice()));

                for matched in cs_report.matched_files {
                    if !report.verified_files.contains(&matched) {
                        report.verified_files.push(matched);
                    }
                }
                for mismatch in cs_report.mismatched_files {
                    report.mismatched_checksums.push(mismatch);
                }
                for missing in cs_report.missing_files {
                    if !report.missing_files.contains(&missing) {
                        report.missing_files.push(missing);
                    }
                }
            }
            Err(e) => {
                report.mismatched_checksums.push(ChecksumMismatch {
                    path: "checksums.sha256".to_string(),
                    expected: "valid coreutils checksums".to_string(),
                    actual: format!("parse error: {e}"),
                    algorithm: ChecksumAlgorithm::Sha256,
                });
            }
        }
    } else {
        report.missing_files.push("checksums.sha256".to_string());
    }

    // 6. Cross-verify with `manifest.json` files inventory
    let mut declared_paths = HashSet::new();
    declared_paths.insert("checksums.sha256".to_string());
    declared_paths.insert("manifest.json".to_string());

    if let Some(ref manifest) = report.manifest {
        for file_entry in &manifest.files {
            declared_paths.insert(file_entry.path.clone());

            match archive_files.get(&file_entry.path) {
                None => {
                    if !report.missing_files.contains(&file_entry.path) {
                        report.missing_files.push(file_entry.path.clone());
                    }
                }
                Some(content) => {
                    // Check size
                    if content.len() as u64 != file_entry.size_bytes {
                        report.mismatched_checksums.push(ChecksumMismatch {
                            path: file_entry.path.clone(),
                            expected: format!("size {} bytes", file_entry.size_bytes),
                            actual: format!("size {} bytes", content.len()),
                            algorithm: ChecksumAlgorithm::Sha256,
                        });
                    }

                    // Check SHA-256
                    let actual_sha = compute_sha256_hex(content);
                    if !actual_sha.eq_ignore_ascii_case(&file_entry.sha256) {
                        report.mismatched_checksums.push(ChecksumMismatch {
                            path: file_entry.path.clone(),
                            expected: file_entry.sha256.clone(),
                            actual: actual_sha,
                            algorithm: ChecksumAlgorithm::Sha256,
                        });
                    }

                    // Check BLAKE3 if declared
                    if let Some(ref expected_blake3) = file_entry.blake3 {
                        let actual_blake3 = compute_blake3_hex(content);
                        if !actual_blake3.eq_ignore_ascii_case(expected_blake3) {
                            report.mismatched_checksums.push(ChecksumMismatch {
                                path: file_entry.path.clone(),
                                expected: expected_blake3.clone(),
                                actual: actual_blake3,
                                algorithm: ChecksumAlgorithm::Blake3,
                            });
                        }
                    }

                    if !report.verified_files.contains(&file_entry.path) {
                        report.verified_files.push(file_entry.path.clone());
                    }
                }
            }
        }
    }

    // 7. Check for unlisted rogue files
    for path in archive_files.keys() {
        if !declared_paths.contains(path) {
            report.unlisted_files.push(path.clone());
        }
    }

    report.verified_files.sort();
    report.missing_files.sort();
    report.unlisted_files.sort();
    report.path_traversal_violations.sort();
    report
        .mismatched_checksums
        .sort_by(|a, b| a.path.cmp(&b.path));

    report.is_valid = !report.has_errors();
    Ok(report)
}

/// Verifies that a generated Markdown artifact adheres to syntactic and structural standards.
///
/// # Errors
/// Returns [`ExporterError::InvalidDocument`] if Markdown is empty or invalid UTF-8.
pub fn verify_markdown_syntax(content: &str) -> Result<()> {
    if content.trim().is_empty() {
        return Err(ExporterError::InvalidDocument(
            "Markdown content is empty".to_string(),
        ));
    }
    // Verify balanced code fences (backticks)
    let mut in_fence = false;
    let mut current_fence_len = 0;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            let backticks = trimmed.chars().take_while(|&c| c == '`').count();
            if in_fence {
                if backticks >= current_fence_len {
                    in_fence = false;
                    current_fence_len = 0;
                }
            } else {
                in_fence = true;
                current_fence_len = backticks;
            }
        }
    }
    if in_fence {
        return Err(ExporterError::InvalidDocument(
            "Unclosed code fence in Markdown export".to_string(),
        ));
    }
    Ok(())
}

/// Verifies that a generated JSON artifact parses correctly.
///
/// # Errors
/// Returns [`ExporterError::Json`] if JSON syntax is invalid.
pub fn verify_json_syntax(content: &str) -> Result<serde_json::Value> {
    serde_json::from_str(content).map_err(ExporterError::Json)
}

/// Verifies that a generated HTML document is non-empty, contains valid DOCTYPE, and closing tags.
///
/// # Errors
/// Returns [`ExporterError::InvalidDocument`] if HTML structure is malformed.
pub fn verify_html_syntax(content: &str, standalone: bool) -> Result<()> {
    if content.trim().is_empty() {
        return Err(ExporterError::InvalidDocument(
            "HTML content is empty".to_string(),
        ));
    }
    if standalone {
        if !content.contains("<!DOCTYPE html>") {
            return Err(ExporterError::InvalidDocument(
                "Standalone HTML missing <!DOCTYPE html> declaration".to_string(),
            ));
        }
        if !content.contains("</html>") {
            return Err(ExporterError::InvalidDocument(
                "Standalone HTML missing closing </html> tag".to_string(),
            ));
        }
    }
    Ok(())
}

/// Verifies that a generated JSONL artifact has valid JSON on every non-empty line.
///
/// # Errors
/// Returns [`ExporterError::Serialization`] if any line is not valid JSON.
pub fn verify_jsonl_syntax(content: &str) -> Result<usize> {
    let mut line_count = 0;
    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        serde_json::from_str::<serde_json::Value>(trimmed).map_err(|e| {
            ExporterError::InvalidDocument(format!("Invalid JSON on line {}: {e}", idx + 1))
        })?;
        line_count += 1;
    }
    Ok(line_count)
}
