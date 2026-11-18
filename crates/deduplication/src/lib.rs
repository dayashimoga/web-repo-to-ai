//! # Content Deduplication Engine (`web-repo-to-ai-deduplication`)
//!
//! Provides deterministic redundancy elimination:
//! - Exact cryptographic content hashing (BLAKE3) across sections and files
//! - Policy-driven canonical instance selection (`FirstSeen`, `ShortestPath`, `StructuralHierarchy`, `PreferredPrefix`)
//! - Deduplication strategies (`Remove`, `ReplaceWithPointer`, `MarkOnly`)
//! - Complete exclusion and relationship provenance accounting (`ExclusionReason::Duplicate`, `RelationshipKind::DuplicateOf`)
//! - Quantitative deduplication metrics, token reduction accounting, and Markdown/JSON reporting

#![warn(missing_docs)]

pub mod cluster;
pub mod config;
pub mod document;
pub mod error;
pub mod exact;
pub mod report;

pub use cluster::DuplicateCluster;
pub use config::{
    CanonicalSelectionPolicy, DeduplicationStrategy, ExactDeduplicationConfig, HashScope,
};
pub use document::DocumentDeduplicator;
pub use error::{DeduplicationError, Result};
pub use exact::{compute_dedup_hash, ExactDeduplicator};
pub use report::ExactDeduplicationReport;

use web_repo_to_ai_cdm::CdmDocument;

/// Returns the unique crate identifier name.
#[must_use]
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

/// Convenience function to deduplicate all identical sections in a [`CdmDocument`].
#[must_use]
pub fn deduplicate_document(
    doc: &CdmDocument,
    config: &ExactDeduplicationConfig,
) -> (CdmDocument, ExactDeduplicationReport) {
    DocumentDeduplicator::new(config.clone()).deduplicate_document(doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-deduplication");
    }
}
