//! # Repository File Classification and Filtering
//!
//! Provides comprehensive categorization of discovered Git tree entries into canonical categories:
//! - Source code (`source_code`)
//! - Documentation (`documentation`)
//! - Configuration files (`configuration`)
//! - Dependency manifests (`dependency_manifest`)
//! - Test suites and fixtures (`test_fixture`)
//! - Generated code and lockfiles (`generated_or_lockfile`)
//! - Binary files and media (`binary`)
//! - Oversized files exceeding budget (`oversized`)
//!
//! Enforces eligibility filtering and produces quantitative classification reports mapping
//! directly into CDM `CompletenessReport` accounting.

use std::collections::{BTreeMap, HashMap};
use std::fmt;

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::CompletenessReport;

use crate::tree::{GitFileMode, GitTreeEntry, GitTreeResponse};

/// Default maximum allowed file size for eligible extraction: 1 MiB (1,048,576 bytes).
pub const DEFAULT_MAX_FILE_SIZE_BYTES: u64 = 1024 * 1024;

/// Canonical classification category for repository files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileCategory {
    /// Programming language source code.
    SourceCode,
    /// Human-readable documentation, guides, or license files.
    Documentation,
    /// Build settings, environment configurations, and tool options.
    Configuration,
    /// Package manager manifests declaring dependencies and project coordinates.
    DependencyManifest,
    /// Automated test suites, unit tests, integration tests, or mock fixtures.
    TestFixture,
    /// Lockfiles, minified distribution bundles, or compiler-generated outputs.
    GeneratedOrLockfile,
    /// Non-text binary assets, compiled artifacts, or multimedia.
    Binary,
    /// Files exceeding the configured byte size limit.
    Oversized,
    /// Other uncategorized or unsupported file types.
    Other,
}

impl FileCategory {
    /// Returns the snake_case string identifier for this category.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceCode => "source_code",
            Self::Documentation => "documentation",
            Self::Configuration => "configuration",
            Self::DependencyManifest => "dependency_manifest",
            Self::TestFixture => "test_fixture",
            Self::GeneratedOrLockfile => "generated_or_lockfile",
            Self::Binary => "binary",
            Self::Oversized => "oversized",
            Self::Other => "other",
        }
    }

    /// Returns whether this category represents general source code.
    #[must_use]
    pub const fn is_source_code(self) -> bool {
        matches!(self, Self::SourceCode)
    }

    /// Returns whether this category represents documentation.
    #[must_use]
    pub const fn is_documentation(self) -> bool {
        matches!(self, Self::Documentation)
    }

    /// Returns whether this category represents a configuration file.
    #[must_use]
    pub const fn is_configuration(self) -> bool {
        matches!(self, Self::Configuration)
    }

    /// Returns whether this category represents a package or dependency manifest.
    #[must_use]
    pub const fn is_dependency_manifest(self) -> bool {
        matches!(self, Self::DependencyManifest)
    }

    /// Returns whether this category represents a test or fixture file.
    #[must_use]
    pub const fn is_test(self) -> bool {
        matches!(self, Self::TestFixture)
    }

    /// Returns whether this category represents generated code or a package lockfile.
    #[must_use]
    pub const fn is_generated_or_lockfile(self) -> bool {
        matches!(self, Self::GeneratedOrLockfile)
    }

    /// Returns whether this category represents a non-text binary file.
    #[must_use]
    pub const fn is_binary(self) -> bool {
        matches!(self, Self::Binary)
    }

    /// Returns whether this category represents an oversized file.
    #[must_use]
    pub const fn is_oversized(self) -> bool {
        matches!(self, Self::Oversized)
    }

    /// Returns whether files in this category are eligible for content extraction into CDM sections.
    #[must_use]
    pub const fn is_extractable(self) -> bool {
        matches!(
            self,
            Self::SourceCode
                | Self::Documentation
                | Self::Configuration
                | Self::DependencyManifest
                | Self::TestFixture
        )
    }
}

impl fmt::Display for FileCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Configuration settings governing file classification and exclusion criteria.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileClassificationConfig {
    /// Upper size ceiling in bytes. Files exceeding this size are categorized as `Oversized`.
    pub max_file_size_bytes: u64,
    /// Path prefixes or directory segments excluded from extraction (e.g. `node_modules/`, `target/`).
    pub excluded_path_prefixes: Vec<String>,
    /// Whether package lockfiles (e.g. `package-lock.json`, `Cargo.lock`) are excluded from extraction.
    pub exclude_lockfiles: bool,
    /// Whether automated test fixtures are excluded from extraction.
    pub exclude_tests: bool,
    /// Explicit path-to-category overrides taking precedence over standard heuristics.
    pub custom_overrides: HashMap<String, FileCategory>,
}

impl Default for FileClassificationConfig {
    fn default() -> Self {
        Self {
            max_file_size_bytes: DEFAULT_MAX_FILE_SIZE_BYTES,
            excluded_path_prefixes: vec![
                ".git/".to_string(),
                "node_modules/".to_string(),
                "target/".to_string(),
                "dist/".to_string(),
                "build/".to_string(),
                ".venv/".to_string(),
                "vendor/".to_string(),
                ".idea/".to_string(),
                ".vscode/".to_string(),
            ],
            exclude_lockfiles: true,
            exclude_tests: false,
            custom_overrides: HashMap::new(),
        }
    }
}

impl FileClassificationConfig {
    /// Creates a new `FileClassificationConfig` with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum allowable file size threshold in bytes.
    #[must_use]
    pub const fn with_max_file_size(mut self, max_bytes: u64) -> Self {
        self.max_file_size_bytes = max_bytes;
        self
    }

    /// Appends an excluded path prefix.
    #[must_use]
    pub fn with_excluded_prefix(mut self, prefix: impl Into<String>) -> Self {
        let mut clean = prefix.into().replace('\\', "/");
        if !clean.ends_with('/') {
            clean.push('/');
        }
        self.excluded_path_prefixes.push(clean);
        self
    }

    /// Configures whether package lockfiles are excluded from extraction.
    #[must_use]
    pub const fn with_exclude_lockfiles(mut self, exclude: bool) -> Self {
        self.exclude_lockfiles = exclude;
        self
    }

    /// Configures whether test fixture files are excluded from extraction.
    #[must_use]
    pub const fn with_exclude_tests(mut self, exclude: bool) -> Self {
        self.exclude_tests = exclude;
        self
    }

    /// Registers an explicit classification override for a specific relative file path.
    #[must_use]
    pub fn with_override(mut self, path: impl Into<String>, category: FileCategory) -> Self {
        let clean = path.into().trim_start_matches('/').replace('\\', "/");
        self.custom_overrides.insert(clean, category);
        self
    }
}

/// A classified repository file entry detailing its category, eligibility, and attributes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassifiedFile {
    /// Normalized relative repository path.
    pub path: String,
    /// Evaluated file category.
    pub category: FileCategory,
    /// POSIX file mode.
    pub mode: GitFileMode,
    /// Git object SHA-1 hash.
    pub sha: String,
    /// File size in bytes, if known.
    pub size: Option<u64>,
    /// Detected programming language or syntax format, if identified.
    pub language: Option<String>,
    /// Whether this file is eligible for downstream source extraction and CDM conversion.
    pub is_eligible: bool,
    /// Explicit reason if the file was deemed ineligible or excluded.
    pub exclusion_reason: Option<String>,
}

impl ClassifiedFile {
    /// Returns the file name or last path segment.
    #[must_use]
    pub fn file_name(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }

    /// Returns the lowercase file extension, if any.
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        let name = self.file_name();
        name.rsplit_once('.').map(|(_, ext)| ext)
    }
}

/// Comprehensive summary report of file classification across an entire Git tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeClassificationReport {
    /// All classified files discovered in the tree.
    pub files: Vec<ClassifiedFile>,
    /// Total count of all files evaluated.
    pub total_files: usize,
    /// Breakdown of file counts per category.
    pub category_counts: BTreeMap<String, usize>,
    /// Total byte size accumulated per category.
    pub category_sizes: BTreeMap<String, u64>,
    /// Total count of files eligible for content extraction.
    pub eligible_count: usize,
    /// Total count of files excluded from extraction.
    pub excluded_count: usize,
    /// Total count of binary files discovered.
    pub binary_count: usize,
    /// Total count of oversized files discovered.
    pub oversized_count: usize,
}

impl TreeClassificationReport {
    /// Filters classified files by exact category.
    #[must_use]
    pub fn filter_by_category(&self, category: FileCategory) -> Vec<&ClassifiedFile> {
        self.files
            .iter()
            .filter(|f| f.category == category)
            .collect()
    }

    /// Returns all files eligible for extraction.
    #[must_use]
    pub fn eligible_files(&self) -> Vec<&ClassifiedFile> {
        self.files.iter().filter(|f| f.is_eligible).collect()
    }

    /// Returns all files excluded from extraction.
    #[must_use]
    pub fn excluded_files(&self) -> Vec<&ClassifiedFile> {
        self.files.iter().filter(|f| !f.is_eligible).collect()
    }

    /// Translates this classification report into a CDM `CompletenessReport`
    /// assuming all currently eligible files will be successfully extracted.
    ///
    /// Evaluates to `formula_valid() == true` enforcing:
    /// `discovered_count == extracted_count + excluded_count + failed_count + unsupported_count`
    #[must_use]
    pub fn to_expected_completeness_report(&self) -> CompletenessReport {
        let discovered = self.total_files as u64;
        let eligible = self.eligible_count as u64;
        let unsupported = self.binary_count as u64;
        let excluded = (self.excluded_count.saturating_sub(self.binary_count)) as u64;

        CompletenessReport::new(
            discovered,
            eligible,
            eligible, // Expected extracted count
            excluded,
            0, // Failed count
            unsupported,
        )
    }

    /// Translates this classification report into an initial pre-extraction CDM `CompletenessReport`.
    #[must_use]
    pub fn to_pre_extraction_completeness_report(&self) -> CompletenessReport {
        let discovered = self.total_files as u64;
        let eligible = self.eligible_count as u64;
        let unsupported = self.binary_count as u64;
        let excluded = (self.excluded_count.saturating_sub(self.binary_count)) as u64;

        CompletenessReport::new(
            discovered,
            eligible,
            0, // Not yet extracted
            excluded,
            0,
            unsupported,
        )
    }

    /// Converts this tree classification report into a structured hierarchical repository directory tree.
    #[must_use]
    pub fn to_repo_tree(&self) -> crate::tree_generator::RepoTree {
        crate::tree_generator::RepoTree::from_classified_files(&self.files)
    }
}

/// File classification engine implementing heuristic pattern matching and rules.
pub struct FileClassifier;

impl FileClassifier {
    /// Classifies a single `GitTreeEntry` according to configuration thresholds and heuristics.
    #[must_use]
    pub fn classify_entry(
        entry: &GitTreeEntry,
        config: &FileClassificationConfig,
    ) -> ClassifiedFile {
        let path = entry.path.clone();
        let file_name = entry.file_name();
        let extension = entry.extension().map(str::to_ascii_lowercase);
        let size = entry.size;

        // 1. Explicit user override takes top priority
        if let Some(&category) = config.custom_overrides.get(&path) {
            let is_eligible = category.is_extractable();
            let exclusion_reason = if is_eligible {
                None
            } else {
                Some("Explicit user override".to_string())
            };
            return ClassifiedFile {
                path,
                category,
                mode: entry.mode.clone(),
                sha: entry.sha.clone(),
                size,
                language: detect_language(file_name, extension.as_deref()),
                is_eligible,
                exclusion_reason,
            };
        }

        // 2. Excluded directory path patterns
        for prefix in &config.excluded_path_prefixes {
            if path.starts_with(prefix) || path.contains(&format!("/{prefix}")) {
                return ClassifiedFile {
                    path,
                    category: FileCategory::Other,
                    mode: entry.mode.clone(),
                    sha: entry.sha.clone(),
                    size,
                    language: detect_language(file_name, extension.as_deref()),
                    is_eligible: false,
                    exclusion_reason: Some(format!("Matches excluded path prefix '{prefix}'")),
                };
            }
        }

        // 3. Oversized check
        if let Some(byte_size) = size {
            if byte_size > config.max_file_size_bytes {
                return ClassifiedFile {
                    path,
                    category: FileCategory::Oversized,
                    mode: entry.mode.clone(),
                    sha: entry.sha.clone(),
                    size,
                    language: detect_language(file_name, extension.as_deref()),
                    is_eligible: false,
                    exclusion_reason: Some(format!(
                        "File size ({byte_size} bytes) exceeds limit ({})",
                        config.max_file_size_bytes
                    )),
                };
            }
        }

        // 4. Binary check
        if is_binary_extension(extension.as_deref()) {
            return ClassifiedFile {
                path,
                category: FileCategory::Binary,
                mode: entry.mode.clone(),
                sha: entry.sha.clone(),
                size,
                language: None,
                is_eligible: false,
                exclusion_reason: Some("Binary file format".to_string()),
            };
        }

        // 5. Dependency Manifest check
        if is_dependency_manifest(file_name) {
            return ClassifiedFile {
                path,
                category: FileCategory::DependencyManifest,
                mode: entry.mode.clone(),
                sha: entry.sha.clone(),
                size,
                language: detect_language(file_name, extension.as_deref()),
                is_eligible: true,
                exclusion_reason: None,
            };
        }

        // 6. Generated code or lockfile check
        if is_generated_or_lockfile(file_name) {
            let is_eligible = !config.exclude_lockfiles;
            let exclusion_reason = if is_eligible {
                None
            } else {
                Some("Package lockfile excluded by configuration".to_string())
            };
            return ClassifiedFile {
                path,
                category: FileCategory::GeneratedOrLockfile,
                mode: entry.mode.clone(),
                sha: entry.sha.clone(),
                size,
                language: detect_language(file_name, extension.as_deref()),
                is_eligible,
                exclusion_reason,
            };
        }

        // 7. Test fixture check
        if is_test_fixture(&path, file_name) {
            let is_eligible = !config.exclude_tests;
            let exclusion_reason = if is_eligible {
                None
            } else {
                Some("Test fixture excluded by configuration".to_string())
            };
            return ClassifiedFile {
                path,
                category: FileCategory::TestFixture,
                mode: entry.mode.clone(),
                sha: entry.sha.clone(),
                size,
                language: detect_language(file_name, extension.as_deref()),
                is_eligible,
                exclusion_reason,
            };
        }

        // 8. Documentation check
        if is_documentation_file(&path, file_name, extension.as_deref()) {
            return ClassifiedFile {
                path,
                category: FileCategory::Documentation,
                mode: entry.mode.clone(),
                sha: entry.sha.clone(),
                size,
                language: detect_language(file_name, extension.as_deref()),
                is_eligible: true,
                exclusion_reason: None,
            };
        }

        // 9. Configuration check
        if is_configuration_file(&path, file_name, extension.as_deref()) {
            return ClassifiedFile {
                path,
                category: FileCategory::Configuration,
                mode: entry.mode.clone(),
                sha: entry.sha.clone(),
                size,
                language: detect_language(file_name, extension.as_deref()),
                is_eligible: true,
                exclusion_reason: None,
            };
        }

        // 10. Source code check
        if is_source_code_extension(extension.as_deref()) {
            return ClassifiedFile {
                path,
                category: FileCategory::SourceCode,
                mode: entry.mode.clone(),
                sha: entry.sha.clone(),
                size,
                language: detect_language(file_name, extension.as_deref()),
                is_eligible: true,
                exclusion_reason: None,
            };
        }

        // 11. Fallback to Other
        ClassifiedFile {
            path,
            category: FileCategory::Other,
            mode: entry.mode.clone(),
            sha: entry.sha.clone(),
            size,
            language: detect_language(file_name, extension.as_deref()),
            is_eligible: false,
            exclusion_reason: Some("Unrecognized file format or category".to_string()),
        }
    }

    /// Classifies all blob entries in a `GitTreeResponse`, producing a consolidated report.
    #[must_use]
    pub fn classify_tree(
        tree_response: &GitTreeResponse,
        config: &FileClassificationConfig,
    ) -> TreeClassificationReport {
        let mut files = Vec::new();
        let mut category_counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut category_sizes: BTreeMap<String, u64> = BTreeMap::new();
        let mut eligible_count = 0;
        let mut excluded_count = 0;
        let mut binary_count = 0;
        let mut oversized_count = 0;

        for entry in tree_response.blobs() {
            let classified = Self::classify_entry(entry, config);
            let category_str = classified.category.as_str().to_string();

            *category_counts.entry(category_str.clone()).or_insert(0) += 1;
            if let Some(s) = classified.size {
                *category_sizes.entry(category_str).or_insert(0) += s;
            }

            if classified.is_eligible {
                eligible_count += 1;
            } else {
                excluded_count += 1;
            }

            if classified.category.is_binary() {
                binary_count += 1;
            } else if classified.category.is_oversized() {
                oversized_count += 1;
            }

            files.push(classified);
        }

        let total_files = files.len();

        TreeClassificationReport {
            files,
            total_files,
            category_counts,
            category_sizes,
            eligible_count,
            excluded_count,
            binary_count,
            oversized_count,
        }
    }
}

/// Detects programming language or syntax format based on file name or extension.
#[must_use]
pub fn detect_language(file_name: &str, extension: Option<&str>) -> Option<String> {
    if file_name.eq_ignore_ascii_case("dockerfile") || file_name.starts_with("Dockerfile.") {
        return Some("dockerfile".to_string());
    }
    if file_name.eq_ignore_ascii_case("makefile") || file_name.starts_with("Makefile.") {
        return Some("makefile".to_string());
    }

    let ext = extension?;
    let lang = match ext {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" | "mjs" | "cjs" => "javascript",
        "py" | "pyi" | "pyw" => "python",
        "go" => "go",
        "java" => "java",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => "cpp",
        "cs" => "csharp",
        "rb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "kt" | "kts" => "kotlin",
        "scala" => "scala",
        "sh" | "bash" | "zsh" => "shell",
        "sql" => "sql",
        "html" | "htm" => "html",
        "css" | "scss" | "sass" | "less" => "css",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "md" | "markdown" => "markdown",
        "xml" => "xml",
        "svelte" => "svelte",
        "vue" => "vue",
        "zig" => "zig",
        "lua" => "lua",
        "r" => "r",
        "dart" => "dart",
        "proto" => "protobuf",
        _ => return None,
    };

    Some(lang.to_string())
}

fn is_binary_extension(ext: Option<&str>) -> bool {
    let Some(e) = ext else { return false };
    matches!(
        e,
        "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "ico"
            | "bmp"
            | "webp"
            | "tiff"
            | "psd"
            | "pdf"
            | "wasm"
            | "so"
            | "dll"
            | "dylib"
            | "exe"
            | "bin"
            | "tar"
            | "gz"
            | "zip"
            | "jar"
            | "class"
            | "pyc"
            | "pyo"
            | "o"
            | "a"
            | "obj"
            | "lib"
            | "mp3"
            | "mp4"
            | "wav"
            | "avi"
            | "mov"
            | "webm"
            | "woff"
            | "woff2"
            | "ttf"
            | "eot"
            | "otf"
    )
}

fn is_dependency_manifest(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "cargo.toml"
            | "package.json"
            | "requirements.txt"
            | "pipfile"
            | "pyproject.toml"
            | "setup.py"
            | "setup.cfg"
            | "go.mod"
            | "go.sum"
            | "pom.xml"
            | "build.gradle"
            | "build.gradle.kts"
            | "settings.gradle"
            | "settings.gradle.kts"
            | "gemfile"
            | "composer.json"
            | "mix.exs"
            | "pubspec.yaml"
            | "flake.nix"
            | "default.nix"
            | "podfile"
            | "cartfile"
            | "package.swift"
    ) || (lower.starts_with("requirements-") && lower.ends_with(".txt"))
}

fn is_generated_or_lockfile(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "cargo.lock"
            | "package-lock.json"
            | "pnpm-lock.yaml"
            | "yarn.lock"
            | "composer.lock"
            | "poetry.lock"
            | "gemfile.lock"
            | "bun.lockb"
            | "flake.lock"
    ) || lower.ends_with(".min.js")
        || lower.ends_with(".min.css")
        || lower.ends_with(".bundle.js")
        || lower.ends_with(".pb.go")
        || lower.contains(".generated.")
}

fn is_test_fixture(path: &str, file_name: &str) -> bool {
    let lower_path = path.to_ascii_lowercase();
    let lower_name = file_name.to_ascii_lowercase();

    lower_path.starts_with("tests/")
        || lower_path.starts_with("test/")
        || lower_path.contains("/tests/")
        || lower_path.contains("/test/")
        || lower_path.contains("/__tests__/")
        || lower_path.contains("/fixtures/")
        || lower_path.contains("/spec/")
        || lower_path.contains("/specs/")
        || lower_name.ends_with("_test.rs")
        || lower_name.ends_with(".test.ts")
        || lower_name.ends_with(".test.js")
        || lower_name.ends_with(".test.tsx")
        || lower_name.ends_with(".test.jsx")
        || lower_name.ends_with(".spec.ts")
        || lower_name.ends_with(".spec.js")
        || lower_name.ends_with("_test.go")
        || lower_name.ends_with("_spec.rb")
        || lower_name.ends_with("_test.py")
        || lower_name.starts_with("test_")
        || lower_name.ends_with("test.java")
        || lower_name.ends_with("tests.java")
        || lower_name.ends_with("spec.scala")
}

fn is_documentation_file(path: &str, file_name: &str, ext: Option<&str>) -> bool {
    let lower_path = path.to_ascii_lowercase();
    let lower_name = file_name.to_ascii_lowercase();

    // Check directory
    if lower_path.starts_with("docs/")
        || lower_path.starts_with("doc/")
        || lower_path.contains("/docs/")
    {
        if let Some(e) = ext {
            if matches!(e, "md" | "markdown" | "rst" | "adoc" | "asciidoc" | "txt") {
                return true;
            }
        }
    }

    // Check specific documentation file names
    matches!(
        lower_name.as_str(),
        "readme"
            | "readme.md"
            | "readme.txt"
            | "readme.rst"
            | "changelog"
            | "changelog.md"
            | "changelog.txt"
            | "contributing"
            | "contributing.md"
            | "code_of_conduct"
            | "code_of_conduct.md"
            | "license"
            | "license.md"
            | "license.txt"
            | "license-mit"
            | "license-apache"
            | "security"
            | "security.md"
            | "roadmap"
            | "roadmap.md"
            | "decisions"
            | "decisions.md"
            | "todo"
            | "todo.md"
            | "authors"
            | "notice"
    ) || matches!(ext, Some("md" | "markdown" | "rst" | "adoc" | "asciidoc"))
}

fn is_configuration_file(path: &str, file_name: &str, ext: Option<&str>) -> bool {
    let lower_path = path.to_ascii_lowercase();
    let lower_name = file_name.to_ascii_lowercase();

    // GitHub workflows
    if lower_path.starts_with(".github/workflows/") || lower_path.contains("/.github/workflows/") {
        return true;
    }

    // Dotfiles and configs
    if matches!(
        lower_name.as_str(),
        ".gitignore"
            | ".gitattributes"
            | ".editorconfig"
            | ".env.example"
            | "tsconfig.json"
            | "deno.json"
            | "makefile"
            | "dockerfile"
            | "podmanfile"
            | "vagrantfile"
            | "procfile"
            | "deny.toml"
            | "rustfmt.toml"
            | "clippy.toml"
    ) || lower_name.starts_with(".eslintrc")
        || lower_name.starts_with(".prettierrc")
        || lower_name.starts_with("tsconfig.")
        || lower_name.starts_with("vitest.config.")
        || lower_name.starts_with("vite.config.")
        || lower_name.starts_with("svelte.config.")
        || lower_name.starts_with("tailwind.config.")
        || lower_name.starts_with("webpack.config.")
        || lower_name.starts_with("docker-compose.")
    {
        return true;
    }

    matches!(ext, Some("toml" | "yaml" | "yml" | "ini" | "cfg" | "conf"))
}

fn is_source_code_extension(ext: Option<&str>) -> bool {
    let Some(e) = ext else { return false };
    matches!(
        e,
        "rs" | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "mjs"
            | "cjs"
            | "py"
            | "pyi"
            | "go"
            | "java"
            | "c"
            | "h"
            | "cpp"
            | "cc"
            | "cxx"
            | "hpp"
            | "cs"
            | "rb"
            | "php"
            | "swift"
            | "kt"
            | "kts"
            | "scala"
            | "sh"
            | "bash"
            | "zsh"
            | "sql"
            | "html"
            | "css"
            | "scss"
            | "sass"
            | "less"
            | "json"
            | "svelte"
            | "vue"
            | "zig"
            | "lua"
            | "r"
            | "dart"
            | "proto"
            | "elm"
            | "clj"
            | "ex"
            | "exs"
            | "erl"
            | "hs"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{GitFileMode, GitTreeEntry, GitTreeEntryType};

    #[test]
    fn test_file_category_methods() {
        assert_eq!(FileCategory::SourceCode.as_str(), "source_code");
        assert_eq!(FileCategory::Documentation.as_str(), "documentation");
        assert_eq!(FileCategory::Configuration.as_str(), "configuration");
        assert_eq!(
            FileCategory::DependencyManifest.as_str(),
            "dependency_manifest"
        );
        assert_eq!(FileCategory::TestFixture.as_str(), "test_fixture");
        assert_eq!(
            FileCategory::GeneratedOrLockfile.as_str(),
            "generated_or_lockfile"
        );
        assert_eq!(FileCategory::Binary.as_str(), "binary");
        assert_eq!(FileCategory::Oversized.as_str(), "oversized");
        assert_eq!(FileCategory::Other.as_str(), "other");

        assert!(FileCategory::SourceCode.is_source_code());
        assert!(FileCategory::Documentation.is_documentation());
        assert!(FileCategory::Configuration.is_configuration());
        assert!(FileCategory::DependencyManifest.is_dependency_manifest());
        assert!(FileCategory::TestFixture.is_test());
        assert!(FileCategory::GeneratedOrLockfile.is_generated_or_lockfile());
        assert!(FileCategory::Binary.is_binary());
        assert!(FileCategory::Oversized.is_oversized());

        assert!(FileCategory::SourceCode.is_extractable());
        assert!(FileCategory::Documentation.is_extractable());
        assert!(FileCategory::Configuration.is_extractable());
        assert!(FileCategory::DependencyManifest.is_extractable());
        assert!(FileCategory::TestFixture.is_extractable());
        assert!(!FileCategory::GeneratedOrLockfile.is_extractable());
        assert!(!FileCategory::Binary.is_extractable());
        assert!(!FileCategory::Oversized.is_extractable());
        assert!(!FileCategory::Other.is_extractable());

        assert_eq!(format!("{}", FileCategory::SourceCode), "source_code");
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(
            detect_language("main.rs", Some("rs")).as_deref(),
            Some("rust")
        );
        assert_eq!(
            detect_language("index.ts", Some("ts")).as_deref(),
            Some("typescript")
        );
        assert_eq!(
            detect_language("app.py", Some("py")).as_deref(),
            Some("python")
        );
        assert_eq!(
            detect_language("Dockerfile", None).as_deref(),
            Some("dockerfile")
        );
        assert_eq!(
            detect_language("Dockerfile.dev", None).as_deref(),
            Some("dockerfile")
        );
        assert_eq!(
            detect_language("Makefile", None).as_deref(),
            Some("makefile")
        );
        assert_eq!(
            detect_language("README.md", Some("md")).as_deref(),
            Some("markdown")
        );
        assert_eq!(detect_language("unknown.xyz", Some("xyz")), None);
    }

    #[test]
    fn test_classify_entry_categories() {
        let config = FileClassificationConfig::default();

        let rs_entry = GitTreeEntry::new(
            "src/main.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha1",
            Some(1024),
        );
        let classified = FileClassifier::classify_entry(&rs_entry, &config);
        assert_eq!(classified.category, FileCategory::SourceCode);
        assert!(classified.is_eligible);
        assert_eq!(classified.language.as_deref(), Some("rust"));

        let doc_entry = GitTreeEntry::new(
            "docs/GUIDE.md",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha2",
            Some(2048),
        );
        let classified = FileClassifier::classify_entry(&doc_entry, &config);
        assert_eq!(classified.category, FileCategory::Documentation);
        assert!(classified.is_eligible);

        let cfg_entry = GitTreeEntry::new(
            "deno.json",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha3",
            Some(512),
        );
        let classified = FileClassifier::classify_entry(&cfg_entry, &config);
        assert_eq!(classified.category, FileCategory::Configuration);
        assert!(classified.is_eligible);

        let manifest_entry = GitTreeEntry::new(
            "Cargo.toml",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha4",
            Some(800),
        );
        let classified = FileClassifier::classify_entry(&manifest_entry, &config);
        assert_eq!(classified.category, FileCategory::DependencyManifest);
        assert!(classified.is_eligible);

        let lock_entry = GitTreeEntry::new(
            "Cargo.lock",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha5",
            Some(4096),
        );
        let classified = FileClassifier::classify_entry(&lock_entry, &config);
        assert_eq!(classified.category, FileCategory::GeneratedOrLockfile);
        assert!(!classified.is_eligible);

        let bin_entry = GitTreeEntry::new(
            "assets/logo.png",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha6",
            Some(8192),
        );
        let classified = FileClassifier::classify_entry(&bin_entry, &config);
        assert_eq!(classified.category, FileCategory::Binary);
        assert!(!classified.is_eligible);

        let huge_entry = GitTreeEntry::new(
            "src/huge.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha7",
            Some(2 * 1024 * 1024),
        );
        let classified = FileClassifier::classify_entry(&huge_entry, &config);
        assert_eq!(classified.category, FileCategory::Oversized);
        assert!(!classified.is_eligible);
    }

    #[test]
    fn test_custom_overrides_and_excluded_paths() {
        let config = FileClassificationConfig::default()
            .with_override("custom/special.xyz", FileCategory::Documentation)
            .with_override("src/internal.rs", FileCategory::Other);

        let entry1 = GitTreeEntry::new(
            "custom/special.xyz",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha1",
            Some(100),
        );
        let classified1 = FileClassifier::classify_entry(&entry1, &config);
        assert_eq!(classified1.category, FileCategory::Documentation);
        assert!(classified1.is_eligible);

        let entry2 = GitTreeEntry::new(
            "src/internal.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha2",
            Some(100),
        );
        let classified2 = FileClassifier::classify_entry(&entry2, &config);
        assert_eq!(classified2.category, FileCategory::Other);
        assert!(!classified2.is_eligible);

        // Excluded path prefix (node_modules)
        let nm_entry = GitTreeEntry::new(
            "node_modules/pkg/index.js",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha3",
            Some(500),
        );
        let classified_nm = FileClassifier::classify_entry(&nm_entry, &config);
        assert!(!classified_nm.is_eligible);
        assert!(classified_nm
            .exclusion_reason
            .unwrap()
            .contains("node_modules/"));
    }

    #[test]
    fn test_completeness_report_mapping() {
        let config = FileClassificationConfig::default();
        let entries = vec![
            GitTreeEntry::new(
                "src/lib.rs",
                GitFileMode::RegularFile,
                GitTreeEntryType::Blob,
                "s1",
                Some(1000),
            ),
            GitTreeEntry::new(
                "README.md",
                GitFileMode::RegularFile,
                GitTreeEntryType::Blob,
                "s2",
                Some(500),
            ),
            GitTreeEntry::new(
                "Cargo.lock",
                GitFileMode::RegularFile,
                GitTreeEntryType::Blob,
                "s3",
                Some(2000),
            ),
            GitTreeEntry::new(
                "logo.png",
                GitFileMode::RegularFile,
                GitTreeEntryType::Blob,
                "s4",
                Some(3000),
            ),
        ];
        let tree = GitTreeResponse::new("root_sha", entries, false);
        let report = tree.classify(&config);

        assert_eq!(report.total_files, 4);
        assert_eq!(report.eligible_count, 2); // src/lib.rs, README.md
        assert_eq!(report.excluded_count, 2); // Cargo.lock, logo.png
        assert_eq!(report.binary_count, 1); // logo.png

        let expected_cdm = report.to_expected_completeness_report();
        assert!(expected_cdm.formula_valid());
        assert_eq!(expected_cdm.discovered_count, 4);
        assert_eq!(expected_cdm.eligible_count, 2);
        assert_eq!(expected_cdm.extracted_count, 2);
        assert_eq!(expected_cdm.unsupported_count, 1);
        assert_eq!(expected_cdm.excluded_count, 1);
        assert_eq!(expected_cdm.failed_count, 0);

        let pre_cdm = report.to_pre_extraction_completeness_report();
        assert_eq!(pre_cdm.extracted_count, 0);
        assert_eq!(pre_cdm.discovered_count, 4);
    }
}
