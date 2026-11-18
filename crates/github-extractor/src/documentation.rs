//! # GitHub Documentation Extraction
//!
//! Provides specialized acquisition, categorization, and structuring of repository documentation:
//! - README files (root and subpackages)
//! - Architectural specifications and decision records (ADRs)
//! - Licensing terms and legal notices
//! - Contributing guidelines and development manuals
//! - Release changelogs and version histories
//! - Security advisories and vulnerability reporting policies
//! - User guides, installation walkthroughs, and tutorials
//!
//! Extracts document titles from Markdown headings or YAML frontmatter and converts
//! documentation into Canonical Context Document Model ([`CdmSection`]) prose sections.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{
    CdmSection, CdmSectionBuilder, CompletenessReport, SectionKind, SectionTokenMetrics,
};

use crate::classification::ClassifiedFile;
use crate::error::GitHubExtractorError;
use crate::source::{
    compute_blake3_hash, compute_git_blob_sha, compute_sha256_hash, FailedSourceExtraction,
    GitBlob, SourceExtractionConfig,
};

/// Canonical functional classification of repository documentation files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocType {
    /// Root or subpackage README overview.
    Readme,
    /// Architecture specifications, system designs, or decision records (ADRs).
    Architecture,
    /// Release history, changelogs, and release notes.
    Changelog,
    /// Contribution guidelines, development workflows, and contributor guides.
    Contributing,
    /// Software license, copyright, and legal attribution notices.
    License,
    /// Security policies, vulnerability disclosure guides, and security advisories.
    Security,
    /// Community code of conduct.
    CodeOfConduct,
    /// User guides, installation walkthroughs, quickstarts, and tutorials.
    Guide,
    /// Project roadmap, milestones, and backlog plans.
    Roadmap,
    /// General or uncategorized documentation.
    General,
}

impl DocType {
    /// Returns the snake_case identifier for this documentation type.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Readme => "readme",
            Self::Architecture => "architecture",
            Self::Changelog => "changelog",
            Self::Contributing => "contributing",
            Self::License => "license",
            Self::Security => "security",
            Self::CodeOfConduct => "code_of_conduct",
            Self::Guide => "guide",
            Self::Roadmap => "roadmap",
            Self::General => "general",
        }
    }

    /// Returns whether this document is a README.
    #[must_use]
    pub const fn is_readme(self) -> bool {
        matches!(self, Self::Readme)
    }

    /// Returns whether this document is a License.
    #[must_use]
    pub const fn is_license(self) -> bool {
        matches!(self, Self::License)
    }

    /// Returns whether this document is an Architectural specification or ADR.
    #[must_use]
    pub const fn is_architecture(self) -> bool {
        matches!(self, Self::Architecture)
    }

    /// Returns whether this document is a Security policy or advisory.
    #[must_use]
    pub const fn is_security(self) -> bool {
        matches!(self, Self::Security)
    }
}

/// Markup syntax or presentation format of a documentation file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocFormat {
    /// GitHub Flavored Markdown or standard Markdown.
    Markdown,
    /// AsciiDoc format.
    Asciidoc,
    /// reStructuredText format.
    RestructuredText,
    /// Plain unformatted text.
    PlainText,
}

impl DocFormat {
    /// Returns the string format identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Asciidoc => "asciidoc",
            Self::RestructuredText => "restructuredtext",
            Self::PlainText => "plaintext",
        }
    }
}

/// Detects the documentation type from repository relative path and file name.
#[must_use]
pub fn detect_doc_type(path: &str, file_name: &str) -> DocType {
    let lower_path = path.to_ascii_lowercase();
    let lower_name = file_name.to_ascii_lowercase();

    // Check README
    if lower_name == "readme"
        || lower_name.starts_with("readme.")
        || lower_name.starts_with("readme_")
    {
        return DocType::Readme;
    }

    // Check LICENSE & legal
    if lower_name == "license"
        || lower_name.starts_with("license.")
        || lower_name.starts_with("license-")
        || lower_name.starts_with("licence")
        || lower_name.starts_with("copying")
        || lower_name.starts_with("notice")
    {
        return DocType::License;
    }

    // Check CHANGELOG & history
    if lower_name == "changelog"
        || lower_name.starts_with("changelog.")
        || lower_name.starts_with("changes")
        || lower_name.starts_with("releases")
        || lower_name.starts_with("history")
    {
        return DocType::Changelog;
    }

    // Check CONTRIBUTING & dev guides
    if lower_name == "contributing"
        || lower_name.starts_with("contributing.")
        || lower_name.starts_with("hacking")
        || lower_name.starts_with("development.")
    {
        return DocType::Contributing;
    }

    // Check SECURITY
    if lower_name == "security"
        || lower_name.starts_with("security.")
        || lower_path.contains("security")
    {
        return DocType::Security;
    }

    // Check CODE OF CONDUCT
    if lower_name == "code_of_conduct" || lower_name.starts_with("code_of_conduct.") {
        return DocType::CodeOfConduct;
    }

    // Check ARCHITECTURE & ADRs
    if lower_name == "architecture"
        || lower_name.starts_with("architecture.")
        || lower_name.starts_with("decisions.")
        || lower_name.starts_with("adr-")
        || lower_name.starts_with("data_model.")
        || lower_path.contains("/architecture")
        || lower_path.contains("/decisions")
        || lower_path.contains("/adr")
    {
        return DocType::Architecture;
    }

    // Check ROADMAP & TODO
    if lower_name == "roadmap"
        || lower_name.starts_with("roadmap.")
        || lower_name == "todo"
        || lower_name.starts_with("todo.")
    {
        return DocType::Roadmap;
    }

    // Check USER GUIDES & TUTORIALS
    if lower_name.starts_with("guide")
        || lower_name.starts_with("user_guide")
        || lower_name.starts_with("install")
        || lower_name.starts_with("quickstart")
        || lower_name.starts_with("api.")
        || lower_path.starts_with("docs/")
        || lower_path.starts_with("doc/")
        || lower_path.contains("/docs/")
    {
        return DocType::Guide;
    }

    DocType::General
}

/// Detects the documentation format from file name and extension.
#[must_use]
pub fn detect_doc_format(file_name: &str, extension: Option<&str>) -> DocFormat {
    if let Some(ext) = extension {
        match ext.to_ascii_lowercase().as_str() {
            "md" | "markdown" => return DocFormat::Markdown,
            "adoc" | "asciidoc" => return DocFormat::Asciidoc,
            "rst" => return DocFormat::RestructuredText,
            "txt" => return DocFormat::PlainText,
            _ => {}
        }
    }

    let lower = file_name.to_ascii_lowercase();
    if lower.ends_with(".md") || lower.ends_with(".markdown") {
        DocFormat::Markdown
    } else if lower.ends_with(".adoc") || lower.ends_with(".asciidoc") {
        DocFormat::Asciidoc
    } else if lower.ends_with(".rst") {
        DocFormat::RestructuredText
    } else {
        DocFormat::PlainText
    }
}

/// Parses the document title and detects frontmatter presence from text content.
#[must_use]
pub fn extract_title_and_frontmatter(content: &str, path: &str) -> (String, bool) {
    let mut has_frontmatter = false;
    let mut title_from_frontmatter = None;

    // 1. Check YAML frontmatter: starts with `---\n` or `---\r\n`
    if let Some(remainder) = content
        .strip_prefix("---\r\n")
        .or_else(|| content.strip_prefix("---\n"))
    {
        if let Some(closing_pos) = remainder
            .find("\n---")
            .or_else(|| remainder.find("\r\n---"))
        {
            let frontmatter_body = &remainder[..closing_pos];
            has_frontmatter = true;

            for line in frontmatter_body.lines() {
                let trimmed = line.trim();
                if let Some(rest) = trimmed.strip_prefix("title:") {
                    let cleaned = rest.trim().trim_matches('"').trim_matches('\'').trim();
                    if !cleaned.is_empty() {
                        title_from_frontmatter = Some(cleaned.to_string());
                        break;
                    }
                }
            }
        }
    }

    if let Some(title) = title_from_frontmatter {
        return (title, has_frontmatter);
    }

    // 2. Scan lines for first Markdown `# Heading`
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let heading_text = trimmed.trim_start_matches('#').trim();
            if !heading_text.is_empty() {
                // Strip markdown bold or italics
                let cleaned = heading_text.trim_matches('*').trim_matches('_').trim();
                return (cleaned.to_string(), has_frontmatter);
            }
        }
    }

    // 3. Fallback: clean filename or path segment
    let file_name = path.rsplit('/').next().unwrap_or(path);
    let title_fallback = file_name
        .rsplit_once('.')
        .map(|(base, _)| base)
        .unwrap_or(file_name)
        .replace(['_', '-'], " ");

    (title_fallback, has_frontmatter)
}

/// An extracted, verified, and structured repository documentation file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedDocFile {
    /// Relative repository path (e.g. `README.md`, `docs/ARCHITECTURE.md`).
    pub path: String,
    /// 40-character Git object SHA-1 hash.
    pub git_sha: String,
    /// Evaluated functional documentation type.
    pub doc_type: DocType,
    /// Presentation markup format.
    pub format: DocFormat,
    /// Document title extracted from heading, frontmatter, or filename.
    pub title: String,
    /// Whether the document contains YAML frontmatter.
    pub has_frontmatter: bool,
    /// Normalized text content.
    pub content: String,
    /// Content length in bytes.
    pub byte_size: usize,
    /// Total lines in the document.
    pub line_count: usize,
    /// Cryptographic BLAKE3 digest of the content bytes.
    pub blake3_hash: String,
    /// Cryptographic SHA-256 digest of the content bytes.
    pub sha256_hash: String,
}

impl ExtractedDocFile {
    /// Constructs an `ExtractedDocFile` from raw blob bytes and path.
    pub fn from_blob(
        path: impl Into<String>,
        blob: &GitBlob,
        config: &SourceExtractionConfig,
    ) -> Result<Self, GitHubExtractorError> {
        let path = path.into();

        if config.verify_git_sha && !blob.verify_git_sha() {
            let actual = compute_git_blob_sha(&blob.raw_bytes);
            return Err(GitHubExtractorError::BlobError {
                sha: blob.sha.clone(),
                message: format!(
                    "Git blob SHA-1 mismatch for doc '{path}': expected '{}', got '{actual}'",
                    blob.sha
                ),
            });
        }

        if blob.is_binary() {
            return Err(GitHubExtractorError::BlobError {
                sha: blob.sha.clone(),
                message: format!("Binary or null bytes detected in documentation file '{path}'"),
            });
        }

        let content = blob.content_lossy().into_owned();
        let byte_size = content.len();
        let line_count = if content.is_empty() {
            0
        } else {
            content.lines().count().max(1)
        };

        let file_name = path.rsplit('/').next().unwrap_or(&path);
        let extension = file_name.rsplit_once('.').map(|(_, ext)| ext);

        let doc_type = detect_doc_type(&path, file_name);
        let format = detect_doc_format(file_name, extension);
        let (title, has_frontmatter) = extract_title_and_frontmatter(&content, &path);

        let blake3_hash = compute_blake3_hash(content.as_bytes());
        let sha256_hash = compute_sha256_hash(content.as_bytes());

        Ok(Self {
            path,
            git_sha: blob.sha.clone(),
            doc_type,
            format,
            title,
            has_frontmatter,
            content,
            byte_size,
            line_count,
            blake3_hash,
            sha256_hash,
        })
    }

    /// Converts this extracted documentation file into a Canonical Context Document Model [`CdmSection`].
    #[must_use]
    pub fn to_cdm_section(
        &self,
        section_id: impl Into<String>,
        source_url: impl Into<String>,
    ) -> CdmSection {
        let lines = self.line_count as u32;
        let mut section =
            CdmSectionBuilder::new(section_id, source_url, SectionKind::Prose, &self.content)
                .title(&self.title)
                .original_path(&self.path)
                .line_range(1, lines.max(1))
                .byte_range(0, self.byte_size as u64)
                .language(self.format.as_str())
                .content_hash(&self.blake3_hash)
                .token_metrics(SectionTokenMetrics::compute_baseline(&self.content))
                .build();

        section
            .attributes
            .insert("doc_type".to_string(), self.doc_type.as_str().to_string());
        section
            .attributes
            .insert("git_blob_sha".to_string(), self.git_sha.clone());
        section
            .attributes
            .insert("sha256".to_string(), self.sha256_hash.clone());
        section.attributes.insert(
            "has_frontmatter".to_string(),
            self.has_frontmatter.to_string(),
        );

        section
    }
}

/// Aggregated outcome and metrics from repository documentation extraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocExtractionReport {
    /// Successfully acquired and verified documentation files.
    pub extracted: Vec<ExtractedDocFile>,
    /// Files that encountered errors during fetch or validation.
    pub failed: Vec<FailedSourceExtraction>,
    /// Files deemed unsupported (e.g. non-text binary).
    pub unsupported: Vec<ClassifiedFile>,
    /// Total bytes acquired across all extracted documentation files.
    pub total_bytes: u64,
    /// Total lines spanned across all extracted documentation files.
    pub total_lines: usize,
    /// Total duration elapsed during the documentation extraction batch.
    pub duration: Duration,
}

impl DocExtractionReport {
    /// Creates a new empty `DocExtractionReport`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            extracted: Vec::new(),
            failed: Vec::new(),
            unsupported: Vec::new(),
            total_bytes: 0,
            total_lines: 0,
            duration: Duration::ZERO,
        }
    }

    /// Appends a successfully extracted documentation file.
    pub fn add_extracted(&mut self, file: ExtractedDocFile) {
        self.total_bytes += file.byte_size as u64;
        self.total_lines += file.line_count;
        self.extracted.push(file);
    }

    /// Records a failed extraction.
    pub fn add_failed(
        &mut self,
        path: impl Into<String>,
        git_sha: impl Into<String>,
        error: impl Into<String>,
    ) {
        self.failed.push(FailedSourceExtraction {
            path: path.into(),
            git_sha: git_sha.into(),
            error: error.into(),
        });
    }

    /// Records an unsupported file.
    pub fn add_unsupported(&mut self, file: ClassifiedFile) {
        self.unsupported.push(file);
    }

    /// Convenience lookup for the primary repository README file (prioritizing root README).
    #[must_use]
    pub fn readme(&self) -> Option<&ExtractedDocFile> {
        self.extracted
            .iter()
            .find(|f| {
                f.path.eq_ignore_ascii_case("readme.md") || f.path.eq_ignore_ascii_case("readme")
            })
            .or_else(|| self.extracted.iter().find(|f| f.doc_type.is_readme()))
    }

    /// Convenience lookup for the primary repository LICENSE file.
    #[must_use]
    pub fn license(&self) -> Option<&ExtractedDocFile> {
        self.extracted
            .iter()
            .find(|f| {
                f.path.eq_ignore_ascii_case("license")
                    || f.path.eq_ignore_ascii_case("license.md")
                    || f.path.eq_ignore_ascii_case("license.txt")
            })
            .or_else(|| self.extracted.iter().find(|f| f.doc_type.is_license()))
    }

    /// Filters extracted documentation files by specific functional type.
    #[must_use]
    pub fn filter_by_type(&self, doc_type: DocType) -> Vec<&ExtractedDocFile> {
        self.extracted
            .iter()
            .filter(|f| f.doc_type == doc_type)
            .collect()
    }

    /// Translates this documentation report into a CDM [`CompletenessReport`].
    #[must_use]
    pub fn to_completeness_report(
        &self,
        total_discovered: u64,
        total_excluded: u64,
    ) -> CompletenessReport {
        let extracted = self.extracted.len() as u64;
        let failed = self.failed.len() as u64;
        let unsupported = self.unsupported.len() as u64;
        let eligible = extracted + failed;

        CompletenessReport::new(
            total_discovered,
            eligible,
            extracted,
            total_excluded,
            failed,
            unsupported,
        )
    }

    /// Converts all extracted documentation files into CDM [`CdmSection`]s.
    #[must_use]
    pub fn to_cdm_sections(&self, base_url: &str) -> Vec<CdmSection> {
        let clean_base = base_url.trim_end_matches('/');
        self.extracted
            .iter()
            .enumerate()
            .map(|(idx, file)| {
                let section_id = format!("sec-doc-{:04}", idx + 1);
                let source_url = format!("{clean_base}/{}", file.path);
                file.to_cdm_section(section_id, source_url)
            })
            .collect()
    }
}

impl Default for DocExtractionReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_type_and_format_detection() {
        assert_eq!(detect_doc_type("README.md", "README.md"), DocType::Readme);
        assert_eq!(
            detect_doc_type("LICENSE-MIT", "LICENSE-MIT"),
            DocType::License
        );
        assert_eq!(
            detect_doc_type("CHANGELOG.md", "CHANGELOG.md"),
            DocType::Changelog
        );
        assert_eq!(
            detect_doc_type("docs/ARCHITECTURE.md", "ARCHITECTURE.md"),
            DocType::Architecture
        );
        assert_eq!(
            detect_doc_type(".github/SECURITY.md", "SECURITY.md"),
            DocType::Security
        );
        assert_eq!(
            detect_doc_type("CONTRIBUTING.md", "CONTRIBUTING.md"),
            DocType::Contributing
        );
        assert_eq!(
            detect_doc_type("ROADMAP.md", "ROADMAP.md"),
            DocType::Roadmap
        );
        assert_eq!(
            detect_doc_type("docs/guides/setup.md", "setup.md"),
            DocType::Guide
        );

        assert_eq!(
            detect_doc_format("README.md", Some("md")),
            DocFormat::Markdown
        );
        assert_eq!(
            detect_doc_format("manual.adoc", Some("adoc")),
            DocFormat::Asciidoc
        );
        assert_eq!(
            detect_doc_format("spec.rst", Some("rst")),
            DocFormat::RestructuredText
        );
        assert_eq!(detect_doc_format("LICENSE", None), DocFormat::PlainText);
    }

    #[test]
    fn test_title_and_frontmatter_extraction() {
        // Markdown heading
        let content_h1 = "# Project Title\n\nSome text here.\n";
        let (title1, fm1) = extract_title_and_frontmatter(content_h1, "README.md");
        assert_eq!(title1, "Project Title");
        assert!(!fm1);

        // YAML frontmatter
        let content_fm = "---\ntitle: \"Advanced Architecture Guide\"\nauthor: \"Core Team\"\n---\n\n# Heading 1\n";
        let (title2, fm2) = extract_title_and_frontmatter(content_fm, "docs/architecture.md");
        assert_eq!(title2, "Advanced Architecture Guide");
        assert!(fm2);

        // Fallback to filename
        let content_no_heading = "Plain text without headers.\nSecond line.\n";
        let (title3, fm3) =
            extract_title_and_frontmatter(content_no_heading, "docs/USER_MANUAL.txt");
        assert_eq!(title3, "USER MANUAL");
        assert!(!fm3);
    }

    #[test]
    fn test_extracted_doc_to_cdm_section() {
        let text = "# Repository Manual\n\nWelcome to our platform.\n";
        let bytes = text.as_bytes().to_vec();
        let blob_sha = compute_git_blob_sha(&bytes);
        let blob = GitBlob::new(blob_sha.clone(), bytes, "raw");

        let config = SourceExtractionConfig::default();
        let doc = ExtractedDocFile::from_blob("docs/MANUAL.md", &blob, &config)
            .expect("Valid documentation must parse");

        assert_eq!(doc.path, "docs/MANUAL.md");
        assert_eq!(doc.title, "Repository Manual");
        assert_eq!(doc.doc_type, DocType::Guide);
        assert_eq!(doc.format, DocFormat::Markdown);

        let section = doc.to_cdm_section(
            "sec-doc-1",
            "https://github.com/org/repo/blob/main/docs/MANUAL.md",
        );
        assert_eq!(section.section_id, "sec-doc-1");
        assert_eq!(section.kind, SectionKind::Prose);
        assert_eq!(section.title.as_deref(), Some("Repository Manual"));
        assert_eq!(section.language.as_deref(), Some("markdown"));
        assert_eq!(
            section.attributes.get("doc_type"),
            Some(&"guide".to_string())
        );
        assert_eq!(section.attributes.get("git_blob_sha"), Some(&blob_sha));
    }
}
