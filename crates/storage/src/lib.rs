//! # Storage Subsystem (`web-repo-to-ai-storage`)
//!
//! Provides relational persistence schema, database migrations, state machines, and durable artifact storage abstractions:
//! - PostgreSQL relational schema DDL and entity models for `jobs`, `job_artifacts`,
//!   `github_installations`, `content_cache_index`, and `job_state_transitions`.
//! - Programmatic database migrations runner and embedded SQLx migrations.
//! - Runtime schema introspection, column audit, and catalog validation.
//! - Deterministic job state machine, transition request builders, and audit history.
//! - Transactional repository with row-level locking (`FOR UPDATE`).
//! - Content-addressed blob storage with SHA-256 and BLAKE3 verification.
//! - Pluggable backend trait (`StorageBackend`).
//! - Ephemeral retention policies and automated cleanup.

#![warn(missing_docs)]

pub mod backend;
pub mod batch;
pub mod checksum;
pub mod database;
pub mod git_cache;
pub mod http_cache;
pub mod leasing;
pub mod local_fs;
pub mod migrations;
pub mod repository;
pub mod retention;
pub mod schema;
pub mod state_machine;
pub mod token_storage;
pub mod validation;

pub use backend::*;
pub use batch::*;
pub use checksum::*;
pub use database::*;
pub use git_cache::*;
pub use http_cache::*;
pub use leasing::*;
pub use local_fs::*;
pub use migrations::*;
pub use repository::*;
pub use retention::*;
pub use schema::*;
pub use state_machine::*;
pub use token_storage::*;
pub use validation::*;

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
