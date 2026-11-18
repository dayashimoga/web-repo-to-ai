//! # GitHub Configuration and Test File Extraction
//!
//! Provides specialized acquisition, categorization, and CDM section generation for:
//! - Repository configuration files (`.github/workflows/*`, `tsconfig.json`, `Dockerfile`, linters, env templates)
//! - Test files and fixtures (unit tests, integration test suites, E2E specs, mock JSON datasets)
//! - Bounded concurrent batch extraction via [`tokio::sync::Semaphore`]
//! - Conversion into Canonical Context Document Model ([`CdmSection`]) code sections ([`SectionKind::CodeFile`]).

use std::time::Duration;

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{
    CdmSection, CdmSectionBuilder, CompletenessReport, SectionKind, SectionTokenMetrics,
};

use crate::classification::{detect_language, ClassifiedFile};
use crate::error::GitHubExtractorError;
use crate::source::{
    compute_blake3_hash, compute_git_blob_sha, compute_sha256_hash, FailedSourceExtraction,
    GitBlob, SourceExtractionConfig,
};

/// Functional category of a repository configuration file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigCategory {
    /// Continuous integration and continuous deployment pipelines (e.g. GitHub Actions, GitLab CI).
    CiCd,
    /// Build systems, compilers, bundlers, and container specifications (e.g. Dockerfile, tsconfig.json).
    Build,
    /// Linters, formatters, and code quality rules (e.g. .eslintrc, rustfmt.toml, clippy.toml).
    LinterFormatter,
    /// Environment variable templates and secret exemplars (e.g. .env.example).
    Environment,
    /// General or uncategorized configuration files.
    General,
}

impl ConfigCategory {
    /// Returns the snake_case identifier for this configuration category.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CiCd => "ci_cd",
            Self::Build => "build",
            Self::LinterFormatter => "linter_formatter",
            Self::Environment => "environment",
            Self::General => "general",
        }
    }

    /// Returns whether this category represents a CI/CD pipeline workflow.
    #[must_use]
    pub const fn is_cicd(self) -> bool {
        matches!(self, Self::CiCd)
    }

    /// Returns whether this category represents build or container tooling.
    #[must_use]
    pub const fn is_build(self) -> bool {
        matches!(self, Self::Build)
    }

    /// Returns whether this category represents a linter or formatter rule.
    #[must_use]
    pub const fn is_linter_formatter(self) -> bool {
        matches!(self, Self::LinterFormatter)
    }
}

/// Detects the configuration category from path and file name.
#[must_use]
pub fn detect_config_category(path: &str, file_name: &str) -> ConfigCategory {
    let lower_path = path.to_ascii_lowercase();
    let lower_name = file_name.to_ascii_lowercase();

    // 1. CI/CD Workflows
    if lower_path.starts_with(".github/workflows/")
        || lower_path.contains("/.github/workflows/")
        || lower_name == ".gitlab-ci.yml"
        || lower_name == "jenkinsfile"
        || lower_name == ".travis.yml"
        || lower_name == "azure-pipelines.yml"
        || lower_path.starts_with(".circleci/")
        || lower_path.contains("/.circleci/")
    {
        return ConfigCategory::CiCd;
    }

    // 2. Linters & Formatters
    if lower_name.starts_with(".eslint")
        || lower_name.starts_with(".prettier")
        || lower_name == "rustfmt.toml"
        || lower_name == ".rustfmt.toml"
        || lower_name == "clippy.toml"
        || lower_name == ".clippy.toml"
        || lower_name == ".flake8"
        || lower_name == ".rubocop.yml"
        || lower_name == ".editorconfig"
        || lower_name == ".golangci.yml"
        || lower_name == "biome.json"
        || lower_name.starts_with("lint")
    {
        return ConfigCategory::LinterFormatter;
    }

    // 3. Environment Templates
    if lower_name == ".env.example"
        || lower_name == ".env.template"
        || lower_name == ".env.sample"
        || lower_name.starts_with(".env.")
    {
        return ConfigCategory::Environment;
    }

    // 4. Build & Container Specs
    if lower_name.starts_with("tsconfig")
        || lower_name.starts_with("webpack.")
        || lower_name.starts_with("vite.config")
        || lower_name.starts_with("rollup.config")
        || lower_name == "makefile"
        || lower_name == "dockerfile"
        || lower_name.starts_with("dockerfile.")
        || lower_name == "docker-compose.yml"
        || lower_name == "docker-compose.yaml"
        || lower_name == "containerfile"
        || lower_name.starts_with("containerfile.")
        || lower_name == "podmanfile"
        || lower_name.starts_with("babel.config")
        || lower_name.starts_with("tailwind.config")
        || lower_name.starts_with("postcss.config")
        || lower_name == "cmakelists.txt"
        || lower_name == "meson.build"
    {
        return ConfigCategory::Build;
    }

    ConfigCategory::General
}

/// Detects the presentation format / language of a configuration file.
#[must_use]
pub fn detect_config_format(file_name: &str, ext: Option<&str>) -> String {
    let lower_name = file_name.to_ascii_lowercase();

    if lower_name == "dockerfile" || lower_name.starts_with("dockerfile.") {
        return "dockerfile".to_string();
    }
    if lower_name == "makefile" {
        return "makefile".to_string();
    }

    if let Some(e) = ext {
        match e.to_ascii_lowercase().as_str() {
            "yml" | "yaml" => return "yaml".to_string(),
            "json" => return "json".to_string(),
            "toml" => return "toml".to_string(),
            "xml" => return "xml".to_string(),
            "ini" | "cfg" | "conf" => return "ini".to_string(),
            "ts" => return "typescript".to_string(),
            "js" => return "javascript".to_string(),
            _ => {}
        }
    }

    "text".to_string()
}

/// Functional category of a test code file or fixture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestCategory {
    /// Unit test file testing isolated components or modules.
    Unit,
    /// Integration test suite verifying multi-component interaction.
    Integration,
    /// End-to-end (E2E) or browser automation test suite.
    E2e,
    /// Static test fixture, mock dataset, or sample payload.
    Fixture,
    /// Performance benchmark or load test suite.
    Benchmark,
    /// General or uncategorized test file.
    General,
}

impl TestCategory {
    /// Returns the snake_case identifier for this test category.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unit => "unit",
            Self::Integration => "integration",
            Self::E2e => "e2e",
            Self::Fixture => "fixture",
            Self::Benchmark => "benchmark",
            Self::General => "general",
        }
    }

    /// Returns whether this file is a unit test.
    #[must_use]
    pub const fn is_unit(self) -> bool {
        matches!(self, Self::Unit)
    }

    /// Returns whether this file is an integration test.
    #[must_use]
    pub const fn is_integration(self) -> bool {
        matches!(self, Self::Integration)
    }

    /// Returns whether this file is a static test fixture or mock dataset.
    #[must_use]
    pub const fn is_fixture(self) -> bool {
        matches!(self, Self::Fixture)
    }
}

/// Detects the test category from relative path and file name.
#[must_use]
pub fn detect_test_category(path: &str, file_name: &str) -> TestCategory {
    let lower_path = path.to_ascii_lowercase();
    let lower_name = file_name.to_ascii_lowercase();

    // 1. Fixtures & Mocks
    if lower_path.contains("/fixtures/")
        || lower_path.contains("/__fixtures__/")
        || lower_path.contains("/mocks/")
        || lower_path.contains("/testdata/")
        || lower_path.contains("/samples/")
        || lower_path.starts_with("fixtures/")
        || lower_path.starts_with("testdata/")
    {
        return TestCategory::Fixture;
    }

    // 2. Benchmarks
    if lower_path.contains("/benches/")
        || lower_path.contains("/benchmarks/")
        || lower_path.starts_with("benches/")
        || lower_name.contains("bench")
    {
        return TestCategory::Benchmark;
    }

    // 3. E2E / UI
    if lower_path.contains("/e2e/")
        || lower_path.contains("/cypress/")
        || lower_path.contains("/playwright/")
        || lower_name.contains(".e2e.")
        || lower_name.contains("_e2e.")
    {
        return TestCategory::E2e;
    }

    // 4. Integration Tests
    if lower_path.starts_with("tests/")
        || lower_path.contains("/integration/")
        || lower_name.contains("integration_test")
        || lower_name.starts_with("it_")
    {
        return TestCategory::Integration;
    }

    // 5. Unit Tests
    if lower_name.ends_with("_test.rs")
        || lower_name.ends_with(".test.ts")
        || lower_name.ends_with(".test.js")
        || lower_name.ends_with(".test.tsx")
        || lower_name.ends_with(".spec.ts")
        || lower_name.ends_with(".spec.js")
        || lower_name.starts_with("test_")
        || lower_name.ends_with("_test.go")
        || lower_path.contains("/__tests__/")
    {
        return TestCategory::Unit;
    }

    TestCategory::General
}

// ---------------------------------------------------------------------------
// Extracted Models
// ---------------------------------------------------------------------------

/// An extracted, verified, and structured repository configuration file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedConfigFile {
    /// Relative repository path (e.g. `.github/workflows/ci.yml`, `tsconfig.json`).
    pub path: String,
    /// 40-character Git object SHA-1 hash.
    pub git_sha: String,
    /// Evaluated configuration category.
    pub config_category: ConfigCategory,
    /// Presentation format / language (e.g. "yaml", "json", "toml", "dockerfile").
    pub format: String,
    /// Normalized text content.
    pub content: String,
    /// Content length in bytes.
    pub byte_size: usize,
    /// Total lines in the file.
    pub line_count: usize,
    /// Cryptographic BLAKE3 digest of the content bytes.
    pub blake3_hash: String,
    /// Cryptographic SHA-256 digest of the content bytes.
    pub sha256_hash: String,
}

impl ExtractedConfigFile {
    /// Constructs an `ExtractedConfigFile` from raw blob bytes and path.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError::BlobError` if Git SHA verification fails or binary content is detected.
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
                    "Git blob SHA-1 mismatch for config '{path}': expected '{}', got '{actual}'",
                    blob.sha
                ),
            });
        }

        if blob.is_binary() {
            return Err(GitHubExtractorError::BlobError {
                sha: blob.sha.clone(),
                message: format!("Binary or null bytes detected in configuration file '{path}'"),
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

        let config_category = detect_config_category(&path, file_name);
        let format = detect_config_format(file_name, extension);

        let blake3_hash = compute_blake3_hash(content.as_bytes());
        let sha256_hash = compute_sha256_hash(content.as_bytes());

        Ok(Self {
            path,
            git_sha: blob.sha.clone(),
            config_category,
            format,
            content,
            byte_size,
            line_count,
            blake3_hash,
            sha256_hash,
        })
    }

    /// Converts this extracted configuration file into a Canonical Context Document Model [`CdmSection`].
    #[must_use]
    pub fn to_cdm_section(
        &self,
        section_id: impl Into<String>,
        source_url: impl Into<String>,
    ) -> CdmSection {
        let lines = self.line_count as u32;
        let title = format!("Config: {}", self.path);
        let mut section =
            CdmSectionBuilder::new(section_id, source_url, SectionKind::CodeFile, &self.content)
                .title(&title)
                .original_path(&self.path)
                .line_range(1, lines.max(1))
                .byte_range(0, self.byte_size as u64)
                .language(&self.format)
                .content_hash(&self.blake3_hash)
                .token_metrics(SectionTokenMetrics::compute_baseline(&self.content))
                .build();

        section
            .attributes
            .insert("file_category".to_string(), "configuration".to_string());
        section.attributes.insert(
            "config_category".to_string(),
            self.config_category.as_str().to_string(),
        );
        section
            .attributes
            .insert("git_blob_sha".to_string(), self.git_sha.clone());
        section
            .attributes
            .insert("sha256".to_string(), self.sha256_hash.clone());

        section
    }
}

/// An extracted, verified, and structured repository test file or test fixture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedTestFile {
    /// Relative repository path (e.g. `tests/integration_test.rs`, `fixtures/sample.json`).
    pub path: String,
    /// 40-character Git object SHA-1 hash.
    pub git_sha: String,
    /// Evaluated test category.
    pub test_category: TestCategory,
    /// Detected programming language or syntax format, if recognized.
    pub language: Option<String>,
    /// Normalized text content.
    pub content: String,
    /// Content length in bytes.
    pub byte_size: usize,
    /// Total lines in the file.
    pub line_count: usize,
    /// Cryptographic BLAKE3 digest of the content bytes.
    pub blake3_hash: String,
    /// Cryptographic SHA-256 digest of the content bytes.
    pub sha256_hash: String,
}

impl ExtractedTestFile {
    /// Constructs an `ExtractedTestFile` from raw blob bytes, path, and optional language hint.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError::BlobError` if Git SHA verification fails or binary content is detected.
    pub fn from_blob(
        path: impl Into<String>,
        blob: &GitBlob,
        config: &SourceExtractionConfig,
        language_hint: Option<String>,
    ) -> Result<Self, GitHubExtractorError> {
        let path = path.into();

        if config.verify_git_sha && !blob.verify_git_sha() {
            let actual = compute_git_blob_sha(&blob.raw_bytes);
            return Err(GitHubExtractorError::BlobError {
                sha: blob.sha.clone(),
                message: format!(
                    "Git blob SHA-1 mismatch for test '{path}': expected '{}', got '{actual}'",
                    blob.sha
                ),
            });
        }

        if blob.is_binary() {
            return Err(GitHubExtractorError::BlobError {
                sha: blob.sha.clone(),
                message: format!("Binary or null bytes detected in test file '{path}'"),
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

        let test_category = detect_test_category(&path, file_name);
        let language = language_hint.or_else(|| detect_language(file_name, extension));

        let blake3_hash = compute_blake3_hash(content.as_bytes());
        let sha256_hash = compute_sha256_hash(content.as_bytes());

        Ok(Self {
            path,
            git_sha: blob.sha.clone(),
            test_category,
            language,
            content,
            byte_size,
            line_count,
            blake3_hash,
            sha256_hash,
        })
    }

    /// Converts this extracted test file into a Canonical Context Document Model [`CdmSection`].
    #[must_use]
    pub fn to_cdm_section(
        &self,
        section_id: impl Into<String>,
        source_url: impl Into<String>,
    ) -> CdmSection {
        let lines = self.line_count as u32;
        let title = format!("Test: {}", self.path);
        let lang = self.language.as_deref().unwrap_or("text");

        let mut section =
            CdmSectionBuilder::new(section_id, source_url, SectionKind::CodeFile, &self.content)
                .title(&title)
                .original_path(&self.path)
                .line_range(1, lines.max(1))
                .byte_range(0, self.byte_size as u64)
                .language(lang)
                .content_hash(&self.blake3_hash)
                .token_metrics(SectionTokenMetrics::compute_baseline(&self.content))
                .build();

        section
            .attributes
            .insert("file_category".to_string(), "test_fixture".to_string());
        section.attributes.insert(
            "test_category".to_string(),
            self.test_category.as_str().to_string(),
        );
        section
            .attributes
            .insert("git_blob_sha".to_string(), self.git_sha.clone());
        section
            .attributes
            .insert("sha256".to_string(), self.sha256_hash.clone());

        section
    }
}

/// Aggregated outcome and metrics from repository configuration and test extraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigTestExtractionReport {
    /// Successfully acquired and verified configuration files.
    pub extracted_configs: Vec<ExtractedConfigFile>,
    /// Successfully acquired and verified test files and fixtures.
    pub extracted_tests: Vec<ExtractedTestFile>,
    /// Files that encountered errors during fetch or validation.
    pub failed: Vec<FailedSourceExtraction>,
    /// Files deemed unsupported (e.g. non-text binary).
    pub unsupported: Vec<ClassifiedFile>,
    /// Total bytes acquired across all extracted config and test files.
    pub total_bytes: u64,
    /// Total lines spanned across all extracted config and test files.
    pub total_lines: usize,
    /// Wall-clock duration elapsed during extraction.
    pub duration: Duration,
}

impl Default for ConfigTestExtractionReport {
    fn default() -> Self {
        Self {
            extracted_configs: Vec::new(),
            extracted_tests: Vec::new(),
            failed: Vec::new(),
            unsupported: Vec::new(),
            total_bytes: 0,
            total_lines: 0,
            duration: Duration::ZERO,
        }
    }
}

impl ConfigTestExtractionReport {
    /// Creates a new empty `ConfigTestExtractionReport`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a successfully extracted configuration file.
    pub fn add_config(&mut self, config: ExtractedConfigFile) {
        self.total_bytes += config.byte_size as u64;
        self.total_lines += config.line_count;
        self.extracted_configs.push(config);
    }

    /// Appends a successfully extracted test file.
    pub fn add_test(&mut self, test: ExtractedTestFile) {
        self.total_bytes += test.byte_size as u64;
        self.total_lines += test.line_count;
        self.extracted_tests.push(test);
    }

    /// Appends a failed extraction attempt.
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

    /// Appends an unsupported file entry.
    pub fn add_unsupported(&mut self, file: ClassifiedFile) {
        self.unsupported.push(file);
    }

    /// Returns total extracted files (configs + tests).
    #[must_use]
    pub fn total_extracted(&self) -> usize {
        self.extracted_configs.len() + self.extracted_tests.len()
    }

    /// Returns all extracted configs matching a specific category.
    #[must_use]
    pub fn filter_configs_by_category(&self, cat: ConfigCategory) -> Vec<&ExtractedConfigFile> {
        self.extracted_configs
            .iter()
            .filter(|f| f.config_category == cat)
            .collect()
    }

    /// Returns all extracted tests matching a specific category.
    #[must_use]
    pub fn filter_tests_by_category(&self, cat: TestCategory) -> Vec<&ExtractedTestFile> {
        self.extracted_tests
            .iter()
            .filter(|f| f.test_category == cat)
            .collect()
    }

    /// Convenience accessor for CI/CD workflow configurations.
    #[must_use]
    pub fn ci_workflows(&self) -> Vec<&ExtractedConfigFile> {
        self.filter_configs_by_category(ConfigCategory::CiCd)
    }

    /// Convenience accessor for static test fixtures and mock datasets.
    #[must_use]
    pub fn fixtures(&self) -> Vec<&ExtractedTestFile> {
        self.filter_tests_by_category(TestCategory::Fixture)
    }

    /// Maps this extraction report into a Canonical Context Document Model [`CompletenessReport`].
    ///
    /// Preserves the fundamental repository completeness accounting formula:
    /// `discovered_count == extracted_count + excluded_count + failed_count + unsupported_count`
    #[must_use]
    pub fn to_completeness_report(
        &self,
        total_discovered: u64,
        total_excluded: u64,
    ) -> CompletenessReport {
        let extracted = (self.extracted_configs.len() + self.extracted_tests.len()) as u64;
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

    /// Converts all extracted configuration and test files into CDM [`CdmSection`]s.
    #[must_use]
    pub fn to_cdm_sections(&self, base_url: &str) -> Vec<CdmSection> {
        let clean_base = base_url.trim_end_matches('/');
        let mut sections = Vec::with_capacity(self.total_extracted());

        for (idx, cfg) in self.extracted_configs.iter().enumerate() {
            let section_id = format!("sec-cfg-{:04}", idx + 1);
            let source_url = format!("{clean_base}/{}", cfg.path);
            sections.push(cfg.to_cdm_section(section_id, source_url));
        }

        for (idx, test) in self.extracted_tests.iter().enumerate() {
            let section_id = format!("sec-test-{:04}", idx + 1);
            let source_url = format!("{clean_base}/{}", test.path);
            sections.push(test.to_cdm_section(section_id, source_url));
        }

        sections
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_config_category() {
        assert_eq!(
            detect_config_category(".github/workflows/ci.yml", "ci.yml"),
            ConfigCategory::CiCd
        );
        assert_eq!(
            detect_config_category(".gitlab-ci.yml", ".gitlab-ci.yml"),
            ConfigCategory::CiCd
        );
        assert_eq!(
            detect_config_category("tsconfig.json", "tsconfig.json"),
            ConfigCategory::Build
        );
        assert_eq!(
            detect_config_category("Dockerfile", "Dockerfile"),
            ConfigCategory::Build
        );
        assert_eq!(
            detect_config_category("docker-compose.yml", "docker-compose.yml"),
            ConfigCategory::Build
        );
        assert_eq!(
            detect_config_category("rustfmt.toml", "rustfmt.toml"),
            ConfigCategory::LinterFormatter
        );
        assert_eq!(
            detect_config_category(".eslintrc.json", ".eslintrc.json"),
            ConfigCategory::LinterFormatter
        );
        assert_eq!(
            detect_config_category(".env.example", ".env.example"),
            ConfigCategory::Environment
        );
        assert_eq!(
            detect_config_category("app.conf", "app.conf"),
            ConfigCategory::General
        );
    }

    #[test]
    fn test_detect_test_category() {
        assert_eq!(
            detect_test_category("crates/core/src/parser_test.rs", "parser_test.rs"),
            TestCategory::Unit
        );
        assert_eq!(
            detect_test_category("apps/web/tests/login.test.ts", "login.test.ts"),
            TestCategory::Unit
        );
        assert_eq!(
            detect_test_category("tests/integration_test.rs", "integration_test.rs"),
            TestCategory::Integration
        );
        assert_eq!(
            detect_test_category("tests/e2e/auth.spec.ts", "auth.spec.ts"),
            TestCategory::E2e
        );
        assert_eq!(
            detect_test_category("tests/fixtures/response.json", "response.json"),
            TestCategory::Fixture
        );
        assert_eq!(
            detect_test_category("benches/benchmark.rs", "benchmark.rs"),
            TestCategory::Benchmark
        );
    }

    #[test]
    fn test_extracted_config_and_test_to_cdm() {
        let content = "name: CI\non: [push]";
        let bytes = content.as_bytes().to_vec();
        let blob_sha = compute_git_blob_sha(&bytes);
        let blob = GitBlob::new(blob_sha, bytes, "raw");
        let config = SourceExtractionConfig::default();

        let extracted_cfg =
            ExtractedConfigFile::from_blob(".github/workflows/ci.yml", &blob, &config)
                .expect("Valid config");
        assert_eq!(extracted_cfg.config_category, ConfigCategory::CiCd);
        assert_eq!(extracted_cfg.format, "yaml");

        let cdm_cfg = extracted_cfg.to_cdm_section("sec-cfg-1", "https://github.com/org/repo");
        assert_eq!(cdm_cfg.kind, SectionKind::CodeFile);
        assert_eq!(
            cdm_cfg.attributes.get("file_category").map(String::as_str),
            Some("configuration")
        );
        assert_eq!(
            cdm_cfg
                .attributes
                .get("config_category")
                .map(String::as_str),
            Some("ci_cd")
        );

        let test_content = "#[test]\nfn test_foo() {}";
        let test_bytes = test_content.as_bytes().to_vec();
        let test_sha = compute_git_blob_sha(&test_bytes);
        let test_blob = GitBlob::new(test_sha, test_bytes, "raw");

        let extracted_test = ExtractedTestFile::from_blob(
            "tests/foo_test.rs",
            &test_blob,
            &config,
            Some("rust".to_string()),
        )
        .expect("Valid test");
        assert_eq!(extracted_test.test_category, TestCategory::Integration);

        let cdm_test = extracted_test.to_cdm_section("sec-test-1", "https://github.com/org/repo");
        assert_eq!(cdm_test.kind, SectionKind::CodeFile);
        assert_eq!(
            cdm_test.attributes.get("file_category").map(String::as_str),
            Some("test_fixture")
        );
        assert_eq!(
            cdm_test.attributes.get("test_category").map(String::as_str),
            Some("integration")
        );
    }
}
