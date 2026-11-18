//! # Exact Deduplication Configuration
//!
//! Fluent configuration and domain profiles governing cryptographic hash comparison,
//! canonical selection priority, deduplication strategies, and exclusion logging.

use serde::{Deserialize, Serialize};

/// Action taken when an exact duplicate section is identified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeduplicationStrategy {
    /// Completely removes the duplicate section from the document body,
    /// recording an auditable exclusion log entry.
    Remove,
    /// Replaces the duplicate section's body with a concise reference pointer
    /// linking to the canonical occurrence to preserve structural hierarchy.
    ReplaceWithPointer,
    /// Preserves the duplicate section body intact, but records telemetry,
    /// relationships, and clusters for downstream inspection.
    MarkOnly,
}

impl DeduplicationStrategy {
    /// Returns a human-readable identifier for display and telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Remove => "remove",
            Self::ReplaceWithPointer => "replace_with_pointer",
            Self::MarkOnly => "mark_only",
        }
    }
}

/// Policy governing which section instance is elected as the authoritative "canonical" occurrence.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalSelectionPolicy {
    /// The first encountered instance in document outline order is elected canonical.
    FirstSeen,
    /// The section with the shortest original file path or title is elected canonical
    /// (e.g. `src/utils.rs` is preferred over `vendor/pkg/src/utils.rs`).
    ShortestPath,
    /// Prioritizes sections according to structural importance:
    /// `CodeFile` > `CodeSnippet` > `Prose` > `Table` > `Admonition` > `Frontmatter` > `Manifest` > `Navigation`.
    StructuralHierarchy,
    /// Prefers sections whose original path begins with a specified prefix (e.g. `"src/"` or `"docs/"`).
    PreferredPrefix(String),
}

/// Scope of content evaluated when computing deduplication hashes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HashScope {
    /// Exact byte-for-byte comparison of the raw section body string.
    ExactBytes,
    /// Compares bodies after canonical line-ending and whitespace trimming,
    /// eliminating trivial line break discrepancies.
    Normalized,
}

/// Master configuration for exact content deduplication across a CDM document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactDeduplicationConfig {
    /// Strategy applied when duplicates are detected.
    pub strategy: DeduplicationStrategy,
    /// Selection rule determining the canonical occurrence in each duplicate cluster.
    pub canonical_policy: CanonicalSelectionPolicy,
    /// Hashing scope (raw bytes vs whitespace-normalized).
    pub hash_scope: HashScope,
    /// Minimum byte size threshold required for a section to be eligible for deduplication.
    /// Tiny snippets (< 32 bytes like single braces or empty stubs) are skipped.
    pub min_byte_threshold: usize,
    /// When true, records an auditable [`web_repo_to_ai_cdm::ExclusionRecord`] for every removed duplicate.
    pub log_exclusions: bool,
}

impl Default for ExactDeduplicationConfig {
    fn default() -> Self {
        Self {
            strategy: DeduplicationStrategy::ReplaceWithPointer,
            canonical_policy: CanonicalSelectionPolicy::StructuralHierarchy,
            hash_scope: HashScope::Normalized,
            min_byte_threshold: 32,
            log_exclusions: true,
        }
    }
}

impl ExactDeduplicationConfig {
    /// Preset: Aggressive deduplication removing all duplicate sections entirely.
    #[must_use]
    pub fn aggressive() -> Self {
        Self {
            strategy: DeduplicationStrategy::Remove,
            canonical_policy: CanonicalSelectionPolicy::StructuralHierarchy,
            hash_scope: HashScope::Normalized,
            min_byte_threshold: 16,
            log_exclusions: true,
        }
    }

    /// Preset: Conservative deduplication replacing duplicate bodies with pointer comments.
    #[must_use]
    pub fn conservative() -> Self {
        Self::default()
    }

    /// Preset: Lossless deduplication marking relationships without removing or altering bodies.
    #[must_use]
    pub fn lossless() -> Self {
        Self {
            strategy: DeduplicationStrategy::MarkOnly,
            canonical_policy: CanonicalSelectionPolicy::FirstSeen,
            hash_scope: HashScope::ExactBytes,
            min_byte_threshold: 32,
            log_exclusions: false,
        }
    }
}
