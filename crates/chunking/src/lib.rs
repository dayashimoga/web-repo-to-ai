//! # Chunking Engine (`web-repo-to-ai-chunking`)
//!
//! Splits documents along semantic and syntactic boundaries:
//! - Heading-aware and section-aware markdown splitting
//! - Function and class AST boundary chunking
//! - Configurable token overlap and citation metadata injection

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
    fn test_chunking_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-chunking");
    }
}
