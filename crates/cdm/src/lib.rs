//! # Canonical Context Document Model (`web-repo-to-ai-cdm`)
//!
//! Defines the versioned intermediate representation (CDM v1.0)
//! shared across extractors, normalizers, optimizers, and exporters.

#![warn(missing_docs)]

/// Returns the supported CDM schema version.
#[must_use]
pub fn schema_version() -> &'static str {
    "1.0.0"
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
    fn test_cdm_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-cdm");
        assert_eq!(schema_version(), "1.0.0");
    }
}
