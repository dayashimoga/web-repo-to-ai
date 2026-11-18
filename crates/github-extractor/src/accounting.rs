//! # Expected-vs-Actual File Accounting & Completeness Verification
//!
//! Provides rigorous mathematical completeness verification, discrepancy taxonomy,
//! set reconciliation, and balance sheet validation between discovered Git tree entries
//! and extracted/classified repository artifacts.
//!
//! Enforces the fundamental repository conversion law:
//! `discovered_blobs == actual_extracted + actual_excluded + actual_failed + actual_unsupported + missing_unaccounted`
//!
//! Ensures that any unaccounted dropped files, phantom extra files, truncated byte counts,
//! or mismatched Git object SHAs are surfaced as explicit discrepancies before marking
//! an extraction job complete.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{
    CdmSection, CdmSectionBuilder, CompletenessReport, SectionKind, SectionTokenMetrics,
};

use crate::classification::{FileCategory, TreeClassificationReport};
use crate::config_test::ConfigTestExtractionReport;
use crate::dependency::DependencyExtractionReport;
use crate::documentation::DocExtractionReport;
use crate::manifest::{FileExtractionStatus, ManifestFileEntry, RepoFileManifest};
use crate::source::{compute_blake3_hash, SourceExtractionReport};
use crate::tree::{GitTreeEntry, GitTreeEntryType, GitTreeResponse};
use crate::tree_generator::format_byte_size;

/// Severity classification for accounting discrepancies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscrepancySeverity {
    /// Informational discrepancy that does not compromise extraction completeness.
    Info,
    /// Warning-level discrepancy (e.g. non-critical category difference).
    Warning,
    /// Critical discrepancy violating completeness or content integrity.
    Critical,
}

impl DiscrepancySeverity {
    /// Returns a static display label for this severity.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warning => "WARN",
            Self::Critical => "CRITICAL",
        }
    }
}

/// Specific type or root cause of an accounting discrepancy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountingDiscrepancyKind {
    /// Discovered in Git tree and eligible for extraction, but completely absent from actual extracted or excluded records.
    MissingUnaccounted,
    /// Present in extracted/manifest records but never discovered in the Git tree (phantom or orphaned file).
    UnexpectedExtra,
    /// Actual extracted byte size does not match expected Git tree blob size.
    ByteSizeMismatch,
    /// Actual Git object SHA-1 does not match expected Git tree blob SHA-1.
    GitShaMismatch,
    /// File category in extraction differs from initial tree classification.
    CategoryMismatch,
    /// Extraction status indicates an invalid or contradictory outcome.
    StatusConflict,
}

impl AccountingDiscrepancyKind {
    /// Returns a static display label for the discrepancy kind.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::MissingUnaccounted => "missing_unaccounted",
            Self::UnexpectedExtra => "unexpected_extra",
            Self::ByteSizeMismatch => "byte_size_mismatch",
            Self::GitShaMismatch => "git_sha_mismatch",
            Self::CategoryMismatch => "category_mismatch",
            Self::StatusConflict => "status_conflict",
        }
    }
}

/// A single detected reconciliation discrepancy between expected and actual file artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountingDiscrepancy {
    /// Normalized relative repository path.
    pub path: String,
    /// Classification kind of the discrepancy.
    pub kind: AccountingDiscrepancyKind,
    /// Expected attribute, count, or behavior from discovery/specification.
    pub expected_detail: String,
    /// Actual observed attribute, count, or behavior from extraction artifacts.
    pub actual_detail: String,
    /// Impact severity of this discrepancy.
    pub severity: DiscrepancySeverity,
}

impl AccountingDiscrepancy {
    /// Creates a new accounting discrepancy record.
    #[must_use]
    pub fn new(
        path: impl Into<String>,
        kind: AccountingDiscrepancyKind,
        expected_detail: impl Into<String>,
        actual_detail: impl Into<String>,
        severity: DiscrepancySeverity,
    ) -> Self {
        Self {
            path: path.into(),
            kind,
            expected_detail: expected_detail.into(),
            actual_detail: actual_detail.into(),
            severity,
        }
    }
}

/// Quantitative reconciliation balance sheet comparing discovered repository tree entries
/// against actual extracted and classified artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileAccountingBalanceSheet {
    /// Target repository identifier (e.g. `owner/repo`).
    pub target_repo: String,
    /// Target ref name (e.g. `main`, `v1.2.0`), if specified.
    pub target_ref: Option<String>,
    /// Pinned 40-character Git commit SHA, if resolved.
    pub pinned_sha: Option<String>,
    /// Total number of blob entries discovered in the Git tree ($E$).
    pub discovered_blobs: usize,
    /// Number of discovered blobs eligible for extraction based on filter rules.
    pub eligible_blobs: usize,
    /// Number of discovered blobs classified as excluded during discovery.
    pub expected_excluded_blobs: usize,
    /// Total number of files successfully extracted into artifacts ($A_{\text{ext}}$).
    pub actual_extracted: usize,
    /// Total number of files intentionally excluded with recorded rationale ($A_{\text{excl}}$).
    pub actual_excluded: usize,
    /// Total number of files that encountered unrecoverable extraction errors ($A_{\text{fail}}$).
    pub actual_failed: usize,
    /// Total number of files deemed unsupported format ($A_{\text{unsup}}$).
    pub actual_unsupported: usize,
    /// Total number of eligible discovered files completely unaccounted for ($M$).
    pub missing_unaccounted: usize,
    /// Total number of files extracted with no corresponding Git tree origin ($X$).
    pub unexpected_extra: usize,
    /// Total number of files with byte size mismatch between tree blob size and extracted content.
    pub byte_size_mismatches: usize,
    /// Total number of files with Git object SHA-1 mismatch.
    pub git_sha_mismatches: usize,
    /// Aggregate byte size of discovered blobs (where sizes are available).
    pub discovered_bytes: u64,
    /// Aggregate byte size of successfully extracted files.
    pub extracted_bytes: u64,
    /// Detailed vector of all detected discrepancies.
    pub discrepancies: Vec<AccountingDiscrepancy>,
}

impl FileAccountingBalanceSheet {
    /// Verifies the arithmetic conservation identity:
    /// `discovered_blobs == actual_extracted + actual_excluded + actual_failed + actual_unsupported + missing_unaccounted`
    #[must_use]
    pub fn is_balanced(&self) -> bool {
        let accounted_sum = self
            .actual_extracted
            .saturating_add(self.actual_excluded)
            .saturating_add(self.actual_failed)
            .saturating_add(self.actual_unsupported)
            .saturating_add(self.missing_unaccounted);
        self.discovered_blobs == accounted_sum
    }

    /// Returns `true` if zero files were dropped without a record and zero phantom extra files were introduced.
    #[must_use]
    pub fn is_fully_accounted(&self) -> bool {
        self.missing_unaccounted == 0 && self.unexpected_extra == 0
    }

    /// Returns `true` if all conservation laws hold, all files are accounted for,
    /// and zero byte size or Git SHA mismatches occurred.
    #[must_use]
    pub fn is_perfectly_consistent(&self) -> bool {
        self.is_balanced()
            && self.is_fully_accounted()
            && self.byte_size_mismatches == 0
            && self.git_sha_mismatches == 0
            && !self.has_critical_discrepancies()
    }

    /// Returns all detected discrepancies matching [`DiscrepancySeverity::Critical`].
    #[must_use]
    pub fn critical_discrepancies(&self) -> Vec<&AccountingDiscrepancy> {
        self.discrepancies
            .iter()
            .filter(|d| d.severity == DiscrepancySeverity::Critical)
            .collect()
    }

    /// Returns `true` if any critical discrepancies were recorded.
    #[must_use]
    pub fn has_critical_discrepancies(&self) -> bool {
        self.discrepancies
            .iter()
            .any(|d| d.severity == DiscrepancySeverity::Critical)
    }

    /// Translates this accounting balance sheet into a standard Canonical Context Document Model [`CompletenessReport`].
    #[must_use]
    pub fn to_completeness_report(&self) -> CompletenessReport {
        CompletenessReport {
            discovered_count: self.discovered_blobs as u64,
            eligible_count: self.eligible_blobs as u64,
            extracted_count: self.actual_extracted as u64,
            excluded_count: self.actual_excluded as u64,
            failed_count: (self.actual_failed + self.missing_unaccounted) as u64,
            unsupported_count: self.actual_unsupported as u64,
        }
    }

    /// Formats this accounting balance sheet as a structured GitHub-Flavored Markdown report.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!(
            "# File Accounting & Completeness Audit: {}\n\n",
            self.target_repo
        ));

        // Metadata badges
        let balanced_badge = if self.is_balanced() {
            "PASS: Balanced"
        } else {
            "FAIL: Unbalanced"
        };
        let accounted_badge = if self.is_fully_accounted() {
            "PASS: 100% Accounted"
        } else {
            "FAIL: Incomplete / Phantom Entries"
        };
        let consistent_badge = if self.is_perfectly_consistent() {
            "PASS: Fully Consistent"
        } else {
            "WARN: Discrepancies Present"
        };

        md.push_str(&format!(
            "- **Status**: `{}` | `{}` | `{}`\n",
            balanced_badge, accounted_badge, consistent_badge
        ));
        if let Some(ref r) = self.target_ref {
            md.push_str(&format!("- **Ref**: `{r}`\n"));
        }
        if let Some(ref sha) = self.pinned_sha {
            md.push_str(&format!("- **Commit**: `{sha}`\n"));
        }
        md.push('\n');

        // Balance Sheet Table
        md.push_str("### Accounting Balance Sheet\n\n");
        md.push_str("| Metric | Count | Bytes | Status |\n");
        md.push_str("| :--- | :---: | :---: | :--- |\n");
        md.push_str(&format!(
            "| **Discovered Blobs (Tree)** | {} | {} | Baseline |\n",
            self.discovered_blobs,
            format_byte_size(self.discovered_bytes)
        ));
        md.push_str(&format!(
            "| ├─ Eligible for Extraction | {} | — | Classification |\n",
            self.eligible_blobs
        ));
        md.push_str(&format!(
            "| └─ Expected Excluded (Tree) | {} | — | Classification |\n",
            self.expected_excluded_blobs
        ));
        md.push_str(&format!(
            "| **Actual Extracted** | {} | {} | {} |\n",
            self.actual_extracted,
            format_byte_size(self.extracted_bytes),
            if self.actual_extracted > 0 {
                "OK"
            } else {
                "Empty"
            }
        ));
        md.push_str(&format!(
            "| **Actual Excluded** | {} | — | Intentional |\n",
            self.actual_excluded
        ));
        md.push_str(&format!(
            "| **Actual Failed** | {} | — | {} |\n",
            self.actual_failed,
            if self.actual_failed == 0 {
                "OK (0)"
            } else {
                "Errors"
            }
        ));
        md.push_str(&format!(
            "| **Actual Unsupported** | {} | — | Binary / Unknown |\n",
            self.actual_unsupported
        ));
        md.push_str(&format!(
            "| **Missing Unaccounted** | **{}** | — | {} |\n",
            self.missing_unaccounted,
            if self.missing_unaccounted == 0 {
                "OK (0)"
            } else {
                "CRITICAL: Unaccounted Drops"
            }
        ));
        md.push_str(&format!(
            "| **Unexpected Extra** | **{}** | — | {} |\n",
            self.unexpected_extra,
            if self.unexpected_extra == 0 {
                "OK (0)"
            } else {
                "CRITICAL: Phantom Files"
            }
        ));
        md.push_str(&format!(
            "| **Byte Size Mismatches** | {} | — | {} |\n",
            self.byte_size_mismatches,
            if self.byte_size_mismatches == 0 {
                "OK (0)"
            } else {
                "Corruption / Truncation"
            }
        ));
        md.push_str(&format!(
            "| **Git SHA Mismatches** | {} | — | {} |\n",
            self.git_sha_mismatches,
            if self.git_sha_mismatches == 0 {
                "OK (0)"
            } else {
                "Hash Mismatches"
            }
        ));
        md.push('\n');

        // Discrepancy Log
        if self.discrepancies.is_empty() {
            md.push_str("### Discrepancy Log\n\n");
            md.push_str("*Zero discrepancies detected. Repository extraction is 100% mathematically balanced and verified against the Git tree.*\n");
        } else {
            md.push_str(&format!(
                "### Discrepancy Log ({} items)\n\n",
                self.discrepancies.len()
            ));
            md.push_str("| Severity | Kind | Path | Expected | Actual |\n");
            md.push_str("| :---: | :--- | :--- | :--- | :--- |\n");
            for d in &self.discrepancies {
                md.push_str(&format!(
                    "| `{}` | `{}` | `{}` | {} | {} |\n",
                    d.severity.as_str(),
                    d.kind.as_str(),
                    d.path,
                    d.expected_detail,
                    d.actual_detail
                ));
            }
            md.push('\n');
        }

        md
    }

    /// Serializes this accounting balance sheet into a formatted pretty JSON string.
    ///
    /// # Errors
    /// Returns `serde_json::Error` if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serializes this accounting balance sheet into a compact single-line JSON string.
    ///
    /// # Errors
    /// Returns `serde_json::Error` if serialization fails.
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Converts this accounting balance sheet into a Canonical Context Document Model [`CdmSection`].
    #[must_use]
    pub fn to_cdm_section(&self, doc_canonical_url: impl Into<String>) -> CdmSection {
        let markdown = self.to_markdown();
        let content_hash = compute_blake3_hash(markdown.as_bytes());
        let token_metrics = SectionTokenMetrics::compute_baseline(&markdown);

        let mut section = CdmSectionBuilder::new(
            "sec-file-accounting-audit",
            doc_canonical_url,
            SectionKind::Manifest,
            &markdown,
        )
        .title("Repository File Accounting & Completeness Audit")
        .original_path("ACCOUNTING_AUDIT.md")
        .content_hash(content_hash)
        .token_metrics(token_metrics)
        .build();

        section.attributes.insert(
            "manifest_role".to_string(),
            "file_accounting_audit".to_string(),
        );
        section
            .attributes
            .insert("is_balanced".to_string(), self.is_balanced().to_string());
        section.attributes.insert(
            "is_fully_accounted".to_string(),
            self.is_fully_accounted().to_string(),
        );
        section.attributes.insert(
            "is_perfectly_consistent".to_string(),
            self.is_perfectly_consistent().to_string(),
        );
        section.attributes.insert(
            "discovered_blobs".to_string(),
            self.discovered_blobs.to_string(),
        );
        section.attributes.insert(
            "actual_extracted".to_string(),
            self.actual_extracted.to_string(),
        );
        section.attributes.insert(
            "missing_unaccounted".to_string(),
            self.missing_unaccounted.to_string(),
        );
        section.attributes.insert(
            "unexpected_extra".to_string(),
            self.unexpected_extra.to_string(),
        );
        section.attributes.insert(
            "byte_size_mismatches".to_string(),
            self.byte_size_mismatches.to_string(),
        );
        section.attributes.insert(
            "git_sha_mismatches".to_string(),
            self.git_sha_mismatches.to_string(),
        );
        section.attributes.insert(
            "discrepancy_count".to_string(),
            self.discrepancies.len().to_string(),
        );

        section
    }
}

/// Auditor engine performing set reconciliation between expected Git tree entries
/// and actual extracted repository artifacts.
pub struct FileAccountingAuditor;

impl FileAccountingAuditor {
    /// Reconciles an expected `GitTreeResponse` against an assembled `RepoFileManifest`.
    #[must_use]
    pub fn audit_manifest(
        expected_tree: &GitTreeResponse,
        classification: Option<&TreeClassificationReport>,
        manifest: &RepoFileManifest,
        target_repo: impl Into<String>,
        target_ref: Option<String>,
        pinned_sha: Option<String>,
    ) -> FileAccountingBalanceSheet {
        let repo_name = target_repo.into();

        // 1. Index expected tree entries (filtering to blobs/files only)
        let mut expected_map: BTreeMap<String, &GitTreeEntry> = BTreeMap::new();
        let mut discovered_bytes: u64 = 0;

        for entry in &expected_tree.tree {
            if entry.entry_type == GitTreeEntryType::Blob {
                expected_map.insert(entry.path.clone(), entry);
                if let Some(size) = entry.size {
                    discovered_bytes = discovered_bytes.saturating_add(size);
                }
            }
        }

        let discovered_blobs = expected_map.len();

        // 2. Derive eligible vs expected excluded from classification if available
        let mut eligible_blobs = 0;
        let mut expected_excluded_blobs = 0;

        if let Some(class_report) = classification {
            for entry in &class_report.files {
                if !entry.category.is_extractable() {
                    expected_excluded_blobs += 1;
                } else {
                    eligible_blobs += 1;
                }
            }
        } else {
            // Default baseline: everything is considered eligible
            eligible_blobs = discovered_blobs;
        }

        // 3. Index actual manifest entries
        let mut actual_map: BTreeMap<String, &ManifestFileEntry> = BTreeMap::new();
        let mut actual_extracted = 0;
        let mut actual_excluded = 0;
        let mut actual_failed = 0;
        let mut actual_unsupported = 0;
        let mut extracted_bytes: u64 = 0;

        for file in &manifest.files {
            actual_map.insert(file.path.clone(), file);
            match &file.status {
                FileExtractionStatus::Extracted => {
                    actual_extracted += 1;
                    extracted_bytes = extracted_bytes.saturating_add(file.size_bytes);
                }
                FileExtractionStatus::Excluded { .. } => {
                    actual_excluded += 1;
                }
                FileExtractionStatus::Failed { .. } => {
                    actual_failed += 1;
                }
                FileExtractionStatus::Unsupported { .. } => {
                    actual_unsupported += 1;
                }
            }
        }

        // 4. Perform bidirectional set reconciliation
        let mut discrepancies: Vec<AccountingDiscrepancy> = Vec::new();
        let mut missing_unaccounted = 0;
        let mut unexpected_extra = 0;
        let mut byte_size_mismatches = 0;
        let mut git_sha_mismatches = 0;

        // Pass A: Check every expected file against actual manifest
        for (path, expected_entry) in &expected_map {
            if let Some(actual_entry) = actual_map.get(path) {
                // Check byte size agreement if expected size was populated in Git tree
                if let Some(exp_size) = expected_entry.size {
                    // For extracted files, byte size should match blob size exactly
                    if actual_entry.status.is_extracted() && actual_entry.size_bytes != exp_size {
                        byte_size_mismatches += 1;
                        discrepancies.push(AccountingDiscrepancy::new(
                            path.as_str(),
                            AccountingDiscrepancyKind::ByteSizeMismatch,
                            format!("{} bytes", exp_size),
                            format!("{} bytes", actual_entry.size_bytes),
                            DiscrepancySeverity::Critical,
                        ));
                    }
                }

                // Check Git SHA-1 consistency
                if !expected_entry.sha.is_empty()
                    && !actual_entry.git_sha.is_empty()
                    && actual_entry.git_sha != expected_entry.sha
                {
                    git_sha_mismatches += 1;
                    discrepancies.push(AccountingDiscrepancy::new(
                        path.as_str(),
                        AccountingDiscrepancyKind::GitShaMismatch,
                        expected_entry.sha.clone(),
                        actual_entry.git_sha.clone(),
                        DiscrepancySeverity::Critical,
                    ));
                }
            } else {
                // Discovered in tree, but entirely absent from manifest!
                missing_unaccounted += 1;
                discrepancies.push(AccountingDiscrepancy::new(
                    path.as_str(),
                    AccountingDiscrepancyKind::MissingUnaccounted,
                    format!("Discovered blob in tree (SHA: {})", expected_entry.sha),
                    "No extraction, exclusion, or failure record present in manifest",
                    DiscrepancySeverity::Critical,
                ));
            }
        }

        // Pass B: Check for phantom actual files not present in the expected tree
        for (path, actual_entry) in &actual_map {
            if !expected_map.contains_key(path) {
                unexpected_extra += 1;
                discrepancies.push(AccountingDiscrepancy::new(
                    path.as_str(),
                    AccountingDiscrepancyKind::UnexpectedExtra,
                    "Not present in discovered Git tree",
                    format!(
                        "Present in manifest with status '{}' (SHA: {})",
                        actual_entry.status.status_name(),
                        actual_entry.git_sha
                    ),
                    DiscrepancySeverity::Critical,
                ));
            }
        }

        // Sort discrepancies deterministically by severity (Critical first) then path
        discrepancies.sort_by(|a, b| {
            b.severity
                .cmp(&a.severity)
                .then_with(|| a.path.cmp(&b.path))
        });

        FileAccountingBalanceSheet {
            target_repo: repo_name,
            target_ref,
            pinned_sha,
            discovered_blobs,
            eligible_blobs,
            expected_excluded_blobs,
            actual_extracted,
            actual_excluded,
            actual_failed,
            actual_unsupported,
            missing_unaccounted,
            unexpected_extra,
            byte_size_mismatches,
            git_sha_mismatches,
            discovered_bytes,
            extracted_bytes,
            discrepancies,
        }
    }

    /// Convenience auditor reconciling an expected `GitTreeResponse` against extraction reports
    /// by assembling an on-the-fly `RepoFileManifest`.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn audit_reports(
        expected_tree: &GitTreeResponse,
        classification: Option<&TreeClassificationReport>,
        sources: Option<&SourceExtractionReport>,
        docs: Option<&DocExtractionReport>,
        dependencies: Option<&DependencyExtractionReport>,
        configs_tests: Option<&ConfigTestExtractionReport>,
        target_repo: impl Into<String>,
        target_ref: Option<String>,
        pinned_sha: Option<String>,
    ) -> FileAccountingBalanceSheet {
        // Derive classification report if not provided
        let class_report = classification
            .cloned()
            .unwrap_or_else(|| expected_tree.classify(&Default::default()));

        // Assemble manifest from reports
        let manifest = RepoFileManifest::from_classification_and_reports(
            &class_report,
            sources,
            docs,
            dependencies,
            configs_tests,
        );

        Self::audit_manifest(
            expected_tree,
            Some(&class_report),
            &manifest,
            target_repo,
            target_ref,
            pinned_sha,
        )
    }

    /// Audits an expected `GitTreeResponse` against a set of `CdmSection`s representing extracted code/prose/manifest files.
    #[must_use]
    pub fn audit_sections(
        expected_tree: &GitTreeResponse,
        classification: Option<&TreeClassificationReport>,
        sections: &[CdmSection],
        target_repo: impl Into<String>,
        target_ref: Option<String>,
        pinned_sha: Option<String>,
    ) -> FileAccountingBalanceSheet {
        // Build manifest file entries from CDM sections having an original_path
        let mut entries = Vec::new();
        for section in sections {
            if let Some(ref path) = section.original_path {
                let category = if section.kind == SectionKind::CodeFile {
                    FileCategory::SourceCode
                } else if section.kind == SectionKind::Prose {
                    FileCategory::Documentation
                } else if section.kind == SectionKind::Manifest {
                    FileCategory::DependencyManifest
                } else {
                    FileCategory::Configuration
                };

                let git_sha = section
                    .attributes
                    .get("git_blob_sha")
                    .cloned()
                    .unwrap_or_default();

                entries.push(ManifestFileEntry {
                    path: path.clone(),
                    git_sha,
                    category,
                    size_bytes: section.body.len() as u64,
                    line_count: Some(section.body.lines().count()),
                    language: section.language.clone(),
                    blake3_hash: Some(section.content_hash.clone()),
                    sha256_hash: None,
                    status: FileExtractionStatus::Extracted,
                    cdm_section_id: Some(section.section_id.clone()),
                });
            }
        }

        let manifest = RepoFileManifest::from_entries(entries);
        Self::audit_manifest(
            expected_tree,
            classification,
            &manifest,
            target_repo,
            target_ref,
            pinned_sha,
        )
    }
}
