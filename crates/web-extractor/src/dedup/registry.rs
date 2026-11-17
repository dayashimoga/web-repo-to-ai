//! # Duplicate Detection Registry
//!
//! Stateful tracking registry detecting exact URL duplicates, exact content duplicates (BLAKE3),
//! and near-duplicate documents via 64-bit SimHash Hamming distance.

use std::collections::HashMap;

use chrono::Utc;

use super::model::{DedupOptions, DuplicateClassification, DuplicateRecord};
use super::simhash::SimHasher;

/// Thread-safe or local registry for tracking seen documents and detecting duplication.
#[derive(Debug, Clone, Default)]
pub struct DuplicateRegistry {
    options: DedupOptions,
    by_url: HashMap<String, DuplicateRecord>,
    by_content_hash: HashMap<String, DuplicateRecord>,
    records: Vec<DuplicateRecord>,
}

impl DuplicateRegistry {
    /// Constructs a new empty `DuplicateRegistry` with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a `DuplicateRegistry` with custom configuration options.
    #[must_use]
    pub fn with_options(options: DedupOptions) -> Self {
        Self {
            options,
            by_url: HashMap::new(),
            by_content_hash: HashMap::new(),
            records: Vec::new(),
        }
    }

    /// Evaluates a document against seen registry entries without modifying registry state.
    #[must_use]
    pub fn check(
        &self,
        canonical_url: &str,
        content_hash_blake3: &str,
        text: &str,
    ) -> DuplicateClassification {
        // 1. Exact canonical URL duplicate
        if let Some(existing) = self.by_url.get(canonical_url) {
            return DuplicateClassification::ExactUrlDuplicate {
                existing_url: existing.canonical_url.clone(),
                seen_at: existing.registered_at,
            };
        }

        // 2. Exact content hash duplicate
        if let Some(existing) = self.by_content_hash.get(content_hash_blake3) {
            return DuplicateClassification::ExactContentDuplicate {
                original_url: existing.canonical_url.clone(),
                content_hash: content_hash_blake3.to_string(),
                seen_at: existing.registered_at,
            };
        }

        // 3. Near-duplicate text via SimHash
        let words_count = text.split_whitespace().count();
        if words_count >= self.options.min_tokens_for_simhash {
            let simhash = SimHasher::compute(text);
            let mut best_match: Option<(&DuplicateRecord, u32)> = None;

            for rec in &self.records {
                let dist = SimHasher::hamming_distance(simhash, rec.simhash);
                if dist <= self.options.max_hamming_distance {
                    match best_match {
                        Some((_, cur_min)) if dist < cur_min => {
                            best_match = Some((rec, dist));
                        }
                        None => {
                            best_match = Some((rec, dist));
                        }
                        _ => {}
                    }
                }
            }

            if let Some((rec, dist)) = best_match {
                return DuplicateClassification::NearDuplicate {
                    original_url: rec.canonical_url.clone(),
                    similarity: SimHasher::similarity(simhash, rec.simhash),
                    hamming_distance: dist,
                    seen_at: rec.registered_at,
                };
            }
        }

        DuplicateClassification::Unique
    }

    /// Evaluates a document against seen registry entries and registers it if unique.
    pub fn check_and_register(
        &mut self,
        canonical_url: &str,
        content_hash_blake3: &str,
        text: &str,
    ) -> DuplicateClassification {
        let classification = self.check(canonical_url, content_hash_blake3, text);

        if classification.is_unique() {
            let simhash = SimHasher::compute(text);
            let record = DuplicateRecord {
                canonical_url: canonical_url.to_string(),
                content_hash_blake3: content_hash_blake3.to_string(),
                simhash,
                registered_at: Utc::now(),
            };

            self.by_url
                .insert(canonical_url.to_string(), record.clone());
            self.by_content_hash
                .insert(content_hash_blake3.to_string(), record.clone());
            self.records.push(record);
        }

        classification
    }

    /// Returns the number of distinct unique documents tracked in the registry.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns true if the registry contains no tracked documents.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Clears all stored records from the registry.
    pub fn clear(&mut self) {
        self.by_url.clear();
        self.by_content_hash.clear();
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_uniqueness_and_exact_duplicates() {
        let mut registry = DuplicateRegistry::new();

        let url1 = "https://example.com/docs/rust";
        let hash1 = "blake3_hash_aaa";
        let text1 = "Rust is a modern systems programming language emphasizing memory safety.";

        // First check must be unique
        let res1 = registry.check_and_register(url1, hash1, text1);
        assert!(res1.is_unique());
        assert_eq!(registry.len(), 1);

        // Exact URL match
        let res2 = registry.check_and_register(url1, "blake3_hash_diff", "Different body");
        assert!(matches!(
            res2,
            DuplicateClassification::ExactUrlDuplicate { .. }
        ));
        assert_eq!(registry.len(), 1);

        // Exact content match with different URL
        let url3 = "https://example.com/mirror/rust";
        let res3 = registry.check_and_register(url3, hash1, text1);
        assert!(matches!(
            res3,
            DuplicateClassification::ExactContentDuplicate { .. }
        ));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_registry_near_duplicate_detection() {
        let mut registry = DuplicateRegistry::new();

        let url1 = "https://example.com/blog/concurrency";
        let hash1 = "hash_1111";
        let text1 = "Fearless concurrency in Rust enables developers to write multi-threaded code with confidence. The compiler checks thread safety statically using Send and Sync traits, preventing data races at compile time rather than during production runtime.";

        let res1 = registry.check_and_register(url1, hash1, text1);
        assert!(res1.is_unique());

        // Near-duplicate: slight modification
        let url2 = "https://example.com/blog/concurrency-v2";
        let hash2 = "hash_2222";
        let text2 = "Fearless concurrency in Rust allows developers to write multi-threaded code with confidence. The compiler checks thread safety statically using Send and Sync traits, preventing data races at compile time rather than during runtime.";

        let res2 = registry.check_and_register(url2, hash2, text2);
        match res2 {
            DuplicateClassification::NearDuplicate {
                ref original_url,
                similarity,
                hamming_distance,
                ..
            } => {
                assert_eq!(original_url, url1);
                assert!(hamming_distance <= 8);
                assert!(similarity >= 0.85);
            }
            other => panic!("Expected NearDuplicate, got {other:?}"),
        }
    }
}
