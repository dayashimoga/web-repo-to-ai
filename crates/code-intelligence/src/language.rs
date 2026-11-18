//! # Programming Language Detection & Classification
//!
//! Provides deterministic file extension, path convention, and content heuristic
//! language detection for multi-lingual source repositories.

use serde::{Deserialize, Serialize};

/// Canonical programming or markup language supported by code intelligence.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    /// Rust programming language.
    Rust,
    /// TypeScript language.
    TypeScript,
    /// JavaScript language.
    JavaScript,
    /// Python language.
    Python,
    /// Go language.
    Go,
    /// Java language.
    Java,
    /// C language.
    C,
    /// C++ language.
    Cpp,
    /// C# language.
    CSharp,
    /// Ruby language.
    Ruby,
    /// PHP language.
    Php,
    /// Swift language.
    Swift,
    /// Kotlin language.
    Kotlin,
    /// Scala language.
    Scala,
    /// Unix Shell script (bash, sh, zsh).
    Shell,
    /// Structured Query Language.
    Sql,
    /// Hypertext Markup Language.
    Html,
    /// Cascading Style Sheets / SCSS / SASS.
    Css,
    /// Markdown documentation.
    Markdown,
    /// YAML configuration.
    Yaml,
    /// TOML configuration.
    Toml,
    /// JSON data.
    Json,
    /// XML data.
    Xml,
    /// Protocol Buffers schema.
    Protobuf,
    /// GraphQL schema / query.
    GraphQL,
    /// Dockerfile container build specification.
    Dockerfile,
    /// Unrecognized or generic language format.
    Unknown(String),
}

impl Language {
    /// Returns the canonical snake_case string representation of the language.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::JavaScript => "javascript",
            Self::Python => "python",
            Self::Go => "go",
            Self::Java => "java",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::CSharp => "csharp",
            Self::Ruby => "ruby",
            Self::Php => "php",
            Self::Swift => "swift",
            Self::Kotlin => "kotlin",
            Self::Scala => "scala",
            Self::Shell => "shell",
            Self::Sql => "sql",
            Self::Html => "html",
            Self::Css => "css",
            Self::Markdown => "markdown",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Json => "json",
            Self::Xml => "xml",
            Self::Protobuf => "protobuf",
            Self::GraphQL => "graphql",
            Self::Dockerfile => "dockerfile",
            Self::Unknown(_) => "unknown",
        }
    }

    /// Resolves a language from a known file extension (case-insensitive, without dot).
    #[must_use]
    pub fn from_extension(ext: &str) -> Option<Self> {
        let clean = ext.trim().trim_start_matches('.').to_ascii_lowercase();
        match clean.as_str() {
            "rs" => Some(Self::Rust),
            "ts" | "mts" | "cts" | "tsx" => Some(Self::TypeScript),
            "js" | "mjs" | "cjs" | "jsx" => Some(Self::JavaScript),
            "py" | "pyw" | "pyi" => Some(Self::Python),
            "go" => Some(Self::Go),
            "java" => Some(Self::Java),
            "c" | "h" => Some(Self::C),
            "cpp" | "cxx" | "cc" | "hpp" | "hxx" | "hh" => Some(Self::Cpp),
            "cs" => Some(Self::CSharp),
            "rb" => Some(Self::Ruby),
            "php" | "phtml" => Some(Self::Php),
            "swift" => Some(Self::Swift),
            "kt" | "kts" => Some(Self::Kotlin),
            "scala" | "sc" => Some(Self::Scala),
            "sh" | "bash" | "zsh" | "env" => Some(Self::Shell),
            "sql" => Some(Self::Sql),
            "html" | "htm" => Some(Self::Html),
            "css" | "scss" | "sass" | "less" => Some(Self::Css),
            "md" | "markdown" | "mdown" | "mkdn" => Some(Self::Markdown),
            "yaml" | "yml" => Some(Self::Yaml),
            "toml" => Some(Self::Toml),
            "json" | "jsonl" | "json5" => Some(Self::Json),
            "xml" | "xsd" | "xsl" | "svg" => Some(Self::Xml),
            "proto" => Some(Self::Protobuf),
            "graphql" | "gql" => Some(Self::GraphQL),
            _ => None,
        }
    }

    /// Resolves a language from a file path or file name.
    #[must_use]
    pub fn from_path(path: &str) -> Option<Self> {
        let normalized = path.replace('\\', "/");
        let file_name = normalized.rsplit('/').next().unwrap_or(&normalized);
        let lower = file_name.to_ascii_lowercase();

        // Exact filename checks
        if lower == "dockerfile" || lower.starts_with("dockerfile.") {
            return Some(Self::Dockerfile);
        }
        if lower == "cargo.toml" {
            return Some(Self::Toml);
        }
        if lower == "package.json" || lower == "tsconfig.json" {
            return Some(Self::Json);
        }
        if lower == "makefile" || lower == "gnumakefile" {
            return Some(Self::Shell);
        }

        // Extension check
        if let Some(dot_idx) = file_name.rfind('.') {
            let ext = &file_name[dot_idx + 1..];
            if let Some(lang) = Self::from_extension(ext) {
                return Some(lang);
            }
        }

        None
    }

    /// Resolves a language from a language string tag or identifier.
    #[must_use]
    pub fn from_identifier(ident: &str) -> Self {
        let clean = ident.trim().to_ascii_lowercase();
        match clean.as_str() {
            "rust" | "rs" => Self::Rust,
            "typescript" | "ts" | "tsx" => Self::TypeScript,
            "javascript" | "js" | "jsx" | "node" => Self::JavaScript,
            "python" | "py" => Self::Python,
            "go" | "golang" => Self::Go,
            "java" => Self::Java,
            "c" => Self::C,
            "cpp" | "c++" | "cxx" => Self::Cpp,
            "csharp" | "c#" | "cs" => Self::CSharp,
            "ruby" | "rb" => Self::Ruby,
            "php" => Self::Php,
            "swift" => Self::Swift,
            "kotlin" | "kt" => Self::Kotlin,
            "scala" => Self::Scala,
            "shell" | "bash" | "sh" | "zsh" => Self::Shell,
            "sql" => Self::Sql,
            "html" => Self::Html,
            "css" | "scss" => Self::Css,
            "markdown" | "md" => Self::Markdown,
            "yaml" | "yml" => Self::Yaml,
            "toml" => Self::Toml,
            "json" => Self::Json,
            "xml" => Self::Xml,
            "protobuf" | "proto" => Self::Protobuf,
            "graphql" | "gql" => Self::GraphQL,
            "dockerfile" | "docker" => Self::Dockerfile,
            other if !other.is_empty() => Self::Unknown(other.to_string()),
            _ => Self::Unknown("unknown".to_string()),
        }
    }

    /// Detects the language using path, shebang, and content heuristics.
    #[must_use]
    pub fn detect(path: Option<&str>, content: &str) -> Self {
        if let Some(p) = path {
            if let Some(lang) = Self::from_path(p) {
                return lang;
            }
        }

        // Shebang inspection
        let trimmed = content.trim_start();
        if trimmed.starts_with("#!") {
            if let Some(first_line) = trimmed.lines().next() {
                let lower = first_line.to_ascii_lowercase();
                if lower.contains("python") {
                    return Self::Python;
                }
                if lower.contains("bash") || lower.contains("sh") || lower.contains("zsh") {
                    return Self::Shell;
                }
                if lower.contains("node") {
                    return Self::JavaScript;
                }
                if lower.contains("ruby") {
                    return Self::Ruby;
                }
                if lower.contains("php") {
                    return Self::Php;
                }
            }
        }

        // Fast content keywords inspection
        if trimmed.starts_with("<?php") {
            return Self::Php;
        }
        if trimmed.starts_with("<!DOCTYPE html") || trimmed.starts_with("<html") {
            return Self::Html;
        }
        if trimmed.starts_with("<?xml") {
            return Self::Xml;
        }
        if trimmed.starts_with('{')
            && trimmed.ends_with('}')
            && serde_json::from_str::<serde_json::Value>(content).is_ok()
        {
            return Self::Json;
        }

        // Distinctive syntax heuristics
        if content.contains("fn main(")
            || content.contains("pub fn ")
            || content.contains("impl ") && content.contains("for ")
        {
            return Self::Rust;
        }
        if content.contains("package ")
            && (content.contains("func ") || content.contains("import ("))
        {
            return Self::Go;
        }
        if content.contains("def ") && content.contains("import ") && !content.contains('{') {
            return Self::Python;
        }
        if (content.contains("interface ") || content.contains("type "))
            && content.contains("export ")
        {
            return Self::TypeScript;
        }
        if content.contains("public class ") || content.contains("public static void main") {
            return Self::Java;
        }

        Self::Unknown("unknown".to_string())
    }

    /// Returns the line-comment prefix for this language, if applicable.
    #[must_use]
    pub const fn line_comment_prefix(&self) -> Option<&'static str> {
        match self {
            Self::Rust
            | Self::TypeScript
            | Self::JavaScript
            | Self::Go
            | Self::Java
            | Self::C
            | Self::Cpp
            | Self::CSharp
            | Self::Swift
            | Self::Kotlin
            | Self::Scala
            | Self::Protobuf => Some("//"),
            Self::Python
            | Self::Ruby
            | Self::Shell
            | Self::Yaml
            | Self::Toml
            | Self::Dockerfile => Some("#"),
            Self::Sql => Some("--"),
            Self::Php => Some("//"),
            _ => None,
        }
    }

    /// Returns the block comment delimiters `(start, end)` for this language, if applicable.
    #[must_use]
    pub const fn block_comment_delimiters(&self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::Rust
            | Self::TypeScript
            | Self::JavaScript
            | Self::Go
            | Self::Java
            | Self::C
            | Self::Cpp
            | Self::CSharp
            | Self::Swift
            | Self::Kotlin
            | Self::Scala
            | Self::Css
            | Self::Sql
            | Self::Protobuf => Some(("/*", "*/")),
            Self::Html | Self::Xml | Self::Markdown => Some(("<!--", "-->")),
            _ => None,
        }
    }

    /// Returns true if this is an executable programming language.
    #[must_use]
    pub const fn is_programming_language(&self) -> bool {
        matches!(
            self,
            Self::Rust
                | Self::TypeScript
                | Self::JavaScript
                | Self::Python
                | Self::Go
                | Self::Java
                | Self::C
                | Self::Cpp
                | Self::CSharp
                | Self::Ruby
                | Self::Php
                | Self::Swift
                | Self::Kotlin
                | Self::Scala
                | Self::Shell
        )
    }

    /// Returns true if this is a configuration, data, or schema language.
    #[must_use]
    pub const fn is_config_or_data(&self) -> bool {
        matches!(
            self,
            Self::Yaml
                | Self::Toml
                | Self::Json
                | Self::Xml
                | Self::Protobuf
                | Self::GraphQL
                | Self::Dockerfile
                | Self::Sql
        )
    }

    /// Returns true if this is a document or markup language.
    #[must_use]
    pub const fn is_markup_or_doc(&self) -> bool {
        matches!(self, Self::Markdown | Self::Html | Self::Css)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_extension() {
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));
        assert_eq!(Language::from_extension(".ts"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("tsx"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("py"), Some(Language::Python));
        assert_eq!(Language::from_extension("go"), Some(Language::Go));
        assert_eq!(Language::from_extension("toml"), Some(Language::Toml));
        assert_eq!(Language::from_extension("unknown_xyz"), None);
    }

    #[test]
    fn test_language_from_path() {
        assert_eq!(
            Language::from_path("src/core/crypto.rs"),
            Some(Language::Rust)
        );
        assert_eq!(Language::from_path("Cargo.toml"), Some(Language::Toml));
        assert_eq!(
            Language::from_path("deploy/Dockerfile"),
            Some(Language::Dockerfile)
        );
        assert_eq!(
            Language::from_path("apps/web/package.json"),
            Some(Language::Json)
        );
    }

    #[test]
    fn test_language_from_identifier() {
        assert_eq!(Language::from_identifier("rust"), Language::Rust);
        assert_eq!(Language::from_identifier("ts"), Language::TypeScript);
        assert_eq!(Language::from_identifier("py"), Language::Python);
        assert_eq!(Language::from_identifier("golang"), Language::Go);
    }

    #[test]
    fn test_language_detect_shebang_and_keywords() {
        let py_script = "#!/usr/bin/env python3\nimport sys\nprint('hello')";
        assert_eq!(Language::detect(None, py_script), Language::Python);

        let sh_script = "#!/bin/bash\necho 'running test'";
        assert_eq!(Language::detect(None, sh_script), Language::Shell);

        let rust_code = "pub fn execute_job(id: u64) -> Result<(), Error> {\n    Ok(())\n}";
        assert_eq!(Language::detect(None, rust_code), Language::Rust);
    }
}
