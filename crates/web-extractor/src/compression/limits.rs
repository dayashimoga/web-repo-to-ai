//! # Body and Decompression Limits
//!
//! Safety bounds for raw network payload size, decompressed memory footprint, and compression bomb expansion ratios.

use serde::{Deserialize, Serialize};

/// Configurable safety limits for HTTP response body ingestion and decompression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BodyLimits {
    /// Maximum allowed raw / compressed bytes read from the network.
    ///
    /// Default: 10 MiB (`10 * 1024 * 1024` bytes).
    pub max_raw_body_bytes: usize,

    /// Maximum allowed decompressed bytes stored in memory.
    ///
    /// Default: 25 MiB (`25 * 1024 * 1024` bytes).
    pub max_decompressed_bytes: usize,

    /// Maximum allowed expansion ratio (`decompressed_bytes / compressed_bytes`).
    ///
    /// If the decompressed output exceeds this multiple of the compressed input,
    /// decompression is immediately aborted as a potential compression bomb.
    ///
    /// Default: `20.0` (e.g. 1 KB compressed expanding to > 20 KB).
    pub max_expansion_ratio: f64,

    /// Minimum decompressed byte threshold before checking the expansion ratio.
    ///
    /// Avoids false positives on small HTML documents (e.g. 500 bytes of HTML with lots
    /// of repeated whitespace) where high compression ratios are normal.
    ///
    /// Default: `4096` bytes (4 KiB).
    pub min_ratio_check_bytes: usize,

    /// Whether to enforce strict equality between declared `Content-Length` and actual bytes read.
    ///
    /// Default: `true`.
    pub enforce_content_length: bool,
}

impl Default for BodyLimits {
    fn default() -> Self {
        Self {
            max_raw_body_bytes: 10 * 1024 * 1024,     // 10 MB
            max_decompressed_bytes: 25 * 1024 * 1024, // 25 MB
            max_expansion_ratio: 20.0,
            min_ratio_check_bytes: 4096, // 4 KB
            enforce_content_length: true,
        }
    }
}

impl BodyLimits {
    /// Creates a new `BodyLimits` with specified raw and decompressed ceilings, using standard ratio defaults.
    pub fn new(max_raw_body_bytes: usize, max_decompressed_bytes: usize) -> Self {
        Self {
            max_raw_body_bytes,
            max_decompressed_bytes,
            ..Default::default()
        }
    }

    /// Sets the maximum allowable expansion ratio.
    #[must_use]
    pub fn with_max_expansion_ratio(mut self, ratio: f64) -> Self {
        self.max_expansion_ratio = ratio;
        self
    }

    /// Sets the minimum decompressed byte threshold before ratio checking begins.
    #[must_use]
    pub fn with_min_ratio_check_bytes(mut self, bytes: usize) -> Self {
        self.min_ratio_check_bytes = bytes;
        self
    }

    /// Configures whether to strictly enforce `Content-Length` match.
    #[must_use]
    pub fn with_enforce_content_length(mut self, enforce: bool) -> Self {
        self.enforce_content_length = enforce;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_limits_defaults() {
        let limits = BodyLimits::default();
        assert_eq!(limits.max_raw_body_bytes, 10 * 1024 * 1024);
        assert_eq!(limits.max_decompressed_bytes, 25 * 1024 * 1024);
        assert_eq!(limits.max_expansion_ratio, 20.0);
        assert_eq!(limits.min_ratio_check_bytes, 4096);
        assert!(limits.enforce_content_length);
    }

    #[test]
    fn test_body_limits_builder() {
        let limits = BodyLimits::new(1024, 2048)
            .with_max_expansion_ratio(10.0)
            .with_min_ratio_check_bytes(512)
            .with_enforce_content_length(false);

        assert_eq!(limits.max_raw_body_bytes, 1024);
        assert_eq!(limits.max_decompressed_bytes, 2048);
        assert_eq!(limits.max_expansion_ratio, 10.0);
        assert_eq!(limits.min_ratio_check_bytes, 512);
        assert!(!limits.enforce_content_length);
    }
}
