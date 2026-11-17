//! # Canonical URL and Deduplication Models
//!
//! Strongly-typed domain structures for canonical URL resolution,
//! content fingerprinting, and duplicate classification.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use web_repo_to_ai_url_security::ValidatedUrl;

/// The origin or heuristic source from which a canonical URL was resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalSource {
    /// Authoritative HTML `<link rel="canonical" href="...">` tag.
    HtmlLinkRelCanonical,
    /// OpenGraph `<meta property="og:url" content="...">` tag.
    OpenGraph,
    /// Schema.org JSON-LD structured data URL (`url` or `mainEntityOfPage`).
    JsonLd,
    /// Final destination URL following HTTP redirect hops.
    HttpRedirect,
    /// Original requested URL when no redirects or metadata overrides were present.
    OriginalRequest,
}

/// The outcome of evaluating competing canonical URL candidates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalResolution {
    /// The normalized, SSRF-validated canonical URL.
    pub canonical_url: ValidatedUrl,
    /// The source from which this canonical URL was derived.
    pub source: CanonicalSource,
    /// The raw candidate string before parsing/normalization, if from metadata.
    pub raw_candidate: Option<String>,
    /// Whether the resolved canonical URL points to a different origin/domain.
    pub is_cross_domain: bool,
    /// Diagnostic confidence score (0.0 to 1.0) of the canonical resolution.
    pub confidence: u8,
    /// Diagnostic warnings emitted during candidate evaluation.
    pub warnings: Vec<String>,
}

impl CanonicalResolution {
    /// Returns true if this resolution is considered high-confidence (>= 80%).
    #[must_use]
    pub fn is_high_confidence(&self) -> bool {
        self.confidence >= 80
    }
}

/// Duplicate classification result for a webpage against seen history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DuplicateClassification {
    /// Webpage is novel and unique; no matching URL, content hash, or near-duplicate found.
    Unique,
    /// Webpage has the exact same canonical URL as a previously seen page.
    ExactUrlDuplicate {
        /// The canonical URL already tracked in registry.
        existing_url: String,
        /// Timestamp when the existing page was first registered.
        seen_at: DateTime<Utc>,
    },
    /// Webpage has the exact same cryptographic content hash (BLAKE3) as a previously seen page.
    ExactContentDuplicate {
        /// URL of the original page possessing this exact content hash.
        original_url: String,
        /// The identical BLAKE3 hex content hash.
        content_hash: String,
        /// Timestamp when the original page was first registered.
        seen_at: DateTime<Utc>,
    },
    /// Webpage text is a near-duplicate based on SimHash 64-bit Hamming distance.
    NearDuplicate {
        /// URL of the original near-identical page.
        original_url: String,
        /// Cosine/SimHash similarity ratio (e.g. 0.95 to 1.00).
        similarity: f64,
        /// Number of differing bits in the 64-bit SimHash fingerprint (0..=64).
        hamming_distance: u32,
        /// Timestamp when the original page was first registered.
        seen_at: DateTime<Utc>,
    },
}

impl DuplicateClassification {
    /// Returns true if the document was classified as unique.
    #[must_use]
    pub fn is_unique(&self) -> bool {
        matches!(self, Self::Unique)
    }

    /// Returns true if the document is an exact duplicate (by URL or content hash).
    #[must_use]
    pub fn is_exact_duplicate(&self) -> bool {
        matches!(
            self,
            Self::ExactUrlDuplicate { .. } | Self::ExactContentDuplicate { .. }
        )
    }

    /// Returns true if the document is a near-duplicate.
    #[must_use]
    pub fn is_near_duplicate(&self) -> bool {
        matches!(self, Self::NearDuplicate { .. })
    }
}

/// Stored entry within the duplicate detection registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuplicateRecord {
    /// Normalized canonical URL of the document.
    pub canonical_url: String,
    /// Lowercase BLAKE3 hex content hash.
    pub content_hash_blake3: String,
    /// 64-bit SimHash fingerprint computed over clean text.
    pub simhash: u64,
    /// Registration timestamp.
    pub registered_at: DateTime<Utc>,
}

/// Options controlling canonical URL resolution and duplicate detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupOptions {
    /// Maximum Hamming distance threshold for near-duplicate classification (default: 3).
    pub max_hamming_distance: u32,
    /// Minimum word token count required before evaluating SimHash near-duplicates (default: 20).
    pub min_tokens_for_simhash: usize,
    /// Whether cross-domain canonical links are honored if valid (default: true).
    pub allow_cross_domain_canonical: bool,
}

impl Default for DedupOptions {
    fn default() -> Self {
        Self {
            max_hamming_distance: 8,
            min_tokens_for_simhash: 10,
            allow_cross_domain_canonical: true,
        }
    }
}
