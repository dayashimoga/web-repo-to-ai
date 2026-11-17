//! # Content Deduplication Engine (`web-repo-to-ai-deduplication`)
//!
//! Provides deterministic redundancy elimination:
//! - Exact content hashing (SHA-256 / BLAKE3)
//! - Near-duplicate section and document detection via MinHash LSH
//! - Deduplication metrics and token reduction accounting

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
    fn test_deduplication_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-deduplication");
    }
}
