//! # Job Engine (`web-repo-to-ai-job-engine`)
//!
//! Manages conversion job lifecycles and concurrency:
//! - State machine transitions (`queued` -> `leased` -> `running` -> `completed` / `failed`)
//! - Distributed lease acquisition with `SKIP LOCKED` semantics
//! - Worker heartbeats and stale lease zombie reclamation

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
    fn test_job_engine_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-job-engine");
    }
}
