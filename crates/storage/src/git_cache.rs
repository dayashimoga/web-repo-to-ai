//! # Git Cache Repository Subsystem
//!
//! Provides transactional repository abstractions and storage implementations for Git commit
//! metadata and content-addressed Git blob caching:
//! - [`GitCacheRepository`] trait defining CRUD, batch lookup, and pruning operations.
//! - [`InMemoryGitCacheRepository`] thread-safe concurrent in-memory store for unit tests and local runs.
//! - [`PostgresGitCacheRepository`] production SQLx-backed PostgreSQL persistent store.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use web_repo_to_ai_core::error::CoreError;

use crate::schema::git_cache::{GitBlobCacheRow, GitCommitCacheRow};

/// Asynchronous repository abstraction for managing cached Git commits and Git blobs.
#[async_trait]
pub trait GitCacheRepository: Send + Sync {
    /// Retrieves a commit cache row by repository owner, name, and commit SHA.
    async fn get_commit(
        &self,
        owner: &str,
        repo: &str,
        commit_sha: &str,
    ) -> Result<Option<GitCommitCacheRow>, CoreError>;

    /// Stores or updates a Git commit cache record.
    async fn upsert_commit(&self, row: &GitCommitCacheRow) -> Result<(), CoreError>;

    /// Touches a commit cache row, updating its `last_accessed_at` timestamp.
    async fn touch_commit(
        &self,
        owner: &str,
        repo: &str,
        commit_sha: &str,
    ) -> Result<bool, CoreError>;

    /// Deletes a cached commit entry.
    async fn delete_commit(
        &self,
        owner: &str,
        repo: &str,
        commit_sha: &str,
    ) -> Result<bool, CoreError>;

    /// Retrieves a single Git blob cache row by Git SHA.
    async fn get_blob(&self, blob_sha: &str) -> Result<Option<GitBlobCacheRow>, CoreError>;

    /// Retrieves a batch of Git blob cache rows by their Git SHAs.
    async fn get_blobs_batch(
        &self,
        blob_shas: &[String],
    ) -> Result<HashMap<String, GitBlobCacheRow>, CoreError>;

    /// Stores or updates a Git blob cache record.
    async fn upsert_blob(&self, row: &GitBlobCacheRow) -> Result<(), CoreError>;

    /// Stores or updates a batch of Git blob cache records.
    async fn upsert_blobs_batch(&self, rows: &[GitBlobCacheRow]) -> Result<(), CoreError>;

    /// Touches a blob cache row, updating its `last_accessed_at` timestamp.
    async fn touch_blob(&self, blob_sha: &str) -> Result<bool, CoreError>;

    /// Deletes a cached blob entry.
    async fn delete_blob(&self, blob_sha: &str) -> Result<bool, CoreError>;

    /// Prunes stale commits whose `last_accessed_at` is older than `before`.
    async fn prune_stale_commits(&self, before: DateTime<Utc>) -> Result<u64, CoreError>;

    /// Prunes stale blobs whose `last_accessed_at` is older than `before`.
    async fn prune_stale_blobs(&self, before: DateTime<Utc>) -> Result<u64, CoreError>;
}

/// Cache key for commit lookup: `(owner, repo, commit_sha)`.
pub type CommitCacheKey = (String, String, String);

type CommitCacheMap = Arc<RwLock<HashMap<CommitCacheKey, GitCommitCacheRow>>>;
type BlobCacheMap = Arc<RwLock<HashMap<String, GitBlobCacheRow>>>;

/// In-memory concurrent implementation of [`GitCacheRepository`] for testing and standalone operation.
#[derive(Debug, Clone, Default)]
pub struct InMemoryGitCacheRepository {
    commits: CommitCacheMap,
    blobs: BlobCacheMap,
}

impl InMemoryGitCacheRepository {
    /// Creates a new, empty `InMemoryGitCacheRepository`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            commits: Arc::new(RwLock::new(HashMap::new())),
            blobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn commit_key(owner: &str, repo: &str, commit_sha: &str) -> (String, String, String) {
        (
            owner.to_lowercase(),
            repo.to_lowercase(),
            commit_sha.to_lowercase(),
        )
    }

    fn blob_key(blob_sha: &str) -> String {
        blob_sha.to_lowercase()
    }
}

#[async_trait]
impl GitCacheRepository for InMemoryGitCacheRepository {
    async fn get_commit(
        &self,
        owner: &str,
        repo: &str,
        commit_sha: &str,
    ) -> Result<Option<GitCommitCacheRow>, CoreError> {
        let key = Self::commit_key(owner, repo, commit_sha);
        let map = self.commits.read().await;
        Ok(map.get(&key).cloned())
    }

    async fn upsert_commit(&self, row: &GitCommitCacheRow) -> Result<(), CoreError> {
        row.validate()?;
        let key = Self::commit_key(&row.owner, &row.repo, &row.commit_sha);
        let mut map = self.commits.write().await;
        map.insert(key, row.clone());
        Ok(())
    }

    async fn touch_commit(
        &self,
        owner: &str,
        repo: &str,
        commit_sha: &str,
    ) -> Result<bool, CoreError> {
        let key = Self::commit_key(owner, repo, commit_sha);
        let mut map = self.commits.write().await;
        if let Some(entry) = map.get_mut(&key) {
            entry.last_accessed_at = Utc::now();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn delete_commit(
        &self,
        owner: &str,
        repo: &str,
        commit_sha: &str,
    ) -> Result<bool, CoreError> {
        let key = Self::commit_key(owner, repo, commit_sha);
        let mut map = self.commits.write().await;
        Ok(map.remove(&key).is_some())
    }

    async fn get_blob(&self, blob_sha: &str) -> Result<Option<GitBlobCacheRow>, CoreError> {
        let key = Self::blob_key(blob_sha);
        let map = self.blobs.read().await;
        Ok(map.get(&key).cloned())
    }

    async fn get_blobs_batch(
        &self,
        blob_shas: &[String],
    ) -> Result<HashMap<String, GitBlobCacheRow>, CoreError> {
        let map = self.blobs.read().await;
        let mut result = HashMap::with_capacity(blob_shas.len());
        for sha in blob_shas {
            let key = Self::blob_key(sha);
            if let Some(row) = map.get(&key) {
                result.insert(sha.clone(), row.clone());
            }
        }
        Ok(result)
    }

    async fn upsert_blob(&self, row: &GitBlobCacheRow) -> Result<(), CoreError> {
        row.validate()?;
        let key = Self::blob_key(&row.blob_sha);
        let mut map = self.blobs.write().await;
        map.insert(key, row.clone());
        Ok(())
    }

    async fn upsert_blobs_batch(&self, rows: &[GitBlobCacheRow]) -> Result<(), CoreError> {
        for row in rows {
            row.validate()?;
        }
        let mut map = self.blobs.write().await;
        for row in rows {
            let key = Self::blob_key(&row.blob_sha);
            map.insert(key, row.clone());
        }
        Ok(())
    }

    async fn touch_blob(&self, blob_sha: &str) -> Result<bool, CoreError> {
        let key = Self::blob_key(blob_sha);
        let mut map = self.blobs.write().await;
        if let Some(entry) = map.get_mut(&key) {
            entry.last_accessed_at = Utc::now();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn delete_blob(&self, blob_sha: &str) -> Result<bool, CoreError> {
        let key = Self::blob_key(blob_sha);
        let mut map = self.blobs.write().await;
        Ok(map.remove(&key).is_some())
    }

    async fn prune_stale_commits(&self, before: DateTime<Utc>) -> Result<u64, CoreError> {
        let mut map = self.commits.write().await;
        let initial = map.len();
        map.retain(|_, v| v.last_accessed_at >= before);
        Ok((initial - map.len()) as u64)
    }

    async fn prune_stale_blobs(&self, before: DateTime<Utc>) -> Result<u64, CoreError> {
        let mut map = self.blobs.write().await;
        let initial = map.len();
        map.retain(|_, v| v.last_accessed_at >= before);
        Ok((initial - map.len()) as u64)
    }
}

/// Production PostgreSQL implementation of [`GitCacheRepository`] using SQLx.
#[derive(Debug, Clone)]
pub struct PostgresGitCacheRepository {
    pool: sqlx::PgPool,
}

impl PostgresGitCacheRepository {
    /// Constructs a new `PostgresGitCacheRepository` backed by a PostgreSQL connection pool.
    #[must_use]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GitCacheRepository for PostgresGitCacheRepository {
    async fn get_commit(
        &self,
        owner: &str,
        repo: &str,
        commit_sha: &str,
    ) -> Result<Option<GitCommitCacheRow>, CoreError> {
        sqlx::query_as::<_, GitCommitCacheRow>(
            r#"
            SELECT * FROM git_commit_cache
            WHERE LOWER(owner) = LOWER($1) AND LOWER(repo) = LOWER($2) AND LOWER(commit_sha) = LOWER($3)
            "#,
        )
        .bind(owner)
        .bind(repo)
        .bind(commit_sha)
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to query git_commit_cache: {err}")))
    }

    async fn upsert_commit(&self, row: &GitCommitCacheRow) -> Result<(), CoreError> {
        row.validate()?;
        sqlx::query(
            r#"
            INSERT INTO git_commit_cache (
                owner, repo, commit_sha, tree_sha, tree_entries_json, total_files, total_bytes, created_at, last_accessed_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (owner, repo, commit_sha) DO UPDATE SET
                tree_sha = EXCLUDED.tree_sha,
                tree_entries_json = COALESCE(EXCLUDED.tree_entries_json, git_commit_cache.tree_entries_json),
                total_files = EXCLUDED.total_files,
                total_bytes = EXCLUDED.total_bytes,
                last_accessed_at = EXCLUDED.last_accessed_at
            "#,
        )
        .bind(&row.owner)
        .bind(&row.repo)
        .bind(&row.commit_sha)
        .bind(&row.tree_sha)
        .bind(&row.tree_entries_json)
        .bind(row.total_files)
        .bind(row.total_bytes)
        .bind(row.created_at)
        .bind(row.last_accessed_at)
        .execute(&self.pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to upsert git_commit_cache: {err}")))?;

        Ok(())
    }

    async fn touch_commit(
        &self,
        owner: &str,
        repo: &str,
        commit_sha: &str,
    ) -> Result<bool, CoreError> {
        let rows = sqlx::query(
            r#"
            UPDATE git_commit_cache
            SET last_accessed_at = NOW()
            WHERE LOWER(owner) = LOWER($1) AND LOWER(repo) = LOWER($2) AND LOWER(commit_sha) = LOWER($3)
            "#,
        )
        .bind(owner)
        .bind(repo)
        .bind(commit_sha)
        .execute(&self.pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to touch git_commit_cache: {err}")))?
        .rows_affected();

        Ok(rows > 0)
    }

    async fn delete_commit(
        &self,
        owner: &str,
        repo: &str,
        commit_sha: &str,
    ) -> Result<bool, CoreError> {
        let rows = sqlx::query(
            r#"
            DELETE FROM git_commit_cache
            WHERE LOWER(owner) = LOWER($1) AND LOWER(repo) = LOWER($2) AND LOWER(commit_sha) = LOWER($3)
            "#,
        )
        .bind(owner)
        .bind(repo)
        .bind(commit_sha)
        .execute(&self.pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to delete git_commit_cache: {err}")))?
        .rows_affected();

        Ok(rows > 0)
    }

    async fn get_blob(&self, blob_sha: &str) -> Result<Option<GitBlobCacheRow>, CoreError> {
        sqlx::query_as::<_, GitBlobCacheRow>(
            r#"
            SELECT * FROM git_blob_cache
            WHERE LOWER(blob_sha) = LOWER($1)
            "#,
        )
        .bind(blob_sha)
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to query git_blob_cache: {err}")))
    }

    async fn get_blobs_batch(
        &self,
        blob_shas: &[String],
    ) -> Result<HashMap<String, GitBlobCacheRow>, CoreError> {
        if blob_shas.is_empty() {
            return Ok(HashMap::new());
        }

        let lower_shas: Vec<String> = blob_shas.iter().map(|s| s.to_lowercase()).collect();
        let rows = sqlx::query_as::<_, GitBlobCacheRow>(
            r#"
            SELECT * FROM git_blob_cache
            WHERE LOWER(blob_sha) = ANY($1)
            "#,
        )
        .bind(&lower_shas)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| {
            CoreError::Storage(format!("Failed to batch query git_blob_cache: {err}"))
        })?;

        let mut map = HashMap::with_capacity(rows.len());
        for row in rows {
            map.insert(row.blob_sha.clone(), row);
        }
        Ok(map)
    }

    async fn upsert_blob(&self, row: &GitBlobCacheRow) -> Result<(), CoreError> {
        row.validate()?;
        sqlx::query(
            r#"
            INSERT INTO git_blob_cache (
                blob_sha, blake3_hex, storage_key, byte_size, is_binary, created_at, last_accessed_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (blob_sha) DO UPDATE SET
                blake3_hex = EXCLUDED.blake3_hex,
                storage_key = EXCLUDED.storage_key,
                byte_size = EXCLUDED.byte_size,
                is_binary = EXCLUDED.is_binary,
                last_accessed_at = EXCLUDED.last_accessed_at
            "#,
        )
        .bind(&row.blob_sha)
        .bind(&row.blake3_hex)
        .bind(&row.storage_key)
        .bind(row.byte_size)
        .bind(row.is_binary)
        .bind(row.created_at)
        .bind(row.last_accessed_at)
        .execute(&self.pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to upsert git_blob_cache: {err}")))?;

        Ok(())
    }

    async fn upsert_blobs_batch(&self, rows: &[GitBlobCacheRow]) -> Result<(), CoreError> {
        for row in rows {
            row.validate()?;
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| CoreError::Storage(format!("Failed to begin transaction: {err}")))?;

        for row in rows {
            sqlx::query(
                r#"
                INSERT INTO git_blob_cache (
                    blob_sha, blake3_hex, storage_key, byte_size, is_binary, created_at, last_accessed_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (blob_sha) DO UPDATE SET
                    blake3_hex = EXCLUDED.blake3_hex,
                    storage_key = EXCLUDED.storage_key,
                    byte_size = EXCLUDED.byte_size,
                    is_binary = EXCLUDED.is_binary,
                    last_accessed_at = EXCLUDED.last_accessed_at
                "#,
            )
            .bind(&row.blob_sha)
            .bind(&row.blake3_hex)
            .bind(&row.storage_key)
            .bind(row.byte_size)
            .bind(row.is_binary)
            .bind(row.created_at)
            .bind(row.last_accessed_at)
            .execute(&mut *tx)
            .await
            .map_err(|err| CoreError::Storage(format!("Failed to batch upsert git_blob_cache: {err}")))?;
        }

        tx.commit()
            .await
            .map_err(|err| CoreError::Storage(format!("Failed to commit transaction: {err}")))?;

        Ok(())
    }

    async fn touch_blob(&self, blob_sha: &str) -> Result<bool, CoreError> {
        let rows = sqlx::query(
            r#"
            UPDATE git_blob_cache
            SET last_accessed_at = NOW()
            WHERE LOWER(blob_sha) = LOWER($1)
            "#,
        )
        .bind(blob_sha)
        .execute(&self.pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to touch git_blob_cache: {err}")))?
        .rows_affected();

        Ok(rows > 0)
    }

    async fn delete_blob(&self, blob_sha: &str) -> Result<bool, CoreError> {
        let rows = sqlx::query(
            r#"
            DELETE FROM git_blob_cache
            WHERE LOWER(blob_sha) = LOWER($1)
            "#,
        )
        .bind(blob_sha)
        .execute(&self.pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to delete git_blob_cache: {err}")))?
        .rows_affected();

        Ok(rows > 0)
    }

    async fn prune_stale_commits(&self, before: DateTime<Utc>) -> Result<u64, CoreError> {
        let rows = sqlx::query(
            r#"
            DELETE FROM git_commit_cache
            WHERE last_accessed_at < $1
            "#,
        )
        .bind(before)
        .execute(&self.pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to prune git_commit_cache: {err}")))?
        .rows_affected();

        Ok(rows)
    }

    async fn prune_stale_blobs(&self, before: DateTime<Utc>) -> Result<u64, CoreError> {
        let rows = sqlx::query(
            r#"
            DELETE FROM git_blob_cache
            WHERE last_accessed_at < $1
            "#,
        )
        .bind(before)
        .execute(&self.pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to prune git_blob_cache: {err}")))?
        .rows_affected();

        Ok(rows)
    }
}
