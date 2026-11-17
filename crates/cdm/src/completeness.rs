//! # Completeness Accounting
//!
//! Models and invariants for tracking extraction completeness across all sources.
//!
//! Enforces the fundamental repository conversion law:
//! `discovered_count = extracted_count + excluded_count + failed_count + unsupported_count`

use serde::{Deserialize, Serialize};

/// Detailed quantitative completeness accounting for an extraction job.
///
/// Ensures compliance with the project's strict completeness invariant.
/// If any discovered item is unaccounted for or if any eligible item fails
/// without an explicit exclusion or error record, `is_complete()` evaluates to `false`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletenessReport {
    /// Total number of items (files, URLs, or DOM blocks) discovered at the source.
    pub discovered_count: u64,
    /// Number of discovered items eligible for processing according to configured filter rules.
    pub eligible_count: u64,
    /// Number of items successfully extracted and converted into CDM sections.
    pub extracted_count: u64,
    /// Number of items intentionally skipped/excluded with recorded reasons.
    pub excluded_count: u64,
    /// Number of items that encountered unrecoverable processing errors.
    pub failed_count: u64,
    /// Number of items discovered whose media type or format is unsupported.
    pub unsupported_count: u64,
}

impl CompletenessReport {
    /// Creates a new completeness report with explicit counts.
    #[must_use]
    pub const fn new(
        discovered_count: u64,
        eligible_count: u64,
        extracted_count: u64,
        excluded_count: u64,
        failed_count: u64,
        unsupported_count: u64,
    ) -> Self {
        Self {
            discovered_count,
            eligible_count,
            extracted_count,
            excluded_count,
            failed_count,
            unsupported_count,
        }
    }

    /// Convenience constructor for a single-source extraction (e.g. single webpage) that succeeded completely.
    #[must_use]
    pub const fn single_item_success() -> Self {
        Self {
            discovered_count: 1,
            eligible_count: 1,
            extracted_count: 1,
            excluded_count: 0,
            failed_count: 0,
            unsupported_count: 0,
        }
    }

    /// Convenience constructor for a single-source extraction that was excluded.
    #[must_use]
    pub const fn single_item_excluded() -> Self {
        Self {
            discovered_count: 1,
            eligible_count: 0,
            extracted_count: 0,
            excluded_count: 1,
            failed_count: 0,
            unsupported_count: 0,
        }
    }

    /// Convenience constructor for a single-source extraction that failed.
    #[must_use]
    pub const fn single_item_failed() -> Self {
        Self {
            discovered_count: 1,
            eligible_count: 1,
            extracted_count: 0,
            excluded_count: 0,
            failed_count: 1,
            unsupported_count: 0,
        }
    }

    /// Verifies the arithmetic invariant:
    /// `discovered_count == extracted_count + excluded_count + failed_count + unsupported_count`
    #[must_use]
    pub fn formula_valid(&self) -> bool {
        self.discovered_count
            == self
                .extracted_count
                .saturating_add(self.excluded_count)
                .saturating_add(self.failed_count)
                .saturating_add(self.unsupported_count)
    }

    /// Evaluates whether the extraction achieved 100% acceptable completeness.
    ///
    /// Requires:
    /// 1. Mathematical formula invariant is strictly satisfied.
    /// 2. Zero failed items (`failed_count == 0`).
    /// 3. All eligible items were successfully extracted (`extracted_count == eligible_count`).
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.formula_valid()
            && self.failed_count == 0
            && self.extracted_count == self.eligible_count
    }

    /// Calculates the extraction yield ratio (`extracted / discovered`), returning a float in `[0.0, 1.0]`.
    #[must_use]
    pub fn extraction_ratio(&self) -> f64 {
        if self.discovered_count == 0 {
            0.0
        } else {
            (self.extracted_count as f64 / self.discovered_count as f64).clamp(0.0, 1.0)
        }
    }
}

impl Default for CompletenessReport {
    fn default() -> Self {
        Self::single_item_success()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completeness_report_invariants() {
        let perfect = CompletenessReport::new(100, 80, 80, 15, 0, 5);
        assert!(perfect.formula_valid());
        assert!(perfect.is_complete());
        assert!((perfect.extraction_ratio() - 0.80).abs() < 1e-6);

        // Discovered count mismatch violates formula
        let invalid = CompletenessReport::new(100, 80, 79, 15, 0, 5);
        assert!(!invalid.formula_valid());
        assert!(!invalid.is_complete());

        // Failed count violates is_complete
        let failed = CompletenessReport::new(100, 80, 78, 15, 2, 5);
        assert!(failed.formula_valid());
        assert!(!failed.is_complete());
    }

    #[test]
    fn test_single_item_convenience_constructors() {
        let success = CompletenessReport::single_item_success();
        assert!(success.is_complete());
        assert_eq!(success.extraction_ratio(), 1.0);

        let excluded = CompletenessReport::single_item_excluded();
        assert!(excluded.formula_valid());
        assert!(excluded.is_complete()); // 0 eligible, 0 extracted, 1 excluded
        assert_eq!(excluded.extraction_ratio(), 0.0);

        let failed = CompletenessReport::single_item_failed();
        assert!(failed.formula_valid());
        assert!(!failed.is_complete());
    }
}
