//! # Storage Subsystem (`web-repo-to-ai-storage`)
//!
//! Provides relational persistence schema and durable artifact storage abstractions:
//! - PostgreSQL relational schema DDL and entity models for `jobs`, `job_artifacts`,
//!   `github_installations`, and `content_cache_index`.
//! - Content-addressed blob storage with SHA-256 and BLAKE3 verification.
//! - Pluggable backend trait (`StorageBackend`).
//! - Ephemeral retention policies and automated cleanup.

#![warn(missing_docs)]

pub mod schema;
pub use schema::*;

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
