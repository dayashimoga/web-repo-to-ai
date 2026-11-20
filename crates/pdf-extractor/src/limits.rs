//! # Resource and Safety Limits for PDF Processing
//!
//! Enforces bounded memory, file size, page count, and stream decompression limits
//! to guard against PDF decompression bombs, infinite loops, and Denial of Service (DoS) attacks.

use serde::{Deserialize, Serialize};

/// Configurable safety limits for PDF parsing and extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PdfLimits {
    /// Maximum allowed file size in bytes (default: 50 MB = 52,428,800 bytes).
    pub max_file_size_bytes: usize,

    /// Maximum number of pages to process (default: 500 pages).
    pub max_page_count: u32,

    /// Maximum text spans allowed per page (default: 50,000 spans).
    pub max_spans_per_page: usize,

    /// Maximum total characters allowed across all pages (default: 5,000,000 chars).
    pub max_total_characters: usize,

    /// Maximum table cells allowed per table (default: 10,000 cells).
    pub max_table_cells: usize,

    /// Maximum decompressed stream size in bytes (default: 20 MB = 20,971,520 bytes).
    pub max_decompressed_stream_bytes: usize,

    /// Behavior when `max_page_count` is reached: if `true`, gracefully truncate and
    /// process pages up to `max_page_count`; if `false`, fail with `PdfError::LimitExceeded`.
    /// Default: `true` (graceful truncation with completeness warning).
    pub truncate_on_page_limit: bool,
}

impl Default for PdfLimits {
    fn default() -> Self {
        Self {
            max_file_size_bytes: 50 * 1024 * 1024, // 50 MiB
            max_page_count: 500,
            max_spans_per_page: 50_000,
            max_total_characters: 5_000_000,
            max_table_cells: 10_000,
            max_decompressed_stream_bytes: 20 * 1024 * 1024, // 20 MiB
            truncate_on_page_limit: true,
        }
    }
}

impl PdfLimits {
    /// Creates a strict limit configuration suitable for untrusted public endpoints.
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            max_file_size_bytes: 20 * 1024 * 1024, // 20 MiB
            max_page_count: 100,
            max_spans_per_page: 15_000,
            max_total_characters: 1_000_000,
            max_table_cells: 5_000,
            max_decompressed_stream_bytes: 10 * 1024 * 1024, // 10 MiB
            truncate_on_page_limit: false,
        }
    }

    /// Creates a permissive limit configuration for large trusted documents.
    #[must_use]
    pub const fn permissive() -> Self {
        Self {
            max_file_size_bytes: 200 * 1024 * 1024, // 200 MiB
            max_page_count: 2_000,
            max_spans_per_page: 100_000,
            max_total_characters: 20_000_000,
            max_table_cells: 50_000,
            max_decompressed_stream_bytes: 50 * 1024 * 1024, // 50 MiB
            truncate_on_page_limit: true,
        }
    }

    /// Builder method to customize `max_file_size_bytes`.
    #[must_use]
    pub const fn with_max_file_size_bytes(mut self, size: usize) -> Self {
        self.max_file_size_bytes = size;
        self
    }

    /// Builder method to customize `max_page_count`.
    #[must_use]
    pub const fn with_max_page_count(mut self, count: u32) -> Self {
        self.max_page_count = count;
        self
    }

    /// Builder method to customize `truncate_on_page_limit`.
    #[must_use]
    pub const fn with_truncate_on_page_limit(mut self, truncate: bool) -> Self {
        self.truncate_on_page_limit = truncate;
        self
    }
}
