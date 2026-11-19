//! # Database Migrations Infrastructure
//!
//! Provides embedded SQLx migrations, programmatic schema application, and
//! migration execution reports for PostgreSQL.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use web_repo_to_ai_core::error::CoreError;

/// Embedded SQLx migrator loading migration files at compile time.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Structured summary report of a migration execution run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationReport {
    /// Number of migrations applied in this execution run.
    pub applied_count: usize,
    /// Latest active migration version after execution.
    pub current_version: Option<i64>,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
    /// UTC timestamp of migration completion.
    pub timestamp: DateTime<Utc>,
}

impl MigrationReport {
    /// Constructs a successful migration execution report.
    #[must_use]
    pub fn new(applied_count: usize, current_version: Option<i64>, duration_ms: u64) -> Self {
        Self {
            applied_count,
            current_version,
            duration_ms,
            timestamp: Utc::now(),
        }
    }

    /// Formats a human-readable summary of the migration execution.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Applied {} migration(s) in {}ms. Current version: {}",
            self.applied_count,
            self.duration_ms,
            self.current_version
                .map_or_else(|| "none".to_string(), |v| v.to_string())
        )
    }
}

/// Applies all pending database migrations against the provided PostgreSQL pool.
///
/// # Errors
///
/// Returns [`CoreError::Storage`] if applying migrations fails due to network or SQL errors.
pub async fn run_migrations(pool: &sqlx::PgPool) -> Result<MigrationReport, CoreError> {
    let start = Instant::now();
    tracing::info!("Starting database migration run...");

    MIGRATOR
        .run(pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Database migration failed: {err}")))?;

    let elapsed = start.elapsed().as_millis() as u64;
    let latest_version = MIGRATOR.migrations.iter().map(|m| m.version).max();
    let total_migrations = MIGRATOR.migrations.len();

    let report = MigrationReport::new(total_migrations, latest_version, elapsed);
    tracing::info!("{}", report.summary());
    Ok(report)
}

/// Returns a reference to all embedded migrations compiled into this binary.
#[must_use]
pub fn embedded_migrations() -> &'static [sqlx::migrate::Migration] {
    &MIGRATOR.migrations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_migrations_loaded() {
        let migrations = embedded_migrations();
        assert!(
            !migrations.is_empty(),
            "Embedded migrations list must not be empty"
        );

        let initial_migration = migrations
            .iter()
            .find(|m| m.version == 20261117000001)
            .expect("Migration 20261117000001 must be present");

        assert_eq!(initial_migration.description, "initial schema");
        assert!(
            initial_migration
                .sql
                .contains("CREATE TABLE IF NOT EXISTS jobs"),
            "Initial migration must contain jobs table definition"
        );
        assert!(
            initial_migration
                .sql
                .contains("CREATE TABLE IF NOT EXISTS job_artifacts"),
            "Initial migration must contain job_artifacts table definition"
        );
        assert!(
            initial_migration
                .sql
                .contains("CREATE TABLE IF NOT EXISTS github_installations"),
            "Initial migration must contain github_installations table definition"
        );
        assert!(
            initial_migration
                .sql
                .contains("CREATE TABLE IF NOT EXISTS content_cache_index"),
            "Initial migration must contain content_cache_index table definition"
        );

        let batch_migration = migrations
            .iter()
            .find(|m| m.version == 20261119000001)
            .expect("Migration 20261119000001 must be present");

        assert_eq!(batch_migration.description, "batch jobs and items");
        assert!(
            batch_migration
                .sql
                .contains("CREATE TABLE IF NOT EXISTS batch_jobs"),
            "Batch migration must contain batch_jobs table definition"
        );
        assert!(
            batch_migration
                .sql
                .contains("CREATE TABLE IF NOT EXISTS batch_items"),
            "Batch migration must contain batch_items table definition"
        );
    }

    #[test]
    fn test_migration_report_formatting() {
        let report = MigrationReport::new(1, Some(20261117000001), 42);
        assert_eq!(report.applied_count, 1);
        assert_eq!(report.current_version, Some(20261117000001));
        assert_eq!(report.duration_ms, 42);
        assert!(report.summary().contains("Applied 1 migration(s) in 42ms"));
        assert!(report.summary().contains("20261117000001"));

        let empty_report = MigrationReport::new(0, None, 5);
        assert!(empty_report.summary().contains("Current version: none"));
    }
}
