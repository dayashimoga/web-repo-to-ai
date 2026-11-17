//! # Cryptographic Checksum Engine (`crates/exporters/src/checksum.rs`)
//!
//! Provides standard cryptographic checksum generation, BSD/GNU coreutils-compatible
//! formatting, parsing, and integrity verification.
//!
//! Compatible with standard shell utilities:
//! - `sha256sum -c checksums.sha256`
//! - `b3sum --check checksums.blake3`

use std::fmt;
use std::io::Write;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{ExporterError, Result};

/// Supported cryptographic digest algorithms for artifact verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChecksumAlgorithm {
    /// SHA-256 (NIST FIPS 180-4, 256-bit).
    #[default]
    Sha256,
    /// BLAKE3 (tree-hashing cryptographic digest, 256-bit).
    Blake3,
}

impl ChecksumAlgorithm {
    /// Returns the canonical algorithm identifier name.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Blake3 => "blake3",
        }
    }

    /// Computes the hexadecimal digest for the given byte slice using this algorithm.
    #[must_use]
    pub fn compute_hex(&self, bytes: &[u8]) -> String {
        match self {
            Self::Sha256 => compute_sha256_hex(bytes),
            Self::Blake3 => compute_blake3_hex(bytes),
        }
    }
}

impl fmt::Display for ChecksumAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Computes the 64-character lowercase hexadecimal SHA-256 digest of a byte slice.
#[must_use]
pub fn compute_sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Computes the 64-character lowercase hexadecimal BLAKE3 digest of a byte slice.
#[must_use]
pub fn compute_blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Individual checksum entry mapping a relative archive path to its cryptographic hash.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChecksumEntry {
    /// Relative path to the target file.
    pub path: String,
    /// 64-character lowercase hexadecimal digest string.
    pub hash: String,
    /// Digest algorithm used.
    pub algorithm: ChecksumAlgorithm,
}

impl ChecksumEntry {
    /// Creates a new checksum entry.
    #[must_use]
    pub fn new(
        path: impl Into<String>,
        hash: impl Into<String>,
        algorithm: ChecksumAlgorithm,
    ) -> Self {
        Self {
            path: path.into(),
            hash: hash.into().to_ascii_lowercase(),
            algorithm,
        }
    }

    /// Creates a new SHA-256 checksum entry.
    #[must_use]
    pub fn sha256(path: impl Into<String>, hash: impl Into<String>) -> Self {
        Self::new(path, hash, ChecksumAlgorithm::Sha256)
    }

    /// Creates a new BLAKE3 checksum entry.
    #[must_use]
    pub fn blake3(path: impl Into<String>, hash: impl Into<String>) -> Self {
        Self::new(path, hash, ChecksumAlgorithm::Blake3)
    }

    /// Creates an entry by hashing content bytes with SHA-256.
    #[must_use]
    pub fn from_sha256_bytes(path: impl Into<String>, bytes: &[u8]) -> Self {
        Self::sha256(path, compute_sha256_hex(bytes))
    }

    /// Creates an entry by hashing content bytes with BLAKE3.
    #[must_use]
    pub fn from_blake3_bytes(path: impl Into<String>, bytes: &[u8]) -> Self {
        Self::blake3(path, compute_blake3_hex(bytes))
    }

    /// Formats this entry according to BSD/GNU coreutils standard (`<hash>  <path>\n`).
    #[must_use]
    pub fn to_coreutils_line(&self) -> String {
        format!("{}  {}\n", self.hash, self.path)
    }
}

/// Generates a standard BSD/GNU coreutils compatible checksums file string.
///
/// Output format: `<hash>  <relative_path>\n` (two spaces between digest and path).
#[must_use]
pub fn generate_checksums_manifest(entries: &[ChecksumEntry]) -> String {
    let mut out = String::with_capacity(entries.len() * 80);
    for entry in entries {
        out.push_str(&entry.to_coreutils_line());
    }
    out
}

/// Streams a standard BSD/GNU coreutils checksums manifest directly to a writer.
///
/// # Errors
/// Returns [`ExporterError::Io`] if writing to destination fails.
pub fn export_checksums_to_writer<W: Write>(
    entries: &[ChecksumEntry],
    mut writer: W,
) -> Result<usize> {
    let mut written = 0;
    for entry in entries {
        let line = entry.to_coreutils_line();
        let bytes = line.as_bytes();
        writer.write_all(bytes)?;
        written += bytes.len();
    }
    Ok(written)
}

/// Parses a BSD/GNU coreutils-compatible checksums file (`checksums.sha256` or `checksums.blake3`).
///
/// Handles standard 2-space separators, binary mode `*` indicators, blank lines, and `#` comments.
///
/// # Errors
/// Returns [`ExporterError::ChecksumValidation`] if any non-empty line cannot be parsed
/// or contains an invalid hexadecimal hash.
pub fn parse_checksums_manifest(
    content: &str,
    algorithm: ChecksumAlgorithm,
) -> Result<Vec<ChecksumEntry>> {
    let mut entries = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Split hash and filename: coreutils format uses 2 spaces, e.g. "hash  file" or "hash *file"
        let (hash_part, file_part) = if let Some(pos) = trimmed.find("  ") {
            (&trimmed[..pos], trimmed[pos + 2..].trim())
        } else if let Some(pos) = trimmed.find(" *") {
            (&trimmed[..pos], trimmed[pos + 2..].trim())
        } else {
            // Fallback: split on first whitespace
            let mut parts = trimmed.splitn(2, char::is_whitespace);
            let h = parts.next().unwrap_or("");
            let f = parts.next().unwrap_or("").trim();
            (h, f)
        };

        let file_path = file_part.strip_prefix('*').unwrap_or(file_part).trim();
        if file_path.is_empty() {
            return Err(ExporterError::ChecksumValidation(format!(
                "Line {line_num}: missing filename in checksum line '{line}'"
            )));
        }

        let hash_clean = hash_part.trim().to_ascii_lowercase();
        if hash_clean.len() != 64 || !hash_clean.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ExporterError::ChecksumValidation(format!(
                "Line {line_num}: invalid {algorithm} digest '{hash_clean}' (expected 64 hex characters)"
            )));
        }

        entries.push(ChecksumEntry::new(file_path, hash_clean, algorithm));
    }

    Ok(entries)
}

/// Diagnostic details when a file's cryptographic hash does not match expectations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChecksumMismatch {
    /// Relative path of the mismatched file.
    pub path: String,
    /// Expected cryptographic hash from manifest.
    pub expected: String,
    /// Actual cryptographic hash computed from content.
    pub actual: String,
    /// Digest algorithm evaluated.
    pub algorithm: ChecksumAlgorithm,
}

/// Comprehensive report detailing verification of a set of files against a checksum manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ChecksumVerificationReport {
    /// Files whose computed digest matched expected hash exactly.
    pub matched_files: Vec<String>,
    /// Files that exist but whose digest did not match expected hash.
    pub mismatched_files: Vec<ChecksumMismatch>,
    /// Files listed in manifest that were not present.
    pub missing_files: Vec<String>,
    /// Total entries evaluated.
    pub total_evaluated: usize,
}

impl ChecksumVerificationReport {
    /// Returns `true` if all files matched and there were no mismatches, no missing files,
    /// and at least one file was checked.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.mismatched_files.is_empty()
            && self.missing_files.is_empty()
            && self.total_evaluated > 0
            && self.matched_files.len() == self.total_evaluated
    }
}

/// Verifies a collection of [`ChecksumEntry`] items against content supplied by a byte reader.
///
/// # Arguments
/// - `entries`: Expected checksum entries.
/// - `reader`: Closure or function taking a relative path and returning `Some(&[u8])` if present,
///   or `None` if missing.
#[must_use]
pub fn verify_checksums<'a, F>(
    entries: &[ChecksumEntry],
    mut reader: F,
) -> ChecksumVerificationReport
where
    F: FnMut(&str) -> Option<&'a [u8]>,
{
    let mut report = ChecksumVerificationReport {
        total_evaluated: entries.len(),
        ..Default::default()
    };

    for entry in entries {
        match reader(&entry.path) {
            None => {
                report.missing_files.push(entry.path.clone());
            }
            Some(bytes) => {
                let actual_hash = entry.algorithm.compute_hex(bytes);
                if actual_hash.eq_ignore_ascii_case(&entry.hash) {
                    report.matched_files.push(entry.path.clone());
                } else {
                    report.mismatched_files.push(ChecksumMismatch {
                        path: entry.path.clone(),
                        expected: entry.hash.clone(),
                        actual: actual_hash,
                        algorithm: entry.algorithm,
                    });
                }
            }
        }
    }

    report
}
