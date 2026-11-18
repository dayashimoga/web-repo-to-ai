//! # Syntax & Structural Token Breakdown
//!
//! Categorizes token consumption across code syntax, natural language prose,
//! markup structures, whitespace, and LLM special tokens.

use crate::model::TargetModel;
use crate::tokenizers::O200kTokenizer;
use serde::{Deserialize, Serialize};

/// Granular allocation of token consumption across syntax and content types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenSyntaxBreakdown {
    /// Target model analyzed.
    pub target_model: TargetModel,
    /// Total tokens evaluated.
    pub total_tokens: usize,
    /// Tokens consumed by programming keywords, identifiers, brackets, and operators.
    pub code_tokens: usize,
    /// Percentage of total tokens spent on code syntax.
    pub code_tokens_pct: f64,
    /// Tokens consumed by natural language prose words.
    pub prose_tokens: usize,
    /// Percentage of total tokens spent on prose.
    pub prose_tokens_pct: f64,
    /// Tokens consumed by Markdown structural syntax (#, *, `, >, |) or HTML/XML tags.
    pub markup_tokens: usize,
    /// Percentage of total tokens spent on markup.
    pub markup_tokens_pct: f64,
    /// Tokens consumed exclusively by whitespace (indentation spaces, blank lines).
    pub whitespace_tokens: usize,
    /// Percentage of total tokens spent on whitespace.
    pub whitespace_tokens_pct: f64,
    /// Tokens consumed by special or control tags.
    pub special_tokens: usize,
    /// Percentage of total tokens spent on special tokens.
    pub special_tokens_pct: f64,
}

impl TokenSyntaxBreakdown {
    /// Analyzes `text` for `target_model` and generates a categorized token breakdown.
    #[must_use]
    pub fn analyze(text: &str, target_model: TargetModel) -> Self {
        if text.is_empty() {
            return Self {
                target_model,
                total_tokens: 0,
                code_tokens: 0,
                code_tokens_pct: 0.0,
                prose_tokens: 0,
                prose_tokens_pct: 0.0,
                markup_tokens: 0,
                markup_tokens_pct: 0.0,
                whitespace_tokens: 0,
                whitespace_tokens_pct: 0.0,
                special_tokens: 0,
                special_tokens_pct: 0.0,
            };
        }

        let mut code_tokens = 0;
        let mut prose_tokens = 0;
        let mut markup_tokens = 0;
        let mut whitespace_tokens = 0;
        let mut special_tokens = 0;

        for line in text.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                // Blank line token
                whitespace_tokens += 1;
                continue;
            }

            // Indentation
            let leading_spaces = line.len() - line.trim_start().len();
            if leading_spaces > 0 {
                whitespace_tokens += leading_spaces.div_ceil(4);
            }

            // Check if Markdown heading or table row
            if trimmed.starts_with('#') || trimmed.starts_with('|') || trimmed.starts_with("```") {
                markup_tokens += O200kTokenizer::count(trimmed);
                continue;
            }

            // Tokenize segments
            for word in trimmed.split_whitespace() {
                if word.starts_with("<|") && word.ends_with("|>") {
                    special_tokens += 1;
                } else if word.starts_with('<') && word.ends_with('>') {
                    markup_tokens += 1;
                } else if word.contains(';')
                    || word.contains('{')
                    || word.contains('(')
                    || word.contains("::")
                    || word.contains("->")
                    || word.contains("=>")
                {
                    code_tokens += O200kTokenizer::count(word);
                } else if word
                    .chars()
                    .all(|c| c.is_alphabetic() || c == '.' || c == ',')
                {
                    prose_tokens += O200kTokenizer::count(word);
                } else {
                    code_tokens += O200kTokenizer::count(word);
                }
            }
        }

        let total = code_tokens + prose_tokens + markup_tokens + whitespace_tokens + special_tokens;
        let total_f = total.max(1) as f64;

        let pct = |val: usize| ((val as f64 / total_f * 100.0) * 10.0).round() / 10.0;

        Self {
            target_model,
            total_tokens: total,
            code_tokens,
            code_tokens_pct: pct(code_tokens),
            prose_tokens,
            prose_tokens_pct: pct(prose_tokens),
            markup_tokens,
            markup_tokens_pct: pct(markup_tokens),
            whitespace_tokens,
            whitespace_tokens_pct: pct(whitespace_tokens),
            special_tokens,
            special_tokens_pct: pct(special_tokens),
        }
    }

    /// Renders a human-readable GitHub Flavored Markdown summary table.
    #[must_use]
    pub fn to_markdown_summary(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!(
            "### Token Syntax Breakdown (`{}`)\n\n",
            self.target_model.as_str()
        ));
        md.push_str("| Category | Tokens | Share (%) |\n");
        md.push_str("| :--- | :---: | :---: |\n");
        md.push_str(&format!(
            "| **Code Syntax** | `{}` | `{}%` |\n",
            self.code_tokens, self.code_tokens_pct
        ));
        md.push_str(&format!(
            "| **Natural Prose** | `{}` | `{}%` |\n",
            self.prose_tokens, self.prose_tokens_pct
        ));
        md.push_str(&format!(
            "| **Markdown / Markup** | `{}` | `{}%` |\n",
            self.markup_tokens, self.markup_tokens_pct
        ));
        md.push_str(&format!(
            "| **Whitespace & Indentation** | `{}` | `{}%` |\n",
            self.whitespace_tokens, self.whitespace_tokens_pct
        ));
        if self.special_tokens > 0 {
            md.push_str(&format!(
                "| **Special / Control** | `{}` | `{}%` |\n",
                self.special_tokens, self.special_tokens_pct
            ));
        }
        md.push_str(&format!(
            "| **Total Tokens** | **`{}`** | `100.0%` |\n",
            self.total_tokens
        ));
        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_breakdown_code_and_prose() {
        let sample =
            "fn calculate_total(a: u32, b: u32) -> u32 {\n    // Return the sum\n    a + b\n}";
        let breakdown = TokenSyntaxBreakdown::analyze(sample, TargetModel::Gpt4o);

        assert!(breakdown.total_tokens > 0);
        assert!(breakdown.code_tokens > 0);
        assert!(breakdown.whitespace_tokens > 0);
        assert_eq!(breakdown.special_tokens, 0);

        let md = breakdown.to_markdown_summary();
        assert!(md.contains("Token Syntax Breakdown"));
        assert!(md.contains("Code Syntax"));
    }
}
