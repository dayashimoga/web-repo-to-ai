//! # Secure Token Storage Repository
//!
//! Provides authenticated envelope-encryption storage abstractions for GitHub installation tokens
//! and OAuth credentials under Rule 05.
//!
//! ## Rule 05 & Architectural Invariants
//!
//! 1. **Envelope Encryption at Rest:** Every credential is encrypted using AES-256-GCM via
//!    [`EncryptionKey`].
//! 2. **Tenant Isolation:** Encryption cryptographically binds `owner_session_id` and
//!    `github_installation_id` as Authenticated Associated Data (AAD).
//! 3. **Secret Redaction:** Plaintext tokens are zero-logged and never retained after decryption.
//! 4. **Revocation Hygiene:** Revoked records immediately nullify encrypted token and nonce bytes.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use web_repo_to_ai_core::crypto::{decrypt_token, encrypt_token, EncryptionKey};
use web_repo_to_ai_core::error::CoreError;

use crate::schema::github::GitHubInstallationRow;

/// Trait defining secure storage and lifecycle operations for GitHub installation access tokens.
#[async_trait]
pub trait TokenStorageRepository: Send + Sync {
    /// Stores or updates an installation access token under authenticated encryption.
    ///
    /// Cryptographically binds the ciphertext to `(owner_session_id, github_installation_id)`.
    ///
    /// # Errors
    /// Returns [`CoreError`] if encryption or persistence fails.
    async fn save_installation_token(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
        account_login: &str,
        raw_token: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<GitHubInstallationRow, CoreError>;

    /// Retrieves and decrypts the active installation access token.
    ///
    /// Returns `None` if the grant does not exist, has been revoked, or the token has expired.
    ///
    /// # Errors
    /// Returns [`CoreError`] if decryption integrity or cryptographic binding verification fails.
    async fn get_active_token(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
    ) -> Result<Option<String>, CoreError>;

    /// Retrieves and decrypts the active installation access token if it will not expire within `safety_margin_secs`.
    ///
    /// Returns `None` if the token does not exist, is revoked, is expired, or will expire within `safety_margin_secs`.
    ///
    /// # Errors
    /// Returns [`CoreError`] if decryption integrity or cryptographic binding verification fails.
    async fn get_active_token_with_margin(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
        safety_margin_secs: u64,
    ) -> Result<Option<String>, CoreError>;

    /// Retrieves the expiration timestamp of an installation token without decrypting it.
    ///
    /// # Errors
    /// Returns [`CoreError`] if query execution fails.
    async fn get_token_expiration(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
    ) -> Result<Option<DateTime<Utc>>, CoreError>;

    /// Retrieves the installation record metadata without decrypting the token.
    ///
    /// # Errors
    /// Returns [`CoreError`] if query execution fails.
    async fn get_installation(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
    ) -> Result<Option<GitHubInstallationRow>, CoreError>;

    /// Lists all active (non-revoked and unexpired) installations for a given owner session.
    ///
    /// # Errors
    /// Returns [`CoreError`] if query execution fails.
    async fn list_active_installations(
        &self,
        owner_session_id: Uuid,
    ) -> Result<Vec<GitHubInstallationRow>, CoreError>;

    /// Revokes an installation grant, immediately zeroing and nullifying the stored ciphertext.
    ///
    /// Returns `true` if an active grant was found and revoked, `false` otherwise.
    ///
    /// # Errors
    /// Returns [`CoreError`] on database error.
    async fn revoke_installation(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
    ) -> Result<bool, CoreError>;

    /// Permanently deletes an installation record from storage.
    ///
    /// Returns `true` if a record was removed, `false` if it did not exist.
    ///
    /// # Errors
    /// Returns [`CoreError`] on database error.
    async fn delete_installation(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
    ) -> Result<bool, CoreError>;

    /// Revokes and zeros out encrypted tokens for all records matching a given GitHub installation ID across all tenant sessions.
    ///
    /// Primarily used for GitHub webhook `installation.deleted` and `installation.suspend` events.
    /// Returns the count of installations revoked.
    ///
    /// # Errors
    /// Returns [`CoreError`] on database error.
    async fn revoke_all_by_installation_id(
        &self,
        github_installation_id: i64,
    ) -> Result<u64, CoreError>;
}

/// Thread-safe in-memory implementation of [`TokenStorageRepository`].
///
/// Suitable for unit tests, local test rigs, and standalone in-process runs.
#[derive(Clone)]
pub struct InMemoryTokenStorage {
    key: EncryptionKey,
    records: Arc<RwLock<HashMap<(Uuid, i64), GitHubInstallationRow>>>,
}

impl InMemoryTokenStorage {
    /// Creates a new `InMemoryTokenStorage` with the given encryption key.
    #[must_use]
    pub fn new(key: EncryptionKey) -> Self {
        Self {
            key,
            records: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl TokenStorageRepository for InMemoryTokenStorage {
    async fn save_installation_token(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
        account_login: &str,
        raw_token: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<GitHubInstallationRow, CoreError> {
        let (ciphertext, nonce) = encrypt_token(
            &self.key,
            raw_token,
            &owner_session_id,
            github_installation_id,
        )?;

        let mut lock = self.records.write().await;
        let key = (owner_session_id, github_installation_id);

        let row = match lock.get_mut(&key) {
            Some(existing) => {
                existing.account_login = account_login.to_string();
                existing.encrypted_token = Some(ciphertext);
                existing.token_nonce = Some(nonce);
                existing.token_expires_at = Some(expires_at);
                existing.revoked_at = None;
                existing.updated_at = Utc::now();
                existing.clone()
            }
            None => {
                let now = Utc::now();
                let row = GitHubInstallationRow {
                    id: Uuid::now_v7(),
                    owner_session_id,
                    github_installation_id,
                    account_login: account_login.to_string(),
                    encrypted_token: Some(ciphertext),
                    token_nonce: Some(nonce),
                    token_expires_at: Some(expires_at),
                    revoked_at: None,
                    created_at: now,
                    updated_at: now,
                };
                lock.insert(key, row.clone());
                row
            }
        };

        row.validate()?;
        Ok(row)
    }

    async fn get_active_token(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
    ) -> Result<Option<String>, CoreError> {
        let lock = self.records.read().await;
        let key = (owner_session_id, github_installation_id);

        let row = match lock.get(&key) {
            Some(r) => r,
            None => return Ok(None),
        };

        if !row.is_active() {
            return Ok(None);
        }

        let (ciphertext, nonce) = match (&row.encrypted_token, &row.token_nonce) {
            (Some(c), Some(n)) => (c, n),
            _ => return Ok(None),
        };

        let token = decrypt_token(
            &self.key,
            ciphertext,
            nonce,
            &owner_session_id,
            github_installation_id,
        )?;

        Ok(Some(token))
    }

    async fn get_active_token_with_margin(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
        safety_margin_secs: u64,
    ) -> Result<Option<String>, CoreError> {
        let lock = self.records.read().await;
        let key = (owner_session_id, github_installation_id);

        let row = match lock.get(&key) {
            Some(r) => r,
            None => return Ok(None),
        };

        let margin = chrono::Duration::seconds(safety_margin_secs as i64);
        if row.is_expiring_soon(margin) {
            return Ok(None);
        }

        let (ciphertext, nonce) = match (&row.encrypted_token, &row.token_nonce) {
            (Some(c), Some(n)) => (c, n),
            _ => return Ok(None),
        };

        let token = decrypt_token(
            &self.key,
            ciphertext,
            nonce,
            &owner_session_id,
            github_installation_id,
        )?;

        Ok(Some(token))
    }

    async fn get_token_expiration(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
    ) -> Result<Option<DateTime<Utc>>, CoreError> {
        let lock = self.records.read().await;
        let key = (owner_session_id, github_installation_id);
        Ok(lock.get(&key).and_then(|r| {
            if r.revoked_at.is_some() {
                None
            } else {
                r.token_expires_at
            }
        }))
    }

    async fn get_installation(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
    ) -> Result<Option<GitHubInstallationRow>, CoreError> {
        let lock = self.records.read().await;
        Ok(lock
            .get(&(owner_session_id, github_installation_id))
            .cloned())
    }

    async fn list_active_installations(
        &self,
        owner_session_id: Uuid,
    ) -> Result<Vec<GitHubInstallationRow>, CoreError> {
        let lock = self.records.read().await;
        let mut list: Vec<GitHubInstallationRow> = lock
            .values()
            .filter(|r| r.owner_session_id == owner_session_id && r.is_active())
            .cloned()
            .collect();
        list.sort_by(|a, b| a.account_login.cmp(&b.account_login));
        Ok(list)
    }

    async fn revoke_installation(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
    ) -> Result<bool, CoreError> {
        let mut lock = self.records.write().await;
        let key = (owner_session_id, github_installation_id);

        if let Some(row) = lock.get_mut(&key) {
            if row.revoked_at.is_none() {
                row.revoke();
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn delete_installation(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
    ) -> Result<bool, CoreError> {
        let mut lock = self.records.write().await;
        let key = (owner_session_id, github_installation_id);
        Ok(lock.remove(&key).is_some())
    }

    async fn revoke_all_by_installation_id(
        &self,
        github_installation_id: i64,
    ) -> Result<u64, CoreError> {
        let mut lock = self.records.write().await;
        let mut count = 0u64;
        for ((_, inst_id), row) in lock.iter_mut() {
            if *inst_id == github_installation_id && row.revoked_at.is_none() {
                row.revoke();
                count += 1;
            }
        }
        Ok(count)
    }
}

/// Production PostgreSQL-backed implementation of [`TokenStorageRepository`].
#[derive(Clone)]
pub struct PostgresTokenStorageRepository {
    pool: sqlx::PgPool,
    key: EncryptionKey,
}

impl PostgresTokenStorageRepository {
    /// Creates a new `PostgresTokenStorageRepository` with the given pool and key.
    #[must_use]
    pub const fn new(pool: sqlx::PgPool, key: EncryptionKey) -> Self {
        Self { pool, key }
    }
}

#[async_trait]
impl TokenStorageRepository for PostgresTokenStorageRepository {
    async fn save_installation_token(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
        account_login: &str,
        raw_token: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<GitHubInstallationRow, CoreError> {
        let (ciphertext, nonce) = encrypt_token(
            &self.key,
            raw_token,
            &owner_session_id,
            github_installation_id,
        )?;

        let new_id = Uuid::now_v7();
        let now = Utc::now();

        let row: GitHubInstallationRow = sqlx::query_as(
            r#"
            INSERT INTO github_installations (
                id,
                owner_session_id,
                github_installation_id,
                account_login,
                encrypted_token,
                token_nonce,
                token_expires_at,
                revoked_at,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, NULL, $8, $8)
            ON CONFLICT (owner_session_id, github_installation_id)
            DO UPDATE SET
                account_login = EXCLUDED.account_login,
                encrypted_token = EXCLUDED.encrypted_token,
                token_nonce = EXCLUDED.token_nonce,
                token_expires_at = EXCLUDED.token_expires_at,
                revoked_at = NULL,
                updated_at = EXCLUDED.updated_at
            RETURNING
                id,
                owner_session_id,
                github_installation_id,
                account_login,
                encrypted_token,
                token_nonce,
                token_expires_at,
                revoked_at,
                created_at,
                updated_at
            "#,
        )
        .bind(new_id)
        .bind(owner_session_id)
        .bind(github_installation_id)
        .bind(account_login)
        .bind(ciphertext)
        .bind(nonce)
        .bind(expires_at)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(format!("Failed to save installation token: {e}")))?;

        Ok(row)
    }

    async fn get_active_token(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
    ) -> Result<Option<String>, CoreError> {
        let row: Option<GitHubInstallationRow> = sqlx::query_as(
            r#"
            SELECT
                id,
                owner_session_id,
                github_installation_id,
                account_login,
                encrypted_token,
                token_nonce,
                token_expires_at,
                revoked_at,
                created_at,
                updated_at
            FROM github_installations
            WHERE owner_session_id = $1
              AND github_installation_id = $2
              AND revoked_at IS NULL
              AND (token_expires_at IS NULL OR token_expires_at > NOW())
            "#,
        )
        .bind(owner_session_id)
        .bind(github_installation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(format!("Failed to fetch active installation: {e}")))?;

        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };

        let (ciphertext, nonce) = match (&row.encrypted_token, &row.token_nonce) {
            (Some(c), Some(n)) => (c, n),
            _ => return Ok(None),
        };

        let token = decrypt_token(
            &self.key,
            ciphertext,
            nonce,
            &owner_session_id,
            github_installation_id,
        )?;

        Ok(Some(token))
    }

    async fn get_active_token_with_margin(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
        safety_margin_secs: u64,
    ) -> Result<Option<String>, CoreError> {
        let margin_secs = safety_margin_secs as f64;
        let row: Option<GitHubInstallationRow> = sqlx::query_as(
            r#"
            SELECT
                id,
                owner_session_id,
                github_installation_id,
                account_login,
                encrypted_token,
                token_nonce,
                token_expires_at,
                revoked_at,
                created_at,
                updated_at
            FROM github_installations
            WHERE owner_session_id = $1
              AND github_installation_id = $2
              AND revoked_at IS NULL
              AND (token_expires_at IS NULL OR token_expires_at > NOW() + ($3 * INTERVAL '1 second'))
            "#,
        )
        .bind(owner_session_id)
        .bind(github_installation_id)
        .bind(margin_secs)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(format!("Failed to fetch active installation with margin: {e}")))?;

        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };

        let (ciphertext, nonce) = match (&row.encrypted_token, &row.token_nonce) {
            (Some(c), Some(n)) => (c, n),
            _ => return Ok(None),
        };

        let token = decrypt_token(
            &self.key,
            ciphertext,
            nonce,
            &owner_session_id,
            github_installation_id,
        )?;

        Ok(Some(token))
    }

    async fn get_token_expiration(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
    ) -> Result<Option<DateTime<Utc>>, CoreError> {
        let row: Option<(Option<DateTime<Utc>>,)> = sqlx::query_as(
            r#"
            SELECT token_expires_at
            FROM github_installations
            WHERE owner_session_id = $1
              AND github_installation_id = $2
              AND revoked_at IS NULL
            "#,
        )
        .bind(owner_session_id)
        .bind(github_installation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(format!("Failed to fetch token expiration: {e}")))?;

        Ok(row.and_then(|(exp,)| exp))
    }

    async fn get_installation(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
    ) -> Result<Option<GitHubInstallationRow>, CoreError> {
        let row: Option<GitHubInstallationRow> = sqlx::query_as(
            r#"
            SELECT
                id,
                owner_session_id,
                github_installation_id,
                account_login,
                encrypted_token,
                token_nonce,
                token_expires_at,
                revoked_at,
                created_at,
                updated_at
            FROM github_installations
            WHERE owner_session_id = $1
              AND github_installation_id = $2
            "#,
        )
        .bind(owner_session_id)
        .bind(github_installation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(format!("Failed to fetch installation metadata: {e}")))?;

        Ok(row)
    }

    async fn list_active_installations(
        &self,
        owner_session_id: Uuid,
    ) -> Result<Vec<GitHubInstallationRow>, CoreError> {
        let rows: Vec<GitHubInstallationRow> = sqlx::query_as(
            r#"
            SELECT
                id,
                owner_session_id,
                github_installation_id,
                account_login,
                encrypted_token,
                token_nonce,
                token_expires_at,
                revoked_at,
                created_at,
                updated_at
            FROM github_installations
            WHERE owner_session_id = $1
              AND revoked_at IS NULL
              AND (token_expires_at IS NULL OR token_expires_at > NOW())
            ORDER BY account_login ASC
            "#,
        )
        .bind(owner_session_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(format!("Failed to list active installations: {e}")))?;

        Ok(rows)
    }

    async fn revoke_installation(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
    ) -> Result<bool, CoreError> {
        let result = sqlx::query(
            r#"
            UPDATE github_installations
            SET
                revoked_at = NOW(),
                encrypted_token = NULL,
                token_nonce = NULL,
                token_expires_at = NULL,
                updated_at = NOW()
            WHERE owner_session_id = $1
              AND github_installation_id = $2
              AND revoked_at IS NULL
            "#,
        )
        .bind(owner_session_id)
        .bind(github_installation_id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(format!("Failed to revoke installation: {e}")))?;

        Ok(result.rows_affected() > 0)
    }

    async fn delete_installation(
        &self,
        owner_session_id: Uuid,
        github_installation_id: i64,
    ) -> Result<bool, CoreError> {
        let result = sqlx::query(
            r#"
            DELETE FROM github_installations
            WHERE owner_session_id = $1
              AND github_installation_id = $2
            "#,
        )
        .bind(owner_session_id)
        .bind(github_installation_id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(format!("Failed to delete installation: {e}")))?;

        Ok(result.rows_affected() > 0)
    }

    async fn revoke_all_by_installation_id(
        &self,
        github_installation_id: i64,
    ) -> Result<u64, CoreError> {
        let result = sqlx::query(
            r#"
            UPDATE github_installations
            SET
                revoked_at = NOW(),
                encrypted_token = NULL,
                token_nonce = NULL,
                token_expires_at = NULL,
                updated_at = NOW()
            WHERE github_installation_id = $1
              AND revoked_at IS NULL
            "#,
        )
        .bind(github_installation_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            CoreError::Storage(format!(
                "Failed to revoke installations for {github_installation_id}: {e}"
            ))
        })?;

        Ok(result.rows_affected())
    }
}
