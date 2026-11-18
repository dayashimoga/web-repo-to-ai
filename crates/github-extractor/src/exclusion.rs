//! # Missing and Excluded Repository File Diagnostics & Audit Reporting
//!
//! Provides structured diagnostics, root-cause categorizations, remediation guidance,
//! and formal reporting for files that were intentionally excluded (oversized, generated/lockfiles,
//! vendor/dependencies, binaries, policy filters, sensitive credentials) or failed extraction.
//!
//! Generates transparent audit trails guaranteeing 100% accounting for every skipped or
//! problematic repository item, with direct translation to Canonical Context Document Model
//! ([`web_repo_to_ai_cdm::ExclusionRecord`]) items.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{
    CdmSection, CdmSectionBuilder, ExclusionReason, ExclusionRecord, SectionKind,
    SectionTokenMetrics,
};

use crate::accounting::FileAccountingBalanceSheet;
use crate::classification::{FileCategory, TreeClassificationReport};
use crate::config_test::ConfigTestExtractionReport;
use crate::dependency::DependencyExtractionReport;
use crate::documentation::DocExtractionReport;
use crate::manifest::{FileExtractionStatus, RepoFileManifest};
use crate::source::{compute_blake3_hash, SourceExtractionReport};
use crate::tree_generator::format_byte_size;

/// Granular functional classification of why an item was intentionally excluded from extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExclusionCategory {
    /// File exceeded the configured maximum byte size ceiling.
    OversizedFile,
    /// Lockfiles, compilation outputs, source maps, or generated distribution bundles.
    GeneratedOrLockfile,
    /// Vendored third-party dependencies (e.g. `node_modules/`, `vendor/`, `third_party/`, `.git/`, `target/`).
    VendorOrThirdParty,
    /// Non-text binary assets, compiled bytecode, images, audio, or archives.
    BinaryOrAsset,
    /// Git submodules, broken symlinks, or non-blob tree entities.
    SubmoduleOrSymlink,
    /// Matches explicit user or system path/glob exclusion patterns (e.g. `*.tmp`, `.gitignore`).
    PolicyPattern,
    /// Blocked by security policy to prevent credential leaks (e.g. private keys, `.env`, tokens).
    SensitiveSecurityPattern,
    /// Uncategorized or miscellaneous exclusion reason.
    Other,
}

impl ExclusionCategory {
    /// Returns a static lowercase identifier for display and telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OversizedFile => "oversized_file",
            Self::GeneratedOrLockfile => "generated_or_lockfile",
            Self::VendorOrThirdParty => "vendor_or_third_party",
            Self::BinaryOrAsset => "binary_or_asset",
            Self::SubmoduleOrSymlink => "submodule_or_symlink",
            Self::PolicyPattern => "policy_pattern",
            Self::SensitiveSecurityPattern => "sensitive_security_pattern",
            Self::Other => "other",
        }
    }

    /// Maps this exclusion category into the official CDM [`ExclusionReason`].
    #[must_use]
    pub const fn to_cdm_reason(self) -> ExclusionReason {
        match self {
            Self::OversizedFile => ExclusionReason::SizeLimit,
            Self::GeneratedOrLockfile => ExclusionReason::ExcludedExtension,
            Self::VendorOrThirdParty | Self::PolicyPattern => ExclusionReason::RulePattern,
            Self::BinaryOrAsset => ExclusionReason::BinaryFile,
            Self::SubmoduleOrSymlink | Self::Other => ExclusionReason::UnsupportedType,
            Self::SensitiveSecurityPattern => ExclusionReason::SecurityRestricted,
        }
    }

    /// Returns default actionable guidance or remediation advice for this category.
    #[must_use]
    pub const fn default_remediation(self) -> &'static str {
        match self {
            Self::OversizedFile => {
                "Increase 'max_file_size_bytes' in FileClassificationConfig if this file must be ingested."
            }
            Self::GeneratedOrLockfile => {
                "Set 'include_lockfiles: true' in job settings if dependency lockfile analysis is required."
            }
            Self::VendorOrThirdParty => {
                "Vendored dependencies are excluded by default to avoid token exhaustion. Ingest package manifests instead."
            }
            Self::BinaryOrAsset => {
                "Binary assets cannot be converted to text/markdown. Store in an external object bucket if required."
            }
            Self::SubmoduleOrSymlink => {
                "Git submodules must be ingested separately using their respective repository URLs."
            }
            Self::PolicyPattern => {
                "Adjust exclusion patterns in repository configuration or custom filter rules."
            }
            Self::SensitiveSecurityPattern => {
                "Sensitive security files are blocked to prevent credential leaks. Redact secrets before conversion."
            }
            Self::Other => "Review repository file classification rules.",
        }
    }

    /// Infers an exclusion category from a relative file path and classified category.
    #[must_use]
    pub fn infer(path: &str, category: Option<FileCategory>) -> Self {
        let p = path.to_lowercase();

        // 1. Sensitive security files
        if p.ends_with(".env")
            || p.contains(".env.")
            || p.ends_with(".pem")
            || p.ends_with(".key")
            || p.ends_with("id_rsa")
            || p.ends_with("id_ed25519")
            || p.ends_with("id_ecdsa")
            || p.contains("credentials")
            || p.contains("secrets.json")
        {
            return Self::SensitiveSecurityPattern;
        }

        // 2. Vendor / Third-party
        if p.starts_with("node_modules/")
            || p.contains("/node_modules/")
            || p.starts_with("vendor/")
            || p.contains("/vendor/")
            || p.starts_with("third_party/")
            || p.contains("/third_party/")
            || p.starts_with("target/")
            || p.contains("/target/")
            || p.starts_with(".git/")
            || p.contains("/.git/")
        {
            return Self::VendorOrThirdParty;
        }

        // 3. Category hints
        if let Some(cat) = category {
            match cat {
                FileCategory::Oversized => return Self::OversizedFile,
                FileCategory::GeneratedOrLockfile => return Self::GeneratedOrLockfile,
                FileCategory::Binary => return Self::BinaryOrAsset,
                _ => {}
            }
        }

        // 4. Filename / Extension fallbacks
        if p.ends_with(".lock")
            || p.ends_with("-lock.json")
            || p.ends_with(".min.js")
            || p.ends_with(".min.css")
            || p.ends_with(".map")
        {
            Self::GeneratedOrLockfile
        } else if p.ends_with(".png")
            || p.ends_with(".jpg")
            || p.ends_with(".jpeg")
            || p.ends_with(".gif")
            || p.ends_with(".ico")
            || p.ends_with(".wasm")
            || p.ends_with(".exe")
            || p.ends_with(".so")
            || p.ends_with(".dylib")
            || p.ends_with(".zip")
            || p.ends_with(".tar.gz")
        {
            Self::BinaryOrAsset
        } else {
            Self::Other
        }
    }
}

/// Root-cause classification for extraction processing failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionFailureCategory {
    /// Transient network or connect timeout while communicating with GitHub API.
    NetworkTimeout,
    /// GitHub API secondary or primary rate limit ceiling encountered.
    RateLimitExceeded,
    /// Blob exceeds GitHub REST API single-blob limits (100 MB ceiling).
    BlobTooLarge,
    /// Extracted content corrupted (Git SHA-1 mismatch or invalid base64 padding).
    CorruptedContent,
    /// Subtree traversal encountered missing commit ref or recursion ceiling.
    SubtreeTraversalFailed,
    /// Local I/O error or memory allocation budget ceiling reached.
    IoOrStorage,
    /// Unclassified extraction processing error.
    Unknown,
}

impl ExtractionFailureCategory {
    /// Returns a static lowercase identifier for display and telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NetworkTimeout => "network_timeout",
            Self::RateLimitExceeded => "rate_limit_exceeded",
            Self::BlobTooLarge => "blob_too_large",
            Self::CorruptedContent => "corrupted_content",
            Self::SubtreeTraversalFailed => "subtree_traversal_failed",
            Self::IoOrStorage => "io_or_storage",
            Self::Unknown => "unknown",
        }
    }

    /// Returns `true` if this failure category represents a transient error eligible for retry.
    #[must_use]
    pub const fn is_retryable(self) -> bool {
        matches!(self, Self::NetworkTimeout | Self::RateLimitExceeded)
    }

    /// Infers the failure category from an error description string.
    #[must_use]
    pub fn infer(error_msg: &str) -> Self {
        let err = error_msg.to_lowercase();
        if err.contains("timeout") || err.contains("timed out") || err.contains("connection reset")
        {
            Self::NetworkTimeout
        } else if err.contains("rate limit") || err.contains("429") || err.contains("secondary") {
            Self::RateLimitExceeded
        } else if err.contains("too large") || err.contains("100 mb") || err.contains("truncated") {
            Self::BlobTooLarge
        } else if err.contains("mismatch")
            || err.contains("corrupt")
            || err.contains("base64")
            || err.contains("sha")
        {
            Self::CorruptedContent
        } else if err.contains("subtree") || err.contains("traversal") || err.contains("recursion")
        {
            Self::SubtreeTraversalFailed
        } else if err.contains("io") || err.contains("storage") || err.contains("disk") {
            Self::IoOrStorage
        } else {
            Self::Unknown
        }
    }
}

/// An audit record detailing a file intentionally excluded from extraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExcludedFileRecord {
    /// Normalized relative repository path.
    pub path: String,
    /// 40-character Git object SHA-1 hash.
    pub git_sha: String,
    /// Classified category of the exclusion.
    pub category: ExclusionCategory,
    /// Byte size of the excluded file, if known.
    pub size_bytes: u64,
    /// Detected language or syntax format, if identified.
    pub language: Option<String>,
    /// Human-readable explanation of why this file was excluded.
    pub rationale: String,
    /// Specific pattern that matched (e.g. `*.lock`, `vendor/**`), if applicable.
    pub pattern: Option<String>,
    /// Actionable remediation or configuration suggestion.
    pub remediation: String,
}

impl ExcludedFileRecord {
    /// Creates a new excluded file record.
    #[must_use]
    pub fn new(
        path: impl Into<String>,
        git_sha: impl Into<String>,
        category: ExclusionCategory,
        size_bytes: u64,
        language: Option<String>,
        rationale: impl Into<String>,
        pattern: Option<String>,
    ) -> Self {
        let remediation = category.default_remediation().to_string();
        Self {
            path: path.into(),
            git_sha: git_sha.into(),
            category,
            size_bytes,
            language,
            rationale: rationale.into(),
            pattern,
            remediation,
        }
    }

    /// Converts this record into a Canonical Context Document Model [`ExclusionRecord`].
    #[must_use]
    pub fn to_cdm_record(&self) -> ExclusionRecord {
        ExclusionRecord::new(
            &self.path,
            self.category.to_cdm_reason(),
            self.pattern.clone(),
            Some(self.rationale.clone()),
            Some(self.size_bytes),
        )
    }
}

/// An audit record detailing an individual file extraction failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailedFileRecord {
    /// Normalized relative repository path.
    pub path: String,
    /// 40-character Git object SHA-1 hash.
    pub git_sha: String,
    /// Classified root-cause failure category.
    pub error_category: ExtractionFailureCategory,
    /// Sanitized error description.
    pub error_message: String,
    /// Whether this failure is eligible for exponential backoff retry.
    pub retry_eligible: bool,
    /// Associated HTTP status code, if applicable.
    pub http_status: Option<u16>,
}

impl FailedFileRecord {
    /// Creates a new failed file record.
    #[must_use]
    pub fn new(
        path: impl Into<String>,
        git_sha: impl Into<String>,
        error_message: impl Into<String>,
        http_status: Option<u16>,
    ) -> Self {
        let msg = error_message.into();
        let error_category = ExtractionFailureCategory::infer(&msg);
        let retry_eligible = error_category.is_retryable();
        Self {
            path: path.into(),
            git_sha: git_sha.into(),
            error_category,
            error_message: msg,
            retry_eligible,
            http_status,
        }
    }
}

/// An audit record detailing an eligible discovered file that was missing without explanation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingFileRecord {
    /// Normalized relative repository path.
    pub path: String,
    /// 40-character Git object SHA-1 hash from the discovered Git tree.
    pub git_sha: String,
    /// Expected byte size from Git tree blob metadata.
    pub expected_size: Option<u64>,
    /// Diagnostic description of the unaccounted drop.
    pub diagnostics: String,
}

impl MissingFileRecord {
    /// Creates a new missing file record.
    #[must_use]
    pub fn new(
        path: impl Into<String>,
        git_sha: impl Into<String>,
        expected_size: Option<u64>,
        diagnostics: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            git_sha: git_sha.into(),
            expected_size,
            diagnostics: diagnostics.into(),
        }
    }
}

/// Consolidated audit report detailing all excluded, failed, and missing repository files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExclusionAuditReport {
    /// Target repository identifier (e.g. `owner/repo`).
    pub target_repo: String,
    /// Target ref name (e.g. `main`, `v2.0.0`), if specified.
    pub target_ref: Option<String>,
    /// Pinned 40-character Git commit SHA, if resolved.
    pub pinned_sha: Option<String>,
    /// Total number of files discovered in the Git tree.
    pub total_discovered_files: usize,
    /// Total count of intentionally excluded files.
    pub total_excluded_files: usize,
    /// Total count of files that encountered extraction errors.
    pub total_failed_files: usize,
    /// Total count of eligible files completely unaccounted for (dropped).
    pub total_missing_files: usize,
    /// Aggregate byte size of all excluded files.
    pub excluded_bytes: u64,
    /// Breakdown of excluded files per exclusion category.
    pub exclusion_breakdown: BTreeMap<String, usize>,
    /// Breakdown of failed files per failure category.
    pub failure_breakdown: BTreeMap<String, usize>,
    /// Itemized collection of excluded file records.
    pub excluded_files: Vec<ExcludedFileRecord>,
    /// Itemized collection of failed file records.
    pub failed_files: Vec<FailedFileRecord>,
    /// Itemized collection of missing file records.
    pub missing_files: Vec<MissingFileRecord>,
}

impl ExclusionAuditReport {
    /// Creates a new empty exclusion audit report.
    #[must_use]
    pub fn new(target_repo: impl Into<String>) -> Self {
        Self {
            target_repo: target_repo.into(),
            target_ref: None,
            pinned_sha: None,
            total_discovered_files: 0,
            total_excluded_files: 0,
            total_failed_files: 0,
            total_missing_files: 0,
            excluded_bytes: 0,
            exclusion_breakdown: BTreeMap::new(),
            failure_breakdown: BTreeMap::new(),
            excluded_files: Vec::new(),
            failed_files: Vec::new(),
            missing_files: Vec::new(),
        }
    }

    /// Returns `true` if zero missing files and zero failed files were detected.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.total_missing_files == 0 && self.total_failed_files == 0
    }

    /// Returns `true` if any critical extraction defects exist (missing or failed files).
    #[must_use]
    pub fn has_critical_issues(&self) -> bool {
        !self.is_clean()
    }

    /// Returns all failed file records eligible for automated HTTP/backoff retry.
    #[must_use]
    pub fn retryable_failures(&self) -> Vec<&FailedFileRecord> {
        self.failed_files
            .iter()
            .filter(|f| f.retry_eligible)
            .collect()
    }

    /// Converts all excluded file records into CDM [`ExclusionRecord`] structures.
    #[must_use]
    pub fn to_cdm_exclusion_records(&self) -> Vec<ExclusionRecord> {
        self.excluded_files
            .iter()
            .map(ExcludedFileRecord::to_cdm_record)
            .collect()
    }

    /// Formats this exclusion and missing file audit into a structured GitHub-Flavored Markdown report.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!(
            "# Missing & Excluded Files Audit: {}\n\n",
            self.target_repo
        ));

        let status_badge = if self.is_clean() {
            "PASS: 100% Accounted & Clean"
        } else {
            "CRITICAL: Missing / Failed Files Detected"
        };

        md.push_str(&format!("- **Integrity Status**: `{status_badge}`\n"));
        if let Some(ref r) = self.target_ref {
            md.push_str(&format!("- **Ref**: `{r}`\n"));
        }
        if let Some(ref sha) = self.pinned_sha {
            md.push_str(&format!("- **Commit**: `{sha}`\n"));
        }
        md.push_str(&format!(
            "- **Discovered Blobs**: {}\n",
            self.total_discovered_files
        ));
        md.push_str(&format!(
            "- **Excluded Files**: {} ({})\n",
            self.total_excluded_files,
            format_byte_size(self.excluded_bytes)
        ));
        md.push_str(&format!(
            "- **Failed Files**: {}\n",
            self.total_failed_files
        ));
        md.push_str(&format!(
            "- **Missing (Dropped) Files**: {}\n\n",
            self.total_missing_files
        ));

        // Exclusion Breakdown Table
        if !self.exclusion_breakdown.is_empty() {
            md.push_str("### Exclusion Category Breakdown\n\n");
            md.push_str("| Exclusion Category | Count | Primary Remediation |\n");
            md.push_str("| :--- | :---: | :--- |\n");
            for (category_str, count) in &self.exclusion_breakdown {
                let remediation = match category_str.as_str() {
                    "oversized_file" => ExclusionCategory::OversizedFile.default_remediation(),
                    "generated_or_lockfile" => {
                        ExclusionCategory::GeneratedOrLockfile.default_remediation()
                    }
                    "vendor_or_third_party" => {
                        ExclusionCategory::VendorOrThirdParty.default_remediation()
                    }
                    "binary_or_asset" => ExclusionCategory::BinaryOrAsset.default_remediation(),
                    "submodule_or_symlink" => {
                        ExclusionCategory::SubmoduleOrSymlink.default_remediation()
                    }
                    "policy_pattern" => ExclusionCategory::PolicyPattern.default_remediation(),
                    "sensitive_security_pattern" => {
                        ExclusionCategory::SensitiveSecurityPattern.default_remediation()
                    }
                    _ => ExclusionCategory::Other.default_remediation(),
                };
                md.push_str(&format!("| `{category_str}` | {count} | {remediation} |\n"));
            }
            md.push('\n');
        }

        // Missing Files Log
        if !self.missing_files.is_empty() {
            md.push_str(&format!(
                "### Missing (Unaccounted Drop) Log ({} files)\n\n",
                self.missing_files.len()
            ));
            md.push_str("| Path | Git Blob SHA | Expected Size | Diagnostic Detail |\n");
            md.push_str("| :--- | :---: | :---: | :--- |\n");
            for m in &self.missing_files {
                let size_str = m
                    .expected_size
                    .map_or_else(|| "unknown".to_string(), format_byte_size);
                md.push_str(&format!(
                    "| `{}` | `{}` | {} | {} |\n",
                    m.path, m.git_sha, size_str, m.diagnostics
                ));
            }
            md.push('\n');
        }

        // Failed Files Log
        if !self.failed_files.is_empty() {
            md.push_str(&format!(
                "### Extraction Failure Log ({} files)\n\n",
                self.failed_files.len()
            ));
            md.push_str("| Path | Error Category | Retryable | HTTP Status | Error Detail |\n");
            md.push_str("| :--- | :---: | :---: | :---: | :--- |\n");
            for f in &self.failed_files {
                md.push_str(&format!(
                    "| `{}` | `{}` | {} | {} | {} |\n",
                    f.path,
                    f.error_category.as_str(),
                    if f.retry_eligible { "Yes" } else { "No" },
                    f.http_status
                        .map_or_else(|| "—".to_string(), |s| s.to_string()),
                    f.error_message
                ));
            }
            md.push('\n');
        }

        // Excluded Files Itemized Table
        if !self.excluded_files.is_empty() {
            md.push_str(&format!(
                "### Excluded Files Index ({} files)\n\n",
                self.excluded_files.len()
            ));
            md.push_str("| Path | Category | Size | Reason | Suggested Action |\n");
            md.push_str("| :--- | :---: | :---: | :--- | :--- |\n");
            for e in &self.excluded_files {
                md.push_str(&format!(
                    "| `{}` | `{}` | {} | {} | {} |\n",
                    e.path,
                    e.category.as_str(),
                    format_byte_size(e.size_bytes),
                    e.rationale,
                    e.remediation
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

    /// Serializes this report into compact single-line JSON.
    ///
    /// # Errors
    /// Returns `serde_json::Error` if serialization fails.
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Converts this audit report into a Canonical Context Document Model [`CdmSection`].
    #[must_use]
    pub fn to_cdm_section(&self, doc_canonical_url: impl Into<String>) -> CdmSection {
        let markdown = self.to_markdown();
        let content_hash = compute_blake3_hash(markdown.as_bytes());
        let token_metrics = SectionTokenMetrics::compute_baseline(&markdown);

        let mut section = CdmSectionBuilder::new(
            "sec-exclusion-and-missing-report",
            doc_canonical_url,
            SectionKind::Manifest,
            &markdown,
        )
        .title("Missing and Excluded Repository Files Audit")
        .original_path("EXCLUSION_REPORT.md")
        .content_hash(content_hash)
        .token_metrics(token_metrics)
        .build();

        section.attributes.insert(
            "manifest_role".to_string(),
            "exclusion_and_missing_report".to_string(),
        );
        section
            .attributes
            .insert("is_clean".to_string(), self.is_clean().to_string());
        section.attributes.insert(
            "total_discovered_files".to_string(),
            self.total_discovered_files.to_string(),
        );
        section.attributes.insert(
            "total_excluded_files".to_string(),
            self.total_excluded_files.to_string(),
        );
        section.attributes.insert(
            "total_failed_files".to_string(),
            self.total_failed_files.to_string(),
        );
        section.attributes.insert(
            "total_missing_files".to_string(),
            self.total_missing_files.to_string(),
        );
        section.attributes.insert(
            "excluded_bytes".to_string(),
            self.excluded_bytes.to_string(),
        );

        section
    }
}

/// Generator engine assembling comprehensive `ExclusionAuditReport`s from repository artifacts.
pub struct ExclusionReportGenerator;

impl ExclusionReportGenerator {
    /// Generates an exclusion and missing file report from a `RepoFileManifest` and optional `FileAccountingBalanceSheet`.
    #[must_use]
    pub fn generate_from_manifest(
        manifest: &RepoFileManifest,
        balance_sheet: Option<&FileAccountingBalanceSheet>,
        target_repo: impl Into<String>,
        target_ref: Option<String>,
        pinned_sha: Option<String>,
    ) -> ExclusionAuditReport {
        let repo_name = target_repo.into();
        let mut report = ExclusionAuditReport::new(repo_name);
        report.target_ref = target_ref;
        report.pinned_sha = pinned_sha;
        report.total_discovered_files = manifest.total_files;

        // Process manifest files
        for entry in &manifest.files {
            match &entry.status {
                FileExtractionStatus::Excluded { reason } => {
                    let category = ExclusionCategory::infer(&entry.path, Some(entry.category));
                    *report
                        .exclusion_breakdown
                        .entry(category.as_str().to_string())
                        .or_insert(0) += 1;
                    report.total_excluded_files += 1;
                    report.excluded_bytes = report.excluded_bytes.saturating_add(entry.size_bytes);

                    report.excluded_files.push(ExcludedFileRecord::new(
                        &entry.path,
                        &entry.git_sha,
                        category,
                        entry.size_bytes,
                        entry.language.clone(),
                        reason.as_str(),
                        None,
                    ));
                }
                FileExtractionStatus::Failed { error } => {
                    let failure_rec =
                        FailedFileRecord::new(&entry.path, &entry.git_sha, error.as_str(), None);
                    *report
                        .failure_breakdown
                        .entry(failure_rec.error_category.as_str().to_string())
                        .or_insert(0) += 1;
                    report.total_failed_files += 1;
                    report.failed_files.push(failure_rec);
                }
                FileExtractionStatus::Unsupported { reason } => {
                    let category = ExclusionCategory::BinaryOrAsset;
                    *report
                        .exclusion_breakdown
                        .entry(category.as_str().to_string())
                        .or_insert(0) += 1;
                    report.total_excluded_files += 1;
                    report.excluded_bytes = report.excluded_bytes.saturating_add(entry.size_bytes);

                    report.excluded_files.push(ExcludedFileRecord::new(
                        &entry.path,
                        &entry.git_sha,
                        category,
                        entry.size_bytes,
                        entry.language.clone(),
                        format!("Unsupported: {reason}"),
                        None,
                    ));
                }
                FileExtractionStatus::Extracted => {}
            }
        }

        // Process missing files from accounting balance sheet if available
        if let Some(sheet) = balance_sheet {
            for d in &sheet.discrepancies {
                if d.kind == crate::accounting::AccountingDiscrepancyKind::MissingUnaccounted {
                    report.total_missing_files += 1;
                    report.missing_files.push(MissingFileRecord::new(
                        &d.path,
                        d.expected_detail.clone(),
                        None,
                        d.actual_detail.clone(),
                    ));
                }
            }
        }

        // Deterministically sort lists by path
        report.excluded_files.sort_by(|a, b| a.path.cmp(&b.path));
        report.failed_files.sort_by(|a, b| a.path.cmp(&b.path));
        report.missing_files.sort_by(|a, b| a.path.cmp(&b.path));

        report
    }

    /// Assembles an exclusion and missing report directly from raw extraction reports and classification.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn generate_from_reports(
        classification: &TreeClassificationReport,
        sources: Option<&SourceExtractionReport>,
        docs: Option<&DocExtractionReport>,
        dependencies: Option<&DependencyExtractionReport>,
        configs_tests: Option<&ConfigTestExtractionReport>,
        balance_sheet: Option<&FileAccountingBalanceSheet>,
        target_repo: impl Into<String>,
        target_ref: Option<String>,
        pinned_sha: Option<String>,
    ) -> ExclusionAuditReport {
        let manifest = RepoFileManifest::from_classification_and_reports(
            classification,
            sources,
            docs,
            dependencies,
            configs_tests,
        );

        Self::generate_from_manifest(
            &manifest,
            balance_sheet,
            target_repo,
            target_ref,
            pinned_sha,
        )
    }
}
