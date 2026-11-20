//! # OCR Safety and Resource Limits
//!
//! Enforces memory ceilings, execution deadlines, and word count constraints
//! to prevent Denial-of-Service (DoS) and resource exhaustion attacks.

use crate::error::{OcrError, Result};
use std::time::Duration;

/// Resource and execution safety limits for OCR operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OcrLimits {
    /// Maximum size of raw input image in bytes (default: 50 MiB).
    pub max_image_bytes: usize,
    /// Maximum execution duration permitted for OCR subprocess (default: 30 seconds).
    pub timeout: Duration,
    /// Maximum word tokens permitted across all pages (default: 100,000).
    pub max_word_count: usize,
    /// Maximum number of pages to process from a multi-page document (default: 200).
    pub max_pages: usize,
}

impl Default for OcrLimits {
    fn default() -> Self {
        Self {
            max_image_bytes: 50 * 1024 * 1024, // 50 MiB
            timeout: Duration::from_secs(30),
            max_word_count: 100_000,
            max_pages: 200,
        }
    }
}

impl OcrLimits {
    /// Creates a new `OcrLimits` with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum permitted image size in bytes.
    #[must_use]
    pub const fn with_max_image_bytes(mut self, bytes: usize) -> Self {
        self.max_image_bytes = bytes;
        self
    }

    /// Sets the maximum execution timeout.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the maximum allowed word count.
    #[must_use]
    pub const fn with_max_word_count(mut self, max_words: usize) -> Self {
        self.max_word_count = max_words;
        self
    }

    /// Sets the maximum allowed page count.
    #[must_use]
    pub const fn with_max_pages(mut self, max_pages: usize) -> Self {
        self.max_pages = max_pages;
        self
    }

    /// Validates raw input bytes against configured size constraints.
    pub fn validate_input(&self, bytes: &[u8]) -> Result<()> {
        if bytes.is_empty() {
            return Err(OcrError::InputLimitExceeded(
                "Input image data is empty (0 bytes)".to_string(),
            ));
        }

        if bytes.len() > self.max_image_bytes {
            return Err(OcrError::InputLimitExceeded(format!(
                "Input image size {} bytes exceeds configured limit of {} bytes",
                bytes.len(),
                self.max_image_bytes
            )));
        }

        Ok(())
    }

    /// Validates extracted word count against configured limits.
    pub fn validate_word_count(&self, word_count: usize) -> Result<()> {
        if word_count > self.max_word_count {
            return Err(OcrError::InputLimitExceeded(format!(
                "Extracted word count {} exceeds configured limit of {} words",
                word_count, self.max_word_count
            )));
        }
        Ok(())
    }
}
