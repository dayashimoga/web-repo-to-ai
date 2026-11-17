//! # Storage Subsystem (`web-repo-to-ai-storage`)
//!
//! Provides durable artifact storage abstractions:
//! - Content-addressed blob storage with SHA-256 verification
//! - Pluggable backend trait (`StorageBackend`)
//! - Ephemeral retention policies and automated cleanup

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
    fn test_storage_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-storage");
    }
}
