//! # HTTP Validator and Content Cache Repository
//!
//! Provides durable persistence for URL-level HTTP caching validators (`ETag`, `Last-Modified`,
//! `Cache-Control`, `Expires`) and content cache index mapping with in-memory and PostgreSQL implementations.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use web_repo_to_ai_core::error::CoreError;

use crate::schema::cache::ContentCacheIndexRow;
use crate::schema::http_cache::HttpValidatorRow;

/// Asynchronous repository abstraction for persisting and querying HTTP validators and content cache index records.
#[async_trait]
pub trait HttpCacheRepository: Send + Sync {
    /// Retrieves the HTTP validator record for a given target URL if cached.
    async fn get_validator(&self, url: &str) -> Result<Option<HttpValidatorRow>, CoreError>;

    /// Stores or updates an HTTP validator record.
    async fn upsert_validator(&self, row: &HttpValidatorRow) -> Result<(), CoreError>;

    /// Updates `last_checked_at` and refreshes metadata for an existing cached URL on 304 Not Modified.
    async fn touch_validator(
        &self,
        url: &str,
        updated_etag: Option<&str>,
        updated_last_modified: Option<&str>,
        updated_expires_at: Option<DateTime<Utc>>,
    ) -> Result<bool, CoreError>;

    /// Removes an HTTP validator record by URL.
    async fn delete_validator(&self, url: &str) -> Result<bool, CoreError>;

    /// Prunes stale HTTP validator entries that have not been accessed or checked since `cutoff`.
    async fn prune_stale_validators(&self, cutoff: DateTime<Utc>) -> Result<u64, CoreError>;

    /// Retrieves a content cache index entry by cache key (e.g. BLAKE3 digest or URL validator key).
    async fn get_content_cache(
        &self,
        cache_key: &str,
    ) -> Result<Option<ContentCacheIndexRow>, CoreError>;

    /// Stores or updates a content cache index record.
    async fn upsert_content_cache(&self, row: &ContentCacheIndexRow) -> Result<(), CoreError>;
}

/// In-memory thread-safe implementation of [`HttpCacheRepository`].
///
/// Ideal for testing, CLI environments, and offline single-node conversion.
#[derive(Debug, Clone, Default)]
pub struct InMemoryHttpCacheRepository {
    validators: Arc<RwLock<HashMap<String, HttpValidatorRow>>>,
    content_cache: Arc<RwLock<HashMap<String, ContentCacheIndexRow>>>,
}

impl InMemoryHttpCacheRepository {
    /// Constructs a new empty `InMemoryHttpCacheRepository`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl HttpCacheRepository for InMemoryHttpCacheRepository {
    async fn get_validator(&self, url: &str) -> Result<Option<HttpValidatorRow>, CoreError> {
        let map = self.validators.read().await;
        Ok(map.get(url).cloned())
    }

    async fn upsert_validator(&self, row: &HttpValidatorRow) -> Result<(), CoreError> {
        row.validate()?;
        let mut map = self.validators.write().await;
        map.insert(row.url.clone(), row.clone());
        Ok(())
    }

    async fn touch_validator(
        &self,
        url: &str,
        updated_etag: Option<&str>,
        updated_last_modified: Option<&str>,
        updated_expires_at: Option<DateTime<Utc>>,
    ) -> Result<bool, CoreError> {
        let mut map = self.validators.write().await;
        if let Some(entry) = map.get_mut(url) {
            entry.touch(updated_etag, updated_last_modified, updated_expires_at);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn delete_validator(&self, url: &str) -> Result<bool, CoreError> {
        let mut map = self.validators.write().await;
        Ok(map.remove(url).is_some())
    }

    async fn prune_stale_validators(&self, cutoff: DateTime<Utc>) -> Result<u64, CoreError> {
        let mut map = self.validators.write().await;
        let before_count = map.len();
        map.retain(|_, row| row.last_checked_at >= cutoff);
        let removed = before_count.saturating_sub(map.len());
        Ok(removed as u64)
    }

    async fn get_content_cache(
        &self,
        cache_key: &str,
    ) -> Result<Option<ContentCacheIndexRow>, CoreError> {
        let map = self.content_cache.read().await;
        Ok(map.get(cache_key).cloned())
    }

    async fn upsert_content_cache(&self, row: &ContentCacheIndexRow) -> Result<(), CoreError> {
        row.validate()?;
        let mut map = self.content_cache.write().await;
        map.insert(row.cache_key.clone(), row.clone());
        Ok(())
    }
}

/// Production PostgreSQL implementation of [`HttpCacheRepository`] backed by SQLx.
#[derive(Debug, Clone)]
pub struct PostgresHttpCacheRepository {
    pool: PgPool,
}

impl PostgresHttpCacheRepository {
    /// Constructs a repository backed by a PostgreSQL connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the underlying connection pool.
    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl HttpCacheRepository for PostgresHttpCacheRepository {
    async fn get_validator(&self, url: &str) -> Result<Option<HttpValidatorRow>, CoreError> {
        sqlx::query_as::<_, HttpValidatorRow>("SELECT * FROM http_validators WHERE url = $1")
            .bind(url)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| {
                CoreError::Storage(format!("Failed to query http_validators by url: {err}"))
            })
    }

    async fn upsert_validator(&self, row: &HttpValidatorRow) -> Result<(), CoreError> {
        row.validate()?;
        sqlx::query(
            r#"
            INSERT INTO http_validators (
                url, canonical_url, etag, last_modified, last_modified_timestamp,
                content_type, blake3_hex, storage_key, byte_size, cache_control,
                expires_at, created_at, last_checked_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (url) DO UPDATE SET
                canonical_url = EXCLUDED.canonical_url,
                etag = EXCLUDED.etag,
                last_modified = EXCLUDED.last_modified,
                last_modified_timestamp = EXCLUDED.last_modified_timestamp,
                content_type = EXCLUDED.content_type,
                blake3_hex = EXCLUDED.blake3_hex,
                storage_key = EXCLUDED.storage_key,
                byte_size = EXCLUDED.byte_size,
                cache_control = EXCLUDED.cache_control,
                expires_at = EXCLUDED.expires_at,
                last_checked_at = EXCLUDED.last_checked_at
            "#,
        )
        .bind(&row.url)
        .bind(&row.canonical_url)
        .bind(&row.etag)
        .bind(&row.last_modified)
        .bind(row.last_modified_timestamp)
        .bind(&row.content_type)
        .bind(&row.blake3_hex)
        .bind(&row.storage_key)
        .bind(row.byte_size)
        .bind(&row.cache_control)
        .bind(row.expires_at)
        .bind(row.created_at)
        .bind(row.last_checked_at)
        .execute(&self.pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to upsert http_validators: {err}")))?;

        Ok(())
    }

    async fn touch_validator(
        &self,
        url: &str,
        updated_etag: Option<&str>,
        updated_last_modified: Option<&str>,
        updated_expires_at: Option<DateTime<Utc>>,
    ) -> Result<bool, CoreError> {
        let result = sqlx::query(
            r#"
            UPDATE http_validators SET
                last_checked_at = NOW(),
                etag = COALESCE($2, etag),
                last_modified = COALESCE($3, last_modified),
                expires_at = COALESCE($4, expires_at)
            WHERE url = $1
            "#,
        )
        .bind(url)
        .bind(updated_etag)
        .bind(updated_last_modified)
        .bind(updated_expires_at)
        .execute(&self.pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to touch http_validator: {err}")))?;

        Ok(result.rows_affected() > 0)
    }

    async fn delete_validator(&self, url: &str) -> Result<bool, CoreError> {
        let result = sqlx::query("DELETE FROM http_validators WHERE url = $1")
            .bind(url)
            .execute(&self.pool)
            .await
            .map_err(|err| CoreError::Storage(format!("Failed to delete http_validator: {err}")))?;

        Ok(result.rows_affected() > 0)
    }

    async fn prune_stale_validators(&self, cutoff: DateTime<Utc>) -> Result<u64, CoreError> {
        let result = sqlx::query("DELETE FROM http_validators WHERE last_checked_at < $1")
            .bind(cutoff)
            .execute(&self.pool)
            .await
            .map_err(|err| {
                CoreError::Storage(format!("Failed to prune stale http_validators: {err}"))
            })?;

        Ok(result.rows_affected())
    }

    async fn get_content_cache(
        &self,
        cache_key: &str,
    ) -> Result<Option<ContentCacheIndexRow>, CoreError> {
        sqlx::query_as::<_, ContentCacheIndexRow>(
            "SELECT * FROM content_cache_index WHERE cache_key = $1",
        )
        .bind(cache_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to query content_cache_index: {err}")))
    }

    async fn upsert_content_cache(&self, row: &ContentCacheIndexRow) -> Result<(), CoreError> {
        row.validate()?;
        sqlx::query(
            r#"
            INSERT INTO content_cache_index (
                cache_key, blake3_hex, storage_key, byte_size, last_accessed_at
            ) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (cache_key) DO UPDATE SET
                blake3_hex = EXCLUDED.blake3_hex,
                storage_key = EXCLUDED.storage_key,
                byte_size = EXCLUDED.byte_size,
                last_accessed_at = EXCLUDED.last_accessed_at
            "#,
        )
        .bind(&row.cache_key)
        .bind(&row.blake3_hex)
        .bind(&row.storage_key)
        .bind(row.byte_size)
        .bind(row.last_accessed_at)
        .execute(&self.pool)
        .await
        .map_err(|err| {
            CoreError::Storage(format!("Failed to upsert content_cache_index: {err}"))
        })?;

        Ok(())
    }
}
