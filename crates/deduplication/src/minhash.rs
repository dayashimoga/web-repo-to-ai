//! # MinHash Signature and Jaccard Similarity Estimator
//!
//! Generates compact locality-sensitive hash signatures from shingle sets and estimates
//! pairwise Jaccard set similarity with sub-linear complexity.

use serde::{Deserialize, Serialize};

/// Number of hash functions in the default MinHash signature bank.
pub const DEFAULT_NUM_HASHES: usize = 64;

/// Generates a deterministic SplitMix64 pseudo-random permutation for MinHash indexing.
#[inline]
fn splitmix64(x: u64, seed: u64) -> u64 {
    let mut z = x.wrapping_add(seed).wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

/// A compact locality-sensitive MinHash signature vector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinHashSignature {
    /// Array of minimum hash values across the permutation function bank.
    pub hashes: Vec<u64>,
}

impl MinHashSignature {
    /// Computes a MinHash signature from a collection of 64-bit shingle hashes.
    ///
    /// If `shingles` is empty, an empty signature is returned.
    #[must_use]
    pub fn compute(shingles: &[u64], num_hashes: usize) -> Self {
        if shingles.is_empty() || num_hashes == 0 {
            return Self { hashes: Vec::new() };
        }

        let mut hashes = vec![u64::MAX; num_hashes];

        for (i, slot) in hashes.iter_mut().enumerate() {
            let seed = (i as u64).wrapping_mul(0x517cc1b727220a95);
            for &shingle in shingles {
                let h = splitmix64(shingle, seed);
                if h < *slot {
                    *slot = h;
                }
            }
        }

        Self { hashes }
    }

    /// Estimates the Jaccard similarity coefficient $\hat{J}(A, B) \in [0.0, 1.0]$ against another signature.
    #[must_use]
    pub fn jaccard_similarity(&self, other: &Self) -> f64 {
        if self.hashes.is_empty() || other.hashes.is_empty() {
            return 0.0;
        }

        let min_len = self.hashes.len().min(other.hashes.len());
        if min_len == 0 {
            return 0.0;
        }

        let mut matches = 0;
        for i in 0..min_len {
            if self.hashes[i] == other.hashes[i] {
                matches += 1;
            }
        }

        matches as f64 / min_len as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shingle::Shingler;

    #[test]
    fn test_identical_text_has_jaccard_one() {
        let text =
            "Modern async distributed system architecture with microservices and event sourcing.";
        let shingles = Shingler::word_shingles(text, 3);
        let sig1 = MinHashSignature::compute(&shingles, DEFAULT_NUM_HASHES);
        let sig2 = MinHashSignature::compute(&shingles, DEFAULT_NUM_HASHES);

        assert!((sig1.jaccard_similarity(&sig2) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_near_duplicate_high_similarity() {
        let text1 = "Modern async distributed system architecture with microservices and event sourcing in Tokio.";
        let text2 = "Modern async distributed system architecture with microservices and event sourcing in Rust.";

        let s1 = Shingler::word_shingles(text1, 3);
        let s2 = Shingler::word_shingles(text2, 3);

        let sig1 = MinHashSignature::compute(&s1, DEFAULT_NUM_HASHES);
        let sig2 = MinHashSignature::compute(&s2, DEFAULT_NUM_HASHES);

        let sim = sig1.jaccard_similarity(&sig2);
        // High similarity (~0.75 - 0.90) expected
        assert!(sim > 0.65, "Expected high similarity, got {sim}");
    }

    #[test]
    fn test_disjoint_text_low_similarity() {
        let text1 = "Alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho.";
        let text2 = "One two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen.";

        let s1 = Shingler::word_shingles(text1, 3);
        let s2 = Shingler::word_shingles(text2, 3);

        let sig1 = MinHashSignature::compute(&s1, DEFAULT_NUM_HASHES);
        let sig2 = MinHashSignature::compute(&s2, DEFAULT_NUM_HASHES);

        let sim = sig1.jaccard_similarity(&sig2);
        assert!(sim < 0.15, "Expected low similarity, got {sim}");
    }
}
