//! Query parameter models and paginated response envelopes.

use serde::{Deserialize, Serialize};
use web_repo_to_ai_core::{CoreError, Result, Validatable};

/// Permitted sort orders.
pub const VALID_SORT_ORDERS: &[&str] = &["asc", "desc"];

/// Permitted job filter statuses.
pub const VALID_STATUS_FILTERS: &[&str] =
    &["queued", "processing", "completed", "failed", "cancelled"];

/// Query parameters for listing conversion jobs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ListJobsQuery {
    /// 1-based page number (defaults to 1).
    pub page: Option<usize>,
    /// Number of items per page (1 to 100, defaults to 20).
    pub limit: Option<usize>,
    /// Optional status filter.
    pub status: Option<String>,
    /// Sort direction (`asc` or `desc`, defaults to `desc`).
    pub sort_order: Option<String>,
}

impl Validatable for ListJobsQuery {
    fn validate(&self) -> Result<()> {
        if let Some(page) = self.page {
            if page == 0 {
                return Err(CoreError::Validation {
                    field: "page".to_string(),
                    reason: "Page number must be greater than or equal to 1".to_string(),
                });
            }
        }

        if let Some(limit) = self.limit {
            if !(1..=100).contains(&limit) {
                return Err(CoreError::Validation {
                    field: "limit".to_string(),
                    reason: format!("Page limit must be between 1 and 100 (received {limit})"),
                });
            }
        }

        if let Some(ref status) = self.status {
            let lower = status.trim().to_ascii_lowercase();
            if !VALID_STATUS_FILTERS.contains(&lower.as_str()) {
                return Err(CoreError::Validation {
                    field: "status".to_string(),
                    reason: format!(
                        "Invalid status filter '{status}': must be one of {:?}",
                        VALID_STATUS_FILTERS
                    ),
                });
            }
        }

        if let Some(ref order) = self.sort_order {
            let lower = order.trim().to_ascii_lowercase();
            if !VALID_SORT_ORDERS.contains(&lower.as_str()) {
                return Err(CoreError::Validation {
                    field: "sort_order".to_string(),
                    reason: format!("Invalid sort order '{order}': must be 'asc' or 'desc'"),
                });
            }
        }

        Ok(())
    }
}

impl ListJobsQuery {
    /// Returns the normalized 1-based page number.
    #[must_use]
    pub fn page(&self) -> usize {
        self.page.unwrap_or(1).max(1)
    }

    /// Returns the normalized page limit (default: 20).
    #[must_use]
    pub fn limit(&self) -> usize {
        self.limit.unwrap_or(20).clamp(1, 100)
    }

    /// Returns true if sorting in ascending order.
    #[must_use]
    pub fn is_ascending(&self) -> bool {
        self.sort_order
            .as_deref()
            .map(|s| s.eq_ignore_ascii_case("asc"))
            .unwrap_or(false)
    }
}

/// Generic envelope for paginated resource collections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// Array of items in the current page slice.
    pub items: Vec<T>,
    /// Current 1-based page number.
    pub page: usize,
    /// Maximum items per page requested.
    pub limit: usize,
    /// Total count of matching resources across all pages.
    pub total: usize,
    /// Whether additional pages are available.
    pub has_more: bool,
}

impl<T> PaginatedResponse<T> {
    /// Wraps items in a paginated envelope computing `has_more`.
    #[must_use]
    pub fn new(items: Vec<T>, page: usize, limit: usize, total: usize) -> Self {
        let has_more = (page * limit) < total;
        Self {
            items,
            page,
            limit,
            total,
            has_more,
        }
    }
}
