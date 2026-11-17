//! # Content Normalizer (`web-repo-to-ai-normalizer`)
//!
//! Provides deterministic document normalization:
//! - UTF-8 encoding canonicalization and replacement character handling
//! - Whitespace and line-ending normalization without semantic loss
//! - Markdown table and code block format standardization

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
    fn test_normalizer_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-normalizer");
    }
}
