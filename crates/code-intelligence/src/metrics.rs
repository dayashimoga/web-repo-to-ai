//! # Code Metrics & Structural Statistics
//!
//! Evaluates quantitative complexity, comment-to-code ratio, line distributions,
//! and minification indicators across source files.

use serde::{Deserialize, Serialize};

use crate::language::Language;
use crate::symbols::{SymbolDeclaration, Visibility};

/// Structural metrics computed across source code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeMetrics {
    /// Total line count including empty lines.
    pub total_lines: usize,
    /// Number of lines containing non-comment code.
    pub code_lines: usize,
    /// Number of comment lines (both single and block comments).
    pub comment_lines: usize,
    /// Number of whitespace-only or empty lines.
    pub blank_lines: usize,
    /// Total number of extracted symbols (functions, types, constants, macros).
    pub symbol_count: usize,
    /// Number of publicly exported declarations.
    pub export_count: usize,
    /// Ratio of comment lines to total non-blank lines in $[0.0, 1.0]$.
    pub comment_ratio: f64,
    /// Average character length across all lines.
    pub average_line_length: f64,
    /// Maximum character length of any single line.
    pub max_line_length: usize,
    /// True if content exhibits minification markers (ultra-long lines, stripped whitespace).
    pub is_minified: bool,
    /// True if any public symbol has associated doc comments.
    pub has_doc_comments: bool,
}

/// Computes [`CodeMetrics`] from source code content and extracted symbols.
pub struct CodeMetricsCalculator;

impl CodeMetricsCalculator {
    /// Evaluates metrics across raw code content.
    #[must_use]
    pub fn compute(
        language: &Language,
        content: &str,
        symbols: &[SymbolDeclaration],
    ) -> CodeMetrics {
        let total_lines = content.lines().count();
        let mut code_lines = 0;
        let mut comment_lines = 0;
        let mut blank_lines = 0;
        let mut max_line_length = 0;
        let mut total_chars = 0;

        let line_prefix = language.line_comment_prefix();
        let block_delims = language.block_comment_delimiters();
        let mut inside_block_comment = false;

        for line in content.lines() {
            let line_len = line.chars().count();
            if line_len > max_line_length {
                max_line_length = line_len;
            }
            total_chars += line_len;

            let trimmed = line.trim();
            if trimmed.is_empty() {
                blank_lines += 1;
                continue;
            }

            if inside_block_comment {
                comment_lines += 1;
                if let Some((_, end)) = block_delims {
                    if trimmed.contains(end) {
                        inside_block_comment = false;
                    }
                }
                continue;
            }

            if let Some((start, end)) = block_delims {
                if trimmed.starts_with(start) {
                    comment_lines += 1;
                    if !trimmed.contains(end) || trimmed == start {
                        inside_block_comment = true;
                    }
                    continue;
                }
            }

            if let Some(prefix) = line_prefix {
                if trimmed.starts_with(prefix) {
                    comment_lines += 1;
                    continue;
                }
            }

            code_lines += 1;
        }

        let non_blank = code_lines + comment_lines;
        let comment_ratio = if non_blank > 0 {
            (comment_lines as f64 / non_blank as f64 * 10000.0).round() / 10000.0
        } else {
            0.0
        };

        let average_line_length = if total_lines > 0 {
            (total_chars as f64 / total_lines as f64 * 100.0).round() / 100.0
        } else {
            0.0
        };

        // Minification heuristics:
        // 1. Any line > 400 characters in code
        // 2. Average line length > 180 characters and total bytes > 400
        // 3. Total lines <= 3 with bytes > 500
        let is_minified = max_line_length > 400
            || (average_line_length > 180.0 && total_chars > 400)
            || (total_lines <= 3 && total_chars > 500);

        let symbol_count = symbols.len();
        let export_count = symbols
            .iter()
            .filter(|s| s.visibility == Visibility::Public)
            .count();
        let has_doc_comments = symbols.iter().any(|s| s.doc_comment.is_some());

        CodeMetrics {
            total_lines,
            code_lines,
            comment_lines,
            blank_lines,
            symbol_count,
            export_count,
            comment_ratio,
            average_line_length,
            max_line_length,
            is_minified,
            has_doc_comments,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_metrics_computation() {
        let code = "\
// Module header
// Description

pub fn calculate() -> u32 {
    let a = 10;
    let b = 20;

    a + b
}
";
        let symbols = vec![SymbolDeclaration {
            name: "calculate".to_string(),
            kind: crate::symbols::SymbolKind::Function,
            visibility: Visibility::Public,
            line_number: 4,
            signature: "pub fn calculate() -> u32 {".to_string(),
            doc_comment: None,
        }];

        let metrics = CodeMetricsCalculator::compute(&Language::Rust, code, &symbols);
        assert_eq!(metrics.total_lines, 9);
        assert_eq!(metrics.comment_lines, 2);
        assert_eq!(metrics.code_lines, 5);
        assert_eq!(metrics.blank_lines, 2);
        assert_eq!(metrics.symbol_count, 1);
        assert_eq!(metrics.export_count, 1);
        assert!(!metrics.is_minified);
        assert!(metrics.comment_ratio > 0.20);
    }

    #[test]
    fn test_minification_detection() {
        let minified = "function a(b,c){var d=1;for(var e=0;e<b.length;e++){d+=b[e]*c}return d;};function f(g){return g.map(function(x){return x*2;})};var z=a([1,2,3],4)+f([5,6]);console.log(z);window.__bundle={init:a,exec:f};";
        let long_minified = minified.repeat(5);

        let metrics = CodeMetricsCalculator::compute(&Language::JavaScript, &long_minified, &[]);
        assert!(metrics.is_minified);
    }
}
