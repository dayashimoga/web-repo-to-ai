//! # 64-Bit SimHash Fingerprinting
//!
//! Generates compact 64-bit Locality Sensitive Hash fingerprints based on Moses Charikar's
//! algorithm, where Hamming distance reflects cosine similarity.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::shingle::fnv1a_hash64;

/// A compact 64-bit SimHash fingerprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SimHash64(pub u64);

impl SimHash64 {
    /// Computes a 64-bit SimHash fingerprint from text tokens.
    ///
    /// Weights each token by its empirical term frequency within the string.
    #[must_use]
    pub fn compute(text: &str) -> Self {
        if text.is_empty() {
            return Self(0);
        }

        // 1. Tokenize into normalized words and count term frequencies
        let mut term_frequencies: BTreeMap<String, i32> = BTreeMap::new();
        for word in text.split_whitespace() {
            let clean = word
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_ascii_lowercase();
            if !clean.is_empty() {
                *term_frequencies.entry(clean).or_default() += 1;
            }
        }

        if term_frequencies.is_empty() {
            return Self(0);
        }

        // 2. Accumulate weighted bit vectors
        let mut bit_weights = [0i64; 64];

        for (word, freq) in term_frequencies {
            let token_hash = fnv1a_hash64(word.as_bytes());
            let weight = i64::from(freq);

            for (bit, slot) in bit_weights.iter_mut().enumerate() {
                if (token_hash & (1u64 << bit)) != 0 {
                    *slot += weight;
                } else {
                    *slot -= weight;
                }
            }
        }

        // 3. Form final 64-bit hash
        let mut fingerprint: u64 = 0;
        for (bit, &weight) in bit_weights.iter().enumerate() {
            if weight > 0 {
                fingerprint |= 1u64 << bit;
            }
        }

        Self(fingerprint)
    }

    /// Computes the Hamming distance (number of differing bits) against another fingerprint.
    #[must_use]
    #[inline]
    pub const fn hamming_distance(self, other: Self) -> u32 {
        (self.0 ^ other.0).count_ones()
    }

    /// Estimates the cosine similarity in $[0.0, 1.0]$ derived from Hamming distance.
    #[must_use]
    pub fn similarity(self, other: Self) -> f64 {
        let dist = self.hamming_distance(other);
        1.0 - (f64::from(dist) / 64.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simhash_identical_text() {
        let text = "Distributed scalable message broker and high performance actor system.";
        let hash1 = SimHash64::compute(text);
        let hash2 = SimHash64::compute(text);

        assert_eq!(hash1.hamming_distance(hash2), 0);
        assert!((hash1.similarity(hash2) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_simhash_near_duplicate_small_distance() {
        let text1 =
            "Distributed scalable message broker and high performance actor system in Rust.";
        let text2 =
            "Distributed scalable message broker and high performance actor system in Tokio.";

        let hash1 = SimHash64::compute(text1);
        let hash2 = SimHash64::compute(text2);

        let distance = hash1.hamming_distance(hash2);
        assert!(distance <= 10, "Expected small distance, got {distance}");
        assert!(hash1.similarity(hash2) >= 0.80);
    }

    #[test]
    fn test_simhash_dissimilar_large_distance() {
        let text1 = "Apple orange banana strawberry pineapple watermelon grape lemon lime.";
        let text2 =
            "Quantum electrodynamics relativistic mechanics gauge symmetry photon emission.";

        let hash1 = SimHash64::compute(text1);
        let hash2 = SimHash64::compute(text2);

        let distance = hash1.hamming_distance(hash2);
        assert!(distance >= 18, "Expected large distance, got {distance}");
    }
}
