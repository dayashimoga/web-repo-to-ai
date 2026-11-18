//! # Content Deduplication Engine (`web-repo-to-ai-deduplication`)
//!
//! Provides deterministic redundancy elimination:
//! - Exact cryptographic content hashing (BLAKE3) across sections and files
//! - Policy-driven canonical instance selection (`FirstSeen`, `ShortestPath`, `StructuralHierarchy`, `PreferredPrefix`)
//! - Deduplication strategies (`Remove`, `ReplaceWithPointer`, `MarkOnly`)
//! - Semantic near-duplicate detection via word shingling, MinHash signatures, and 64-bit SimHash fingerprints
//! - Near-duplicate consolidation with annotated pointers, exclusions, and directed relationship graphs
//! - Complete exclusion and relationship provenance accounting (`ExclusionReason::Duplicate`, `RelationshipKind::DuplicateOf`)
//! - Quantitative deduplication metrics, token reduction accounting, and Markdown/JSON reporting

#![warn(missing_docs)]

pub mod cluster;
pub mod config;
pub mod document;
pub mod error;
pub mod exact;
pub mod minhash;
pub mod report;
pub mod semantic;
pub mod semantic_cluster;
pub mod semantic_config;
pub mod semantic_report;
pub mod shingle;
pub mod simhash;

pub use cluster::DuplicateCluster;
pub use config::{
    CanonicalSelectionPolicy, DeduplicationStrategy, ExactDeduplicationConfig, HashScope,
};
pub use document::DocumentDeduplicator;
pub use error::{DeduplicationError, Result};
pub use exact::{compute_dedup_hash, ExactDeduplicator};
pub use minhash::{MinHashSignature, DEFAULT_NUM_HASHES};
pub use report::ExactDeduplicationReport;
pub use semantic::SemanticDeduplicator;
pub use semantic_cluster::{NearDuplicateCluster, NearDuplicateMember};
pub use semantic_config::{
    SemanticAlgorithm, SemanticCanonicalPolicy, SemanticDeduplicationConfig,
    SemanticDeduplicationStrategy,
};
pub use semantic_report::SemanticDeduplicationReport;
pub use shingle::Shingler;
pub use simhash::SimHash64;

use web_repo_to_ai_cdm::CdmDocument;

/// Returns the unique crate identifier name.
#[must_use]
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

/// Convenience function to deduplicate all exact identical sections in a [`CdmDocument`].
#[must_use]
pub fn deduplicate_document(
    doc: &CdmDocument,
    config: &ExactDeduplicationConfig,
) -> (CdmDocument, ExactDeduplicationReport) {
    DocumentDeduplicator::new(config.clone()).deduplicate_document(doc)
}

/// Convenience function to consolidate all semantic near-duplicate sections in a [`CdmDocument`].
#[must_use]
pub fn deduplicate_semantic(
    doc: &CdmDocument,
    config: &SemanticDeduplicationConfig,
) -> (CdmDocument, SemanticDeduplicationReport) {
    SemanticDeduplicator::new(config.clone()).deduplicate_document(doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-deduplication");
    }
}
