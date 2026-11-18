//! # Automated Code-Review Package Generation
//!
//! Provides targeted AI code-review package assembly, review prioritization tiering,
//! review checklist synthesis, and end-to-end Canonical Context Document Model
//! ([`CdmDocument`]) packaging.

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{
    CdmDocument, CdmDocumentBuilder, CdmSectionBuilder, SectionKind, SectionTokenMetrics,
    SourceIdentity, SourceType,
};

use crate::architecture::RepoArchitectureContext;
use crate::classification::{FileCategory, TreeClassificationReport};
use crate::config_test::ConfigTestExtractionReport;
use crate::dependency::DependencyExtractionReport;
use crate::documentation::DocExtractionReport;
use crate::manifest::RepoFileManifest;
use crate::metadata::GitHubRepositoryMetadata;
use crate::source::{compute_blake3_hash, SourceExtractionReport};
use crate::tree_generator::{format_byte_size, RepoTree};

/// Specialized review focus or perspective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReviewFocus {
    /// Comprehensive overall review of repository architecture, code, and tests.
    #[default]
    FullRepository,
    /// Targeted security audit (SSRF, credentials, crypto, input validation, memory safety).
    SecurityAudit,
    /// High-level architecture, subsystem boundaries, decoupling, and dependency analysis.
    ArchitectureReview,
    /// Code quality, conventions, test coverage, and performance bottlenecks.
    QualityAudit,
    /// Focused review of modified changesets or key entry points.
    DiffChangeset,
}

impl ReviewFocus {
    /// Returns the snake_case string identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FullRepository => "full_repository",
            Self::SecurityAudit => "security_audit",
            Self::ArchitectureReview => "architecture_review",
            Self::QualityAudit => "quality_audit",
            Self::DiffChangeset => "diff_changeset",
        }
    }

    /// Returns a human-readable display title.
    #[must_use]
    pub const fn display_title(self) -> &'static str {
        match self {
            Self::FullRepository => "Full Repository Code Review",
            Self::SecurityAudit => "Application Security & Defense Audit",
            Self::ArchitectureReview => "System Architecture & Decoupling Review",
            Self::QualityAudit => "Code Quality & Engineering Standards Audit",
            Self::DiffChangeset => "Changeset & Entry Points Review",
        }
    }
}

/// Evaluated priority tier for reviewing an individual file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewPriorityTier {
    /// Priority 1: Critical security, authentication, cryptography, or core runtime entry points.
    Critical = 1,
    /// Priority 2: Primary API services, domain logic, and core crate modules.
    High = 2,
    /// Priority 3: Secondary utilities, configurations, and data models.
    Medium = 3,
    /// Priority 4: Test suites, test fixtures, and documentation.
    Low = 4,
    /// Priority 5: Excluded or non-reviewable files (lockfiles, vendor shims, binaries).
    Excluded = 5,
}

impl ReviewPriorityTier {
    /// Returns the snake_case string identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Excluded => "excluded",
        }
    }

    /// Returns a visual badge label for markdown tables.
    #[must_use]
    pub const fn badge(self) -> &'static str {
        match self {
            Self::Critical => "[CRITICAL]",
            Self::High => "[HIGH]",
            Self::Medium => "[MEDIUM]",
            Self::Low => "[LOW]",
            Self::Excluded => "[EXCLUDED]",
        }
    }
}

/// A prioritized repository file selected for inclusion in the code-review package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrioritizedReviewFile {
    /// Relative repository path (e.g. `services/api/src/main.rs`).
    pub path: String,
    /// 40-character Git object SHA-1.
    pub git_sha: String,
    /// Evaluated priority tier.
    pub priority_tier: ReviewPriorityTier,
    /// Functional file category.
    pub category: FileCategory,
    /// Programming language.
    pub language: Option<String>,
    /// Byte size in bytes.
    pub byte_size: usize,
    /// Total lines in file content.
    pub line_count: usize,
    /// Rational justification for the assigned priority tier.
    pub priority_reason: String,
    /// Full normalized text content, if acquired.
    pub content: Option<String>,
    /// Cryptographic BLAKE3 content digest.
    pub blake3_hash: Option<String>,
}

/// An individual review prompt checklist item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewChecklistItem {
    /// Functional category (e.g. "SSRF & Network Defense", "Memory Safety").
    pub category: String,
    /// Actionable review verification instruction for the reviewer.
    pub instruction: String,
    /// Severity level ("Mandatory", "Advisory", "Best Practice").
    pub severity: String,
}

/// Configuration governing code-review package assembly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeReviewPackageConfig {
    /// Selected review focus.
    pub focus: ReviewFocus,
    /// Maximum count of source code files to include in full detail.
    pub max_files: usize,
    /// Maximum total lines ceiling across included code files.
    pub max_total_lines: usize,
    /// Whether to include the rendered directory tree diagram.
    pub include_tree: bool,
    /// Whether to include the file manifest table.
    pub include_manifest: bool,
    /// Whether to include the architecture overview.
    pub include_architecture: bool,
    /// Whether to include the review checklist and prompt instructions.
    pub include_checklist: bool,
}

impl Default for CodeReviewPackageConfig {
    fn default() -> Self {
        Self {
            focus: ReviewFocus::FullRepository,
            max_files: 50,
            max_total_lines: 10_000,
            include_tree: true,
            include_manifest: true,
            include_architecture: true,
            include_checklist: true,
        }
    }
}

impl CodeReviewPackageConfig {
    /// Creates a new configuration with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the review focus.
    #[must_use]
    pub const fn with_focus(mut self, focus: ReviewFocus) -> Self {
        self.focus = focus;
        self
    }

    /// Sets the maximum number of code files to include.
    #[must_use]
    pub const fn with_max_files(mut self, max_files: usize) -> Self {
        self.max_files = max_files;
        self
    }

    /// Sets the maximum line count budget across included code files.
    #[must_use]
    pub const fn with_max_total_lines(mut self, max_lines: usize) -> Self {
        self.max_total_lines = max_lines;
        self
    }
}

/// Consolidated, self-contained AI code-review package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeReviewPackage {
    /// Selected review focus.
    pub focus: ReviewFocus,
    /// Target repository identifier (e.g. `owner/repo`).
    pub target_repo: String,
    /// Target commit SHA-1 or pinned revision.
    pub pinned_sha: Option<String>,
    /// Git reference branch or tag name.
    pub target_ref: Option<String>,
    /// Total files discovered in the source repository.
    pub total_repo_files: usize,
    /// Number of files selected for detailed review.
    pub reviewed_file_count: usize,
    /// Total lines spanned across selected review files.
    pub reviewed_line_count: usize,
    /// Total bytes spanned across selected review files.
    pub reviewed_byte_size: usize,
    /// Prioritized list of review files ordered from Critical to Low.
    pub files: Vec<PrioritizedReviewFile>,
    /// Tailored review checklist items for this package.
    pub checklist: Vec<ReviewChecklistItem>,
    /// Rendered directory tree diagram, if included.
    pub tree_rendered: Option<String>,
    /// Formatted file manifest table, if included.
    pub manifest_summary: Option<String>,
    /// Formatted architecture overview, if included.
    pub architecture_summary: Option<String>,
}

impl CodeReviewPackage {
    /// Assembles a complete, prioritized `CodeReviewPackage` from repository artifacts and extraction reports.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn assemble(
        config: &CodeReviewPackageConfig,
        classification: &TreeClassificationReport,
        tree: Option<&RepoTree>,
        manifest: Option<&RepoFileManifest>,
        architecture: Option<&RepoArchitectureContext>,
        sources: Option<&SourceExtractionReport>,
        docs: Option<&DocExtractionReport>,
        dependencies: Option<&DependencyExtractionReport>,
        configs_tests: Option<&ConfigTestExtractionReport>,
        metadata: Option<&GitHubRepositoryMetadata>,
        target_ref: Option<String>,
        pinned_sha: Option<String>,
    ) -> Self {
        let target_repo = metadata
            .map(|m| m.full_name.clone())
            .unwrap_or_else(|| "repository".to_string());

        let total_repo_files = classification.files.len();

        // 1. Prioritize and select files
        let mut all_prioritized = Self::prioritize_files(
            classification,
            config.focus,
            sources,
            docs,
            dependencies,
            configs_tests,
        );

        // Sort by priority tier ascending (Critical=1 -> Low=4), then by line count descending, then path
        all_prioritized.sort_by(|a, b| {
            a.priority_tier
                .cmp(&b.priority_tier)
                .then_with(|| b.line_count.cmp(&a.line_count))
                .then_with(|| a.path.cmp(&b.path))
        });

        // Filter according to budgets
        let mut reviewed_files = Vec::new();
        let mut current_lines = 0;

        for file in all_prioritized {
            if file.priority_tier == ReviewPriorityTier::Excluded {
                continue;
            }
            if reviewed_files.len() >= config.max_files {
                break;
            }
            if current_lines + file.line_count > config.max_total_lines
                && !reviewed_files.is_empty()
            {
                // If it's Critical tier, we still attempt to include it if total files is low
                if file.priority_tier != ReviewPriorityTier::Critical {
                    continue;
                }
            }

            current_lines += file.line_count;
            reviewed_files.push(file);
        }

        let reviewed_file_count = reviewed_files.len();
        let reviewed_line_count = current_lines;
        let reviewed_byte_size = reviewed_files.iter().map(|f| f.byte_size).sum();

        // 2. Generate review checklist
        let checklist = if config.include_checklist {
            Self::generate_checklist(config.focus)
        } else {
            Vec::new()
        };

        // 3. Tree rendering
        let tree_rendered = if config.include_tree {
            tree.map(|t| t.render_default())
        } else {
            None
        };

        // 4. Manifest summary
        let manifest_summary = if config.include_manifest {
            manifest.map(|m| m.to_markdown_table())
        } else {
            None
        };

        // 5. Architecture summary
        let architecture_summary = if config.include_architecture {
            architecture.map(|a| a.to_markdown())
        } else {
            None
        };

        Self {
            focus: config.focus,
            target_repo,
            pinned_sha,
            target_ref,
            total_repo_files,
            reviewed_file_count,
            reviewed_line_count,
            reviewed_byte_size,
            files: reviewed_files,
            checklist,
            tree_rendered,
            manifest_summary,
            architecture_summary,
        }
    }

    /// Evaluates priority tier and collects content for each repository file.
    fn prioritize_files(
        classification: &TreeClassificationReport,
        focus: ReviewFocus,
        sources: Option<&SourceExtractionReport>,
        _docs: Option<&DocExtractionReport>,
        _dependencies: Option<&DependencyExtractionReport>,
        configs_tests: Option<&ConfigTestExtractionReport>,
    ) -> Vec<PrioritizedReviewFile> {
        let mut results = Vec::new();

        for file in &classification.files {
            if !file.is_eligible {
                continue;
            }

            let (tier, reason) = Self::evaluate_tier(&file.path, file.category, focus);

            // Retrieve extracted content and metrics
            let mut content = None;
            let mut line_count = 0;
            let mut byte_size = file.size.unwrap_or(0) as usize;
            let mut blake3_hash = None;

            if let Some(src_report) = sources {
                if let Some(src) = src_report.extracted.iter().find(|e| e.path == file.path) {
                    content = Some(src.content.clone());
                    line_count = src.line_count;
                    byte_size = src.byte_size;
                    blake3_hash = Some(src.blake3_hash.clone());
                }
            }

            if content.is_none() {
                if let Some(cfg_report) = configs_tests {
                    if let Some(cfg) = cfg_report
                        .extracted_configs
                        .iter()
                        .find(|e| e.path == file.path)
                    {
                        content = Some(cfg.content.clone());
                        line_count = cfg.line_count;
                        byte_size = cfg.byte_size;
                        blake3_hash = Some(cfg.blake3_hash.clone());
                    } else if let Some(t) = cfg_report
                        .extracted_tests
                        .iter()
                        .find(|e| e.path == file.path)
                    {
                        content = Some(t.content.clone());
                        line_count = t.line_count;
                        byte_size = t.byte_size;
                        blake3_hash = Some(t.blake3_hash.clone());
                    }
                }
            }

            results.push(PrioritizedReviewFile {
                path: file.path.clone(),
                git_sha: file.sha.clone(),
                priority_tier: tier,
                category: file.category,
                language: file.language.clone(),
                byte_size,
                line_count,
                priority_reason: reason,
                content,
                blake3_hash,
            });
        }

        results
    }

    /// Evaluates priority tier and rationale for a specific path and category.
    fn evaluate_tier(
        path: &str,
        category: FileCategory,
        focus: ReviewFocus,
    ) -> (ReviewPriorityTier, String) {
        let lower = path.to_ascii_lowercase();

        // 1. Excluded categories
        if category == FileCategory::GeneratedOrLockfile
            || category == FileCategory::Binary
            || category == FileCategory::Oversized
        {
            return (
                ReviewPriorityTier::Excluded,
                "Excluded artifact, generated code, or non-text binary".to_string(),
            );
        }

        // 2. Security-sensitive keywords
        let security_keywords = [
            "security",
            "auth",
            "crypto",
            "token",
            "secret",
            "ssrf",
            "credential",
            "password",
            "cert",
            "tls",
            "url-security",
            "url_security",
        ];
        for kw in &security_keywords {
            if lower.contains(kw) {
                return (
                    ReviewPriorityTier::Critical,
                    format!("Contains security-sensitive keyword `{kw}`"),
                );
            }
        }

        // 3. Execution entry points
        if lower == "src/main.rs"
            || lower.ends_with("/src/main.rs")
            || lower.contains("services/api")
            || lower == "main.go"
        {
            return (
                ReviewPriorityTier::Critical,
                "Primary application execution or API entry point".to_string(),
            );
        }

        // 4. Focus-specific evaluations
        match focus {
            ReviewFocus::SecurityAudit => {
                if lower.contains("network")
                    || lower.contains("http")
                    || lower.contains("fetch")
                    || lower.contains("proxy")
                    || lower.contains("dns")
                    || lower.contains("storage")
                {
                    return (
                        ReviewPriorityTier::Critical,
                        "Network, I/O, or storage interface under Security Audit".to_string(),
                    );
                }
                if category == FileCategory::Configuration {
                    return (
                        ReviewPriorityTier::High,
                        "Configuration or container specification under Security Audit".to_string(),
                    );
                }
            }
            ReviewFocus::ArchitectureReview => {
                if lower == "src/lib.rs"
                    || lower.ends_with("/src/lib.rs")
                    || lower.contains("crates/core")
                    || lower.contains("crates/cdm")
                {
                    return (
                        ReviewPriorityTier::Critical,
                        "Core domain interface or CDM model under Architecture Review".to_string(),
                    );
                }
            }
            ReviewFocus::QualityAudit => {
                if category == FileCategory::TestFixture {
                    return (
                        ReviewPriorityTier::High,
                        "Test suite verifying quality and invariants under Quality Audit"
                            .to_string(),
                    );
                }
            }
            ReviewFocus::FullRepository | ReviewFocus::DiffChangeset => {}
        }

        // 5. Library roots and core services
        if lower.ends_with("/lib.rs") || lower == "src/lib.rs" {
            return (
                ReviewPriorityTier::High,
                "Library root public API definition".to_string(),
            );
        }

        if lower.starts_with("crates/") || lower.starts_with("services/") {
            return (
                ReviewPriorityTier::High,
                "Core business logic within workspace subsystem".to_string(),
            );
        }

        // 6. Web frontend routes
        if lower.contains("+page.svelte") || lower.contains("app/page.tsx") {
            return (
                ReviewPriorityTier::High,
                "Primary interactive UI view route".to_string(),
            );
        }

        // 7. Configs & Tooling
        if category == FileCategory::Configuration {
            return (
                ReviewPriorityTier::Medium,
                "Repository configuration, linter, or CI/CD workflow".to_string(),
            );
        }

        // 8. Tests
        if category == FileCategory::TestFixture {
            return (
                ReviewPriorityTier::Low,
                "Automated test suite or validation fixture".to_string(),
            );
        }

        // 9. Documentation
        if category == FileCategory::Documentation {
            return (
                ReviewPriorityTier::Low,
                "Technical documentation or guide".to_string(),
            );
        }

        (
            ReviewPriorityTier::Medium,
            "General source code module".to_string(),
        )
    }

    /// Generates review guidance checklist tailored to the review focus.
    fn generate_checklist(focus: ReviewFocus) -> Vec<ReviewChecklistItem> {
        let mut list = Vec::new();

        match focus {
            ReviewFocus::SecurityAudit => {
                list.push(ReviewChecklistItem {
                    category: "SSRF & Network Defense".to_string(),
                    instruction: "Verify strict scheme allowlisting, private IP range blocks (RFC1918, loopback, link-local, cloud metadata), and DNS rebinding protections.".to_string(),
                    severity: "Mandatory".to_string(),
                });
                list.push(ReviewChecklistItem {
                    category: "Secret & Credential Redaction".to_string(),
                    instruction: "Ensure tokens, passwords, and API keys are never logged in plaintext, exposed in errors, or embedded in export bundles.".to_string(),
                    severity: "Mandatory".to_string(),
                });
                list.push(ReviewChecklistItem {
                    category: "Path Traversal & Injection Defense".to_string(),
                    instruction: "Verify path normalization and traversal rejection (`..`), zero shell execution, and sanitized file writes.".to_string(),
                    severity: "Mandatory".to_string(),
                });
                list.push(ReviewChecklistItem {
                    category: "Resource Bounds & DoS Prevention".to_string(),
                    instruction: "Confirm bounded streaming reads, maximum payload byte ceilings, decompression bomb protections, and request timeouts.".to_string(),
                    severity: "Mandatory".to_string(),
                });
            }
            ReviewFocus::ArchitectureReview => {
                list.push(ReviewChecklistItem {
                    category: "Decoupling & Modularity".to_string(),
                    instruction: "Verify clean crate boundaries without circular dependencies or leaky abstractions.".to_string(),
                    severity: "Mandatory".to_string(),
                });
                list.push(ReviewChecklistItem {
                    category: "Canonical Representation".to_string(),
                    instruction: "Ensure all source transformations flow through the Canonical Context Document Model (CDM) intermediate representation.".to_string(),
                    severity: "Mandatory".to_string(),
                });
                list.push(ReviewChecklistItem {
                    category: "Workspace Hygiene".to_string(),
                    instruction: "Check that single-responsibility components and entry points follow explicit naming and separation conventions.".to_string(),
                    severity: "Advisory".to_string(),
                });
            }
            ReviewFocus::QualityAudit => {
                list.push(ReviewChecklistItem {
                    category: "Algorithmic Complexity".to_string(),
                    instruction: "Ensure parsing and tokenization algorithms maintain linear O(N) or O(N log N) deterministic throughput.".to_string(),
                    severity: "Mandatory".to_string(),
                });
                list.push(ReviewChecklistItem {
                    category: "Error Contracts & Diagnostics".to_string(),
                    instruction: "Verify exhaustive error handling (e.g. thiserror/RFC 7807) without panics, unwrap() in production paths, or opaque error strings.".to_string(),
                    severity: "Mandatory".to_string(),
                });
                list.push(ReviewChecklistItem {
                    category: "Test Coverage & Invariants".to_string(),
                    instruction: "Check unit, integration, and golden dataset test coverage for all edge cases and boundary conditions.".to_string(),
                    severity: "Mandatory".to_string(),
                });
            }
            ReviewFocus::FullRepository | ReviewFocus::DiffChangeset => {
                list.push(ReviewChecklistItem {
                    category: "Security & Defenses".to_string(),
                    instruction: "Inspect input validation, authentication boundaries, and denial-of-service / resource limit controls.".to_string(),
                    severity: "Mandatory".to_string(),
                });
                list.push(ReviewChecklistItem {
                    category: "Architecture & Decoupling".to_string(),
                    instruction: "Verify module boundaries, circular dependency prevention, and adherence to workspace conventions.".to_string(),
                    severity: "Mandatory".to_string(),
                });
                list.push(ReviewChecklistItem {
                    category: "Error Handling & Invariants".to_string(),
                    instruction: "Ensure explicit error contracts without unhandled exceptions or hidden panics.".to_string(),
                    severity: "Mandatory".to_string(),
                });
                list.push(ReviewChecklistItem {
                    category: "Test Coverage".to_string(),
                    instruction:
                        "Verify unit, integration, and golden test coverage for critical paths."
                            .to_string(),
                    severity: "Best Practice".to_string(),
                });
            }
        }

        list
    }

    /// Formats the code-review package as a comprehensive, self-contained GitHub-Flavored Markdown dossier.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "# Code Review Package: {}\n\n",
            self.focus.display_title()
        ));

        out.push_str("## Review Package Metadata\n\n");
        out.push_str("| Attribute | Value |\n");
        out.push_str("| :--- | :--- |\n");
        out.push_str(&format!("| Target Repository | `{}` |\n", self.target_repo));
        if let Some(ref r) = self.target_ref {
            out.push_str(&format!("| Target Reference | `{r}` |\n"));
        }
        if let Some(ref sha) = self.pinned_sha {
            let short = if sha.len() >= 7 { &sha[..7] } else { sha };
            out.push_str(&format!("| Pinned Commit SHA | `{short}` (`{sha}`) |\n"));
        }
        out.push_str(&format!("| Review Focus | `{}` |\n", self.focus.as_str()));
        out.push_str(&format!(
            "| Total Repository Files | {} |\n",
            self.total_repo_files
        ));
        out.push_str(&format!(
            "| Files Selected for Review | {} |\n",
            self.reviewed_file_count
        ));
        out.push_str(&format!(
            "| Total Lines Under Review | {} |\n",
            self.reviewed_line_count
        ));
        out.push_str(&format!(
            "| Total Content Volume | {} ({}) |\n\n",
            self.reviewed_byte_size,
            format_byte_size(self.reviewed_byte_size as u64)
        ));

        // Review Checklist
        if !self.checklist.is_empty() {
            out.push_str("## Reviewer Instructions & Verification Checklist\n\n");
            out.push_str("| Category | Severity | Verification Instruction |\n");
            out.push_str("| :--- | :---: | :--- |\n");
            for item in &self.checklist {
                out.push_str(&format!(
                    "| **{}** | `{}` | {} |\n",
                    item.category, item.severity, item.instruction
                ));
            }
            out.push('\n');
        }

        // Prioritized Files Table
        if !self.files.is_empty() {
            out.push_str("## Prioritized Review Files Index\n\n");
            out.push_str("| Priority | Path | Category | Language | Lines | Size | Rationale |\n");
            out.push_str("| :---: | :--- | :--- | :--- | :---: | :---: | :--- |\n");
            for file in &self.files {
                let lang = file.language.as_deref().unwrap_or("-");
                out.push_str(&format!(
                    "| `{}` | `{}` | {} | {} | {} | {} | {} |\n",
                    file.priority_tier.as_str(),
                    file.path,
                    file.category.as_str(),
                    lang,
                    file.line_count,
                    format_byte_size(file.byte_size as u64),
                    file.priority_reason
                ));
            }
            out.push('\n');
        }

        // Architecture overview if present
        if let Some(ref arch_md) = self.architecture_summary {
            out.push_str("---\n\n");
            out.push_str(arch_md);
            out.push('\n');
        }

        // Directory tree if present
        if let Some(ref tree_str) = self.tree_rendered {
            out.push_str("---\n\n## Repository Directory Tree\n\n```text\n");
            out.push_str(tree_str);
            out.push_str("\n```\n\n");
        }

        // File Manifest if present
        if let Some(ref manifest_md) = self.manifest_summary {
            out.push_str("---\n\n");
            out.push_str(manifest_md);
            out.push('\n');
        }

        // Reviewed Code Files Content
        if !self.files.is_empty() {
            out.push_str("---\n\n# Prioritized Code Review Files\n\n");
            for file in &self.files {
                if let Some(ref code) = file.content {
                    let lang = file.language.as_deref().unwrap_or("text");
                    out.push_str(&format!(
                        "## File: `{}` {}\n\n",
                        file.path,
                        file.priority_tier.badge()
                    ));
                    out.push_str(&format!(
                        "- **Category:** `{}` | **Lines:** {} | **Size:** {}\n",
                        file.category.as_str(),
                        file.line_count,
                        format_byte_size(file.byte_size as u64)
                    ));
                    out.push_str(&format!(
                        "- **Priority Rationale:** {}\n\n",
                        file.priority_reason
                    ));
                    out.push_str(&format!("```{lang}\n"));
                    out.push_str(code);
                    if !code.ends_with('\n') {
                        out.push('\n');
                    }
                    out.push_str("```\n\n");
                }
            }
        }

        out
    }

    /// Serializes this code review package into a pretty-printed JSON string.
    ///
    /// # Errors
    /// Returns `serde_json::Error` if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serializes this code review package into a compact single-line JSON string.
    ///
    /// # Errors
    /// Returns `serde_json::Error` if serialization fails.
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Converts this code review package into an end-to-end Canonical Context Document Model [`CdmDocument`].
    #[must_use]
    pub fn to_cdm_document(&self, doc_canonical_url: impl Into<String>) -> CdmDocument {
        let canonical_url = doc_canonical_url.into();
        let mut source_identity = SourceIdentity::new(&self.target_repo, &canonical_url);
        let (owner, repo) = if let Some((o, r)) = self.target_repo.split_once('/') {
            (o.to_string(), r.to_string())
        } else {
            ("unknown".to_string(), self.target_repo.clone())
        };
        source_identity.repo_owner = Some(owner);
        source_identity.repo_name = Some(repo);
        source_identity.git_ref = self.target_ref.clone();

        let mut builder =
            CdmDocumentBuilder::new(source_identity, SourceType::GitHubPublic, &canonical_url)
                .title(format!("Code Review Package: {}", self.target_repo))
                .content_type("application/x-cdm-review-package");

        // Section 1: Review Dossier Frontmatter / Overview
        let overview_md = format!(
            "# Code Review Package: {}\n\n- Focus: `{}`\n- Reviewed Files: {}\n- Total Lines: {}\n",
            self.target_repo,
            self.focus.as_str(),
            self.reviewed_file_count,
            self.reviewed_line_count
        );
        let sec_overview = CdmSectionBuilder::new(
            "sec-review-overview",
            &canonical_url,
            SectionKind::Frontmatter,
            &overview_md,
        )
        .title("Code Review Dossier Overview")
        .original_path("REVIEW_OVERVIEW.md")
        .content_hash(compute_blake3_hash(overview_md.as_bytes()))
        .token_metrics(SectionTokenMetrics::compute_baseline(&overview_md))
        .build();
        builder = builder.add_section(sec_overview);

        // Section 2: Directory Tree if present
        if let Some(ref tree_str) = self.tree_rendered {
            let sec_tree = CdmSectionBuilder::new(
                "sec-review-tree",
                &canonical_url,
                SectionKind::Manifest,
                tree_str,
            )
            .title("Repository Directory Tree")
            .original_path("TREE.md")
            .content_hash(compute_blake3_hash(tree_str.as_bytes()))
            .token_metrics(SectionTokenMetrics::compute_baseline(tree_str))
            .build();
            builder = builder.add_section(sec_tree);
        }

        // Section 3: File Manifest if present
        if let Some(ref manifest_md) = self.manifest_summary {
            let sec_manifest = CdmSectionBuilder::new(
                "sec-review-manifest",
                &canonical_url,
                SectionKind::Manifest,
                manifest_md,
            )
            .title("Repository File Manifest")
            .original_path("MANIFEST.md")
            .content_hash(compute_blake3_hash(manifest_md.as_bytes()))
            .token_metrics(SectionTokenMetrics::compute_baseline(manifest_md))
            .build();
            builder = builder.add_section(sec_manifest);
        }

        // Section 4: Architecture Overview if present
        if let Some(ref arch_md) = self.architecture_summary {
            let sec_arch = CdmSectionBuilder::new(
                "sec-review-architecture",
                &canonical_url,
                SectionKind::Prose,
                arch_md,
            )
            .title("System Architecture Overview")
            .original_path("ARCHITECTURE.md")
            .content_hash(compute_blake3_hash(arch_md.as_bytes()))
            .token_metrics(SectionTokenMetrics::compute_baseline(arch_md))
            .build();
            builder = builder.add_section(sec_arch);
        }

        // Sections 5..N: Prioritized Code Files
        for (i, file) in self.files.iter().enumerate() {
            if let Some(ref code) = file.content {
                let sec_id = format!("sec-code-{:03}", i + 1);
                let lang = file.language.as_deref().unwrap_or("text");
                let hash_str = compute_blake3_hash(code.as_bytes());

                let mut sec =
                    CdmSectionBuilder::new(sec_id, &canonical_url, SectionKind::CodeFile, code)
                        .title(&file.path)
                        .original_path(&file.path)
                        .line_range(1, (file.line_count as u32).max(1))
                        .byte_range(0, file.byte_size as u64)
                        .language(lang)
                        .content_hash(&hash_str)
                        .token_metrics(SectionTokenMetrics::compute_baseline(code))
                        .build();

                sec.attributes.insert(
                    "priority_tier".to_string(),
                    file.priority_tier.as_str().to_string(),
                );
                sec.attributes
                    .insert("git_blob_sha".to_string(), file.git_sha.clone());
                sec.attributes
                    .insert("priority_reason".to_string(), file.priority_reason.clone());

                builder = builder.add_section(sec);
            }
        }

        // Section Last: Review Checklist & Instructions
        if !self.checklist.is_empty() {
            let mut checklist_md = String::from("## Reviewer Verification Checklist\n\n");
            for item in &self.checklist {
                checklist_md.push_str(&format!(
                    "- [{}] **{}**: {}\n",
                    item.severity, item.category, item.instruction
                ));
            }

            let sec_checklist = CdmSectionBuilder::new(
                "sec-review-checklist",
                &canonical_url,
                SectionKind::Admonition,
                &checklist_md,
            )
            .title("Review Verification Checklist")
            .original_path("CHECKLIST.md")
            .content_hash(compute_blake3_hash(checklist_md.as_bytes()))
            .token_metrics(SectionTokenMetrics::compute_baseline(&checklist_md))
            .build();
            builder = builder.add_section(sec_checklist);
        }

        builder.build()
    }
}
