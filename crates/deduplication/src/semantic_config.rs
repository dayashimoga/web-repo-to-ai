//! # Semantic Deduplication Configuration
//!
//! Fluent configuration and domain profiles governing near-duplicate MinHash / SimHash
//! similarity thresholds, canonical election, and replacement strategies.

use serde::{Deserialize, Serialize};

/// Algorithmic strategy for calculating text similarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticAlgorithm {
    /// Sliding window word shingling with MinHash signature estimation.
    MinHash {
        /// Number of consecutive words per shingle (default: 3).
        shingle_size: usize,
        /// Size of permutation signature bank (default: 64).
        num_hashes: usize,
    },
    /// 64-bit SimHash with maximum Hamming distance ceiling.
    SimHash {
        /// Maximum bit difference allowing near-duplicate classification (default: 4).
        max_hamming_distance: u32,
    },
}

impl Default for SemanticAlgorithm {
    fn default() -> Self {
        Self::MinHash {
            shingle_size: 2,
            num_hashes: 64,
        }
    }
}

/// Action taken when a near-duplicate section is identified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticDeduplicationStrategy {
    /// Replaces the near-duplicate section body with an annotated reference pointer
    /// stating the canonical target and estimated similarity percentage.
    ReplaceWithAnnotatedPointer,
    /// Completely removes the near-duplicate section from the document outline,
    /// recording an auditable [`web_repo_to_ai_cdm::ExclusionRecord`].
    Remove,
    /// Preserves the section body unmodified, but registers `RelationshipKind::DuplicateOf`
    /// edges and records cluster telemetry.
    MarkOnly,
}

/// Rule governing which section in a near-duplicate cluster is elected canonical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticCanonicalPolicy {
    /// Elects the most comprehensive section with the highest character and word count.
    InformationRichness,
    /// Prioritizes structural authority: `CodeFile` > `CodeSnippet` > `Prose` > `Table` > `Admonition`.
    StructuralHierarchy,
    /// First encountered instance in document outline order.
    FirstSeen,
    /// Section with the shortest original path or title.
    ShortestPath,
}

/// Master configuration for semantic near-duplicate deduplication.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticDeduplicationConfig {
    /// Algorithmic similarity estimator.
    pub algorithm: SemanticAlgorithm,
    /// Minimum similarity score in $[0.0, 1.0]$ required to classify two sections as near-duplicates (default: 0.80).
    pub similarity_threshold: f64,
    /// Strategy applied when near-duplicates are detected.
    pub strategy: SemanticDeduplicationStrategy,
    /// Rule for selecting the canonical instance in each near-duplicate cluster.
    pub canonical_policy: SemanticCanonicalPolicy,
    /// Minimum word count required for a section to be eligible for semantic evaluation.
    /// Short phrases or trivial snippets (< 10 words) are skipped.
    pub min_words_threshold: usize,
    /// When true, records an auditable [`web_repo_to_ai_cdm::ExclusionRecord`] for every removed near-duplicate.
    pub log_exclusions: bool,
}

impl Default for SemanticDeduplicationConfig {
    fn default() -> Self {
        Self {
            algorithm: SemanticAlgorithm::default(),
            similarity_threshold: 0.75,
            strategy: SemanticDeduplicationStrategy::ReplaceWithAnnotatedPointer,
            canonical_policy: SemanticCanonicalPolicy::InformationRichness,
            min_words_threshold: 8,
            log_exclusions: true,
        }
    }
}

impl SemanticDeduplicationConfig {
    /// Preset: Balanced near-duplicate consolidation using MinHash (0.80 threshold).
    #[must_use]
    pub fn balanced() -> Self {
        Self::default()
    }

    /// Preset: Aggressive near-duplicate removal with 0.70 similarity threshold.
    #[must_use]
    pub fn aggressive() -> Self {
        Self {
            algorithm: SemanticAlgorithm::MinHash {
                shingle_size: 2,
                num_hashes: 64,
            },
            similarity_threshold: 0.70,
            strategy: SemanticDeduplicationStrategy::Remove,
            canonical_policy: SemanticCanonicalPolicy::StructuralHierarchy,
            min_words_threshold: 6,
            log_exclusions: true,
        }
    }

    /// Preset: Conservative near-duplicate pointer substitution with 0.85 similarity threshold.
    #[must_use]
    pub fn conservative() -> Self {
        Self {
            algorithm: SemanticAlgorithm::MinHash {
                shingle_size: 2,
                num_hashes: 64,
            },
            similarity_threshold: 0.85,
            strategy: SemanticDeduplicationStrategy::ReplaceWithAnnotatedPointer,
            canonical_policy: SemanticCanonicalPolicy::StructuralHierarchy,
            min_words_threshold: 10,
            log_exclusions: true,
        }
    }

    /// Preset: SimHash-based near-duplicate consolidation.
    #[must_use]
    pub fn simhash_based(max_hamming_distance: u32) -> Self {
        Self {
            algorithm: SemanticAlgorithm::SimHash {
                max_hamming_distance,
            },
            similarity_threshold: 0.80,
            strategy: SemanticDeduplicationStrategy::ReplaceWithAnnotatedPointer,
            canonical_policy: SemanticCanonicalPolicy::InformationRichness,
            min_words_threshold: 8,
            log_exclusions: true,
        }
    }
}
