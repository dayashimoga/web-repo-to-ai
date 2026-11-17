//! # Canonical URL Resolution and Deduplication Subsystem
//!
//! Provides zero-trust canonical URL resolution and content deduplication:
//! - Canonical URL candidate evaluation under SSRF and security policies
//! - 64-bit SimHash fingerprinting and Hamming similarity computation
//! - In-memory duplicate registry tracking exact URL, exact content, and near-duplicates

pub mod canonical;
pub mod model;
pub mod registry;
pub mod simhash;

pub use canonical::CanonicalUrlResolver;
pub use model::{
    CanonicalResolution, CanonicalSource, DedupOptions, DuplicateClassification, DuplicateRecord,
};
pub use registry::DuplicateRegistry;
pub use simhash::SimHasher;
