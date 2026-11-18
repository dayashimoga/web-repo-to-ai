//! # Repository File Manifest & Completeness Index
//!
//! Provides unified repository file indexing, per-file extraction status accounting,
//! multi-report aggregation, GitHub-Flavored Markdown summary table formatting,
//! JSON serialization, and Canonical Context Document Model ([`CdmSection`]) packaging.
//!
//! Enforces the fundamental repository extraction invariant:
//! `total_files == extracted_files + excluded_files + failed_files + unsupported_files`

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{
    CdmSection, CdmSectionBuilder, CompletenessReport, SectionKind, SectionTokenMetrics,
};

use crate::classification::{ClassifiedFile, FileCategory, TreeClassificationReport};
use crate::config_test::ConfigTestExtractionReport;
use crate::dependency::DependencyExtractionReport;
use crate::documentation::DocExtractionReport;
use crate::source::{compute_blake3_hash, SourceExtractionReport};
use crate::tree_generator::format_byte_size;

/// Status of an individual repository file during extraction and packaging.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileExtractionStatus {
    /// File was successfully extracted, verified, and converted into CDM content.
    Extracted,
    /// File was deliberately excluded from extraction (e.g. oversized, vendor, lockfile).
    Excluded {
        /// Explanation or policy category for the exclusion.
        reason: String,
    },
    /// File extraction encountered an unrecoverable processing error.
    Failed {
        /// Sanitized error description.
        error: String,
    },
    /// File format or media type is unsupported (e.g. non-text binary).
    Unsupported {
        /// Description of the unsupported format or attribute.
        reason: String,
    },
}

impl FileExtractionStatus {
    /// Returns `true` if the file was successfully extracted.
    #[must_use]
    pub const fn is_extracted(&self) -> bool {
        matches!(self, Self::Extracted)
    }

    /// Returns `true` if the file was deliberately excluded.
    #[must_use]
    pub const fn is_excluded(&self) -> bool {
        matches!(self, Self::Excluded { .. })
    }

    /// Returns `true` if the file encountered an extraction failure.
    #[must_use]
    pub const fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }

    /// Returns `true` if the file is deemed unsupported.
    #[must_use]
    pub const fn is_unsupported(&self) -> bool {
        matches!(self, Self::Unsupported { .. })
    }

    /// Returns a static lowercase identifier for the status variant.
    #[must_use]
    pub const fn status_name(&self) -> &'static str {
        match self {
            Self::Extracted => "extracted",
            Self::Excluded { .. } => "excluded",
            Self::Failed { .. } => "failed",
            Self::Unsupported { .. } => "unsupported",
        }
    }

    /// Returns the associated reason or error description, if present.
    #[must_use]
    pub fn reason_or_error(&self) -> Option<&str> {
        match self {
            Self::Extracted => None,
            Self::Excluded { reason } | Self::Unsupported { reason } => Some(reason.as_str()),
            Self::Failed { error } => Some(error.as_str()),
        }
    }
}

/// Structured index entry for a single repository file in the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestFileEntry {
    /// Normalized relative repository path (e.g. `src/lib.rs`).
    pub path: String,
    /// 40-character Git object SHA-1 hash.
    pub git_sha: String,
    /// Classified functional category of the file.
    pub category: FileCategory,
    /// Size in bytes of the file, if known.
    pub size_bytes: u64,
    /// Number of lines in the file content, if extracted as text.
    pub line_count: Option<usize>,
    /// Detected programming language or syntax format, if identified.
    pub language: Option<String>,
    /// Cryptographic BLAKE3 digest of the content bytes, if extracted.
    pub blake3_hash: Option<String>,
    /// Cryptographic SHA-256 digest of the content bytes, if extracted.
    pub sha256_hash: Option<String>,
    /// Final extraction outcome status for this file.
    pub status: FileExtractionStatus,
    /// Identifier of the corresponding CDM section, if extracted into a section.
    pub cdm_section_id: Option<String>,
}

impl ManifestFileEntry {
    /// Creates a new manifest file entry with baseline attributes.
    #[must_use]
    pub fn new(
        path: impl Into<String>,
        git_sha: impl Into<String>,
        category: FileCategory,
        size_bytes: u64,
        status: FileExtractionStatus,
    ) -> Self {
        Self {
            path: path.into(),
            git_sha: git_sha.into(),
            category,
            size_bytes,
            line_count: None,
            language: None,
            blake3_hash: None,
            sha256_hash: None,
            status,
            cdm_section_id: None,
        }
    }

    /// Sets the line count for text files.
    #[must_use]
    pub const fn with_line_count(mut self, lines: usize) -> Self {
        self.line_count = Some(lines);
        self
    }

    /// Sets the programming language or syntax format.
    #[must_use]
    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.language = Some(lang.into());
        self
    }

    /// Sets the BLAKE3 cryptographic digest.
    #[must_use]
    pub fn with_blake3_hash(mut self, hash: impl Into<String>) -> Self {
        self.blake3_hash = Some(hash.into());
        self
    }

    /// Sets the SHA-256 cryptographic digest.
    #[must_use]
    pub fn with_sha256_hash(mut self, hash: impl Into<String>) -> Self {
        self.sha256_hash = Some(hash.into());
        self
    }

    /// Sets the associated CDM section ID.
    #[must_use]
    pub fn with_cdm_section_id(mut self, id: impl Into<String>) -> Self {
        self.cdm_section_id = Some(id.into());
        self
    }

    /// Convenience query indicating if this entry was successfully extracted.
    #[must_use]
    pub const fn is_extracted(&self) -> bool {
        self.status.is_extracted()
    }
}

/// Consolidated repository file manifest and completeness accounting index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoFileManifest {
    /// Alphabetically sorted list of all discovered repository files.
    pub files: Vec<ManifestFileEntry>,
    /// Total number of files discovered in the repository tree.
    pub total_files: usize,
    /// Number of files successfully extracted.
    pub extracted_files: usize,
    /// Number of files intentionally excluded.
    pub excluded_files: usize,
    /// Number of files that encountered extraction errors.
    pub failed_files: usize,
    /// Number of files deemed unsupported (e.g. binary media).
    pub unsupported_files: usize,
    /// Aggregate byte size of all discovered files.
    pub total_bytes: u64,
    /// Aggregate byte size of successfully extracted files.
    pub extracted_bytes: u64,
    /// Aggregate line count across all extracted text files.
    pub total_lines: usize,
    /// File count breakdown per functional category.
    pub category_counts: BTreeMap<String, usize>,
    /// File count breakdown per programming language / format.
    pub language_counts: BTreeMap<String, usize>,
    /// Count breakdown per extraction status name.
    pub status_counts: BTreeMap<String, usize>,
}

impl RepoFileManifest {
    /// Builds a consolidated `RepoFileManifest` from explicit file entries.
    ///
    /// Entries are sorted deterministically by lowercase path, and aggregate
    /// metrics and category distributions are computed automatically.
    #[must_use]
    pub fn from_entries(mut entries: Vec<ManifestFileEntry>) -> Self {
        entries.sort_by_key(|a| a.path.to_lowercase());

        let total_files = entries.len();
        let mut extracted_files = 0;
        let mut excluded_files = 0;
        let mut failed_files = 0;
        let mut unsupported_files = 0;
        let mut total_bytes = 0;
        let mut extracted_bytes = 0;
        let mut total_lines = 0;
        let mut category_counts = BTreeMap::new();
        let mut language_counts = BTreeMap::new();
        let mut status_counts = BTreeMap::new();

        for entry in &entries {
            total_bytes += entry.size_bytes;

            *category_counts
                .entry(entry.category.as_str().to_string())
                .or_insert(0) += 1;

            if let Some(ref lang) = entry.language {
                *language_counts.entry(lang.clone()).or_insert(0) += 1;
            }

            *status_counts
                .entry(entry.status.status_name().to_string())
                .or_insert(0) += 1;

            match &entry.status {
                FileExtractionStatus::Extracted => {
                    extracted_files += 1;
                    extracted_bytes += entry.size_bytes;
                    total_lines += entry.line_count.unwrap_or(0);
                }
                FileExtractionStatus::Excluded { .. } => {
                    excluded_files += 1;
                }
                FileExtractionStatus::Failed { .. } => {
                    failed_files += 1;
                }
                FileExtractionStatus::Unsupported { .. } => {
                    unsupported_files += 1;
                }
            }
        }

        Self {
            files: entries,
            total_files,
            extracted_files,
            excluded_files,
            failed_files,
            unsupported_files,
            total_bytes,
            extracted_bytes,
            total_lines,
            category_counts,
            language_counts,
            status_counts,
        }
    }

    /// Assembles a cohesive manifest by stitching a tree classification report with
    /// results from arbitrary downstream extraction reports (sources, documentation,
    /// dependencies, configurations, and tests).
    ///
    /// Every file in `classification.files` is guaranteed to be represented in the
    /// resulting manifest, ensuring strict invariant compliance.
    #[must_use]
    pub fn from_classification_and_reports(
        classification: &TreeClassificationReport,
        sources: Option<&SourceExtractionReport>,
        docs: Option<&DocExtractionReport>,
        dependencies: Option<&DependencyExtractionReport>,
        configs_tests: Option<&ConfigTestExtractionReport>,
    ) -> Self {
        let mut entries = Vec::with_capacity(classification.files.len());

        for file in &classification.files {
            let entry = Self::build_entry(file, sources, docs, dependencies, configs_tests);
            entries.push(entry);
        }

        Self::from_entries(entries)
    }

    fn build_entry(
        file: &ClassifiedFile,
        sources: Option<&SourceExtractionReport>,
        docs: Option<&DocExtractionReport>,
        dependencies: Option<&DependencyExtractionReport>,
        configs_tests: Option<&ConfigTestExtractionReport>,
    ) -> ManifestFileEntry {
        let size_bytes = file.size.unwrap_or(0);

        // 1. If file was classified as ineligible, record exclusion or unsupported
        if !file.is_eligible {
            let status = if file.category == FileCategory::Binary {
                FileExtractionStatus::Unsupported {
                    reason: file
                        .exclusion_reason
                        .clone()
                        .unwrap_or_else(|| "Binary file format".to_string()),
                }
            } else {
                FileExtractionStatus::Excluded {
                    reason: file
                        .exclusion_reason
                        .clone()
                        .unwrap_or_else(|| "Excluded by filter policy".to_string()),
                }
            };

            return ManifestFileEntry::new(
                &file.path,
                &file.sha,
                file.category,
                size_bytes,
                status,
            )
            .with_language(file.language.clone().unwrap_or_default());
        }

        // 2. Check source extraction report
        if let Some(src_report) = sources {
            if let Some(found) = src_report.extracted.iter().find(|e| e.path == file.path) {
                let mut entry = ManifestFileEntry::new(
                    &file.path,
                    &found.git_sha,
                    file.category,
                    found.byte_size as u64,
                    FileExtractionStatus::Extracted,
                )
                .with_line_count(found.line_count)
                .with_blake3_hash(&found.blake3_hash)
                .with_sha256_hash(&found.sha256_hash);

                if let Some(lang) = found.language.as_ref().or(file.language.as_ref()) {
                    entry = entry.with_language(lang.as_str());
                }
                return entry;
            }
            if let Some(failed) = src_report.failed.iter().find(|f| f.path == file.path) {
                return ManifestFileEntry::new(
                    &file.path,
                    &failed.git_sha,
                    file.category,
                    size_bytes,
                    FileExtractionStatus::Failed {
                        error: failed.error.clone(),
                    },
                );
            }
            if src_report.unsupported.iter().any(|u| u.path == file.path) {
                return ManifestFileEntry::new(
                    &file.path,
                    &file.sha,
                    file.category,
                    size_bytes,
                    FileExtractionStatus::Unsupported {
                        reason: "Binary or null bytes detected during source inspection"
                            .to_string(),
                    },
                );
            }
        }

        // 3. Check doc extraction report
        if let Some(doc_report) = docs {
            if let Some(found) = doc_report.extracted.iter().find(|e| e.path == file.path) {
                return ManifestFileEntry::new(
                    &file.path,
                    &found.git_sha,
                    file.category,
                    found.byte_size as u64,
                    FileExtractionStatus::Extracted,
                )
                .with_line_count(found.line_count)
                .with_language(found.format.as_str())
                .with_blake3_hash(&found.blake3_hash)
                .with_sha256_hash(&found.sha256_hash);
            }
            if let Some(failed) = doc_report.failed.iter().find(|f| f.path == file.path) {
                return ManifestFileEntry::new(
                    &file.path,
                    &failed.git_sha,
                    file.category,
                    size_bytes,
                    FileExtractionStatus::Failed {
                        error: failed.error.clone(),
                    },
                );
            }
            if doc_report.unsupported.iter().any(|u| u.path == file.path) {
                return ManifestFileEntry::new(
                    &file.path,
                    &file.sha,
                    file.category,
                    size_bytes,
                    FileExtractionStatus::Unsupported {
                        reason: "Binary documentation file format".to_string(),
                    },
                );
            }
        }

        // 4. Check dependency extraction report
        if let Some(dep_report) = dependencies {
            if let Some(found) = dep_report.extracted.iter().find(|e| e.path == file.path) {
                return ManifestFileEntry::new(
                    &file.path,
                    &found.git_sha,
                    file.category,
                    found.byte_size as u64,
                    FileExtractionStatus::Extracted,
                )
                .with_line_count(found.line_count)
                .with_language(found.ecosystem.format_name())
                .with_blake3_hash(&found.blake3_hash)
                .with_sha256_hash(&found.sha256_hash);
            }
            if let Some(failed) = dep_report.failed.iter().find(|f| f.path == file.path) {
                return ManifestFileEntry::new(
                    &file.path,
                    &failed.git_sha,
                    file.category,
                    size_bytes,
                    FileExtractionStatus::Failed {
                        error: failed.error.clone(),
                    },
                );
            }
            if dep_report.unsupported.iter().any(|u| u.path == file.path) {
                return ManifestFileEntry::new(
                    &file.path,
                    &file.sha,
                    file.category,
                    size_bytes,
                    FileExtractionStatus::Unsupported {
                        reason: "Binary dependency manifest file".to_string(),
                    },
                );
            }
        }

        // 5. Check configuration and test extraction report
        if let Some(cfg_test_report) = configs_tests {
            if let Some(found) = cfg_test_report
                .extracted_configs
                .iter()
                .find(|e| e.path == file.path)
            {
                return ManifestFileEntry::new(
                    &file.path,
                    &found.git_sha,
                    file.category,
                    found.byte_size as u64,
                    FileExtractionStatus::Extracted,
                )
                .with_line_count(found.line_count)
                .with_language(&found.format)
                .with_blake3_hash(&found.blake3_hash)
                .with_sha256_hash(&found.sha256_hash);
            }
            if let Some(found) = cfg_test_report
                .extracted_tests
                .iter()
                .find(|e| e.path == file.path)
            {
                let mut entry = ManifestFileEntry::new(
                    &file.path,
                    &found.git_sha,
                    file.category,
                    found.byte_size as u64,
                    FileExtractionStatus::Extracted,
                )
                .with_line_count(found.line_count)
                .with_blake3_hash(&found.blake3_hash)
                .with_sha256_hash(&found.sha256_hash);

                if let Some(lang) = found.language.as_ref().or(file.language.as_ref()) {
                    entry = entry.with_language(lang.as_str());
                }
                return entry;
            }
            if let Some(failed) = cfg_test_report.failed.iter().find(|f| f.path == file.path) {
                return ManifestFileEntry::new(
                    &file.path,
                    &failed.git_sha,
                    file.category,
                    size_bytes,
                    FileExtractionStatus::Failed {
                        error: failed.error.clone(),
                    },
                );
            }
            if cfg_test_report
                .unsupported
                .iter()
                .any(|u| u.path == file.path)
            {
                return ManifestFileEntry::new(
                    &file.path,
                    &file.sha,
                    file.category,
                    size_bytes,
                    FileExtractionStatus::Unsupported {
                        reason: "Binary configuration or test fixture format".to_string(),
                    },
                );
            }
        }

        // Fallback: eligible file not found in any provided extraction report
        let mut entry = ManifestFileEntry::new(
            &file.path,
            &file.sha,
            file.category,
            size_bytes,
            FileExtractionStatus::Excluded {
                reason: "Not queued for active extraction".to_string(),
            },
        );
        if let Some(ref lang) = file.language {
            entry = entry.with_language(lang);
        }
        entry
    }

    /// Verifies the fundamental completeness conservation equation:
    /// `total_files == extracted_files + excluded_files + failed_files + unsupported_files`
    #[must_use]
    pub fn verify_completeness(&self) -> bool {
        self.total_files
            == self
                .extracted_files
                .saturating_add(self.excluded_files)
                .saturating_add(self.failed_files)
                .saturating_add(self.unsupported_files)
    }

    /// Returns `true` if the manifest is valid, all eligible files were extracted,
    /// and zero failures or unsupported formats occurred.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.verify_completeness() && self.failed_files == 0 && self.unsupported_files == 0
    }

    /// Converts the manifest accounting into a Canonical Context Document Model [`CompletenessReport`].
    #[must_use]
    pub fn to_completeness_report(&self) -> CompletenessReport {
        let discovered = self.total_files as u64;
        let extracted = self.extracted_files as u64;
        let excluded = self.excluded_files as u64;
        let failed = self.failed_files as u64;
        let unsupported = self.unsupported_files as u64;
        let eligible = extracted + failed + unsupported;

        CompletenessReport::new(
            discovered,
            eligible,
            extracted,
            excluded,
            failed,
            unsupported,
        )
    }

    /// Returns an iterator over all successfully extracted files.
    pub fn extracted_entries(&self) -> impl Iterator<Item = &ManifestFileEntry> {
        self.files.iter().filter(|f| f.is_extracted())
    }

    /// Returns an iterator over all excluded files.
    pub fn excluded_entries(&self) -> impl Iterator<Item = &ManifestFileEntry> {
        self.files.iter().filter(|f| f.status.is_excluded())
    }

    /// Returns an iterator over all failed files.
    pub fn failed_entries(&self) -> impl Iterator<Item = &ManifestFileEntry> {
        self.files.iter().filter(|f| f.status.is_failed())
    }

    /// Returns an iterator over all unsupported files.
    pub fn unsupported_entries(&self) -> impl Iterator<Item = &ManifestFileEntry> {
        self.files.iter().filter(|f| f.status.is_unsupported())
    }

    /// Searches for a manifest file entry by exact relative repository path.
    #[must_use]
    pub fn find_by_path(&self, path: &str) -> Option<&ManifestFileEntry> {
        self.files.iter().find(|f| f.path == path)
    }

    /// Formats the manifest as an informative GitHub-Flavored Markdown summary document.
    #[must_use]
    pub fn to_markdown_table(&self) -> String {
        let mut out = String::new();
        out.push_str("# Repository File Manifest & Completeness Index\n\n");

        out.push_str("## Executive Summary\n\n");
        out.push_str("| Metric | Value |\n");
        out.push_str("| :--- | :--- |\n");
        out.push_str(&format!(
            "| Total Discovered Files | {} |\n",
            self.total_files
        ));
        out.push_str(&format!(
            "| Successfully Extracted | {} |\n",
            self.extracted_files
        ));
        out.push_str(&format!(
            "| Intentionally Excluded | {} |\n",
            self.excluded_files
        ));
        out.push_str(&format!("| Failed Extraction | {} |\n", self.failed_files));
        out.push_str(&format!(
            "| Unsupported Media/Format | {} |\n",
            self.unsupported_files
        ));
        out.push_str(&format!(
            "| Total Discovered Bytes | {} ({}) |\n",
            self.total_bytes,
            format_byte_size(self.total_bytes)
        ));
        out.push_str(&format!(
            "| Extracted Content Bytes | {} ({}) |\n",
            self.extracted_bytes,
            format_byte_size(self.extracted_bytes)
        ));
        out.push_str(&format!(
            "| Total Extracted Lines | {} |\n",
            self.total_lines
        ));
        let formula_status = if self.verify_completeness() {
            "Valid (`total == extracted + excluded + failed + unsupported`)"
        } else {
            "INVALID (Conservation invariant violated)"
        };
        out.push_str(&format!(
            "| Completeness Invariant | {} |\n\n",
            formula_status
        ));

        // Category breakdown
        if !self.category_counts.is_empty() {
            out.push_str("## Category Breakdown\n\n");
            out.push_str("| Category | File Count | Percentage |\n");
            out.push_str("| :--- | :---: | :---: |\n");
            for (category, count) in &self.category_counts {
                let pct = if self.total_files > 0 {
                    (*count as f64 / self.total_files as f64) * 100.0
                } else {
                    0.0
                };
                out.push_str(&format!("| `{category}` | {count} | {pct:.1}% |\n"));
            }
            out.push('\n');
        }

        // File Index Table
        out.push_str("## File Index\n\n");
        out.push_str(
            "| Path | Category | Language | Size | Lines | Status | Details | Git SHA |\n",
        );
        out.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");

        for file in &self.files {
            let lang = file.language.as_deref().unwrap_or("-");
            let lines = file
                .line_count
                .map(|l| l.to_string())
                .unwrap_or_else(|| "-".to_string());
            let size = format_byte_size(file.size_bytes);
            let status_badge = file.status.status_name();
            let details = match &file.status {
                FileExtractionStatus::Extracted => file
                    .blake3_hash
                    .as_ref()
                    .map(|h| {
                        if h.len() >= 8 {
                            format!("blake3:{}", &h[..8])
                        } else {
                            format!("blake3:{h}")
                        }
                    })
                    .unwrap_or_else(|| "ok".to_string()),
                FileExtractionStatus::Excluded { reason }
                | FileExtractionStatus::Unsupported { reason } => {
                    let sanitized = reason.replace('|', "\\|");
                    if sanitized.len() > 40 {
                        format!("{}...", &sanitized[..37])
                    } else {
                        sanitized
                    }
                }
                FileExtractionStatus::Failed { error } => {
                    let sanitized = error.replace('|', "\\|");
                    if sanitized.len() > 40 {
                        format!("{}...", &sanitized[..37])
                    } else {
                        sanitized
                    }
                }
            };
            let short_sha = if file.git_sha.len() >= 7 {
                &file.git_sha[..7]
            } else {
                &file.git_sha
            };

            out.push_str(&format!(
                "| `{}` | {} | {} | {} | {} | `{}` | {} | `{}` |\n",
                file.path,
                file.category.as_str(),
                lang,
                size,
                lines,
                status_badge,
                details,
                short_sha,
            ));
        }

        out
    }

    /// Serializes this repository manifest into a pretty-printed JSON string.
    ///
    /// # Errors
    /// Returns `serde_json::Error` if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serializes this repository manifest into a compact single-line JSON string.
    ///
    /// # Errors
    /// Returns `serde_json::Error` if serialization fails.
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Converts this repository manifest into a Canonical Context Document Model [`CdmSection`].
    #[must_use]
    pub fn to_cdm_section(
        &self,
        section_id: impl Into<String>,
        source_url: impl Into<String>,
    ) -> CdmSection {
        let markdown = self.to_markdown_table();
        let line_count = markdown.lines().count().max(1) as u32;
        let byte_size = markdown.len() as u64;
        let blake3_hash = compute_blake3_hash(markdown.as_bytes());
        let token_metrics = SectionTokenMetrics::compute_baseline(&markdown);

        let mut section =
            CdmSectionBuilder::new(section_id, source_url, SectionKind::Manifest, &markdown)
                .title("Repository File Manifest & Completeness Index")
                .original_path("MANIFEST.md")
                .line_range(1, line_count)
                .byte_range(0, byte_size)
                .language("markdown")
                .content_hash(&blake3_hash)
                .token_metrics(token_metrics)
                .build();

        section
            .attributes
            .insert("manifest_role".to_string(), "file_manifest".to_string());
        section
            .attributes
            .insert("total_files".to_string(), self.total_files.to_string());
        section.attributes.insert(
            "extracted_files".to_string(),
            self.extracted_files.to_string(),
        );
        section.attributes.insert(
            "excluded_files".to_string(),
            self.excluded_files.to_string(),
        );
        section
            .attributes
            .insert("failed_files".to_string(), self.failed_files.to_string());
        section.attributes.insert(
            "unsupported_files".to_string(),
            self.unsupported_files.to_string(),
        );
        section
            .attributes
            .insert("total_bytes".to_string(), self.total_bytes.to_string());
        section.attributes.insert(
            "extracted_bytes".to_string(),
            self.extracted_bytes.to_string(),
        );
        section
            .attributes
            .insert("total_lines".to_string(), self.total_lines.to_string());
        section.attributes.insert(
            "completeness_valid".to_string(),
            self.verify_completeness().to_string(),
        );

        section
    }
}
