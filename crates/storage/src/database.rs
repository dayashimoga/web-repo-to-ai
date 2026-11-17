//! # Database Connection & Lifecycle Manager
//!
//! Provides [`DatabaseManager`] managing connection pools, migration execution,
//! schema validation, and health probing against PostgreSQL.

use std::time::Duration;
use web_repo_to_ai_core::config::DatabaseConfig;
use web_repo_to_ai_core::error::CoreError;

use crate::migrations::{run_migrations, MigrationReport};
use crate::validation::{validate_schema, SchemaValidationReport};

/// Central manager for PostgreSQL connection pooling and schema operations.
#[derive(Debug, Clone)]
pub struct DatabaseManager {
    pool: sqlx::PgPool,
}

impl DatabaseManager {
    /// Connects to a PostgreSQL database using a connection URL with default pool settings.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if establishing the connection pool fails.
    pub async fn connect(database_url: &str) -> Result<Self, CoreError> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(10))
            .connect(database_url)
            .await
            .map_err(|err| {
                CoreError::Storage(format!(
                    "Failed to connect to PostgreSQL at '{database_url}': {err}"
                ))
            })?;

        Ok(Self { pool })
    }

    /// Connects to a PostgreSQL database using structured [`DatabaseConfig`].
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if pool configuration or connection fails.
    pub async fn connect_with_config(config: &DatabaseConfig) -> Result<Self, CoreError> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.connect_timeout_secs))
            .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
            .connect(&config.url)
            .await
            .map_err(|err| {
                CoreError::Storage(format!(
                    "Failed to connect to PostgreSQL using DatabaseConfig: {err}"
                ))
            })?;

        Ok(Self { pool })
    }

    /// Wraps an existing [`sqlx::PgPool`] into a [`DatabaseManager`].
    #[must_use]
    pub fn from_pool(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the underlying [`sqlx::PgPool`].
    #[must_use]
    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }

    /// Executes all pending database migrations against this connection pool.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if migration execution fails.
    pub async fn migrate(&self) -> Result<MigrationReport, CoreError> {
        run_migrations(&self.pool).await
    }

    /// Validates that the active database schema conforms to all tables, columns, indexes, and triggers.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if schema introspection fails.
    pub async fn validate_schema(&self) -> Result<SchemaValidationReport, CoreError> {
        validate_schema(&self.pool).await
    }

    /// Executes a low-overhead liveness probe query (`SELECT 1`).
    pub async fn is_healthy(&self) -> bool {
        sqlx::query("SELECT 1").execute(&self.pool).await.is_ok()
    }

    /// Gracefully closes all connections in the connection pool.
    pub async fn close(&self) {
        self.pool.close().await;
    }
}
