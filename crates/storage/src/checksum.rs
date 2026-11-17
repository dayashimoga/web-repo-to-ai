//! # Artifact Checksum Validation & Integrity Verification
//!
//! Provides cryptographic digestion, dual-hash verification (SHA-256 and BLAKE3),
//! and automated integrity audits for stored artifacts:
//! - [`ArtifactDigests`]: Single-pass and streaming computation of SHA-256 and BLAKE3 digests.
//! - [`ChecksumVerificationStatus`]: Enumeration of verification outcomes (`Verified`, `Mismatch`, `Missing`, `Corrupted`).
//! - [`ArtifactIntegrityReport`]: Comprehensive audit record for an artifact's storage integrity.
//! - [`StorageIntegrityAuditSummary`]: Aggregate metrics across an audited batch of stored artifacts.
//! - [`ArtifactVerifier`]: Coordinator verifying storage files, relational records, and checksum manifests.
//! - [`generate_sha256_manifest`] & [`parse_sha256_manifest`]: Standard `checksums.sha256` manifest generation and validation.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::AsyncReadExt;
use uuid::Uuid;
use web_repo_to_ai_core::error::CoreError;

use crate::backend::DynStorage;
use crate::schema::JobArtifactRow;

/// Supported cryptographic checksum algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChecksumAlgorithm {
    /// NIST Standard SHA-256 digest (256 bits, 64 hex characters).
    Sha256,
    /// High-performance BLAKE3 tree-hashing digest (256 bits, 64 hex characters).
    Blake3,
}

impl ChecksumAlgorithm {
    /// Returns the canonical string representation of the algorithm.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Blake3 => "blake3",
        }
    }

    /// Expected lowercase hexadecimal digest length in characters.
    #[must_use]
    pub const fn hex_len(&self) -> usize {
        64
    }
}

impl fmt::Display for ChecksumAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ChecksumAlgorithm {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "sha256" | "sha-256" => Ok(Self::Sha256),
            "blake3" => Ok(Self::Blake3),
            other => Err(CoreError::Validation {
                field: "checksum_algorithm".to_string(),
                reason: format!(
                    "Unsupported checksum algorithm '{other}' (expected 'sha256' or 'blake3')"
                ),
            }),
        }
    }
}

/// Cryptographic digests computed simultaneously over an artifact payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactDigests {
    /// 64-character lowercase hexadecimal SHA-256 digest.
    pub sha256_hex: String,
    /// 64-character lowercase hexadecimal BLAKE3 digest.
    pub blake3_hex: String,
    /// Exact byte size of the digested payload.
    pub byte_size: u64,
}

impl ArtifactDigests {
    /// Computes both SHA-256 and BLAKE3 digests over an in-memory byte slice in a single pass.
    #[must_use]
    pub fn compute(data: &[u8]) -> Self {
        let mut sha = Sha256::new();
        sha.update(data);
        let sha256_hex = format!("{:x}", sha.finalize());

        let mut blake = blake3::Hasher::new();
        blake.update(data);
        let blake3_hex = blake.finalize().to_hex().to_string();

        Self {
            sha256_hex,
            blake3_hex,
            byte_size: data.len() as u64,
        }
    }

    /// Streams an asynchronous reader through SHA-256 and BLAKE3 hashers in 64 KB chunks.
    ///
    /// Preserves memory bounds for multi-gigabyte artifacts within the 16 GB baseline ceiling.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if streaming read fails.
    pub async fn compute_streaming<R: tokio::io::AsyncRead + Unpin>(
        mut reader: R,
    ) -> Result<Self, CoreError> {
        let mut sha = Sha256::new();
        let mut blake = blake3::Hasher::new();
        let mut buf = [0u8; 65536]; // 64 KB streaming buffer
        let mut total_bytes = 0u64;

        loop {
            let bytes_read = reader.read(&mut buf).await.map_err(|err| {
                CoreError::Storage(format!(
                    "Failed to stream artifact during checksum calculation: {err}"
                ))
            })?;

            if bytes_read == 0 {
                break;
            }

            let slice = &buf[..bytes_read];
            sha.update(slice);
            blake.update(slice);
            total_bytes += bytes_read as u64;
        }

        let sha256_hex = format!("{:x}", sha.finalize());
        let blake3_hex = blake.finalize().to_hex().to_string();

        Ok(Self {
            sha256_hex,
            blake3_hex,
            byte_size: total_bytes,
        })
    }

    /// Validates hexadecimal lengths and character encoding.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Validation`] if either digest is malformed.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.sha256_hex.len() != 64 || !self.sha256_hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CoreError::Validation {
                field: "sha256_hex".to_string(),
                reason: format!(
                    "sha256_hex must be exactly 64 hexadecimal characters (actual: '{}')",
                    self.sha256_hex
                ),
            });
        }

        if self.blake3_hex.len() != 64 || !self.blake3_hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CoreError::Validation {
                field: "blake3_hex".to_string(),
                reason: format!(
                    "blake3_hex must be exactly 64 hexadecimal characters (actual: '{}')",
                    self.blake3_hex
                ),
            });
        }

        Ok(())
    }

    /// Compares the computed SHA-256 digest against an expected hex string (case-insensitively).
    #[must_use]
    pub fn matches_sha256(&self, expected: &str) -> bool {
        self.sha256_hex.eq_ignore_ascii_case(expected.trim())
    }

    /// Compares the computed BLAKE3 digest against an expected hex string (case-insensitively).
    #[must_use]
    pub fn matches_blake3(&self, expected: &str) -> bool {
        self.blake3_hex.eq_ignore_ascii_case(expected.trim())
    }

    /// Compares both SHA-256 and BLAKE3 digests simultaneously.
    #[must_use]
    pub fn matches(&self, expected_sha256: &str, expected_blake3: &str) -> bool {
        self.matches_sha256(expected_sha256) && self.matches_blake3(expected_blake3)
    }
}

/// Verification status for an artifact integrity check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChecksumVerificationStatus {
    /// Physical file exists, byte size matches, and both SHA-256 and BLAKE3 match.
    Verified,
    /// Physical file exists, but one or both hashes (or byte size) do not match expected values.
    Mismatch,
    /// Physical file is missing from the storage backend.
    Missing,
    /// File could not be fully streamed or contains I/O corruption.
    Corrupted,
}

impl ChecksumVerificationStatus {
    /// Returns the canonical status string representation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Mismatch => "mismatch",
            Self::Missing => "missing",
            Self::Corrupted => "corrupted",
        }
    }

    /// Returns `true` if the artifact successfully passed verification.
    #[must_use]
    pub const fn is_verified(&self) -> bool {
        matches!(self, Self::Verified)
    }
}

impl fmt::Display for ChecksumVerificationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Detailed audit report recording the outcome of an artifact integrity check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactIntegrityReport {
    /// Optional relational `job_artifacts.id`.
    pub artifact_id: Option<Uuid>,
    /// Optional parent `jobs.id`.
    pub job_id: Option<Uuid>,
    /// Storage key or path in the active backend.
    pub storage_key: String,
    /// Export format (`markdown`, `zip`, etc.) if known.
    pub format: Option<String>,
    /// Expected SHA-256 digest from metadata.
    pub expected_sha256: String,
    /// Computed SHA-256 digest from storage file, if read succeeded.
    pub actual_sha256: Option<String>,
    /// Expected BLAKE3 digest from metadata.
    pub expected_blake3: String,
    /// Computed BLAKE3 digest from storage file, if read succeeded.
    pub actual_blake3: Option<String>,
    /// Expected file size in bytes from metadata.
    pub expected_bytes: u64,
    /// Actual file size in bytes from storage file, if read succeeded.
    pub actual_bytes: Option<u64>,
    /// Outcome of the checksum and size evaluation.
    pub status: ChecksumVerificationStatus,
    /// Timestamp when verification was conducted.
    pub verified_at: DateTime<Utc>,
    /// Optional error details or mismatch explanations.
    pub error_detail: Option<String>,
}

impl ArtifactIntegrityReport {
    /// Returns `true` if verification succeeded.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.status == ChecksumVerificationStatus::Verified
    }

    /// Returns `true` if a cryptographic hash or size mismatch was discovered.
    #[must_use]
    pub fn has_mismatch(&self) -> bool {
        self.status == ChecksumVerificationStatus::Mismatch
    }

    /// Returns `true` if the physical storage file was missing.
    #[must_use]
    pub fn is_missing(&self) -> bool {
        self.status == ChecksumVerificationStatus::Missing
    }
}

/// Aggregated summary of a storage-wide or job-wide integrity verification run.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageIntegrityAuditSummary {
    /// Total number of artifacts evaluated during this audit.
    pub total_audited: usize,
    /// Count of artifacts passing verification.
    pub verified_count: usize,
    /// Count of artifacts exhibiting hash or size mismatches.
    pub mismatch_count: usize,
    /// Count of artifacts missing from physical storage.
    pub missing_count: usize,
    /// Count of artifacts encountering I/O or read corruption.
    pub corrupted_count: usize,
    /// Total verified payload volume in bytes.
    pub total_bytes_verified: u64,
    /// Individual artifact integrity records.
    pub reports: Vec<ArtifactIntegrityReport>,
}

impl StorageIntegrityAuditSummary {
    /// Returns `true` if all audited artifacts passed without a single failure or mismatch.
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.mismatch_count == 0 && self.missing_count == 0 && self.corrupted_count == 0
    }

    /// Calculates the percentage of artifacts that passed verification (0.0 to 100.0).
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.total_audited == 0 {
            100.0
        } else {
            (self.verified_count as f64 / self.total_audited as f64) * 100.0
        }
    }
}

/// Verification coordinator validating physical files and database metadata.
#[derive(Debug, Clone)]
pub struct ArtifactVerifier {
    storage: DynStorage,
}

impl ArtifactVerifier {
    /// Instantiates an artifact verifier bound to a storage backend.
    #[must_use]
    pub fn new(storage: DynStorage) -> Self {
        Self { storage }
    }

    /// Returns a reference to the active storage backend.
    #[must_use]
    pub fn storage(&self) -> &DynStorage {
        &self.storage
    }

    /// Verifies in-memory byte content against expected SHA-256 and BLAKE3 digests.
    #[must_use]
    pub fn verify_bytes(
        &self,
        data: &[u8],
        expected_sha256: &str,
        expected_blake3: &str,
    ) -> ChecksumVerificationStatus {
        let digests = ArtifactDigests::compute(data);
        if digests.matches(expected_sha256, expected_blake3) {
            ChecksumVerificationStatus::Verified
        } else {
            ChecksumVerificationStatus::Mismatch
        }
    }

    /// Streams and verifies a physical storage file against expected digests and size.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if low-level storage communication fails catastrophically.
    pub async fn verify_storage_file(
        &self,
        key: &str,
        expected_sha256: &str,
        expected_blake3: &str,
        expected_bytes: u64,
    ) -> Result<ArtifactIntegrityReport, CoreError> {
        let now = Utc::now();

        // 1. Probe existence
        let exists = match self.storage.exists(key).await {
            Ok(ex) => ex,
            Err(err) => {
                return Ok(ArtifactIntegrityReport {
                    artifact_id: None,
                    job_id: None,
                    storage_key: key.to_string(),
                    format: None,
                    expected_sha256: expected_sha256.to_string(),
                    actual_sha256: None,
                    expected_blake3: expected_blake3.to_string(),
                    actual_blake3: None,
                    expected_bytes,
                    actual_bytes: None,
                    status: ChecksumVerificationStatus::Missing,
                    verified_at: now,
                    error_detail: Some(format!("Failed to probe artifact existence: {err}")),
                });
            }
        };

        if !exists {
            return Ok(ArtifactIntegrityReport {
                artifact_id: None,
                job_id: None,
                storage_key: key.to_string(),
                format: None,
                expected_sha256: expected_sha256.to_string(),
                actual_sha256: None,
                expected_blake3: expected_blake3.to_string(),
                actual_blake3: None,
                expected_bytes,
                actual_bytes: None,
                status: ChecksumVerificationStatus::Missing,
                verified_at: now,
                error_detail: Some(format!("Artifact file '{key}' not found in storage")),
            });
        }

        // 2. Open read stream
        let read_stream = match self.storage.open_read(key).await {
            Ok(stream) => stream,
            Err(err) => {
                return Ok(ArtifactIntegrityReport {
                    artifact_id: None,
                    job_id: None,
                    storage_key: key.to_string(),
                    format: None,
                    expected_sha256: expected_sha256.to_string(),
                    actual_sha256: None,
                    expected_blake3: expected_blake3.to_string(),
                    actual_blake3: None,
                    expected_bytes,
                    actual_bytes: None,
                    status: ChecksumVerificationStatus::Corrupted,
                    verified_at: now,
                    error_detail: Some(format!("Failed to open artifact read stream: {err}")),
                });
            }
        };

        // 3. Compute streaming digests
        let digests = match ArtifactDigests::compute_streaming(read_stream).await {
            Ok(d) => d,
            Err(err) => {
                return Ok(ArtifactIntegrityReport {
                    artifact_id: None,
                    job_id: None,
                    storage_key: key.to_string(),
                    format: None,
                    expected_sha256: expected_sha256.to_string(),
                    actual_sha256: None,
                    expected_blake3: expected_blake3.to_string(),
                    actual_blake3: None,
                    expected_bytes,
                    actual_bytes: None,
                    status: ChecksumVerificationStatus::Corrupted,
                    verified_at: now,
                    error_detail: Some(format!("Streaming digest computation failed: {err}")),
                });
            }
        };

        // 4. Invariant checks
        let sha_matches = digests.matches_sha256(expected_sha256);
        let blake_matches = digests.matches_blake3(expected_blake3);
        let size_matches = digests.byte_size == expected_bytes;

        let (status, error_detail) = if sha_matches && blake_matches && size_matches {
            (ChecksumVerificationStatus::Verified, None)
        } else {
            let mut diffs = Vec::new();
            if !sha_matches {
                diffs.push(format!(
                    "SHA-256 mismatch (expected: {expected_sha256}, actual: {})",
                    digests.sha256_hex
                ));
            }
            if !blake_matches {
                diffs.push(format!(
                    "BLAKE3 mismatch (expected: {expected_blake3}, actual: {})",
                    digests.blake3_hex
                ));
            }
            if !size_matches {
                diffs.push(format!(
                    "Byte size mismatch (expected: {expected_bytes}, actual: {})",
                    digests.byte_size
                ));
            }
            (ChecksumVerificationStatus::Mismatch, Some(diffs.join("; ")))
        };

        Ok(ArtifactIntegrityReport {
            artifact_id: None,
            job_id: None,
            storage_key: key.to_string(),
            format: None,
            expected_sha256: expected_sha256.to_string(),
            actual_sha256: Some(digests.sha256_hex),
            expected_blake3: expected_blake3.to_string(),
            actual_blake3: Some(digests.blake3_hex),
            expected_bytes,
            actual_bytes: Some(digests.byte_size),
            status,
            verified_at: now,
            error_detail,
        })
    }

    /// Verifies physical storage consistency against a relational `JobArtifactRow`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if storage access fails.
    pub async fn verify_artifact_record(
        &self,
        artifact: &JobArtifactRow,
    ) -> Result<ArtifactIntegrityReport, CoreError> {
        let mut report = self
            .verify_storage_file(
                &artifact.storage_key,
                &artifact.sha256_hex,
                &artifact.blake3_hex,
                artifact.byte_size as u64,
            )
            .await?;

        report.artifact_id = Some(artifact.id);
        report.job_id = Some(artifact.job_id);
        report.format = Some(artifact.format.clone());
        Ok(report)
    }

    /// Fetches a relational artifact record by ID and verifies physical storage integrity.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::NotFound`] if the artifact record does not exist.
    /// Returns [`CoreError::Storage`] if database or storage operations fail.
    pub async fn verify_job_artifact(
        &self,
        pool: &sqlx::PgPool,
        artifact_id: Uuid,
    ) -> Result<ArtifactIntegrityReport, CoreError> {
        let artifact =
            match crate::retention::ArtifactRepository::get_artifact(pool, artifact_id).await? {
                Some(a) => a,
                None => {
                    return Err(CoreError::NotFound {
                        resource_type: "job_artifact",
                        id: artifact_id.to_string(),
                    });
                }
            };

        self.verify_artifact_record(&artifact).await
    }

    /// Audits all artifacts registered under a specific parent job.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database or storage queries fail.
    pub async fn verify_all_job_artifacts(
        &self,
        pool: &sqlx::PgPool,
        job_id: Uuid,
    ) -> Result<Vec<ArtifactIntegrityReport>, CoreError> {
        let artifacts =
            crate::retention::ArtifactRepository::list_artifacts_for_job(pool, job_id).await?;
        let mut reports = Vec::with_capacity(artifacts.len());

        for artifact in artifacts {
            reports.push(self.verify_artifact_record(&artifact).await?);
        }

        Ok(reports)
    }

    /// Executes a system-wide audit of stored artifacts up to a configured batch limit.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database queries fail.
    pub async fn audit_storage_integrity(
        &self,
        pool: &sqlx::PgPool,
        limit: usize,
    ) -> Result<StorageIntegrityAuditSummary, CoreError> {
        let artifacts =
            crate::retention::ArtifactRepository::find_oldest_artifacts(pool, limit as i64).await?;
        let mut summary = StorageIntegrityAuditSummary::default();

        for artifact in artifacts {
            let report = self.verify_artifact_record(&artifact).await?;
            match report.status {
                ChecksumVerificationStatus::Verified => {
                    summary.verified_count += 1;
                    summary.total_bytes_verified += report.actual_bytes.unwrap_or(0);
                }
                ChecksumVerificationStatus::Mismatch => {
                    summary.mismatch_count += 1;
                }
                ChecksumVerificationStatus::Missing => {
                    summary.missing_count += 1;
                }
                ChecksumVerificationStatus::Corrupted => {
                    summary.corrupted_count += 1;
                }
            }
            summary.total_audited += 1;
            summary.reports.push(report);
        }

        Ok(summary)
    }
}

/// Generates a standard BSD/GNU-style `checksums.sha256` manifest string.
///
/// Format: `<sha256_hex>  <filename>\n` (two spaces between hash and filename).
#[must_use]
pub fn generate_sha256_manifest(entries: &[(String, String)]) -> String {
    let mut manifest = String::new();
    for (sha256, filename) in entries {
        manifest.push_str(&format!("{}  {}\n", sha256.to_ascii_lowercase(), filename));
    }
    manifest
}

/// Parses a standard `checksums.sha256` manifest file into `(sha256, filename)` pairs.
///
/// Supports standard two-space BSD/GNU coreutils format and ignores comment and blank lines.
///
/// # Errors
///
/// Returns [`CoreError::Validation`] if lines are malformed or SHA-256 digests are invalid.
pub fn parse_sha256_manifest(manifest: &str) -> Result<Vec<(String, String)>, CoreError> {
    let mut results = Vec::new();

    for (line_idx, line) in manifest.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(CoreError::Validation {
                field: "manifest".to_string(),
                reason: format!(
                    "Line {}: invalid checksum manifest format (expected '<hash>  <filename>')",
                    line_idx + 1
                ),
            });
        }

        let sha256 = parts[0].to_ascii_lowercase();
        if sha256.len() != 64 || !sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CoreError::Validation {
                field: "manifest".to_string(),
                reason: format!(
                    "Line {}: invalid SHA-256 digest length or character content: '{}'",
                    line_idx + 1,
                    sha256
                ),
            });
        }

        let filename = parts[1..].join(" ");
        let clean_filename = filename.strip_prefix('*').unwrap_or(&filename).to_string();

        results.push((sha256, clean_filename));
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{InMemoryStorage, StorageBackend};
    use crate::schema::{ArtifactFormat, JobArtifactRow};
    use std::sync::Arc;

    #[test]
    fn test_artifact_digests_empty_payload() {
        let empty_data = b"";
        let digests = ArtifactDigests::compute(empty_data);

        // Standard test vectors for empty string
        assert_eq!(
            digests.sha256_hex,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            digests.blake3_hex,
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );
        assert_eq!(digests.byte_size, 0);
        assert!(digests.validate().is_ok());
    }

    #[test]
    fn test_artifact_digests_matching_and_validation() {
        let payload = b"Hello, web-repo-to-ai!";
        let digests = ArtifactDigests::compute(payload);

        assert!(digests.matches_sha256(&digests.sha256_hex));
        assert!(digests.matches_blake3(&digests.blake3_hex));
        assert!(digests.matches(&digests.sha256_hex, &digests.blake3_hex));

        // Case-insensitivity
        let upper_sha = digests.sha256_hex.to_ascii_uppercase();
        let upper_blake = digests.blake3_hex.to_ascii_uppercase();
        assert!(digests.matches_sha256(&upper_sha));
        assert!(digests.matches_blake3(&upper_blake));
        assert!(digests.matches(&upper_sha, &upper_blake));

        // Mismatched
        assert!(!digests
            .matches_sha256("0000000000000000000000000000000000000000000000000000000000000000"));
        assert!(!digests
            .matches_blake3("0000000000000000000000000000000000000000000000000000000000000000"));

        // Malformed lengths
        let invalid_sha = ArtifactDigests {
            sha256_hex: "short".to_string(),
            blake3_hex: digests.blake3_hex.clone(),
            byte_size: 10,
        };
        assert!(invalid_sha.validate().is_err());
    }

    #[tokio::test]
    async fn test_streaming_digests_matches_slice_compute() {
        let payload =
            b"Deterministic payload streaming test across 64KB boundary buffers.".repeat(1000);
        let slice_digests = ArtifactDigests::compute(&payload);

        let reader = std::io::Cursor::new(payload);
        let streaming_digests = ArtifactDigests::compute_streaming(reader).await.unwrap();

        assert_eq!(slice_digests.sha256_hex, streaming_digests.sha256_hex);
        assert_eq!(slice_digests.blake3_hex, streaming_digests.blake3_hex);
        assert_eq!(slice_digests.byte_size, streaming_digests.byte_size);
    }

    #[tokio::test]
    async fn test_artifact_verifier_outcomes() {
        let storage = Arc::new(InMemoryStorage::new());
        let verifier = ArtifactVerifier::new(storage.clone());

        let payload = b"# Generated Context Bundle\nSection content.";
        let digests = ArtifactDigests::compute(payload);
        let key = "jobs/test_job/artifact.md";

        // 1. Missing file check
        let missing_report = verifier
            .verify_storage_file(
                key,
                &digests.sha256_hex,
                &digests.blake3_hex,
                digests.byte_size,
            )
            .await
            .unwrap();
        assert_eq!(missing_report.status, ChecksumVerificationStatus::Missing);
        assert!(missing_report.is_missing());
        assert!(!missing_report.is_valid());

        // 2. Put file and verify success
        storage.put(key, payload).await.unwrap();

        let verified_report = verifier
            .verify_storage_file(
                key,
                &digests.sha256_hex,
                &digests.blake3_hex,
                digests.byte_size,
            )
            .await
            .unwrap();
        assert_eq!(verified_report.status, ChecksumVerificationStatus::Verified);
        assert!(verified_report.is_valid());

        // 3. Hash mismatch check
        let wrong_sha = "0000000000000000000000000000000000000000000000000000000000000000";
        let mismatch_report = verifier
            .verify_storage_file(key, wrong_sha, &digests.blake3_hex, digests.byte_size)
            .await
            .unwrap();
        assert_eq!(mismatch_report.status, ChecksumVerificationStatus::Mismatch);
        assert!(mismatch_report.has_mismatch());
        assert!(!mismatch_report.is_valid());

        // 4. Byte size mismatch check
        let size_mismatch_report = verifier
            .verify_storage_file(
                key,
                &digests.sha256_hex,
                &digests.blake3_hex,
                digests.byte_size + 10,
            )
            .await
            .unwrap();
        assert_eq!(
            size_mismatch_report.status,
            ChecksumVerificationStatus::Mismatch
        );
    }

    #[tokio::test]
    async fn test_verify_artifact_record() {
        let storage = Arc::new(InMemoryStorage::new());
        let verifier = ArtifactVerifier::new(storage.clone());

        let job_id = Uuid::now_v7();
        let payload = b"{\"model\": \"gpt-4o\", \"tokens\": 1024}";
        let digests = ArtifactDigests::compute(payload);
        let key = format!("jobs/{job_id}/artifact.json");
        storage.put(&key, payload).await.unwrap();

        let artifact_row = JobArtifactRow::new(
            job_id,
            ArtifactFormat::Json,
            &key,
            digests.byte_size as i64,
            &digests.sha256_hex,
            &digests.blake3_hex,
            chrono::Duration::days(7),
        );

        let report = verifier
            .verify_artifact_record(&artifact_row)
            .await
            .unwrap();
        assert_eq!(report.status, ChecksumVerificationStatus::Verified);
        assert_eq!(report.job_id, Some(job_id));
        assert_eq!(report.format, Some("json".to_string()));
        assert_eq!(report.actual_bytes, Some(digests.byte_size));
    }

    #[test]
    fn test_manifest_generation_and_parsing_roundtrip() {
        let entries = vec![
            (
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
                "manifest.json".to_string(),
            ),
            (
                "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262".to_string(),
                "artifact.md".to_string(),
            ),
            (
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_string(),
                "subfolder/nested file with spaces.txt".to_string(),
            ),
        ];

        let manifest_content = generate_sha256_manifest(&entries);
        assert!(manifest_content.contains("manifest.json"));
        assert!(manifest_content.contains("artifact.md"));

        let parsed = parse_sha256_manifest(&manifest_content).unwrap();
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].0, entries[0].0);
        assert_eq!(parsed[0].1, "manifest.json");
        assert_eq!(parsed[1].1, "artifact.md");
        assert_eq!(parsed[2].1, "subfolder/nested file with spaces.txt");

        // Comments and empty lines ignored
        let manifest_with_comments = format!(
            "# Checksums manifest\n\n{}\n# End of manifest\n",
            manifest_content
        );
        let parsed_comments = parse_sha256_manifest(&manifest_with_comments).unwrap();
        assert_eq!(parsed_comments.len(), 3);

        // Malformed line rejected
        assert!(parse_sha256_manifest("short_hash  file.txt").is_err());
        assert!(parse_sha256_manifest("only_one_column").is_err());
    }

    #[test]
    fn test_storage_integrity_audit_summary_health() {
        let mut summary = StorageIntegrityAuditSummary::default();
        assert!(summary.is_healthy());
        assert_eq!(summary.success_rate(), 100.0);

        summary.total_audited = 10;
        summary.verified_count = 10;
        assert!(summary.is_healthy());
        assert_eq!(summary.success_rate(), 100.0);

        summary.mismatch_count = 1;
        summary.verified_count = 9;
        assert!(!summary.is_healthy());
        assert_eq!(summary.success_rate(), 90.0);
    }
}
