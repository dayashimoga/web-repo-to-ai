//! Persistent rate limiting storage repositories for sliding window counters.
//!
//! Provides [`RateLimitStore`] trait, [`InMemoryRateLimitStore`], and [`PostgresRateLimitStore`].

use async_trait::async_trait;
use chrono::{Duration, Utc};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::RwLock;
use web_repo_to_ai_core::CoreError;

/// Asynchronous repository contract for persistent rate limiting counters.
#[async_trait]
pub trait RateLimitStore: Send + Sync {
    /// Increments the request counter for the specified `(key, window_bucket)` by `cost`
    /// and returns the resulting counter value.
    ///
    /// # Errors
    /// Returns [`CoreError::Storage`] if persistence fails.
    async fn increment_and_count(
        &self,
        key: &str,
        window_bucket: i64,
        cost: u32,
        ttl_seconds: u64,
    ) -> Result<u32, CoreError>;

    /// Reads the current counter value without incrementing.
    ///
    /// # Errors
    /// Returns [`CoreError::Storage`] if retrieval fails.
    async fn get_count(&self, key: &str, window_bucket: i64) -> Result<u32, CoreError>;

    /// Prunes expired rate limit buckets. Returns the count of removed records.
    ///
    /// # Errors
    /// Returns [`CoreError::Storage`] if deletion fails.
    async fn prune_expired(&self) -> Result<u64, CoreError>;
}

/// Compound key tuple `(client_key, window_bucket)` for rate limit tracking.
pub type RateLimitKey = (String, i64);
/// Value entry representing `(count, expires_at)` in the in-memory rate limit store.
pub type RateLimitEntry = (u32, chrono::DateTime<Utc>);
/// Internal mapping table from compound keys to rate limit entries.
type RateLimitMap = HashMap<RateLimitKey, RateLimitEntry>;

/// Thread-safe in-memory rate limiting store for unit testing and ephemeral nodes.
#[derive(Debug, Default)]
pub struct InMemoryRateLimitStore {
    counters: RwLock<RateLimitMap>,
}

impl InMemoryRateLimitStore {
    /// Creates a new empty in-memory store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl RateLimitStore for InMemoryRateLimitStore {
    async fn increment_and_count(
        &self,
        key: &str,
        window_bucket: i64,
        cost: u32,
        ttl_seconds: u64,
    ) -> Result<u32, CoreError> {
        let expires_at = Utc::now() + Duration::seconds(ttl_seconds as i64);
        let mut map = self
            .counters
            .write()
            .map_err(|e| CoreError::Storage(format!("Lock poisoned: {e}")))?;

        let entry = map
            .entry((key.to_string(), window_bucket))
            .or_insert((0, expires_at));
        entry.0 = entry.0.saturating_add(cost);
        entry.1 = expires_at;
        Ok(entry.0)
    }

    async fn get_count(&self, key: &str, window_bucket: i64) -> Result<u32, CoreError> {
        let map = self
            .counters
            .read()
            .map_err(|e| CoreError::Storage(format!("Lock poisoned: {e}")))?;
        Ok(map
            .get(&(key.to_string(), window_bucket))
            .map(|(count, _)| *count)
            .unwrap_or(0))
    }

    async fn prune_expired(&self) -> Result<u64, CoreError> {
        let now = Utc::now();
        let mut map = self
            .counters
            .write()
            .map_err(|e| CoreError::Storage(format!("Lock poisoned: {e}")))?;
        let before_len = map.len();
        map.retain(|_, (_, expires_at)| *expires_at > now);
        let pruned = before_len.saturating_sub(map.len());
        Ok(pruned as u64)
    }
}

/// Production PostgreSQL-backed rate limiting repository.
#[derive(Debug, Clone)]
pub struct PostgresRateLimitStore {
    pool: PgPool,
}

impl PostgresRateLimitStore {
    /// Creates a new repository bound to the given connection pool.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RateLimitStore for PostgresRateLimitStore {
    async fn increment_and_count(
        &self,
        key: &str,
        window_bucket: i64,
        cost: u32,
        ttl_seconds: u64,
    ) -> Result<u32, CoreError> {
        let expires_at = Utc::now() + Duration::seconds(ttl_seconds as i64);
        let row = sqlx::query(
            r#"
            INSERT INTO rate_limit_counters (client_key, window_bucket, request_count, expires_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            ON CONFLICT (client_key, window_bucket)
            DO UPDATE SET request_count = rate_limit_counters.request_count + EXCLUDED.request_count,
                          updated_at = NOW()
            RETURNING request_count
            "#,
        )
        .bind(key)
        .bind(window_bucket)
        .bind(cost as i32)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(format!("Failed to increment rate limit counter: {e}")))?;

        let count: i32 = row
            .try_get("request_count")
            .map_err(|e| CoreError::Storage(format!("Failed to parse request_count: {e}")))?;

        Ok(count.max(0) as u32)
    }

    async fn get_count(&self, key: &str, window_bucket: i64) -> Result<u32, CoreError> {
        let row_opt = sqlx::query(
            r#"
            SELECT request_count FROM rate_limit_counters
            WHERE client_key = $1 AND window_bucket = $2
            "#,
        )
        .bind(key)
        .bind(window_bucket)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(format!("Failed to get rate limit counter: {e}")))?;

        match row_opt {
            Some(row) => {
                let count: i32 = row.try_get("request_count").map_err(|e| {
                    CoreError::Storage(format!("Failed to read request_count: {e}"))
                })?;
                Ok(count.max(0) as u32)
            }
            None => Ok(0),
        }
    }

    async fn prune_expired(&self) -> Result<u64, CoreError> {
        let now = Utc::now();
        let result = sqlx::query(
            r#"
            DELETE FROM rate_limit_counters
            WHERE expires_at <= $1
            "#,
        )
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            CoreError::Storage(format!("Failed to prune expired rate limit counters: {e}"))
        })?;

        Ok(result.rows_affected())
    }
}
