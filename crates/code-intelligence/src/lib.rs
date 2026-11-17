//! # Code Intelligence (`web-repo-to-ai-code-intelligence`)
//!
//! Provides AST-level structural inspection:
//! - Tree-sitter multi-language syntax tree traversal
//! - Symbol, signature, and dependency boundary extraction
//! - Source code language classification

#![warn(missing_docs)]

/// Returns the unique crate identifier name.
#[must_use]
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_intelligence_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-code-intelligence");
    }
}
