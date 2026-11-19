//! # API Key Persistence Repository
//!
//! Provides database and in-memory abstractions for storing, querying, revoking,
//! and auditing API keys for developer and AI agent authentication under Stage 9.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use web_repo_to_ai_core::auth::{ApiKeyRecord, ApiKeyScope};
use web_repo_to_ai_core::error::CoreError;

/// Database row mapping entity for the `api_keys` table.
#[derive(Debug, Clone, FromRow)]
pub struct ApiKeyRow {
    /// Unique identifier of the key record.
    pub id: Uuid,
    /// Tenant or account UUID that owns this key.
    pub tenant_id: Uuid,
    /// Human-readable label.
    pub name: String,
    /// Display-safe prefix of the key (e.g. "wr_live_ab12cd...").
    pub key_prefix: String,
    /// BLAKE3 cryptographic hash of the raw plaintext API key token.
    pub key_hash: String,
    /// Set of authorized permissions granted to this key.
    pub scopes: Vec<String>,
    /// UTC timestamp of key creation.
    pub created_at: DateTime<Utc>,
    /// Optional UTC timestamp after which the key is expired.
    pub expires_at: Option<DateTime<Utc>>,
    /// Optional UTC timestamp of most recent authenticated invocation.
    pub last_used_at: Option<DateTime<Utc>>,
    /// Optional UTC timestamp when the key was revoked.
    pub revoked_at: Option<DateTime<Utc>>,
}

impl From<ApiKeyRow> for ApiKeyRecord {
    fn from(r: ApiKeyRow) -> Self {
        let mut scopes = HashSet::new();
        for s in r.scopes {
            if let Ok(scope) = ApiKeyScope::parse(&s) {
                scopes.insert(scope);
            }
        }
        Self {
            id: r.id,
            tenant_id: r.tenant_id,
            name: r.name,
            key_prefix: r.key_prefix,
            key_hash: r.key_hash,
            scopes,
            created_at: r.created_at,
            expires_at: r.expires_at,
            last_used_at: r.last_used_at,
            revoked_at: r.revoked_at,
        }
    }
}

/// Asynchronous trait defining persistent operations for API keys.
#[async_trait]
pub trait ApiKeyRepository: Send + Sync {
    /// Saves a new API key record to storage.
    ///
    /// # Errors
    /// Returns [`CoreError::Storage`] if a key with the same hash already exists or persistence fails.
    async fn save_key(&self, record: &ApiKeyRecord) -> Result<ApiKeyRecord, CoreError>;

    /// Finds an API key record by its BLAKE3 cryptographic hash.
    ///
    /// Returns `None` if no matching record is found.
    ///
    /// # Errors
    /// Returns [`CoreError::Storage`] if query execution fails.
    async fn find_by_hash(&self, key_hash: &str) -> Result<Option<ApiKeyRecord>, CoreError>;

    /// Finds an API key record by its unique UUID.
    ///
    /// # Errors
    /// Returns [`CoreError::Storage`] if query execution fails.
    async fn find_by_id(&self, key_id: Uuid) -> Result<Option<ApiKeyRecord>, CoreError>;

    /// Lists all active (non-revoked) API keys belonging to the given tenant.
    ///
    /// # Errors
    /// Returns [`CoreError::Storage`] if query execution fails.
    async fn list_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<ApiKeyRecord>, CoreError>;

    /// Revokes an API key belonging to the given tenant.
    ///
    /// Returns `true` if the key was found and revoked, or `false` if it did not exist or belonged to another tenant.
    ///
    /// # Errors
    /// Returns [`CoreError::Storage`] if update execution fails.
    async fn revoke_key(&self, key_id: Uuid, tenant_id: Uuid) -> Result<bool, CoreError>;

    /// Updates the `last_used_at` timestamp for the specified key to the current time.
    ///
    /// # Errors
    /// Returns [`CoreError::Storage`] if update execution fails.
    async fn record_usage(&self, key_id: Uuid) -> Result<(), CoreError>;
}

/// Thread-safe in-memory API key repository for testing, local CLI execution, and standalone setups.
#[derive(Debug, Clone, Default)]
pub struct InMemoryApiKeyRepository {
    keys: Arc<RwLock<HashMap<Uuid, ApiKeyRecord>>>,
    hash_index: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl InMemoryApiKeyRepository {
    /// Constructs a new empty in-memory API key repository.
    #[must_use]
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            hash_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ApiKeyRepository for InMemoryApiKeyRepository {
    async fn save_key(&self, record: &ApiKeyRecord) -> Result<ApiKeyRecord, CoreError> {
        let mut hash_index = self.hash_index.write().await;
        if hash_index.contains_key(&record.key_hash) {
            return Err(CoreError::Storage(format!(
                "API key with hash '{}' already exists",
                record.key_hash
            )));
        }

        let mut keys = self.keys.write().await;
        keys.insert(record.id, record.clone());
        hash_index.insert(record.key_hash.clone(), record.id);

        Ok(record.clone())
    }

    async fn find_by_hash(&self, key_hash: &str) -> Result<Option<ApiKeyRecord>, CoreError> {
        let hash_index = self.hash_index.read().await;
        let key_id = match hash_index.get(key_hash) {
            Some(id) => *id,
            None => return Ok(None),
        };

        let keys = self.keys.read().await;
        Ok(keys.get(&key_id).cloned())
    }

    async fn find_by_id(&self, key_id: Uuid) -> Result<Option<ApiKeyRecord>, CoreError> {
        let keys = self.keys.read().await;
        Ok(keys.get(&key_id).cloned())
    }

    async fn list_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<ApiKeyRecord>, CoreError> {
        let keys = self.keys.read().await;
        let now = Utc::now();
        let mut tenant_keys: Vec<ApiKeyRecord> = keys
            .values()
            .filter(|k| k.tenant_id == tenant_id && k.revoked_at.is_none() && k.is_active(now))
            .cloned()
            .collect();

        tenant_keys.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        Ok(tenant_keys)
    }

    async fn revoke_key(&self, key_id: Uuid, tenant_id: Uuid) -> Result<bool, CoreError> {
        let mut keys = self.keys.write().await;
        if let Some(record) = keys.get_mut(&key_id) {
            if record.tenant_id == tenant_id {
                record.revoked_at = Some(Utc::now());
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn record_usage(&self, key_id: Uuid) -> Result<(), CoreError> {
        let mut keys = self.keys.write().await;
        if let Some(record) = keys.get_mut(&key_id) {
            record.last_used_at = Some(Utc::now());
        }
        Ok(())
    }
}

/// PostgreSQL persistence adapter for API keys.
#[derive(Debug, Clone)]
pub struct PostgresApiKeyRepository {
    pool: sqlx::PgPool,
}

impl PostgresApiKeyRepository {
    /// Constructs a new PostgreSQL API key repository wrapping a connection pool.
    #[must_use]
    pub const fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ApiKeyRepository for PostgresApiKeyRepository {
    async fn save_key(&self, record: &ApiKeyRecord) -> Result<ApiKeyRecord, CoreError> {
        let scopes_vec: Vec<String> = record.scopes.iter().map(ToString::to_string).collect();

        sqlx::query(
            r#"
            INSERT INTO api_keys (
                id, tenant_id, name, key_prefix, key_hash, scopes,
                created_at, expires_at, last_used_at, revoked_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(record.id)
        .bind(record.tenant_id)
        .bind(&record.name)
        .bind(&record.key_prefix)
        .bind(&record.key_hash)
        .bind(&scopes_vec)
        .bind(record.created_at)
        .bind(record.expires_at)
        .bind(record.last_used_at)
        .bind(record.revoked_at)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(format!("Failed to insert API key record: {e}")))?;

        Ok(record.clone())
    }

    async fn find_by_hash(&self, key_hash: &str) -> Result<Option<ApiKeyRecord>, CoreError> {
        let row: Option<ApiKeyRow> = sqlx::query_as(
            r#"
            SELECT
                id, tenant_id, name, key_prefix, key_hash, scopes,
                created_at, expires_at, last_used_at, revoked_at
            FROM api_keys
            WHERE key_hash = $1
            "#,
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(format!("Failed to query API key by hash: {e}")))?;

        Ok(row.map(Into::into))
    }

    async fn find_by_id(&self, key_id: Uuid) -> Result<Option<ApiKeyRecord>, CoreError> {
        let row: Option<ApiKeyRow> = sqlx::query_as(
            r#"
            SELECT
                id, tenant_id, name, key_prefix, key_hash, scopes,
                created_at, expires_at, last_used_at, revoked_at
            FROM api_keys
            WHERE id = $1
            "#,
        )
        .bind(key_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(format!("Failed to query API key by id: {e}")))?;

        Ok(row.map(Into::into))
    }

    async fn list_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<ApiKeyRecord>, CoreError> {
        let now = Utc::now();
        let rows: Vec<ApiKeyRow> = sqlx::query_as(
            r#"
            SELECT
                id, tenant_id, name, key_prefix, key_hash, scopes,
                created_at, expires_at, last_used_at, revoked_at
            FROM api_keys
            WHERE tenant_id = $1
              AND revoked_at IS NULL
              AND (expires_at IS NULL OR expires_at > $2)
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id)
        .bind(now)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(format!("Failed to list API keys for tenant: {e}")))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn revoke_key(&self, key_id: Uuid, tenant_id: Uuid) -> Result<bool, CoreError> {
        let now = Utc::now();
        let result = sqlx::query(
            r#"
            UPDATE api_keys
            SET revoked_at = $1
            WHERE id = $2 AND tenant_id = $3 AND revoked_at IS NULL
            "#,
        )
        .bind(now)
        .bind(key_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(format!("Failed to revoke API key: {e}")))?;

        Ok(result.rows_affected() > 0)
    }

    async fn record_usage(&self, key_id: Uuid) -> Result<(), CoreError> {
        let now = Utc::now();
        let _ = sqlx::query(
            r#"
            UPDATE api_keys
            SET last_used_at = $1
            WHERE id = $2
            "#,
        )
        .bind(now)
        .bind(key_id)
        .execute(&self.pool)
        .await;

        Ok(())
    }
}
