//! # Cryptographic Content Hash Validation & Integrity Verification
//!
//! Provides multi-algorithm cryptographic checksum verification (Git Object SHA-1, BLAKE3, and SHA-256)
//! validating extracted repository artifact bodies against calculated hashes and source Git tree blob digests.
//!
//! Enforces the integrity invariant:
//! Every extracted file content body must identically match:
//! 1. Git Object SHA-1 (`blob <length>\0<content>`) matching the Git tree blob SHA.
//! 2. Primary BLAKE3 hash matching the stored 64-character lowercase hex digest.
//! 3. Secondary SHA-256 hash matching the stored 64-character lowercase hex digest.
//!
//! Surfaces any corruptions, in-transit network truncations, payload tampering, or malformed hashes.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{CdmSection, CdmSectionBuilder, SectionKind, SectionTokenMetrics};

use crate::config_test::ConfigTestExtractionReport;
use crate::dependency::DependencyExtractionReport;
use crate::documentation::DocExtractionReport;
use crate::manifest::{FileExtractionStatus, RepoFileManifest};
use crate::source::{
    compute_blake3_hash, compute_git_blob_sha, compute_sha256_hash, SourceExtractionReport,
};

/// Supported cryptographic hash algorithms for content integrity verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HashAlgorithm {
    /// Git Object SHA-1 format: `SHA1("blob " + length + "\0" + content)`. 40 lowercase hex characters.
    GitBlobSha1,
    /// BLAKE3 256-bit cryptographic digest. 64 lowercase hex characters.
    Blake3,
    /// SHA-256 standard cryptographic digest. 64 lowercase hex characters.
    Sha256,
}

impl HashAlgorithm {
    /// Returns a static display label for this algorithm.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GitBlobSha1 => "git_blob_sha1",
            Self::Blake3 => "blake3",
            Self::Sha256 => "sha256",
        }
    }

    /// Returns the expected character length of the hexadecimal digest.
    #[must_use]
    pub const fn expected_hex_len(self) -> usize {
        match self {
            Self::GitBlobSha1 => 40,
            Self::Blake3 | Self::Sha256 => 64,
        }
    }

    /// Validates whether a given string is a syntactically valid hex digest for this algorithm.
    #[must_use]
    pub fn is_valid_hex_format(&self, hash_str: &str) -> bool {
        hash_str.len() == self.expected_hex_len()
            && hash_str
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    }
}

/// Quantitative check metric tracking verification passes and failures for a specific algorithm.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashCheckMetric {
    /// Total number of checks evaluated for this algorithm.
    pub checked: usize,
    /// Number of checks that matched successfully.
    pub passed: usize,
    /// Number of checks where calculated hash differed from expected.
    pub failed: usize,
}

impl HashCheckMetric {
    /// Records a successful verification match.
    pub fn record_pass(&mut self) {
        self.checked += 1;
        self.passed += 1;
    }

    /// Records a failed verification match.
    pub fn record_fail(&mut self) {
        self.checked += 1;
        self.failed += 1;
    }

    /// Returns `true` if all performed checks passed with zero failures.
    #[must_use]
    pub fn is_perfect(&self) -> bool {
        self.failed == 0
    }
}

/// A specific detected checksum or content hash validation failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashValidationFailure {
    /// Normalized relative repository path.
    pub path: String,
    /// Hash algorithm that failed verification.
    pub algorithm: HashAlgorithm,
    /// Expected hash value recorded in discovery or manifest.
    pub expected_hash: String,
    /// Actual hash value calculated over the extracted payload bytes.
    pub calculated_hash: String,
    /// Contextual explanation or error detail.
    pub context: String,
}

impl HashValidationFailure {
    /// Creates a new hash validation failure record.
    #[must_use]
    pub fn new(
        path: impl Into<String>,
        algorithm: HashAlgorithm,
        expected_hash: impl Into<String>,
        calculated_hash: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            algorithm,
            expected_hash: expected_hash.into(),
            calculated_hash: calculated_hash.into(),
            context: context.into(),
        }
    }
}

/// Configuration settings controlling cryptographic hash verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashValidationConfig {
    /// Whether to verify Git Object SHA-1 digests against Git tree blob SHAs.
    pub verify_git_sha: bool,
    /// Whether to verify primary BLAKE3 content hashes.
    pub verify_blake3: bool,
    /// Whether to verify secondary SHA-256 content hashes.
    pub verify_sha256: bool,
    /// Whether to abort validation immediately upon the first failure.
    pub fail_fast: bool,
}

impl Default for HashValidationConfig {
    fn default() -> Self {
        Self {
            verify_git_sha: true,
            verify_blake3: true,
            verify_sha256: true,
            fail_fast: false,
        }
    }
}

impl HashValidationConfig {
    /// Returns a strict validation configuration with all algorithms enabled.
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            verify_git_sha: true,
            verify_blake3: true,
            verify_sha256: true,
            fail_fast: false,
        }
    }

    /// Returns a configuration verifying Git object SHAs only.
    #[must_use]
    pub const fn git_sha_only() -> Self {
        Self {
            verify_git_sha: true,
            verify_blake3: false,
            verify_sha256: false,
            fail_fast: false,
        }
    }

    /// Sets whether to stop on the first failure.
    #[must_use]
    pub const fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }
}

/// Consolidated audit report summarizing cryptographic content hash verification across repository files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashValidationReport {
    /// Target repository identifier (e.g. `owner/repo`).
    pub target_repo: String,
    /// Total number of distinct file content items audited.
    pub total_files_checked: usize,
    /// Metrics for Git Object SHA-1 verification checks.
    pub git_sha_metrics: HashCheckMetric,
    /// Metrics for BLAKE3 verification checks.
    pub blake3_metrics: HashCheckMetric,
    /// Metrics for SHA-256 verification checks.
    pub sha256_metrics: HashCheckMetric,
    /// Total count of hash mismatches and malformed digest failures.
    pub total_failures: usize,
    /// List of all individual verification failures detected.
    pub failures: Vec<HashValidationFailure>,
}

impl HashValidationReport {
    /// Creates a new empty hash validation report for a target repository.
    #[must_use]
    pub fn new(target_repo: impl Into<String>) -> Self {
        Self {
            target_repo: target_repo.into(),
            total_files_checked: 0,
            git_sha_metrics: HashCheckMetric::default(),
            blake3_metrics: HashCheckMetric::default(),
            sha256_metrics: HashCheckMetric::default(),
            total_failures: 0,
            failures: Vec::new(),
        }
    }

    /// Returns `true` if all cryptographic checksums were verified with zero failures.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.total_failures == 0
    }

    /// Records a failure and increments the total failures counter.
    pub fn add_failure(&mut self, failure: HashValidationFailure) {
        self.total_failures += 1;
        self.failures.push(failure);
    }

    /// Formats this validation report into a structured GitHub-Flavored Markdown summary.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!(
            "# Cryptographic Content Hash Validation: {}\n\n",
            self.target_repo
        ));

        let status_badge = if self.is_valid() {
            "PASS: 100% Verified"
        } else {
            "FAIL: Checksum Mismatches Detected"
        };

        md.push_str(&format!("- **Integrity Status**: `{status_badge}`\n"));
        md.push_str(&format!(
            "- **Files Checked**: {}\n",
            self.total_files_checked
        ));
        md.push_str(&format!(
            "- **Total Failures**: {}\n\n",
            self.total_failures
        ));

        // Metric Breakdown Table
        md.push_str("### Checksum Algorithm Breakdown\n\n");
        md.push_str("| Algorithm | Checked | Passed | Failed | Status |\n");
        md.push_str("| :--- | :---: | :---: | :---: | :--- |\n");
        md.push_str(&format!(
            "| **Git Blob SHA-1** | {} | {} | {} | {} |\n",
            self.git_sha_metrics.checked,
            self.git_sha_metrics.passed,
            self.git_sha_metrics.failed,
            if self.git_sha_metrics.is_perfect() {
                "PASS"
            } else {
                "FAIL"
            }
        ));
        md.push_str(&format!(
            "| **BLAKE3 (Primary)** | {} | {} | {} | {} |\n",
            self.blake3_metrics.checked,
            self.blake3_metrics.passed,
            self.blake3_metrics.failed,
            if self.blake3_metrics.is_perfect() {
                "PASS"
            } else {
                "FAIL"
            }
        ));
        md.push_str(&format!(
            "| **SHA-256 (Secondary)** | {} | {} | {} | {} |\n",
            self.sha256_metrics.checked,
            self.sha256_metrics.passed,
            self.sha256_metrics.failed,
            if self.sha256_metrics.is_perfect() {
                "PASS"
            } else {
                "FAIL"
            }
        ));
        md.push('\n');

        // Failures Table
        if self.failures.is_empty() {
            md.push_str("### Verification Failures Log\n\n");
            md.push_str("*Zero cryptographic hash failures detected. All extracted files identically match their expected Git tree digests and content hashes.*\n");
        } else {
            md.push_str(&format!(
                "### Verification Failures Log ({} failures)\n\n",
                self.failures.len()
            ));
            md.push_str("| Path | Algorithm | Expected Hash | Calculated Hash | Detail |\n");
            md.push_str("| :--- | :---: | :--- | :--- | :--- |\n");
            for f in &self.failures {
                md.push_str(&format!(
                    "| `{}` | `{}` | `{}` | `{}` | {} |\n",
                    f.path,
                    f.algorithm.as_str(),
                    f.expected_hash,
                    f.calculated_hash,
                    f.context
                ));
            }
            md.push('\n');
        }

        md
    }

    /// Serializes this report into pretty formatted JSON.
    ///
    /// # Errors
    /// Returns `serde_json::Error` if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serializes this report into a compact single-line JSON string.
    ///
    /// # Errors
    /// Returns `serde_json::Error` if serialization fails.
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Converts this report into a Canonical Context Document Model [`CdmSection`].
    #[must_use]
    pub fn to_cdm_section(&self, doc_canonical_url: impl Into<String>) -> CdmSection {
        let markdown = self.to_markdown();
        let content_hash = compute_blake3_hash(markdown.as_bytes());
        let token_metrics = SectionTokenMetrics::compute_baseline(&markdown);

        let mut section = CdmSectionBuilder::new(
            "sec-content-hash-validation",
            doc_canonical_url,
            SectionKind::Manifest,
            &markdown,
        )
        .title("Cryptographic Content Hash Validation Audit")
        .original_path("HASH_VALIDATION.md")
        .content_hash(content_hash)
        .token_metrics(token_metrics)
        .build();

        section.attributes.insert(
            "manifest_role".to_string(),
            "content_hash_validation".to_string(),
        );
        section
            .attributes
            .insert("is_valid".to_string(), self.is_valid().to_string());
        section.attributes.insert(
            "total_files_checked".to_string(),
            self.total_files_checked.to_string(),
        );
        section.attributes.insert(
            "total_failures".to_string(),
            self.total_failures.to_string(),
        );
        section.attributes.insert(
            "git_sha_passed".to_string(),
            self.git_sha_metrics.passed.to_string(),
        );
        section.attributes.insert(
            "git_sha_failed".to_string(),
            self.git_sha_metrics.failed.to_string(),
        );
        section.attributes.insert(
            "blake3_passed".to_string(),
            self.blake3_metrics.passed.to_string(),
        );
        section.attributes.insert(
            "blake3_failed".to_string(),
            self.blake3_metrics.failed.to_string(),
        );
        section.attributes.insert(
            "sha256_passed".to_string(),
            self.sha256_metrics.passed.to_string(),
        );
        section.attributes.insert(
            "sha256_failed".to_string(),
            self.sha256_metrics.failed.to_string(),
        );

        section
    }
}

/// Cryptographic content hash validator evaluating files, manifests, reports, and CDM sections.
pub struct ContentHashValidator;

impl ContentHashValidator {
    /// Validates a single file content buffer against expected Git SHA-1, BLAKE3, and SHA-256 hashes.
    pub fn validate_file_bytes(
        path: &str,
        content: &[u8],
        expected_git_sha: Option<&str>,
        expected_blake3: Option<&str>,
        expected_sha256: Option<&str>,
        config: &HashValidationConfig,
        report: &mut HashValidationReport,
    ) {
        report.total_files_checked += 1;

        // 1. Git Object SHA-1 check
        if config.verify_git_sha {
            if let Some(exp_sha) = expected_git_sha {
                if !exp_sha.is_empty() {
                    let actual_git_sha = compute_git_blob_sha(content);
                    if !HashAlgorithm::GitBlobSha1.is_valid_hex_format(exp_sha) {
                        report.git_sha_metrics.record_fail();
                        report.add_failure(HashValidationFailure::new(
                            path,
                            HashAlgorithm::GitBlobSha1,
                            exp_sha,
                            actual_git_sha,
                            "Expected Git SHA-1 is not a valid 40-character lowercase hex digest",
                        ));
                    } else if actual_git_sha == exp_sha {
                        report.git_sha_metrics.record_pass();
                    } else {
                        report.git_sha_metrics.record_fail();
                        report.add_failure(HashValidationFailure::new(
                            path,
                            HashAlgorithm::GitBlobSha1,
                            exp_sha,
                            actual_git_sha,
                            "Git Object SHA-1 mismatch: calculated blob SHA does not match expected SHA",
                        ));
                        if config.fail_fast {
                            return;
                        }
                    }
                }
            }
        }

        // 2. Primary BLAKE3 check
        if config.verify_blake3 {
            if let Some(exp_blake3) = expected_blake3 {
                if !exp_blake3.is_empty() {
                    let actual_blake3 = compute_blake3_hash(content);
                    if !HashAlgorithm::Blake3.is_valid_hex_format(exp_blake3) {
                        report.blake3_metrics.record_fail();
                        report.add_failure(HashValidationFailure::new(
                            path,
                            HashAlgorithm::Blake3,
                            exp_blake3,
                            actual_blake3,
                            "Expected BLAKE3 hash is not a valid 64-character lowercase hex digest",
                        ));
                    } else if actual_blake3 == exp_blake3 {
                        report.blake3_metrics.record_pass();
                    } else {
                        report.blake3_metrics.record_fail();
                        report.add_failure(HashValidationFailure::new(
                            path,
                            HashAlgorithm::Blake3,
                            exp_blake3,
                            actual_blake3,
                            "BLAKE3 content hash mismatch: calculated digest does not match expected",
                        ));
                        if config.fail_fast {
                            return;
                        }
                    }
                }
            }
        }

        // 3. Secondary SHA-256 check
        if config.verify_sha256 {
            if let Some(exp_sha256) = expected_sha256 {
                if !exp_sha256.is_empty() {
                    let actual_sha256 = compute_sha256_hash(content);
                    if !HashAlgorithm::Sha256.is_valid_hex_format(exp_sha256) {
                        report.sha256_metrics.record_fail();
                        report.add_failure(HashValidationFailure::new(
                            path,
                            HashAlgorithm::Sha256,
                            exp_sha256,
                            actual_sha256,
                            "Expected SHA-256 hash is not a valid 64-character lowercase hex digest",
                        ));
                    } else if actual_sha256 == exp_sha256 {
                        report.sha256_metrics.record_pass();
                    } else {
                        report.sha256_metrics.record_fail();
                        report.add_failure(HashValidationFailure::new(
                            path,
                            HashAlgorithm::Sha256,
                            exp_sha256,
                            actual_sha256,
                            "SHA-256 content hash mismatch: calculated digest does not match expected",
                        ));
                    }
                }
            }
        }
    }

    /// Validates all files across extraction reports (`SourceExtractionReport`, `DocExtractionReport`,
    /// `DependencyExtractionReport`, and `ConfigTestExtractionReport`).
    #[must_use]
    pub fn validate_reports(
        sources: Option<&SourceExtractionReport>,
        docs: Option<&DocExtractionReport>,
        dependencies: Option<&DependencyExtractionReport>,
        configs_tests: Option<&ConfigTestExtractionReport>,
        config: &HashValidationConfig,
        target_repo: impl Into<String>,
    ) -> HashValidationReport {
        let mut report = HashValidationReport::new(target_repo);

        // Sources
        if let Some(src_report) = sources {
            for file in &src_report.extracted {
                Self::validate_file_bytes(
                    &file.path,
                    file.content.as_bytes(),
                    Some(&file.git_sha),
                    Some(&file.blake3_hash),
                    Some(&file.sha256_hash),
                    config,
                    &mut report,
                );
                if config.fail_fast && !report.is_valid() {
                    return report;
                }
            }
        }

        // Docs
        if let Some(doc_report) = docs {
            for file in &doc_report.extracted {
                Self::validate_file_bytes(
                    &file.path,
                    file.content.as_bytes(),
                    Some(&file.git_sha),
                    Some(&file.blake3_hash),
                    Some(&file.sha256_hash),
                    config,
                    &mut report,
                );
                if config.fail_fast && !report.is_valid() {
                    return report;
                }
            }
        }

        // Dependencies
        if let Some(dep_report) = dependencies {
            for file in &dep_report.extracted {
                Self::validate_file_bytes(
                    &file.path,
                    file.content.as_bytes(),
                    Some(&file.git_sha),
                    Some(&file.blake3_hash),
                    Some(&file.sha256_hash),
                    config,
                    &mut report,
                );
                if config.fail_fast && !report.is_valid() {
                    return report;
                }
            }
        }

        // Configs & Tests
        if let Some(cfg_report) = configs_tests {
            for file in &cfg_report.extracted_configs {
                Self::validate_file_bytes(
                    &file.path,
                    file.content.as_bytes(),
                    Some(&file.git_sha),
                    Some(&file.blake3_hash),
                    Some(&file.sha256_hash),
                    config,
                    &mut report,
                );
                if config.fail_fast && !report.is_valid() {
                    return report;
                }
            }
            for file in &cfg_report.extracted_tests {
                Self::validate_file_bytes(
                    &file.path,
                    file.content.as_bytes(),
                    Some(&file.git_sha),
                    Some(&file.blake3_hash),
                    Some(&file.sha256_hash),
                    config,
                    &mut report,
                );
                if config.fail_fast && !report.is_valid() {
                    return report;
                }
            }
        }

        report
    }

    /// Validates entries in a `RepoFileManifest` against provided content payloads.
    #[must_use]
    pub fn validate_manifest(
        manifest: &RepoFileManifest,
        content_map: &HashMap<String, &[u8]>,
        config: &HashValidationConfig,
        target_repo: impl Into<String>,
    ) -> HashValidationReport {
        let mut report = HashValidationReport::new(target_repo);

        for entry in &manifest.files {
            if entry.status == FileExtractionStatus::Extracted {
                if let Some(bytes) = content_map.get(&entry.path) {
                    Self::validate_file_bytes(
                        &entry.path,
                        bytes,
                        Some(&entry.git_sha),
                        entry.blake3_hash.as_deref(),
                        entry.sha256_hash.as_deref(),
                        config,
                        &mut report,
                    );
                    if config.fail_fast && !report.is_valid() {
                        return report;
                    }
                }
            }
        }

        report
    }

    /// Validates `CdmSection` list ensuring section content hashes match their body.
    #[must_use]
    pub fn validate_cdm_sections(
        sections: &[CdmSection],
        config: &HashValidationConfig,
        target_repo: impl Into<String>,
    ) -> HashValidationReport {
        let mut report = HashValidationReport::new(target_repo);

        for section in sections {
            let path = section
                .original_path
                .as_deref()
                .unwrap_or(&section.section_id);

            let expected_git_sha = section.attributes.get("git_blob_sha").map(String::as_str);

            Self::validate_file_bytes(
                path,
                section.body.as_bytes(),
                expected_git_sha,
                Some(&section.content_hash),
                None, // CDM sections store BLAKE3 as section.content_hash
                config,
                &mut report,
            );

            if config.fail_fast && !report.is_valid() {
                return report;
            }
        }

        report
    }
}
