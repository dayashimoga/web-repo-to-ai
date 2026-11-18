//! # GitHub Dependency and Manifest Extraction
//!
//! Provides specialized acquisition, categorization, and deterministic parsing of package
//! manifests and dependency lockfiles:
//! - Package manifests (`Cargo.toml`, `package.json`, `requirements.txt`, `go.mod`, `pyproject.toml`)
//! - Dependency lockfiles (`Cargo.lock`, `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, `go.sum`)
//! - Multi-ecosystem detection across Rust, Node.js/TypeScript, Python, Go, Java/Gradle, Ruby, PHP, Dart/Flutter, and Elixir
//! - Extraction of package identity (name, version, description, license) and declared dependencies (Direct, Dev, Build, Peer, Optional)
//! - Conversion of manifests into Canonical Context Document Model ([`CdmSection`]) manifest sections ([`SectionKind::Manifest`]).

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

/// Supported package and build ecosystems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageEcosystem {
    /// Rust Cargo (`Cargo.toml`, `Cargo.lock`).
    Cargo,
    /// Node.js / TypeScript npm / pnpm / yarn / bun (`package.json`, `package-lock.json`, etc.).
    Npm,
    /// Python pip / poetry / pipenv (`requirements.txt`, `pyproject.toml`, `Pipfile`, etc.).
    Pip,
    /// Go Modules (`go.mod`, `go.sum`).
    GoMod,
    /// Java / Kotlin Maven and Gradle (`pom.xml`, `build.gradle`, etc.).
    MavenGradle,
    /// Ruby Bundler (`Gemfile`, `Gemfile.lock`).
    Bundler,
    /// PHP Composer (`composer.json`, `composer.lock`).
    Composer,
    /// Dart and Flutter Pub (`pubspec.yaml`, `pubspec.lock`).
    Pub,
    /// Elixir Mix (`mix.exs`, `mix.lock`).
    Mix,
    /// Other or unrecognized package ecosystem.
    Other,
}

impl PackageEcosystem {
    /// Returns the snake_case identifier for this package ecosystem.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cargo => "cargo",
            Self::Npm => "npm",
            Self::Pip => "pip",
            Self::GoMod => "go_mod",
            Self::MavenGradle => "maven_gradle",
            Self::Bundler => "bundler",
            Self::Composer => "composer",
            Self::Pub => "pub",
            Self::Mix => "mix",
            Self::Other => "other",
        }
    }

    /// Returns the primary manifest language format identifier (e.g. "toml", "json").
    #[must_use]
    pub const fn format_name(self) -> &'static str {
        match self {
            Self::Cargo => "toml",
            Self::Npm | Self::Composer => "json",
            Self::Pip => "python",
            Self::GoMod => "go",
            Self::MavenGradle => "xml",
            Self::Bundler | Self::Mix => "ruby",
            Self::Pub => "yaml",
            Self::Other => "text",
        }
    }
}

/// Detects the package ecosystem from the file name.
#[must_use]
pub fn detect_ecosystem(file_name: &str) -> PackageEcosystem {
    let lower = file_name.to_ascii_lowercase();
    match lower.as_str() {
        "cargo.toml" | "cargo.lock" => PackageEcosystem::Cargo,
        "package.json" | "package-lock.json" | "pnpm-lock.yaml" | "yarn.lock" | "bun.lockb" => {
            PackageEcosystem::Npm
        }
        "requirements.txt" | "pipfile" | "pipfile.lock" | "pyproject.toml" | "setup.py"
        | "setup.cfg" | "poetry.lock" => PackageEcosystem::Pip,
        "go.mod" | "go.sum" => PackageEcosystem::GoMod,
        "pom.xml"
        | "build.gradle"
        | "build.gradle.kts"
        | "settings.gradle"
        | "settings.gradle.kts" => PackageEcosystem::MavenGradle,
        "gemfile" | "gemfile.lock" => PackageEcosystem::Bundler,
        "composer.json" | "composer.lock" => PackageEcosystem::Composer,
        "pubspec.yaml" | "pubspec.lock" => PackageEcosystem::Pub,
        "mix.exs" | "mix.lock" => PackageEcosystem::Mix,
        _ => {
            if lower.starts_with("requirements-") && lower.ends_with(".txt") {
                PackageEcosystem::Pip
            } else {
                PackageEcosystem::Other
            }
        }
    }
}

/// Functional role of a dependency configuration file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestRole {
    /// Semantic manifest declaring package specifications and direct dependencies.
    Manifest,
    /// Resolved lockfile pinning transitive dependencies and cryptographic digests.
    Lockfile,
}

impl ManifestRole {
    /// Returns the snake_case identifier for this manifest role.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Manifest => "manifest",
            Self::Lockfile => "lockfile",
        }
    }

    /// Returns whether this file is a manifest.
    #[must_use]
    pub const fn is_manifest(self) -> bool {
        matches!(self, Self::Manifest)
    }

    /// Returns whether this file is a lockfile.
    #[must_use]
    pub const fn is_lockfile(self) -> bool {
        matches!(self, Self::Lockfile)
    }
}

/// Detects the manifest role (manifest or lockfile) from the file name.
#[must_use]
pub fn detect_manifest_role(file_name: &str) -> ManifestRole {
    let lower = file_name.to_ascii_lowercase();
    if lower.ends_with(".lock")
        || lower == "package-lock.json"
        || lower == "pnpm-lock.yaml"
        || lower == "yarn.lock"
        || lower == "bun.lockb"
        || lower == "go.sum"
        || lower == "pipfile.lock"
        || lower == "poetry.lock"
        || lower == "pubspec.lock"
        || lower == "mix.lock"
        || lower == "composer.lock"
    {
        ManifestRole::Lockfile
    } else {
        ManifestRole::Manifest
    }
}

/// Functional scope/type of a declared dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
    /// Runtime / production library dependency.
    Direct,
    /// Development, testing, or benchmark dependency.
    Dev,
    /// Build-script or compiler plugin dependency.
    Build,
    /// Peer / host dependency (e.g. npm peerDependencies).
    Peer,
    /// Optional or conditional feature dependency.
    Optional,
}

impl DependencyType {
    /// Returns the string representation of this dependency type.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Dev => "dev",
            Self::Build => "build",
            Self::Peer => "peer",
            Self::Optional => "optional",
        }
    }
}

/// An individual declared dependency extracted from a manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclaredDependency {
    /// Package or module identifier.
    pub name: String,
    /// Version constraint or requirement string (e.g. `^1.0`, `>=2.0.0`, `1.2.3`), if specified.
    pub version_req: Option<String>,
    /// Functional dependency classification.
    pub dependency_type: DependencyType,
    /// Whether this dependency is conditionally enabled via an optional flag.
    pub optional: bool,
    /// Feature flags requested for this dependency.
    pub features: Vec<String>,
}

impl DeclaredDependency {
    /// Creates a new `DeclaredDependency`.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        version_req: Option<String>,
        dependency_type: DependencyType,
    ) -> Self {
        Self {
            name: name.into(),
            version_req,
            dependency_type,
            optional: false,
            features: Vec::new(),
        }
    }

    /// Creates a new `DeclaredDependency` without a specified version requirement.
    #[must_use]
    pub fn without_version(name: impl Into<String>, dependency_type: DependencyType) -> Self {
        Self::new(name, None, dependency_type)
    }

    /// Sets the optional flag for this dependency.
    #[must_use]
    pub const fn with_optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }

    /// Sets enabled features for this dependency.
    #[must_use]
    pub fn with_features(mut self, features: Vec<String>) -> Self {
        self.features = features;
        self
    }
}

/// Parsed metadata and dependency declarations from a package manifest.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedManifest {
    /// Declared package name, if present.
    pub package_name: Option<String>,
    /// Declared package version, if present.
    pub package_version: Option<String>,
    /// Short package description, if present.
    pub description: Option<String>,
    /// Declared package license, if present.
    pub license: Option<String>,
    /// Extracted list of declared dependencies.
    pub dependencies: Vec<DeclaredDependency>,
}

impl ParsedManifest {
    /// Creates a new empty `ParsedManifest`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns direct runtime dependencies.
    #[must_use]
    pub fn direct_dependencies(&self) -> Vec<&DeclaredDependency> {
        self.dependencies
            .iter()
            .filter(|d| d.dependency_type == DependencyType::Direct)
            .collect()
    }

    /// Returns development/test dependencies.
    #[must_use]
    pub fn dev_dependencies(&self) -> Vec<&DeclaredDependency> {
        self.dependencies
            .iter()
            .filter(|d| d.dependency_type == DependencyType::Dev)
            .collect()
    }

    /// Returns the total number of dependencies declared.
    #[must_use]
    pub fn dependency_count(&self) -> usize {
        self.dependencies.len()
    }

    /// Finds a dependency by name.
    #[must_use]
    pub fn find_dependency(&self, name: &str) -> Option<&DeclaredDependency> {
        self.dependencies.iter().find(|d| d.name == name)
    }
}

// ---------------------------------------------------------------------------
// Deterministic Manifest Parsers
// ---------------------------------------------------------------------------

/// Lightweight deterministic parser for `Cargo.toml`.
#[must_use]
pub fn parse_cargo_toml(content: &str) -> ParsedManifest {
    let mut manifest = ParsedManifest::new();
    let mut current_section = String::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();

        // Skip blank lines and pure comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Section header
        if line.starts_with('[') && line.ends_with(']') {
            let inner = line.trim_start_matches('[').trim_end_matches(']').trim();
            current_section = inner.to_ascii_lowercase();
            continue;
        }

        // Package metadata
        if current_section == "package" {
            if let Some((key, val)) = parse_toml_key_value(line) {
                match key.as_str() {
                    "name" => manifest.package_name = Some(val),
                    "version" => manifest.package_version = Some(val),
                    "description" => manifest.description = Some(val),
                    "license" => manifest.license = Some(val),
                    _ => {}
                }
            }
            continue;
        }

        // Dependency sections
        let dep_type = if current_section == "dependencies"
            || current_section == "workspace.dependencies"
            || current_section.ends_with(".dependencies")
        {
            Some(DependencyType::Direct)
        } else if current_section == "dev-dependencies"
            || current_section.ends_with(".dev-dependencies")
        {
            Some(DependencyType::Dev)
        } else if current_section == "build-dependencies"
            || current_section.ends_with(".build-dependencies")
        {
            Some(DependencyType::Build)
        } else {
            None
        };

        if let Some(dtype) = dep_type {
            if let Some(dep) = parse_cargo_dependency_line(line, dtype) {
                manifest.dependencies.push(dep);
            }
        }
    }

    manifest
}

fn parse_toml_key_value(line: &str) -> Option<(String, String)> {
    let (key_part, val_part) = line.split_once('=')?;
    let key = key_part.trim().to_ascii_lowercase();
    let val_trimmed = val_part.trim();

    // Strip comments after value
    let val_clean = val_trimmed.split('#').next().unwrap_or(val_trimmed).trim();

    // Strip outer quotes if string
    let val = if (val_clean.starts_with('"') && val_clean.ends_with('"'))
        || (val_clean.starts_with('\'') && val_clean.ends_with('\''))
    {
        val_clean[1..val_clean.len().saturating_sub(1)].to_string()
    } else {
        val_clean.to_string()
    };

    Some((key, val))
}

fn parse_cargo_dependency_line(line: &str, dtype: DependencyType) -> Option<DeclaredDependency> {
    let (key_part, val_part) = line.split_once('=')?;
    let name = key_part
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string();
    if name.is_empty() {
        return None;
    }

    let val_trimmed = val_part.trim();
    // Strip trailing comment
    let val_clean = val_trimmed.split('#').next().unwrap_or(val_trimmed).trim();

    // Case 1: Simple string version requirement: `dep = "1.0.0"`
    if val_clean.starts_with('"') || val_clean.starts_with('\'') {
        let ver = val_clean.trim_matches('"').trim_matches('\'').to_string();
        return Some(DeclaredDependency::new(name, Some(ver), dtype));
    }

    // Case 2: Inline table: `dep = { version = "1.0", features = ["derive"], optional = true }`
    if val_clean.starts_with('{') {
        let inner = val_clean
            .trim_start_matches('{')
            .trim_end_matches('}')
            .trim();
        let mut version_req = None;
        let mut optional = false;
        let mut features = Vec::new();

        // Split key-values by comma
        for item in inner.split(',') {
            if let Some((k, v)) = item.split_once('=') {
                let k_trim = k.trim().to_ascii_lowercase();
                let v_trim = v.trim();
                match k_trim.as_str() {
                    "version" => {
                        let ver = v_trim.trim_matches('"').trim_matches('\'').to_string();
                        version_req = Some(ver);
                    }
                    "optional" => {
                        optional = v_trim.starts_with("true");
                    }
                    "features" => {
                        let array_content = v_trim.trim_matches('[').trim_matches(']');
                        for feat in array_content.split(',') {
                            let feat_trim = feat.trim().trim_matches('"').trim_matches('\'');
                            if !feat_trim.is_empty() {
                                features.push(feat_trim.to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut dep = DeclaredDependency::new(name, version_req, dtype);
        if optional {
            dep = dep.with_optional(true);
        }
        if !features.is_empty() {
            dep = dep.with_features(features);
        }
        return Some(dep);
    }

    Some(DeclaredDependency::without_version(name, dtype))
}

/// Deterministic parser for Node.js `package.json`.
#[must_use]
pub fn parse_package_json(content: &str) -> ParsedManifest {
    let mut manifest = ParsedManifest::new();

    let Ok(val) = serde_json::from_str::<serde_json::Value>(content) else {
        return manifest;
    };

    if let Some(obj) = val.as_object() {
        manifest.package_name = obj
            .get("name")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        manifest.package_version = obj
            .get("version")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        manifest.description = obj
            .get("description")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        manifest.license = obj
            .get("license")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let sections = [
            ("dependencies", DependencyType::Direct),
            ("devDependencies", DependencyType::Dev),
            ("peerDependencies", DependencyType::Peer),
            ("optionalDependencies", DependencyType::Optional),
        ];

        for (section_name, dep_type) in sections {
            if let Some(deps_obj) = obj.get(section_name).and_then(|v| v.as_object()) {
                for (pkg, ver_val) in deps_obj {
                    let version_str = match ver_val {
                        serde_json::Value::String(s) => Some(s.clone()),
                        serde_json::Value::Number(n) => Some(n.to_string()),
                        _ => None,
                    };
                    let optional = dep_type == DependencyType::Optional;
                    let dep =
                        DeclaredDependency::new(pkg, version_str, dep_type).with_optional(optional);
                    manifest.dependencies.push(dep);
                }
            }
        }
    }

    manifest
}

/// Deterministic parser for Python `requirements.txt`.
#[must_use]
pub fn parse_requirements_txt(content: &str) -> ParsedManifest {
    let mut manifest = ParsedManifest::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();

        // Skip blank lines, comments, and option flags (-r, -i, -f, --extra-index-url)
        if line.is_empty() || line.starts_with('#') || line.starts_with('-') {
            continue;
        }

        // Strip inline comments
        let clean = line.split('#').next().unwrap_or(line).trim();
        if clean.is_empty() {
            continue;
        }

        // Split package name and version operators (==, >=, <=, ~=, !=, >, <)
        let ops = ["==", ">=", "<=", "~=", "!=", "===", ">", "<"];
        let mut found_op = None;

        for op in ops {
            if let Some(idx) = clean.find(op) {
                found_op = Some((idx, op));
                break;
            }
        }

        if let Some((idx, op)) = found_op {
            let name = clean[..idx].trim().to_string();
            let version = clean[idx + op.len()..].trim().to_string();
            if !name.is_empty() {
                let full_version = format!("{op}{version}");
                manifest.dependencies.push(DeclaredDependency::new(
                    name,
                    Some(full_version),
                    DependencyType::Direct,
                ));
            }
        } else {
            // Package without pinned version
            manifest
                .dependencies
                .push(DeclaredDependency::without_version(
                    clean,
                    DependencyType::Direct,
                ));
        }
    }

    manifest
}

/// Deterministic parser for Go `go.mod`.
#[must_use]
pub fn parse_go_mod(content: &str) -> ParsedManifest {
    let mut manifest = ParsedManifest::new();
    let mut in_require_block = false;

    for raw_line in content.lines() {
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        if let Some(mod_name) = line.strip_prefix("module ") {
            manifest.package_name = Some(mod_name.trim().to_string());
            continue;
        }

        if let Some(go_ver) = line.strip_prefix("go ") {
            manifest.package_version = Some(go_ver.trim().to_string());
            continue;
        }

        if line.starts_with("require (") {
            in_require_block = true;
            continue;
        }

        if in_require_block {
            if line == ")" {
                in_require_block = false;
                continue;
            }
            if let Some(dep) = parse_go_require_entry(line) {
                manifest.dependencies.push(dep);
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("require ") {
            if let Some(dep) = parse_go_require_entry(rest) {
                manifest.dependencies.push(dep);
            }
        }
    }

    manifest
}

fn parse_go_require_entry(line: &str) -> Option<DeclaredDependency> {
    let clean = line.split("//").next().unwrap_or(line).trim();
    let mut parts = clean.split_whitespace();
    let mod_path = parts.next()?;
    let version = parts.next().map(ToString::to_string);

    let is_indirect = line.contains("// indirect");
    let dep_type = if is_indirect {
        DependencyType::Dev
    } else {
        DependencyType::Direct
    };

    Some(DeclaredDependency::new(mod_path, version, dep_type))
}

/// Deterministic parser for Python `pyproject.toml`.
#[must_use]
pub fn parse_pyproject_toml(content: &str) -> ParsedManifest {
    let mut manifest = ParsedManifest::new();
    let mut current_section = String::new();
    let mut in_dependencies_array = false;

    for raw_line in content.lines() {
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            let inner = line.trim_start_matches('[').trim_end_matches(']').trim();
            current_section = inner.to_ascii_lowercase();
            in_dependencies_array = false;
            continue;
        }

        // PEP 621 [project]
        if current_section == "project" {
            if let Some((k, v)) = parse_toml_key_value(line) {
                match k.as_str() {
                    "name" => manifest.package_name = Some(v),
                    "version" => manifest.package_version = Some(v),
                    "description" => manifest.description = Some(v),
                    _ => {}
                }
            }

            if line.starts_with("dependencies = [") || line == "dependencies = [" {
                in_dependencies_array = true;
                continue;
            }
        }

        // Poetry [tool.poetry]
        if current_section == "tool.poetry" {
            if let Some((k, v)) = parse_toml_key_value(line) {
                match k.as_str() {
                    "name" => manifest.package_name = Some(v),
                    "version" => manifest.package_version = Some(v),
                    "description" => manifest.description = Some(v),
                    "license" => manifest.license = Some(v),
                    _ => {}
                }
            }
        }

        // Poetry dependencies
        if current_section == "tool.poetry.dependencies" {
            if let Some(dep) = parse_cargo_dependency_line(line, DependencyType::Direct) {
                manifest.dependencies.push(dep);
            }
            continue;
        }

        if in_dependencies_array {
            if line.starts_with(']') {
                in_dependencies_array = false;
                continue;
            }
            let item_clean = line
                .trim_matches(',')
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if !item_clean.is_empty() {
                // Parse version specifiers if present
                let ops = ["==", ">=", "<=", "~=", "!=", "===", ">", "<"];
                let mut found_op = None;
                for op in ops {
                    if let Some(idx) = item_clean.find(op) {
                        found_op = Some((idx, op));
                        break;
                    }
                }
                if let Some((idx, op)) = found_op {
                    let name = item_clean[..idx].trim().to_string();
                    let ver = item_clean[idx + op.len()..].trim().to_string();
                    manifest.dependencies.push(DeclaredDependency::new(
                        name,
                        Some(format!("{op}{ver}")),
                        DependencyType::Direct,
                    ));
                } else {
                    manifest
                        .dependencies
                        .push(DeclaredDependency::without_version(
                            item_clean,
                            DependencyType::Direct,
                        ));
                }
            }
        }
    }

    manifest
}

/// Parses manifest text based on the detected package ecosystem and file name.
#[must_use]
pub fn parse_manifest(
    ecosystem: PackageEcosystem,
    file_name: &str,
    content: &str,
) -> Option<ParsedManifest> {
    let lower = file_name.to_ascii_lowercase();

    match ecosystem {
        PackageEcosystem::Cargo => {
            if lower == "cargo.toml" {
                Some(parse_cargo_toml(content))
            } else {
                None
            }
        }
        PackageEcosystem::Npm => {
            if lower == "package.json" {
                Some(parse_package_json(content))
            } else {
                None
            }
        }
        PackageEcosystem::Pip => {
            if lower == "requirements.txt"
                || (lower.starts_with("requirements-") && lower.ends_with(".txt"))
            {
                Some(parse_requirements_txt(content))
            } else if lower == "pyproject.toml" {
                Some(parse_pyproject_toml(content))
            } else {
                None
            }
        }
        PackageEcosystem::GoMod => {
            if lower == "go.mod" {
                Some(parse_go_mod(content))
            } else {
                None
            }
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Extracted Models
// ---------------------------------------------------------------------------

/// An extracted, verified, and structured repository dependency or lockfile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedDependencyFile {
    /// Relative repository path (e.g. `Cargo.toml`, `package.json`, `Cargo.lock`).
    pub path: String,
    /// 40-character Git object SHA-1 hash.
    pub git_sha: String,
    /// Evaluated package ecosystem.
    pub ecosystem: PackageEcosystem,
    /// Functional manifest role (`Manifest` vs `Lockfile`).
    pub role: ManifestRole,
    /// Normalized text content.
    pub content: String,
    /// Parsed manifest metadata and dependencies, if this file is a supported manifest.
    pub parsed: Option<ParsedManifest>,
    /// Content length in bytes.
    pub byte_size: usize,
    /// Total lines in the file.
    pub line_count: usize,
    /// Cryptographic BLAKE3 digest of the content bytes.
    pub blake3_hash: String,
    /// Cryptographic SHA-256 digest of the content bytes.
    pub sha256_hash: String,
}

impl ExtractedDependencyFile {
    /// Constructs an `ExtractedDependencyFile` from raw blob bytes and path.
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
                    "Git blob SHA-1 mismatch for dependency file '{path}': expected '{}', got '{actual}'",
                    blob.sha
                ),
            });
        }

        if blob.is_binary() {
            return Err(GitHubExtractorError::BlobError {
                sha: blob.sha.clone(),
                message: format!("Binary or null bytes detected in dependency file '{path}'"),
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
        let ecosystem = detect_ecosystem(file_name);
        let role = detect_manifest_role(file_name);

        let parsed = if role.is_manifest() {
            parse_manifest(ecosystem, file_name, &content)
        } else {
            None
        };

        let blake3_hash = compute_blake3_hash(content.as_bytes());
        let sha256_hash = compute_sha256_hash(content.as_bytes());

        Ok(Self {
            path,
            git_sha: blob.sha.clone(),
            ecosystem,
            role,
            content,
            parsed,
            byte_size,
            line_count,
            blake3_hash,
            sha256_hash,
        })
    }

    /// Converts this extracted dependency file into a Canonical Context Document Model [`CdmSection`].
    #[must_use]
    pub fn to_cdm_section(
        &self,
        section_id: impl Into<String>,
        source_url: impl Into<String>,
    ) -> CdmSection {
        let lines = self.line_count as u32;
        let title = format!("Manifest: {}", self.path);
        let mut section =
            CdmSectionBuilder::new(section_id, source_url, SectionKind::Manifest, &self.content)
                .title(&title)
                .original_path(&self.path)
                .line_range(1, lines.max(1))
                .byte_range(0, self.byte_size as u64)
                .language(self.ecosystem.format_name())
                .content_hash(&self.blake3_hash)
                .token_metrics(SectionTokenMetrics::compute_baseline(&self.content))
                .build();

        section
            .attributes
            .insert("ecosystem".to_string(), self.ecosystem.as_str().to_string());
        section
            .attributes
            .insert("manifest_role".to_string(), self.role.as_str().to_string());
        section
            .attributes
            .insert("git_blob_sha".to_string(), self.git_sha.clone());
        section
            .attributes
            .insert("sha256".to_string(), self.sha256_hash.clone());

        if let Some(parsed) = &self.parsed {
            if let Some(pkg) = &parsed.package_name {
                section
                    .attributes
                    .insert("package_name".to_string(), pkg.clone());
            }
            if let Some(ver) = &parsed.package_version {
                section
                    .attributes
                    .insert("package_version".to_string(), ver.clone());
            }
            section.attributes.insert(
                "dependency_count".to_string(),
                parsed.dependency_count().to_string(),
            );
        }

        section
    }
}

/// Aggregated outcome and metrics from repository dependency extraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyExtractionReport {
    /// Successfully acquired and verified dependency files.
    pub extracted: Vec<ExtractedDependencyFile>,
    /// Files that encountered errors during fetch or validation.
    pub failed: Vec<FailedSourceExtraction>,
    /// Files deemed unsupported (e.g. non-text binary).
    pub unsupported: Vec<ClassifiedFile>,
    /// Total bytes acquired across all extracted dependency files.
    pub total_bytes: u64,
    /// Total lines spanned across all extracted dependency files.
    pub total_lines: usize,
    /// Wall-clock duration elapsed during extraction.
    pub duration: Duration,
}

impl Default for DependencyExtractionReport {
    fn default() -> Self {
        Self {
            extracted: Vec::new(),
            failed: Vec::new(),
            unsupported: Vec::new(),
            total_bytes: 0,
            total_lines: 0,
            duration: Duration::ZERO,
        }
    }
}

impl DependencyExtractionReport {
    /// Creates a new empty `DependencyExtractionReport`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a successfully extracted dependency file.
    pub fn add_extracted(&mut self, file: ExtractedDependencyFile) {
        self.total_bytes += file.byte_size as u64;
        self.total_lines += file.line_count;
        self.extracted.push(file);
    }

    /// Appends a failed dependency extraction attempt.
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

    /// Appends an unsupported dependency file entry.
    pub fn add_unsupported(&mut self, file: ClassifiedFile) {
        self.unsupported.push(file);
    }

    /// Returns all extracted dependency files matching a specific package ecosystem.
    #[must_use]
    pub fn filter_by_ecosystem(
        &self,
        ecosystem: PackageEcosystem,
    ) -> Vec<&ExtractedDependencyFile> {
        self.extracted
            .iter()
            .filter(|f| f.ecosystem == ecosystem)
            .collect()
    }

    /// Returns all semantic manifests (excluding lockfiles).
    #[must_use]
    pub fn manifests(&self) -> Vec<&ExtractedDependencyFile> {
        self.extracted
            .iter()
            .filter(|f| f.role.is_manifest())
            .collect()
    }

    /// Returns all resolved dependency lockfiles.
    #[must_use]
    pub fn lockfiles(&self) -> Vec<&ExtractedDependencyFile> {
        self.extracted
            .iter()
            .filter(|f| f.role.is_lockfile())
            .collect()
    }

    /// Collects all direct dependencies declared across all parsed manifests.
    #[must_use]
    pub fn all_direct_dependencies(&self) -> Vec<DeclaredDependency> {
        let mut all_deps = Vec::new();
        for file in &self.extracted {
            if let Some(parsed) = &file.parsed {
                for dep in parsed.direct_dependencies() {
                    all_deps.push(dep.clone());
                }
            }
        }
        all_deps
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

    /// Converts all extracted dependency files into CDM [`CdmSection`]s.
    #[must_use]
    pub fn to_cdm_sections(&self, base_url: &str) -> Vec<CdmSection> {
        let clean_base = base_url.trim_end_matches('/');
        self.extracted
            .iter()
            .enumerate()
            .map(|(idx, file)| {
                let section_id = format!("sec-manifest-{:04}", idx + 1);
                let source_url = format!("{clean_base}/{}", file.path);
                file.to_cdm_section(section_id, source_url)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ecosystem_and_role() {
        assert_eq!(detect_ecosystem("Cargo.toml"), PackageEcosystem::Cargo);
        assert_eq!(detect_manifest_role("Cargo.toml"), ManifestRole::Manifest);

        assert_eq!(detect_ecosystem("Cargo.lock"), PackageEcosystem::Cargo);
        assert_eq!(detect_manifest_role("Cargo.lock"), ManifestRole::Lockfile);

        assert_eq!(detect_ecosystem("package.json"), PackageEcosystem::Npm);
        assert_eq!(detect_manifest_role("package.json"), ManifestRole::Manifest);

        assert_eq!(detect_ecosystem("package-lock.json"), PackageEcosystem::Npm);
        assert_eq!(
            detect_manifest_role("package-lock.json"),
            ManifestRole::Lockfile
        );

        assert_eq!(detect_ecosystem("requirements.txt"), PackageEcosystem::Pip);
        assert_eq!(
            detect_manifest_role("requirements.txt"),
            ManifestRole::Manifest
        );

        assert_eq!(
            detect_ecosystem("requirements-dev.txt"),
            PackageEcosystem::Pip
        );
        assert_eq!(
            detect_manifest_role("requirements-dev.txt"),
            ManifestRole::Manifest
        );

        assert_eq!(detect_ecosystem("go.mod"), PackageEcosystem::GoMod);
        assert_eq!(detect_manifest_role("go.mod"), ManifestRole::Manifest);

        assert_eq!(detect_ecosystem("go.sum"), PackageEcosystem::GoMod);
        assert_eq!(detect_manifest_role("go.sum"), ManifestRole::Lockfile);
    }

    #[test]
    fn test_parse_cargo_toml() {
        let content = r#"
[package]
name = "my-crate"
version = "0.2.1"
description = "A great crate"
license = "MIT"

[dependencies]
tokio = "1.40"
serde = { version = "1.0", features = ["derive"], optional = true }

[dev-dependencies]
tempfile = "3.8"
"#;
        let parsed = parse_cargo_toml(content);
        assert_eq!(parsed.package_name.as_deref(), Some("my-crate"));
        assert_eq!(parsed.package_version.as_deref(), Some("0.2.1"));
        assert_eq!(parsed.description.as_deref(), Some("A great crate"));
        assert_eq!(parsed.license.as_deref(), Some("MIT"));
        assert_eq!(parsed.dependencies.len(), 3);

        let tokio_dep = parsed.find_dependency("tokio").expect("tokio found");
        assert_eq!(tokio_dep.version_req.as_deref(), Some("1.40"));
        assert_eq!(tokio_dep.dependency_type, DependencyType::Direct);
        assert!(!tokio_dep.optional);

        let serde_dep = parsed.find_dependency("serde").expect("serde found");
        assert_eq!(serde_dep.version_req.as_deref(), Some("1.0"));
        assert!(serde_dep.optional);
        assert_eq!(serde_dep.features, vec!["derive".to_string()]);

        let tempfile_dep = parsed.find_dependency("tempfile").expect("tempfile found");
        assert_eq!(tempfile_dep.dependency_type, DependencyType::Dev);
    }

    #[test]
    fn test_parse_package_json() {
        let content = r#"{
  "name": "my-web-app",
  "version": "1.0.0",
  "description": "Frontend app",
  "license": "Apache-2.0",
  "dependencies": {
    "svelte": "^5.0.0",
    "tailwindcss": "^3.4.0"
  },
  "devDependencies": {
    "typescript": "^5.5.0",
    "vitest": "^2.0.0"
  }
}"#;
        let parsed = parse_package_json(content);
        assert_eq!(parsed.package_name.as_deref(), Some("my-web-app"));
        assert_eq!(parsed.package_version.as_deref(), Some("1.0.0"));
        assert_eq!(parsed.dependencies.len(), 4);

        let svelte = parsed.find_dependency("svelte").expect("svelte found");
        assert_eq!(svelte.version_req.as_deref(), Some("^5.0.0"));
        assert_eq!(svelte.dependency_type, DependencyType::Direct);

        let ts = parsed.find_dependency("typescript").expect("ts found");
        assert_eq!(ts.dependency_type, DependencyType::Dev);
    }

    #[test]
    fn test_parse_requirements_txt() {
        let content = r#"
# Core libraries
fastapi==0.112.0
uvicorn>=0.30.0
pydantic~=2.8.0
pytest
-r other-requirements.txt
"#;
        let parsed = parse_requirements_txt(content);
        assert_eq!(parsed.dependencies.len(), 4);

        let fastapi = parsed.find_dependency("fastapi").expect("fastapi found");
        assert_eq!(fastapi.version_req.as_deref(), Some("==0.112.0"));

        let pytest = parsed.find_dependency("pytest").expect("pytest found");
        assert_eq!(pytest.version_req, None);
    }

    #[test]
    fn test_parse_go_mod() {
        let content = r#"
module github.com/owner/myrepo

go 1.22.0

require (
	github.com/gin-gonic/gin v1.9.1
	github.com/stretchr/testify v1.8.4 // indirect
)
"#;
        let parsed = parse_go_mod(content);
        assert_eq!(
            parsed.package_name.as_deref(),
            Some("github.com/owner/myrepo")
        );
        assert_eq!(parsed.package_version.as_deref(), Some("1.22.0"));
        assert_eq!(parsed.dependencies.len(), 2);

        let gin = parsed
            .find_dependency("github.com/gin-gonic/gin")
            .expect("gin found");
        assert_eq!(gin.version_req.as_deref(), Some("v1.9.1"));
        assert_eq!(gin.dependency_type, DependencyType::Direct);

        let testify = parsed
            .find_dependency("github.com/stretchr/testify")
            .expect("testify found");
        assert_eq!(testify.dependency_type, DependencyType::Dev);
    }

    #[test]
    fn test_cdm_section_generation() {
        let content = "tokio = \"1.0\"";
        let bytes = content.as_bytes().to_vec();
        let blob_sha = compute_git_blob_sha(&bytes);
        let blob = GitBlob::new(blob_sha, bytes, "raw");
        let config = SourceExtractionConfig::default();

        let extracted = ExtractedDependencyFile::from_blob("Cargo.toml", &blob, &config)
            .expect("blob extraction ok");

        assert_eq!(extracted.ecosystem, PackageEcosystem::Cargo);
        assert_eq!(extracted.role, ManifestRole::Manifest);
        assert!(extracted.parsed.is_some());

        let section = extracted.to_cdm_section("sec-1", "https://github.com/owner/repo/Cargo.toml");
        assert_eq!(section.kind, SectionKind::Manifest);
        assert_eq!(
            section.attributes.get("ecosystem").map(String::as_str),
            Some("cargo")
        );
        assert_eq!(
            section.attributes.get("manifest_role").map(String::as_str),
            Some("manifest")
        );
    }
}
