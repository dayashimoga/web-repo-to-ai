//! # Syntax and Programming Language Detection
//!
//! Provides heuristic detection for programming languages and configuration formats
//! based on shebang declarations, keyword patterns, and structural syntax signatures.

/// Normalizes raw language identifiers extracted from HTML attributes or class names.
///
/// Strips common prefixes like `language-`, `lang-`, `brush:`, `highlight-source-`,
/// and maps standard aliases to canonical lowercase identifiers.
#[must_use]
pub fn normalize_language_tag(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Strip common prefixes
    let lower = trimmed.to_ascii_lowercase();
    let stripped = lower
        .strip_prefix("language-")
        .or_else(|| lower.strip_prefix("lang-"))
        .or_else(|| lower.strip_prefix("brush:"))
        .or_else(|| lower.strip_prefix("highlight-source-"))
        .or_else(|| lower.strip_prefix("source-"))
        .unwrap_or(&lower)
        .trim();

    if stripped.is_empty() {
        return None;
    }

    // Map common aliases to canonical names
    let canonical = match stripped {
        "rs" | "rust" => "rust",
        "py" | "python" | "python3" => "python",
        "js" | "javascript" | "mjs" | "cjs" | "node" => "javascript",
        "ts" | "typescript" | "mts" | "cts" => "typescript",
        "sh" | "bash" | "zsh" | "shell" | "console" => "bash",
        "c" => "c",
        "cpp" | "c++" | "cxx" | "cc" | "hpp" | "h" => "cpp",
        "cs" | "csharp" | "c#" => "csharp",
        "go" | "golang" => "go",
        "java" => "java",
        "kt" | "kotlin" => "kotlin",
        "swift" => "swift",
        "rb" | "ruby" => "ruby",
        "php" => "php",
        "html" | "htm" | "xhtml" => "html",
        "css" | "scss" | "sass" | "less" => "css",
        "sql" | "pgsql" | "mysql" => "sql",
        "json" | "jsonc" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" | "svg" => "xml",
        "md" | "markdown" => "markdown",
        "docker" | "dockerfile" => "dockerfile",
        "proto" | "protobuf" => "protobuf",
        "graphql" | "gql" => "graphql",
        other => {
            // Keep alphanumeric plus hyphen/underscore identifiers
            if other
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                return Some(other.to_string());
            }
            return None;
        }
    };

    Some(canonical.to_string())
}

/// Heuristically detects the programming language or syntax format of a code block.
///
/// Analyzes shebang lines first, then structural signatures and keyword frequencies.
#[must_use]
pub fn detect_syntax(code: &str) -> Option<&'static str> {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return None;
    }

    // 1. Shebang line check
    if let Some(first_line) = trimmed.lines().next() {
        if let Some(rest) = first_line.strip_prefix("#!") {
            let lower_shebang = rest.to_ascii_lowercase();
            if lower_shebang.contains("python") {
                return Some("python");
            }
            if lower_shebang.contains("bash") || lower_shebang.contains("sh") {
                return Some("bash");
            }
            if lower_shebang.contains("node") {
                return Some("javascript");
            }
            if lower_shebang.contains("ruby") {
                return Some("ruby");
            }
            if lower_shebang.contains("perl") {
                return Some("perl");
            }
        }
    }

    // 2. Format checks: JSON, HTML, XML, YAML, TOML
    if ((trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']')))
        && (trimmed.contains("\":") || trimmed.contains("\": "))
        && !trimmed.contains("function ")
    {
        return Some("json");
    }

    if (trimmed.starts_with("<!DOCTYPE html") || trimmed.starts_with("<!doctype html"))
        || (trimmed.contains("<html") && trimmed.contains("</html>"))
        || (trimmed.contains("<div") && trimmed.contains("</div>"))
    {
        return Some("html");
    }

    if trimmed.starts_with("<?xml") {
        return Some("xml");
    }

    // TOML signature: [package] or [dependencies] or [table.sub]
    if (trimmed.starts_with("[package]") || trimmed.starts_with("[dependencies]"))
        || (trimmed
            .lines()
            .any(|l| l.starts_with('[') && l.ends_with(']'))
            && trimmed.contains(" = "))
    {
        return Some("toml");
    }

    // 3. Score programming languages by keyword patterns
    let mut rust_score = 0;
    let mut python_score = 0;
    let mut js_score = 0;
    let mut ts_score = 0;
    let mut go_score = 0;
    let mut cpp_score = 0;
    let mut java_score = 0;
    let mut sql_score = 0;
    let mut bash_score = 0;
    let mut css_score = 0;
    let mut docker_score = 0;

    for line in trimmed.lines() {
        let l = line.trim();

        // Dockerfile signatures
        if l.starts_with("FROM ") && (l.contains(':') || l.contains(" AS ") || l.contains(" as ")) {
            docker_score += 5;
        } else if l.starts_with("RUN ")
            || l.starts_with("WORKDIR ")
            || l.starts_with("ENTRYPOINT ")
            || l.starts_with("COPY ")
        {
            docker_score += 4;
        } else if l.starts_with("EXPOSE ") || l.starts_with("ENV ") || l.starts_with("CMD ") {
            docker_score += 3;
        }

        // Rust signatures
        if l.starts_with("pub fn ") || l.starts_with("fn ") || l.starts_with("impl ") {
            rust_score += 4;
        }
        if l.contains("let mut ") || l.contains("match ") || l.contains("println!(") {
            rust_score += 3;
        }
        if l.starts_with("use std::")
            || l.starts_with("#[derive(")
            || l.contains("Ok(")
            || l.contains("Err(")
        {
            rust_score += 3;
        }

        // Python signatures
        if l.starts_with("def ") || l.starts_with("class ") && l.ends_with(':') {
            python_score += 4;
        }
        if l.starts_with("from ") && l.contains(" import ") || l.starts_with("import ") {
            python_score += 2;
        }
        if l.contains("self.") || l.contains("__init__") || l.starts_with("elif ") {
            python_score += 3;
        }
        if l.starts_with("if __name__ == ") {
            python_score += 5;
        }

        // TypeScript & JavaScript signatures
        if l.starts_with("interface ")
            || l.starts_with("export interface ")
            || l.starts_with("type ") && l.contains('=')
        {
            ts_score += 4;
        }
        if l.contains(": string")
            || l.contains(": number")
            || l.contains(": boolean")
            || l.contains("as const")
        {
            ts_score += 3;
        }
        if l.starts_with("const ") || l.starts_with("let ") || l.starts_with("var ") {
            js_score += 2;
        }
        if l.contains("console.log(") || l.contains("=> {") || l.contains("require(") {
            js_score += 3;
        }
        if l.starts_with("export default ") || l.starts_with("import {") && l.contains("from '") {
            js_score += 3;
        }

        // Go signatures
        if l.starts_with("package ") || l.starts_with("func ") {
            go_score += 4;
        }
        if l.contains("fmt.Print") || l.contains("go func(") || l.contains("chan ") {
            go_score += 3;
        }
        if l.contains(":= ") && !l.contains("let ") {
            go_score += 2;
        }

        // C / C++ signatures
        if l.starts_with("#include <") || l.starts_with("#include \"") {
            cpp_score += 5;
        }
        if l.contains("std::") || l.contains("cout <<") || l.contains("nullptr") {
            cpp_score += 4;
        }

        // Java signatures
        if l.starts_with("public class ") || l.contains("public static void main") {
            java_score += 5;
        }
        if l.contains("System.out.println(")
            || l.starts_with("package com.")
            || l.starts_with("import java.")
        {
            java_score += 4;
        }

        // SQL signatures
        let upper = l.to_ascii_uppercase();
        if upper.starts_with("SELECT ") {
            sql_score += 4;
        }
        if upper.starts_with("FROM ") {
            sql_score += 3;
        }
        if upper.starts_with("WHERE ")
            || upper.starts_with("JOIN ")
            || upper.starts_with("LEFT JOIN ")
            || upper.starts_with("INNER JOIN ")
        {
            sql_score += 3;
        }
        if upper.starts_with("GROUP BY ")
            || upper.starts_with("ORDER BY ")
            || upper.starts_with("HAVING ")
        {
            sql_score += 3;
        }
        if upper.starts_with("INSERT INTO ")
            || upper.starts_with("CREATE TABLE ")
            || upper.starts_with("ALTER TABLE ")
            || upper.starts_with("DROP TABLE ")
        {
            sql_score += 5;
        }
        if upper.starts_with("UPDATE ") && upper.contains(" SET ") {
            sql_score += 4;
        }

        // Bash signatures
        if l.starts_with("curl ")
            || l.starts_with("chmod ")
            || l.starts_with("export ")
            || l.starts_with("sudo ")
        {
            bash_score += 3;
        }
        if l.starts_with("npm ")
            || l.starts_with("cargo ")
            || l.starts_with("pnpm ")
            || l.starts_with("git ")
        {
            bash_score += 3;
        }

        // CSS signatures
        if (l.contains('{') && l.ends_with('{')) && (l.starts_with('.') || l.starts_with('#')) {
            css_score += 2;
        }
        if l.contains("margin:")
            || l.contains("padding:")
            || l.contains("display: flex")
            || l.contains("color:")
        {
            css_score += 2;
        }
    }

    // Resolve TypeScript vs JavaScript
    if ts_score >= 3 {
        js_score = 0; // TypeScript takes priority if TS-specific tokens found
    }

    // Rank candidates
    let candidates = [
        ("rust", rust_score),
        ("python", python_score),
        ("typescript", ts_score),
        ("javascript", js_score),
        ("go", go_score),
        ("cpp", cpp_score),
        ("java", java_score),
        ("sql", sql_score),
        ("bash", bash_score),
        ("css", css_score),
        ("dockerfile", docker_score),
    ];

    let mut best_lang = None;
    let mut highest_score = 0;

    for (lang, score) in candidates {
        if score > highest_score && score >= 3 {
            highest_score = score;
            best_lang = Some(lang);
        }
    }

    best_lang
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_language_tags() {
        assert_eq!(
            normalize_language_tag("language-rust"),
            Some("rust".to_string())
        );
        assert_eq!(
            normalize_language_tag("lang-python"),
            Some("python".to_string())
        );
        assert_eq!(
            normalize_language_tag("brush: js"),
            Some("javascript".to_string())
        );
        assert_eq!(
            normalize_language_tag("highlight-source-ts"),
            Some("typescript".to_string())
        );
        assert_eq!(normalize_language_tag("rs"), Some("rust".to_string()));
        assert_eq!(normalize_language_tag("c++"), Some("cpp".to_string()));
        assert_eq!(normalize_language_tag("golang"), Some("go".to_string()));
        assert_eq!(normalize_language_tag(""), None);
    }

    #[test]
    fn test_detect_shebang() {
        let py = "#!/usr/bin/env python3\nprint('hello')";
        assert_eq!(detect_syntax(py), Some("python"));

        let sh = "#!/bin/bash\necho 'hello'";
        assert_eq!(detect_syntax(sh), Some("bash"));
    }

    #[test]
    fn test_detect_rust_syntax() {
        let rust_code = r#"
            pub fn process_data(input: &str) -> Result<String, Error> {
                let mut output = String::new();
                match input {
                    "a" => println!("matched"),
                    _ => {}
                }
                Ok(output)
            }
        "#;
        assert_eq!(detect_syntax(rust_code), Some("rust"));
    }

    #[test]
    fn test_detect_python_syntax() {
        let py_code = r#"
            import os
            from typing import List

            class Processor:
                def __init__(self, name: str):
                    self.name = name

                def run(self):
                    print(f"Running {self.name}")
        "#;
        assert_eq!(detect_syntax(py_code), Some("python"));
    }

    #[test]
    fn test_detect_sql_syntax() {
        let sql_code = "SELECT id, name, created_at FROM users WHERE status = 'active' ORDER BY created_at DESC;";
        assert_eq!(detect_syntax(sql_code), Some("sql"));
    }

    #[test]
    fn test_detect_json_syntax() {
        let json_code = "{\n  \"name\": \"web-repo-to-ai\",\n  \"version\": \"1.0.0\"\n}";
        assert_eq!(detect_syntax(json_code), Some("json"));
    }
}
