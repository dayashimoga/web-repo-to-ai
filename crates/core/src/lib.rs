//! # Core Foundation (`web-repo-to-ai-core`)
//!
//! Provides the foundational primitives for `web-repo-to-ai`:
//! - Unified error taxonomy and error classification
//! - Domain identifiers (Job, Document, Tenant)
//! - Core traits and configuration contracts

#![warn(missing_docs)]

/// Returns the semantic crate version.
#[must_use]
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Returns the unique crate identifier name.
#[must_use]
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-core");
        assert!(!crate_version().is_empty());
    }
}
