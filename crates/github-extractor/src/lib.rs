//! # GitHub Extractor (`web-repo-to-ai-github-extractor`)
//!
//! Provides deep repository ingestion for public and private GitHub sources:
//! - Git Trees API traversal with recursive subtree recovery
//! - Completeness manifest accounting (Discovered = Extracted + Excluded + Failed)
//! - Ephemeral GitHub App installation token lifecycle management

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
    fn test_github_extractor_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-github-extractor");
    }
}
