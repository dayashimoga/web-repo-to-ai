//! # Shingling and N-Gram Tokenizer
//!
//! Decomposes text into normalized word shingles and character k-grams for
//! MinHash and SimHash similarity estimation.

/// Word shingler generating sliding windows of consecutive normalized words.
pub struct Shingler;

impl Shingler {
    /// Extracts $k$-word shingles from an input text string.
    ///
    /// Words are lowercased and stripped of leading/trailing non-alphanumeric punctuation.
    /// Returns 64-bit FNV-1a hashes of each shingle.
    #[must_use]
    pub fn word_shingles(text: &str, k: usize) -> Vec<u64> {
        if k == 0 || text.is_empty() {
            return Vec::new();
        }

        // 1. Tokenize into normalized words
        let words: Vec<String> = text
            .split_whitespace()
            .map(|w| {
                w.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_ascii_lowercase()
            })
            .filter(|w| !w.is_empty())
            .collect();

        if words.len() < k {
            return Vec::new();
        }

        let mut shingles = Vec::with_capacity(words.len() - k + 1);

        for window in words.windows(k) {
            let combined = window.join(" ");
            shingles.push(fnv1a_hash64(combined.as_bytes()));
        }

        shingles
    }

    /// Extracts character $k$-grams from an input string.
    #[must_use]
    pub fn char_kgrams(text: &str, k: usize) -> Vec<u64> {
        if k == 0 || text.is_empty() {
            return Vec::new();
        }

        let chars: Vec<char> = text.chars().collect();
        if chars.len() < k {
            return Vec::new();
        }

        let mut kgrams = Vec::with_capacity(chars.len() - k + 1);
        for window in chars.windows(k) {
            let s: String = window.iter().collect();
            kgrams.push(fnv1a_hash64(s.as_bytes()));
        }

        kgrams
    }
}

/// 64-bit FNV-1a non-cryptographic hash for fast, uniform shingle hashing.
#[inline]
pub fn fnv1a_hash64(bytes: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_shingling() {
        let text = "The quick brown fox jumps over the lazy dog";
        let shingles = Shingler::word_shingles(text, 3);
        // 9 words -> 7 3-word shingles
        assert_eq!(shingles.len(), 7);
    }

    #[test]
    fn test_shingles_short_text() {
        let text = "hello world";
        let shingles = Shingler::word_shingles(text, 3);
        assert!(shingles.is_empty());
    }

    #[test]
    fn test_char_kgrams() {
        let text = "abcdef";
        let kgrams = Shingler::char_kgrams(text, 3);
        assert_eq!(kgrams.len(), 4); // "abc", "bcd", "cde", "def"
    }
}
