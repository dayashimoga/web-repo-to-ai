//! # CDM Schema Validation and Invariant Enforcement
//!
//! Validates structural integrity, referential consistency, cryptographic
//! digest authenticity, tree acyclicity, line/byte ranges, and completeness
//! invariants for [`CdmDocument`].

use crate::error::{CdmError, Result};
use crate::hierarchy::HierarchyNode;
use crate::model::{CdmDocument, ExtractionStatus, SourceType};
use crate::section::CdmSection;
use crate::SCHEMA_VERSION;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;

/// Severity level of a detected validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationSeverity {
    /// Non-fatal diagnostic warning that does not invalidate the document.
    Warning,
    /// Fatal violation of schema rules or invariants.
    Error,
}

impl ValidationSeverity {
    /// Returns true if this issue represents a fatal validation error.
    #[must_use]
    pub const fn is_error(self) -> bool {
        matches!(self, Self::Error)
    }

    /// Returns true if this issue is an advisory warning.
    #[must_use]
    pub const fn is_warning(self) -> bool {
        matches!(self, Self::Warning)
    }

    /// Returns a human-readable uppercase label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Warning => "WARNING",
            Self::Error => "ERROR",
        }
    }
}

impl std::fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A specific violation or diagnostic emitted during validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Machine-readable invariant or rule code (e.g. `SECTION_ID_DUPLICATE`).
    pub code: String,
    /// Human-readable explanation of the issue.
    pub message: String,
    /// Path or coordinate in the document where the issue occurred (e.g. `sections[0].line_range`).
    pub path: String,
    /// Severity classification.
    pub severity: ValidationSeverity,
}

impl ValidationIssue {
    /// Creates a fatal error issue.
    #[must_use]
    pub fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: path.into(),
            severity: ValidationSeverity::Error,
        }
    }

    /// Creates an advisory warning issue.
    #[must_use]
    pub fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: path.into(),
            severity: ValidationSeverity::Warning,
        }
    }
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] ({}) {}: {}",
            self.severity, self.code, self.path, self.message
        )
    }
}

/// Configuration switches governing validator strictness and rule enforcement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// If true, warnings are treated as fatal errors causing `is_valid()` to return false.
    pub strict_mode: bool,
    /// Recomputes BLAKE3 cryptographic hashes for every section body.
    pub verify_hashes: bool,
    /// Validates completeness accounting invariants.
    pub enforce_completeness: bool,
    /// Validates referential integrity for section IDs referenced by hierarchy nodes and relationships.
    pub check_referential_integrity: bool,
    /// Validates acyclicity and unique IDs in the hierarchy tree.
    pub check_tree_acyclicity: bool,
    /// Maximum permitted hierarchy tree depth to guard against malicious recursion/stack overflow.
    pub max_tree_depth: usize,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            verify_hashes: true,
            enforce_completeness: true,
            check_referential_integrity: true,
            check_tree_acyclicity: true,
            max_tree_depth: 64,
        }
    }
}

impl ValidationConfig {
    /// Returns a strict validation configuration where warnings are treated as fatal errors.
    #[must_use]
    pub fn strict() -> Self {
        Self {
            strict_mode: true,
            verify_hashes: true,
            enforce_completeness: true,
            check_referential_integrity: true,
            check_tree_acyclicity: true,
            max_tree_depth: 64,
        }
    }

    /// Returns a permissive validation configuration with minimal checks.
    #[must_use]
    pub fn permissive() -> Self {
        Self {
            strict_mode: false,
            verify_hashes: false,
            enforce_completeness: false,
            check_referential_integrity: false,
            check_tree_acyclicity: true,
            max_tree_depth: 128,
        }
    }

    /// Sets whether strict mode is enabled.
    #[must_use]
    pub const fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Sets whether section BLAKE3 cryptographic hashes are verified.
    #[must_use]
    pub const fn with_verify_hashes(mut self, verify: bool) -> Self {
        self.verify_hashes = verify;
        self
    }

    /// Sets whether completeness accounting is enforced.
    #[must_use]
    pub const fn with_enforce_completeness(mut self, enforce: bool) -> Self {
        self.enforce_completeness = enforce;
        self
    }

    /// Sets whether referential integrity is checked.
    #[must_use]
    pub const fn with_check_referential_integrity(mut self, check: bool) -> Self {
        self.check_referential_integrity = check;
        self
    }

    /// Sets whether hierarchy tree acyclicity is checked.
    #[must_use]
    pub const fn with_check_tree_acyclicity(mut self, check: bool) -> Self {
        self.check_tree_acyclicity = check;
        self
    }

    /// Sets the maximum permitted hierarchy tree depth.
    #[must_use]
    pub const fn with_max_tree_depth(mut self, depth: usize) -> Self {
        self.max_tree_depth = depth;
        self
    }
}

/// Comprehensive report containing all issues discovered during validation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    /// List of all recorded issues (both errors and warnings).
    pub issues: Vec<ValidationIssue>,
    /// Number of content sections inspected.
    pub checked_sections_count: usize,
    /// Number of hierarchy tree nodes inspected.
    pub checked_nodes_count: usize,
    /// Number of relationships inspected.
    pub checked_relationships_count: usize,
    /// Whether strict mode was active during this validation run.
    pub strict_mode: bool,
}

impl ValidationReport {
    /// Returns true if the document satisfies all validation invariants.
    ///
    /// In normal mode, returns true if there are zero errors.
    /// In strict mode, returns true if there are zero errors AND zero warnings.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        if self.strict_mode {
            self.issues.is_empty()
        } else {
            !self.has_errors()
        }
    }

    /// Returns true if at least one fatal error was recorded.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|i| i.severity.is_error())
    }

    /// Returns true if at least one warning was recorded.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        self.issues.iter().any(|i| i.severity.is_warning())
    }

    /// Returns the number of fatal errors.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity.is_error()).count()
    }

    /// Returns the number of warnings.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity.is_warning())
            .count()
    }

    /// Returns an iterator over fatal errors.
    pub fn errors(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues.iter().filter(|i| i.severity.is_error())
    }

    /// Returns an iterator over warnings.
    pub fn warnings(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues.iter().filter(|i| i.severity.is_warning())
    }

    /// Appends a fatal error to the report.
    pub fn add_error(
        &mut self,
        code: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<String>,
    ) {
        self.issues
            .push(ValidationIssue::error(code, message, path));
    }

    /// Appends a warning to the report.
    pub fn add_warning(
        &mut self,
        code: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<String>,
    ) {
        self.issues
            .push(ValidationIssue::warning(code, message, path));
    }

    /// Filters recorded issues by exact code match.
    #[must_use]
    pub fn filter_by_code<'a>(&'a self, code: &'a str) -> Vec<&'a ValidationIssue> {
        self.issues.iter().filter(|i| i.code == code).collect()
    }

    /// Filters recorded issues whose path begins with a given prefix.
    #[must_use]
    pub fn filter_by_path_prefix<'a>(&'a self, prefix: &'a str) -> Vec<&'a ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.path.starts_with(prefix))
            .collect()
    }

    /// Formats a human-readable summary of the validation report.
    #[must_use]
    pub fn summary(&self) -> String {
        if self.is_valid() {
            let warn_text = if self.has_warnings() {
                format!(" with {} warning(s)", self.warning_count())
            } else {
                String::new()
            };
            format!(
                "CDM validation passed{}: {} section(s), {} node(s), {} relationship(s) verified",
                warn_text,
                self.checked_sections_count,
                self.checked_nodes_count,
                self.checked_relationships_count
            )
        } else {
            let mut summary = format!(
                "CDM validation failed: {} error(s), {} warning(s) across {} section(s):\n",
                self.error_count(),
                self.warning_count(),
                self.checked_sections_count
            );
            for issue in &self.issues {
                summary.push_str(&format!("  - {}\n", issue));
            }
            summary
        }
    }
}

/// The Canonical Context Document Model validator engine.
#[derive(Debug, Clone)]
pub struct CdmValidator {
    config: ValidationConfig,
}

impl Default for CdmValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl CdmValidator {
    /// Creates a new validator with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ValidationConfig::default(),
        }
    }

    /// Creates a validator with a custom configuration.
    #[must_use]
    pub const fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Creates a validator in strict mode.
    #[must_use]
    pub fn strict() -> Self {
        Self {
            config: ValidationConfig::strict(),
        }
    }

    /// Creates a validator in permissive mode.
    #[must_use]
    pub fn permissive() -> Self {
        Self {
            config: ValidationConfig::permissive(),
        }
    }

    /// Returns a reference to the active validator configuration.
    #[must_use]
    pub const fn config(&self) -> &ValidationConfig {
        &self.config
    }

    /// Validates a [`CdmDocument`] according to the active configuration.
    #[must_use]
    pub fn validate(&self, doc: &CdmDocument) -> ValidationReport {
        let mut report = ValidationReport {
            strict_mode: self.config.strict_mode,
            checked_sections_count: doc.sections.len(),
            checked_relationships_count: doc.relationships.len(),
            ..Default::default()
        };

        // 1. Schema version check
        self.validate_schema_version(doc, &mut report);

        // 2. Document identity & metadata
        self.validate_identity_and_metadata(doc, &mut report);

        // 3. Sections validation
        let mut section_ids: HashSet<&str> = HashSet::with_capacity(doc.sections.len());
        self.validate_sections(doc, &mut report, &mut section_ids);

        // 4. Hierarchy tree validation
        self.validate_hierarchy(doc, &mut report, &section_ids);

        // 5. Relationships validation
        self.validate_relationships(doc, &mut report, &section_ids);

        // 6. Citations validation
        self.validate_citations(doc, &mut report, &section_ids);

        // 7. Completeness accounting invariants
        self.validate_completeness(doc, &mut report);

        // 8. Warning records validation
        self.validate_warnings_and_exclusions(doc, &mut report);

        report
    }

    /// Static convenience method validating a document with default settings.
    #[must_use]
    pub fn validate_document(doc: &CdmDocument) -> ValidationReport {
        Self::new().validate(doc)
    }

    /// Static convenience method validating a document in strict mode, returning `Ok(())` or `Err(CdmError)`.
    pub fn validate_document_strict(doc: &CdmDocument) -> Result<()> {
        let report = Self::strict().validate(doc);
        if report.is_valid() {
            Ok(())
        } else {
            Err(CdmError::from_report(&report))
        }
    }

    fn validate_schema_version(&self, doc: &CdmDocument, report: &mut ValidationReport) {
        if doc.schema_version.trim().is_empty() {
            report.add_error(
                "SCHEMA_VERSION_EMPTY",
                "schema_version cannot be empty",
                "schema_version",
            );
            return;
        }

        let expected_major = SCHEMA_VERSION.split('.').next().unwrap_or("1");
        let found_major = doc.schema_version.split('.').next().unwrap_or("");

        if expected_major != found_major {
            report.add_error(
                "SCHEMA_VERSION_INCOMPATIBLE",
                format!(
                    "schema_version '{}' is incompatible with supported major version '{}'",
                    doc.schema_version, SCHEMA_VERSION
                ),
                "schema_version",
            );
        }
    }

    fn validate_identity_and_metadata(&self, doc: &CdmDocument, report: &mut ValidationReport) {
        // Document ID cannot be nil
        if doc.document_id.as_uuid().is_nil() {
            report.add_error(
                "DOCUMENT_ID_NIL",
                "document_id cannot be a nil UUID",
                "document_id",
            );
        }

        // Canonical URL must be valid HTTP/HTTPS (or file for document sources)
        match Url::parse(&doc.canonical_url) {
            Ok(parsed) => {
                let scheme = parsed.scheme();
                let is_file_allowed = matches!(
                    doc.source_type,
                    SourceType::DocumentPdf
                        | SourceType::DocumentOcr
                        | SourceType::StructuredDocument
                );
                if scheme != "http" && scheme != "https" && (!is_file_allowed || scheme != "file") {
                    report.add_error(
                        "CANONICAL_URL_SCHEME_INVALID",
                        format!("canonical_url scheme must be 'http' or 'https', found '{scheme}'"),
                        "canonical_url",
                    );
                }
            }
            Err(e) => {
                report.add_error(
                    "CANONICAL_URL_INVALID",
                    format!("canonical_url failed to parse: {e}"),
                    "canonical_url",
                );
            }
        }

        // SourceIdentity checks
        if doc.source_identity.raw_input.trim().is_empty() {
            report.add_error(
                "RAW_INPUT_EMPTY",
                "source_identity.raw_input cannot be empty",
                "source_identity.raw_input",
            );
        }

        if doc.source_identity.canonical_url.trim().is_empty() {
            report.add_error(
                "SOURCE_CANONICAL_URL_EMPTY",
                "source_identity.canonical_url cannot be empty",
                "source_identity.canonical_url",
            );
        }

        match doc.source_type {
            SourceType::WebpageStatic | SourceType::WebpageDynamic => {
                if doc
                    .source_identity
                    .domain
                    .as_deref()
                    .unwrap_or("")
                    .trim()
                    .is_empty()
                {
                    report.add_warning(
                        "DOMAIN_MISSING",
                        "webpage extraction should include a domain in source_identity",
                        "source_identity.domain",
                    );
                }
            }
            SourceType::GitHubPublic | SourceType::GitHubPrivate => {
                let has_owner = !doc
                    .source_identity
                    .repo_owner
                    .as_deref()
                    .unwrap_or("")
                    .trim()
                    .is_empty();
                let has_repo = !doc
                    .source_identity
                    .repo_name
                    .as_deref()
                    .unwrap_or("")
                    .trim()
                    .is_empty();

                if !has_owner || !has_repo {
                    report.add_error(
                        "REPO_COORDINATES_MISSING",
                        "GitHub source must specify non-empty repo_owner and repo_name in source_identity",
                        "source_identity",
                    );
                }
            }
            SourceType::DocumentPdf | SourceType::DocumentOcr | SourceType::StructuredDocument => {}
        }

        // Revision hash formats if present
        if let Some(ref h) = doc.revision.content_hash_blake3 {
            if h.len() != 64 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
                report.add_error(
                    "HASH_FORMAT_INVALID",
                    "revision.content_hash_blake3 must be a 64-character lowercase hex digest",
                    "revision.content_hash_blake3",
                );
            }
        }
        if let Some(ref h) = doc.revision.content_hash_sha256 {
            if h.len() != 64 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
                report.add_error(
                    "HASH_FORMAT_INVALID",
                    "revision.content_hash_sha256 must be a 64-character lowercase hex digest",
                    "revision.content_hash_sha256",
                );
            }
        }

        // Title advisory warning
        if doc
            .metadata
            .title
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
        {
            report.add_warning(
                "TITLE_EMPTY",
                "document metadata should contain a descriptive title",
                "metadata.title",
            );
        }
    }

    fn validate_sections<'a>(
        &self,
        doc: &'a CdmDocument,
        report: &mut ValidationReport,
        section_ids: &mut HashSet<&'a str>,
    ) {
        if doc.sections.is_empty() && doc.extraction_status == ExtractionStatus::Completed {
            report.add_warning(
                "SECTIONS_EMPTY",
                "document status is Completed but sections array is empty",
                "sections",
            );
        }

        for (idx, section) in doc.sections.iter().enumerate() {
            let path = format!("sections[{idx}]");

            // Section ID validation
            if section.section_id.trim().is_empty() {
                report.add_error(
                    "SECTION_ID_EMPTY",
                    "section_id cannot be empty or solely whitespace",
                    format!("{path}.section_id"),
                );
            } else if !section_ids.insert(section.section_id.as_str()) {
                report.add_error(
                    "SECTION_ID_DUPLICATE",
                    format!("duplicate section_id '{}' found", section.section_id),
                    format!("{path}.section_id"),
                );
            }

            // Source URL validation
            if section.source_url.trim().is_empty() {
                report.add_error(
                    "SECTION_URL_EMPTY",
                    "source_url cannot be empty",
                    format!("{path}.source_url"),
                );
            } else if Url::parse(&section.source_url).is_err()
                && !section.source_url.starts_with('/')
                && !section.source_url.starts_with('#')
            {
                report.add_error(
                    "SECTION_URL_INVALID",
                    format!(
                        "source_url '{}' is not a valid URL or path",
                        section.source_url
                    ),
                    format!("{path}.source_url"),
                );
            }

            // Line range checks
            if let Some(ref lr) = section.line_range {
                if lr.start_line == 0 {
                    report.add_error(
                        "LINE_RANGE_ZERO",
                        "start_line cannot be 0 (line numbers are 1-indexed)",
                        format!("{path}.line_range"),
                    );
                }
                if lr.end_line < lr.start_line {
                    report.add_error(
                        "LINE_RANGE_INVERTED",
                        format!(
                            "end_line ({}) cannot be less than start_line ({})",
                            lr.end_line, lr.start_line
                        ),
                        format!("{path}.line_range"),
                    );
                }
            }

            // Byte range checks
            if let Some(ref br) = section.byte_range {
                if br.end_byte < br.start_byte {
                    report.add_error(
                        "BYTE_RANGE_INVERTED",
                        format!(
                            "end_byte ({}) cannot be less than start_byte ({})",
                            br.end_byte, br.start_byte
                        ),
                        format!("{path}.byte_range"),
                    );
                }
            }

            // Content hash verification
            if section.content_hash.len() != 64
                || !section.content_hash.chars().all(|c| c.is_ascii_hexdigit())
            {
                report.add_error(
                    "HASH_FORMAT_INVALID",
                    "section content_hash must be a 64-character lowercase hex digest",
                    format!("{path}.content_hash"),
                );
            } else if self.config.verify_hashes {
                let computed = CdmSection::compute_hash(&section.body);
                if !section.content_hash.eq_ignore_ascii_case(&computed) {
                    report.add_error(
                        "SECTION_HASH_MISMATCH",
                        format!(
                            "content_hash mismatch: recorded '{}', computed '{}'",
                            section.content_hash, computed
                        ),
                        format!("{path}.content_hash"),
                    );
                }
            }

            // Token metrics checks
            if let Some(ref tm) = section.token_metrics {
                let actual_chars = section.body.chars().count();
                if tm.char_count != actual_chars {
                    report.add_warning(
                        "TOKEN_METRICS_CHAR_MISMATCH",
                        format!(
                            "token_metrics.char_count ({}) does not match body char count ({})",
                            tm.char_count, actual_chars
                        ),
                        format!("{path}.token_metrics.char_count"),
                    );
                }
                if tm.word_count > tm.char_count && actual_chars > 0 {
                    report.add_error(
                        "TOKEN_METRICS_INVALID",
                        format!(
                            "word_count ({}) cannot exceed char_count ({})",
                            tm.word_count, tm.char_count
                        ),
                        format!("{path}.token_metrics"),
                    );
                }
            }
        }
    }

    fn validate_hierarchy(
        &self,
        doc: &CdmDocument,
        report: &mut ValidationReport,
        section_ids: &HashSet<&str>,
    ) {
        let mut visited_node_ids: HashSet<String> = HashSet::new();
        let mut node_count = 0;

        self.validate_hierarchy_node(
            &doc.hierarchy,
            0,
            "hierarchy",
            report,
            section_ids,
            &mut visited_node_ids,
            &mut node_count,
        );

        report.checked_nodes_count = node_count;
    }

    #[allow(clippy::too_many_arguments)]
    fn validate_hierarchy_node(
        &self,
        node: &HierarchyNode,
        depth: usize,
        path: &str,
        report: &mut ValidationReport,
        section_ids: &HashSet<&str>,
        visited_ids: &mut HashSet<String>,
        node_count: &mut usize,
    ) {
        *node_count += 1;

        if depth > self.config.max_tree_depth {
            report.add_error(
                "TREE_DEPTH_EXCEEDED",
                format!(
                    "hierarchy tree depth ({depth}) exceeds maximum limit of {}",
                    self.config.max_tree_depth
                ),
                path,
            );
            return;
        }

        if node.id.trim().is_empty() {
            report.add_error(
                "TREE_NODE_ID_EMPTY",
                "hierarchy node id cannot be empty",
                format!("{path}.id"),
            );
        } else if self.config.check_tree_acyclicity && !visited_ids.insert(node.id.clone()) {
            report.add_error(
                "TREE_NODE_ID_DUPLICATE",
                format!(
                    "duplicate hierarchy node id '{}' indicates cycle or collision",
                    node.id
                ),
                format!("{path}.id"),
            );
        }

        // Referential integrity: if node specifies section_id, that section must exist
        if self.config.check_referential_integrity {
            if let Some(ref sec_id) = node.section_id {
                if !section_ids.contains(sec_id.as_str()) {
                    report.add_error(
                        "REFERENTIAL_INTEGRITY_MISSING_SECTION",
                        format!(
                            "hierarchy node '{}' references non-existent section_id '{}'",
                            node.id, sec_id
                        ),
                        format!("{path}.section_id"),
                    );
                }
            }
        }

        for (idx, child) in node.children.iter().enumerate() {
            let child_path = format!("{path}.children[{idx}]");
            self.validate_hierarchy_node(
                child,
                depth + 1,
                &child_path,
                report,
                section_ids,
                visited_ids,
                node_count,
            );
        }
    }

    fn validate_relationships(
        &self,
        doc: &CdmDocument,
        report: &mut ValidationReport,
        section_ids: &HashSet<&str>,
    ) {
        for (idx, rel) in doc.relationships.iter().enumerate() {
            let path = format!("relationships[{idx}]");

            if rel.source_id.trim().is_empty() {
                report.add_error(
                    "RELATIONSHIP_SOURCE_EMPTY",
                    "relationship source_id cannot be empty",
                    format!("{path}.source_id"),
                );
            }

            if rel.target_id.trim().is_empty() {
                report.add_error(
                    "RELATIONSHIP_TARGET_EMPTY",
                    "relationship target_id cannot be empty",
                    format!("{path}.target_id"),
                );
            }

            if self.config.check_referential_integrity {
                if !rel.source_id.trim().is_empty() && !section_ids.contains(rel.source_id.as_str())
                {
                    report.add_error(
                        "RELATIONSHIP_SOURCE_NOT_FOUND",
                        format!(
                            "relationship source_id '{}' does not exist in document sections",
                            rel.source_id
                        ),
                        format!("{path}.source_id"),
                    );
                }

                // If target_id is an internal ID (not an external URL or root-relative path)
                let is_external = rel.target_id.starts_with("http://")
                    || rel.target_id.starts_with("https://")
                    || rel.target_id.starts_with('/');

                if !is_external
                    && !rel.target_id.trim().is_empty()
                    && !section_ids.contains(rel.target_id.as_str())
                {
                    report.add_warning(
                        "RELATIONSHIP_TARGET_NOT_FOUND",
                        format!(
                            "internal relationship target_id '{}' does not match any document section",
                            rel.target_id
                        ),
                        format!("{path}.target_id"),
                    );
                }
            }

            if rel.source_id == rel.target_id && !rel.source_id.is_empty() {
                report.add_warning(
                    "RELATIONSHIP_SELF_REFERENCE",
                    format!("relationship connects entity '{}' to itself", rel.source_id),
                    path,
                );
            }
        }
    }

    fn validate_citations(
        &self,
        doc: &CdmDocument,
        report: &mut ValidationReport,
        section_ids: &HashSet<&str>,
    ) {
        for (idx, cit) in doc.citations.iter().enumerate() {
            let path = format!("citations[{idx}]");

            if cit.citation_id.trim().is_empty() {
                report.add_error(
                    "CITATION_ID_EMPTY",
                    "citation_id cannot be empty",
                    format!("{path}.citation_id"),
                );
            }

            if Url::parse(&cit.url).is_err() {
                report.add_error(
                    "CITATION_URL_INVALID",
                    format!("citation url '{}' is not a valid URL", cit.url),
                    format!("{path}.url"),
                );
            }

            if self.config.check_referential_integrity {
                if let Some(ref sec_id) = cit.section_id {
                    if !section_ids.contains(sec_id.as_str()) {
                        report.add_warning(
                            "CITATION_SECTION_NOT_FOUND",
                            format!(
                                "citation '{}' references non-existent section_id '{}'",
                                cit.citation_id, sec_id
                            ),
                            format!("{path}.section_id"),
                        );
                    }
                }
            }
        }
    }

    fn validate_completeness(&self, doc: &CdmDocument, report: &mut ValidationReport) {
        // Enforce the mathematical invariant formula:
        // discovered = extracted + excluded + failed + unsupported
        if !doc.completeness.formula_valid() {
            report.add_error(
                "COMPLETENESS_INVARIANT_VIOLATED",
                format!(
                    "completeness formula invariant violated: discovered ({}) != extracted ({}) + excluded ({}) + failed ({}) + unsupported ({})",
                    doc.completeness.discovered_count,
                    doc.completeness.extracted_count,
                    doc.completeness.excluded_count,
                    doc.completeness.failed_count,
                    doc.completeness.unsupported_count,
                ),
                "completeness",
            );
        }

        if self.config.enforce_completeness
            && doc.extraction_status == ExtractionStatus::Completed
            && !doc.completeness.is_complete()
        {
            report.add_error(
                "COMPLETENESS_STATUS_INCONSISTENT",
                format!(
                    "document status is Completed but completeness report indicates incomplete conversion (extracted: {}, eligible: {}, failed: {})",
                    doc.completeness.extracted_count,
                    doc.completeness.eligible_count,
                    doc.completeness.failed_count,
                ),
                "completeness",
            );
        }
    }

    fn validate_warnings_and_exclusions(&self, doc: &CdmDocument, report: &mut ValidationReport) {
        for (idx, warning) in doc.warnings.iter().enumerate() {
            let path = format!("warnings[{idx}]");
            if warning.code.trim().is_empty() {
                report.add_error(
                    "WARNING_CODE_EMPTY",
                    "extraction warning code cannot be empty",
                    format!("{path}.code"),
                );
            }
            if warning.message.trim().is_empty() {
                report.add_error(
                    "WARNING_MESSAGE_EMPTY",
                    "extraction warning message cannot be empty",
                    format!("{path}.message"),
                );
            }
        }

        for (idx, excl) in doc.exclusions.iter().enumerate() {
            let path = format!("exclusions[{idx}]");
            if excl.original_path.trim().is_empty() {
                report.add_error(
                    "EXCLUSION_PATH_EMPTY",
                    "exclusion record original_path cannot be empty",
                    format!("{path}.original_path"),
                );
            }
            if let Some(ref details) = excl.details {
                if details.trim().is_empty() {
                    report.add_warning(
                        "EXCLUSION_DETAILS_EMPTY",
                        "exclusion record details string is empty",
                        format!("{path}.details"),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::{CdmDocumentBuilder, CdmSectionBuilder};
    use crate::completeness::CompletenessReport;
    use crate::hierarchy::HierarchyNode;
    use crate::model::{ExtractionStatus, SourceIdentity, SourceType};
    use crate::relationships::{Relationship, RelationshipKind};
    use crate::section::{ByteRange, LineRange, SectionKind};

    fn make_valid_document() -> CdmDocument {
        let identity = SourceIdentity {
            raw_input: "https://example.com/docs".to_string(),
            canonical_url: "https://example.com/docs".to_string(),
            domain: Some("example.com".to_string()),
            repo_owner: None,
            repo_name: None,
            git_ref: None,
            subpath: None,
        };

        let sec = CdmSectionBuilder::new(
            "sec-1",
            "https://example.com/docs/intro",
            SectionKind::Prose,
            "Introduction to the platform documentation.\n",
        )
        .title("Introduction")
        .original_path("intro.md")
        .language("markdown")
        .line_range(1, 2)
        .byte_range(0, 44)
        .build();

        let mut root = HierarchyNode::root("Docs");
        root.add_child(HierarchyNode::file(
            "node-1",
            "intro.md",
            "intro.md",
            Some("sec-1".to_string()),
            Some(44),
            Some(2),
        ));

        CdmDocumentBuilder::new(
            identity,
            SourceType::WebpageStatic,
            "https://example.com/docs",
        )
        .title("Example Docs")
        .hierarchy(root)
        .add_section(sec)
        .completeness(CompletenessReport::single_item_success())
        .build()
    }

    #[test]
    fn test_valid_document_passes_validation() {
        let doc = make_valid_document();
        let report = doc.validation_report();
        assert!(report.is_valid(), "Report should be valid: {:?}", report);
        assert_eq!(report.error_count(), 0);
        assert!(doc.validate().is_ok());
        assert!(doc.validate_strict().is_ok());
    }

    #[test]
    fn test_duplicate_section_ids_detected() {
        let mut doc = make_valid_document();
        let dup_sec = CdmSectionBuilder::new(
            "sec-1", // duplicate ID
            "https://example.com/docs/other",
            SectionKind::Prose,
            "Duplicate section content\n",
        )
        .build();
        doc.sections.push(dup_sec);

        let report = doc.validation_report();
        assert!(!report.is_valid());
        assert!(report.has_errors());
        let dup_issues = report.filter_by_code("SECTION_ID_DUPLICATE");
        assert_eq!(dup_issues.len(), 1);
        assert_eq!(dup_issues[0].path, "sections[1].section_id");
    }

    #[test]
    fn test_line_range_bounds_validation() {
        let mut doc = make_valid_document();
        // Line number 0 is invalid (1-indexed)
        doc.sections[0].line_range = Some(LineRange::new(0, 10));
        let report = doc.validation_report();
        assert!(!report.is_valid());
        assert_eq!(report.filter_by_code("LINE_RANGE_ZERO").len(), 1);

        // Inverted range
        doc.sections[0].line_range = Some(LineRange::new(10, 5));
        let report = doc.validation_report();
        assert!(!report.is_valid());
        assert_eq!(report.filter_by_code("LINE_RANGE_INVERTED").len(), 1);
    }

    #[test]
    fn test_byte_range_bounds_validation() {
        let mut doc = make_valid_document();
        doc.sections[0].byte_range = Some(ByteRange::new(100, 50));
        let report = doc.validation_report();
        assert!(!report.is_valid());
        assert_eq!(report.filter_by_code("BYTE_RANGE_INVERTED").len(), 1);
    }

    #[test]
    fn test_section_hash_tamper_detection() {
        let mut doc = make_valid_document();
        // Tamper with body without updating hash
        doc.sections[0].body = "Tampered body content".to_string();

        let report = doc.validation_report();
        assert!(!report.is_valid());
        assert_eq!(report.filter_by_code("SECTION_HASH_MISMATCH").len(), 1);

        // Permissive config skipping hash verification passes
        let permissive = ValidationConfig::default().with_verify_hashes(false);
        let report_permissive = doc.validate_with_config(&permissive);
        assert!(report_permissive.is_valid());
    }

    #[test]
    fn test_tree_duplicate_node_ids_detected() {
        let mut doc = make_valid_document();
        doc.hierarchy.add_child(HierarchyNode::file(
            "node-1", // duplicate node ID
            "intro2.md",
            "intro2.md",
            Some("sec-1".to_string()),
            None,
            None,
        ));

        let report = doc.validation_report();
        assert!(!report.is_valid());
        assert_eq!(report.filter_by_code("TREE_NODE_ID_DUPLICATE").len(), 1);
    }

    #[test]
    fn test_referential_integrity_missing_section() {
        let mut doc = make_valid_document();
        doc.hierarchy.add_child(HierarchyNode::file(
            "node-ghost",
            "ghost.md",
            "ghost.md",
            Some("non-existent-section-id".to_string()),
            None,
            None,
        ));

        let report = doc.validation_report();
        assert!(!report.is_valid());
        assert_eq!(
            report
                .filter_by_code("REFERENTIAL_INTEGRITY_MISSING_SECTION")
                .len(),
            1
        );
    }

    #[test]
    fn test_relationships_referential_integrity() {
        let mut doc = make_valid_document();
        doc.relationships.push(Relationship {
            source_id: "sec-ghost".to_string(),
            target_id: "sec-1".to_string(),
            kind: RelationshipKind::Imports,
            description: None,
        });

        let report = doc.validation_report();
        assert!(!report.is_valid());
        assert_eq!(
            report.filter_by_code("RELATIONSHIP_SOURCE_NOT_FOUND").len(),
            1
        );
    }

    #[test]
    fn test_completeness_formula_invariant_enforcement() {
        let mut doc = make_valid_document();
        // Break formula: discovered (10) != extracted (1) + excluded (0) + failed (0) + unsupported (0)
        doc.completeness.discovered_count = 10;

        let report = doc.validation_report();
        assert!(!report.is_valid());
        assert_eq!(
            report
                .filter_by_code("COMPLETENESS_INVARIANT_VIOLATED")
                .len(),
            1
        );
    }

    #[test]
    fn test_completed_status_with_failed_items_rejected() {
        let mut doc = make_valid_document();
        // 1 discovered, 1 eligible, 0 extracted, 1 failed
        doc.completeness = CompletenessReport::new(1, 1, 0, 0, 1, 0);
        doc.extraction_status = ExtractionStatus::Completed;

        let report = doc.validation_report();
        assert!(!report.is_valid());
        assert_eq!(
            report
                .filter_by_code("COMPLETENESS_STATUS_INCONSISTENT")
                .len(),
            1
        );
    }

    #[test]
    fn test_strict_mode_escalates_warnings() {
        let mut doc = make_valid_document();
        // Missing title triggers a warning
        doc.metadata.title = None;

        let normal_report = doc.validation_report();
        assert!(normal_report.is_valid(), "Normal mode allows warnings");
        assert!(normal_report.has_warnings());
        assert_eq!(normal_report.filter_by_code("TITLE_EMPTY").len(), 1);

        // Strict mode treats warnings as fatal
        let strict_report = doc.validate_with_config(&ValidationConfig::strict());
        assert!(!strict_report.is_valid(), "Strict mode rejects warnings");
        assert!(doc.validate_strict().is_err());
    }

    #[test]
    fn test_tree_depth_exceeded_defense() {
        let mut doc = make_valid_document();
        // Build nested directories: root -> d1 -> d2 -> d3 -> d4
        let mut d4 = HierarchyNode::directory("d-4", "d-4", "d-4");
        d4.add_child(HierarchyNode::file(
            "f-5",
            "f-5",
            "f-5",
            Some("sec-1".to_string()),
            None,
            None,
        ));
        let mut d3 = HierarchyNode::directory("d-3", "d-3", "d-3");
        d3.add_child(d4);
        let mut d2 = HierarchyNode::directory("d-2", "d-2", "d-2");
        d2.add_child(d3);
        let mut d1 = HierarchyNode::directory("d-1", "d-1", "d-1");
        d1.add_child(d2);
        doc.hierarchy.add_child(d1);

        let shallow_config = ValidationConfig::default().with_max_tree_depth(2);
        let report = doc.validate_with_config(&shallow_config);
        assert!(!report.is_valid());
        assert_eq!(report.filter_by_code("TREE_DEPTH_EXCEEDED").len(), 1);
    }
}
