//! # SimHash Content Fingerprinting
//!
//! Implements Charikar's 64-bit SimHash algorithm over word n-grams and shingles
//! for detecting near-duplicate documents and computing Hamming similarity.

use std::collections::HashSet;

/// Computes 64-bit locality-sensitive hash fingerprints for near-duplicate text detection.
#[derive(Debug, Clone, Default)]
pub struct SimHasher;

impl SimHasher {
    /// Computes a 64-bit SimHash fingerprint from normalized text using word tokens with term-frequency weighting.
    #[must_use]
    pub fn compute(text: &str) -> u64 {
        let tokens = Self::tokenize(text);
        if tokens.is_empty() {
            return 0;
        }

        let mut term_counts: std::collections::HashMap<&str, i32> =
            std::collections::HashMap::new();
        for token in &tokens {
            *term_counts.entry(token.as_str()).or_insert(0) += 1;
        }

        let mut v = [0i32; 64];

        for (term, count) in term_counts {
            let hash = Self::hash64(term.as_bytes());
            for (bit, v_slot) in v.iter_mut().enumerate() {
                if (hash >> bit) & 1 == 1 {
                    *v_slot += count;
                } else {
                    *v_slot -= count;
                }
            }
        }

        let mut fingerprint: u64 = 0;
        for (bit, &weight) in v.iter().enumerate() {
            if weight > 0 {
                fingerprint |= 1u64 << bit;
            }
        }

        fingerprint
    }

    /// Computes the Hamming distance (number of differing bits) between two SimHash fingerprints.
    #[must_use]
    pub fn hamming_distance(a: u64, b: u64) -> u32 {
        (a ^ b).count_ones()
    }

    /// Computes the cosine similarity ratio (0.0 to 1.0) derived from SimHash Hamming distance.
    #[must_use]
    pub fn similarity(a: u64, b: u64) -> f64 {
        1.0 - (Self::hamming_distance(a, b) as f64 / 64.0)
    }

    /// Checks if two fingerprints represent near-duplicate content within the threshold.
    #[must_use]
    pub fn is_near_duplicate(a: u64, b: u64, max_hamming_distance: u32) -> bool {
        Self::hamming_distance(a, b) <= max_hamming_distance
    }

    /// Computes exact token Jaccard similarity between two texts ($|A \cap B| / |A \cup B|$).
    #[must_use]
    pub fn jaccard_similarity(text_a: &str, text_b: &str) -> f64 {
        let set_a: HashSet<String> = Self::tokenize(text_a).into_iter().collect();
        let set_b: HashSet<String> = Self::tokenize(text_b).into_iter().collect();

        if set_a.is_empty() && set_b.is_empty() {
            return 1.0;
        }
        if set_a.is_empty() || set_b.is_empty() {
            return 0.0;
        }

        let intersection_count = set_a.intersection(&set_b).count();
        let union_count = set_a.union(&set_b).count();

        intersection_count as f64 / union_count as f64
    }

    /// Tokenizes input text into lowercase alphanumeric words.
    fn tokenize(text: &str) -> Vec<String> {
        text.split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_ascii_lowercase())
            .collect()
    }

    /// 64-bit mixer hash using BLAKE3.
    fn hash64(data: &[u8]) -> u64 {
        let digest = blake3::hash(data);
        let bytes: [u8; 8] = digest.as_bytes()[..8]
            .try_into()
            .expect("slice with 8 elements");
        u64::from_le_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simhash_identical_text() {
        let text = "Rust is a systems programming language focused on safety and speed.";
        let h1 = SimHasher::compute(text);
        let h2 = SimHasher::compute(text);

        assert_eq!(h1, h2);
        assert_eq!(SimHasher::hamming_distance(h1, h2), 0);
        assert!((SimHasher::similarity(h1, h2) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_simhash_near_duplicate_detection() {
        let text1 = "Rust is a multi-paradigm, general-purpose programming language designed for performance and safety, especially safe concurrency. Rust is syntactically similar to C++.";
        let text2 = "Rust is a multi-paradigm, general-purpose programming language built for performance and safety, especially safe concurrency. Rust is syntactically similar to C++.";

        let h1 = SimHasher::compute(text1);
        let h2 = SimHasher::compute(text2);

        let dist = SimHasher::hamming_distance(h1, h2);
        assert!(
            dist <= 3,
            "Hamming distance should be <= 3 for near-duplicates, got {dist}"
        );
        assert!(SimHasher::is_near_duplicate(h1, h2, 3));
        assert!(SimHasher::similarity(h1, h2) >= 0.95);
    }

    #[test]
    fn test_simhash_distinct_documents_have_high_distance() {
        let text1 = "Quantum computing leverages superposition and entanglement to perform complex matrix calculations exponentially faster than classical computers.";
        let text2 = "The Mediterranean diet emphasizes fresh vegetables, olive oil, legumes, and lean proteins for cardiovascular health.";

        let h1 = SimHasher::compute(text1);
        let h2 = SimHasher::compute(text2);

        let dist = SimHasher::hamming_distance(h1, h2);
        assert!(
            dist > 10,
            "Completely different topics should have high Hamming distance, got {dist}"
        );
        assert!(!SimHasher::is_near_duplicate(h1, h2, 3));
    }

    #[test]
    fn test_jaccard_similarity() {
        let text1 = "the quick brown fox";
        let text2 = "the fast brown fox";

        let score = SimHasher::jaccard_similarity(text1, text2);
        // intersection: {the, brown, fox} (3)
        // union: {the, quick, fast, brown, fox} (5)
        // 3/5 = 0.6
        assert!((score - 0.6).abs() < 1e-4);
    }
}
