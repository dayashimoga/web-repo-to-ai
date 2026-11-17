//! # Database Schema Validation
//!
//! Provides runtime schema introspection and catalog verification to guarantee that
//! live PostgreSQL databases conform exactly to the Canonical Data Model in `docs/DATA_MODEL.md`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use web_repo_to_ai_core::error::CoreError;

use crate::schema::{EXPECTED_INDEXES, EXPECTED_TABLES};

/// Expected database triggers defined in PostgreSQL schema.
pub const EXPECTED_TRIGGERS: &[&str] =
    &["trg_jobs_updated_at", "trg_github_installations_updated_at"];

/// Returns the expected columns for each core table.
#[must_use]
pub fn expected_table_columns() -> HashMap<&'static str, &'static [&'static str]> {
    let mut map = HashMap::new();
    map.insert(
        "jobs",
        &[
            "id",
            "kind",
            "status",
            "source_url",
            "config_json",
            "progress_phase",
            "progress_pct",
            "leased_by",
            "lease_expires_at",
            "attempt_count",
            "error_code",
            "error_message",
            "owner_session_id",
            "created_at",
            "updated_at",
            "completed_at",
        ][..],
    );
    map.insert(
        "job_artifacts",
        &[
            "id",
            "job_id",
            "format",
            "storage_key",
            "byte_size",
            "sha256_hex",
            "blake3_hex",
            "expires_at",
            "created_at",
        ][..],
    );
    map.insert(
        "github_installations",
        &[
            "id",
            "owner_session_id",
            "github_installation_id",
            "account_login",
            "encrypted_token",
            "token_nonce",
            "token_expires_at",
            "revoked_at",
            "created_at",
            "updated_at",
        ][..],
    );
    map.insert(
        "content_cache_index",
        &[
            "cache_key",
            "blake3_hex",
            "storage_key",
            "byte_size",
            "last_accessed_at",
        ][..],
    );
    map.insert(
        "job_state_transitions",
        &[
            "id",
            "job_id",
            "from_status",
            "to_status",
            "phase",
            "progress_pct",
            "message",
            "error_code",
            "worker_id",
            "transitioned_at",
        ][..],
    );
    map
}

/// Detailed audit report of runtime database schema inspection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaValidationReport {
    /// True if all expected tables, columns, indexes, and triggers are present.
    pub is_valid: bool,
    /// Successfully verified table names.
    pub verified_tables: Vec<String>,
    /// Expected tables missing from the database.
    pub missing_tables: Vec<String>,
    /// Successfully verified column names (`table.column`).
    pub verified_columns: Vec<String>,
    /// Expected columns missing from the database (`table.column`).
    pub missing_columns: Vec<String>,
    /// Successfully verified index names.
    pub verified_indexes: Vec<String>,
    /// Expected indexes missing from the database.
    pub missing_indexes: Vec<String>,
    /// Successfully verified trigger names.
    pub verified_triggers: Vec<String>,
    /// Expected triggers missing from the database.
    pub missing_triggers: Vec<String>,
    /// UTC timestamp of validation audit.
    pub validation_timestamp: DateTime<Utc>,
}

impl SchemaValidationReport {
    /// Formats a concise human-readable summary of validation results.
    #[must_use]
    pub fn summary(&self) -> String {
        if self.is_valid {
            format!(
                "Schema valid: {} tables, {} columns, {} indexes, {} triggers verified.",
                self.verified_tables.len(),
                self.verified_columns.len(),
                self.verified_indexes.len(),
                self.verified_triggers.len()
            )
        } else {
            let mut issues = Vec::new();
            if !self.missing_tables.is_empty() {
                issues.push(format!("missing tables: {:?}", self.missing_tables));
            }
            if !self.missing_columns.is_empty() {
                issues.push(format!("missing columns: {:?}", self.missing_columns));
            }
            if !self.missing_indexes.is_empty() {
                issues.push(format!("missing indexes: {:?}", self.missing_indexes));
            }
            if !self.missing_triggers.is_empty() {
                issues.push(format!("missing triggers: {:?}", self.missing_triggers));
            }
            format!("Schema INVALID: {}", issues.join("; "))
        }
    }
}

/// Evaluates catalog metadata against required schema invariants.
#[must_use]
pub fn validate_catalog_metadata(
    discovered_tables: &[String],
    discovered_columns: &[(&str, &str)],
    discovered_indexes: &[String],
    discovered_triggers: &[String],
) -> SchemaValidationReport {
    let table_set: HashSet<&str> = discovered_tables.iter().map(String::as_str).collect();
    let index_set: HashSet<&str> = discovered_indexes.iter().map(String::as_str).collect();
    let trigger_set: HashSet<&str> = discovered_triggers.iter().map(String::as_str).collect();

    let mut col_set: HashSet<(String, String)> = HashSet::new();
    for (t, c) in discovered_columns {
        col_set.insert(((*t).to_string(), (*c).to_string()));
    }

    let mut verified_tables = Vec::new();
    let mut missing_tables = Vec::new();
    for &expected in EXPECTED_TABLES {
        if table_set.contains(expected) {
            verified_tables.push(expected.to_string());
        } else {
            missing_tables.push(expected.to_string());
        }
    }

    let expected_cols = expected_table_columns();
    let mut verified_columns = Vec::new();
    let mut missing_columns = Vec::new();
    for (&tbl, cols) in &expected_cols {
        for &col in *cols {
            let key = format!("{tbl}.{col}");
            if col_set.contains(&(tbl.to_string(), col.to_string())) {
                verified_columns.push(key);
            } else {
                missing_columns.push(key);
            }
        }
    }

    let mut verified_indexes = Vec::new();
    let mut missing_indexes = Vec::new();
    for &expected in EXPECTED_INDEXES {
        if index_set.contains(expected) {
            verified_indexes.push(expected.to_string());
        } else {
            missing_indexes.push(expected.to_string());
        }
    }

    let mut verified_triggers = Vec::new();
    let mut missing_triggers = Vec::new();
    for &expected in EXPECTED_TRIGGERS {
        if trigger_set.contains(expected) {
            verified_triggers.push(expected.to_string());
        } else {
            missing_triggers.push(expected.to_string());
        }
    }

    let is_valid = missing_tables.is_empty()
        && missing_columns.is_empty()
        && missing_indexes.is_empty()
        && missing_triggers.is_empty();

    SchemaValidationReport {
        is_valid,
        verified_tables,
        missing_tables,
        verified_columns,
        missing_columns,
        verified_indexes,
        missing_indexes,
        verified_triggers,
        missing_triggers,
        validation_timestamp: Utc::now(),
    }
}

/// Connects to a live PostgreSQL database pool, queries system catalogs,
/// and validates complete structural compliance with schema specifications.
///
/// # Errors
///
/// Returns [`CoreError::Storage`] if catalog queries fail.
pub async fn validate_schema(pool: &sqlx::PgPool) -> Result<SchemaValidationReport, CoreError> {
    // 1. Query base tables
    let tables: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| CoreError::Storage(format!("Failed to query tables catalog: {e}")))?;

    let discovered_tables: Vec<String> = tables.into_iter().map(|(t,)| t).collect();

    // 2. Query columns
    let columns: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT table_name, column_name
        FROM information_schema.columns
        WHERE table_schema = 'public'
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| CoreError::Storage(format!("Failed to query columns catalog: {e}")))?;

    let discovered_columns: Vec<(&str, &str)> = columns
        .iter()
        .map(|(t, c)| (t.as_str(), c.as_str()))
        .collect();

    // 3. Query indexes
    let indexes: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT indexname
        FROM pg_indexes
        WHERE schemaname = 'public'
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| CoreError::Storage(format!("Failed to query indexes catalog: {e}")))?;

    let discovered_indexes: Vec<String> = indexes.into_iter().map(|(i,)| i).collect();

    // 4. Query triggers
    let triggers: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT trigger_name
        FROM information_schema.triggers
        WHERE trigger_schema = 'public'
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| CoreError::Storage(format!("Failed to query triggers catalog: {e}")))?;

    let discovered_triggers: Vec<String> = triggers.into_iter().map(|(tr,)| tr).collect();

    let report = validate_catalog_metadata(
        &discovered_tables,
        &discovered_columns,
        &discovered_indexes,
        &discovered_triggers,
    );

    if report.is_valid {
        tracing::info!("{}", report.summary());
    } else {
        tracing::warn!("{}", report.summary());
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCatalog {
        tables: Vec<String>,
        columns: Vec<(&'static str, &'static str)>,
        indexes: Vec<String>,
        triggers: Vec<String>,
    }

    fn generate_mock_valid_catalog() -> MockCatalog {
        let tables: Vec<String> = EXPECTED_TABLES.iter().map(|s| (*s).to_string()).collect();

        let mut columns: Vec<(&'static str, &'static str)> = Vec::new();
        let expected_cols = expected_table_columns();
        for (&tbl, cols) in &expected_cols {
            for &col in *cols {
                columns.push((tbl, col));
            }
        }

        let indexes: Vec<String> = EXPECTED_INDEXES.iter().map(|s| (*s).to_string()).collect();
        let triggers: Vec<String> = EXPECTED_TRIGGERS.iter().map(|s| (*s).to_string()).collect();

        MockCatalog {
            tables,
            columns,
            indexes,
            triggers,
        }
    }

    #[test]
    fn test_validation_report_full_compliance() {
        let cat = generate_mock_valid_catalog();
        let report =
            validate_catalog_metadata(&cat.tables, &cat.columns, &cat.indexes, &cat.triggers);

        assert!(report.is_valid);
        assert!(report.missing_tables.is_empty());
        assert!(report.missing_columns.is_empty());
        assert!(report.missing_indexes.is_empty());
        assert!(report.missing_triggers.is_empty());
        assert_eq!(report.verified_tables.len(), EXPECTED_TABLES.len());
        assert_eq!(report.verified_indexes.len(), EXPECTED_INDEXES.len());
        assert_eq!(report.verified_triggers.len(), EXPECTED_TRIGGERS.len());
        assert!(report.summary().contains("Schema valid:"));
    }

    #[test]
    fn test_validation_detects_missing_table() {
        let mut cat = generate_mock_valid_catalog();
        cat.tables.retain(|t| t != "job_artifacts");

        let report =
            validate_catalog_metadata(&cat.tables, &cat.columns, &cat.indexes, &cat.triggers);

        assert!(!report.is_valid);
        assert_eq!(report.missing_tables, vec!["job_artifacts".to_string()]);
        assert!(report.summary().contains("missing tables"));
        assert!(report.summary().contains("job_artifacts"));
    }

    #[test]
    fn test_validation_detects_missing_column() {
        let mut cat = generate_mock_valid_catalog();
        cat.columns
            .retain(|&(t, c)| !(t == "jobs" && c == "leased_by"));

        let report =
            validate_catalog_metadata(&cat.tables, &cat.columns, &cat.indexes, &cat.triggers);

        assert!(!report.is_valid);
        assert_eq!(report.missing_columns, vec!["jobs.leased_by".to_string()]);
        assert!(report.summary().contains("missing columns"));
    }

    #[test]
    fn test_validation_detects_missing_index() {
        let mut cat = generate_mock_valid_catalog();
        cat.indexes.retain(|i| i != "idx_jobs_status_lease");

        let report =
            validate_catalog_metadata(&cat.tables, &cat.columns, &cat.indexes, &cat.triggers);

        assert!(!report.is_valid);
        assert_eq!(
            report.missing_indexes,
            vec!["idx_jobs_status_lease".to_string()]
        );
        assert!(report.summary().contains("missing indexes"));
    }

    #[test]
    fn test_validation_detects_missing_trigger() {
        let mut cat = generate_mock_valid_catalog();
        cat.triggers.retain(|t| t != "trg_jobs_updated_at");

        let report =
            validate_catalog_metadata(&cat.tables, &cat.columns, &cat.indexes, &cat.triggers);

        assert!(!report.is_valid);
        assert_eq!(
            report.missing_triggers,
            vec!["trg_jobs_updated_at".to_string()]
        );
        assert!(report.summary().contains("missing triggers"));
    }
}
