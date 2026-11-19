//! Data Transfer Objects for API key generation, listing, and revocation.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;
use web_repo_to_ai_core::auth::{ApiKeyRecord, ApiKeyScope};
use web_repo_to_ai_core::{CoreError, Validatable};

/// Request payload for generating a new API key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    /// Human-readable descriptive name or label for the API key (1-100 characters).
    pub name: String,
    /// Permission scopes granted to the key (e.g. `["read", "write"]` or `["admin"]`).
    pub scopes: Vec<String>,
    /// Optional expiration in days from now (1 to 365 days). If omitted, key does not expire.
    pub expires_in_days: Option<u32>,
}

impl Validatable for CreateApiKeyRequest {
    fn validate(&self) -> Result<(), CoreError> {
        let trimmed_name = self.name.trim();
        if trimmed_name.is_empty() {
            return Err(CoreError::Validation {
                field: "name".to_string(),
                reason: "API key name cannot be empty".to_string(),
            });
        }
        if trimmed_name.len() > 100 {
            return Err(CoreError::Validation {
                field: "name".to_string(),
                reason: "API key name cannot exceed 100 characters".to_string(),
            });
        }

        if self.scopes.is_empty() {
            return Err(CoreError::Validation {
                field: "scopes".to_string(),
                reason: "At least one permission scope must be specified".to_string(),
            });
        }

        for scope_str in &self.scopes {
            scope_str.parse::<ApiKeyScope>()?;
        }

        if let Some(days) = self.expires_in_days {
            if days == 0 || days > 365 {
                return Err(CoreError::Validation {
                    field: "expires_in_days".to_string(),
                    reason: "Expiration must be between 1 and 365 days".to_string(),
                });
            }
        }

        Ok(())
    }
}

impl CreateApiKeyRequest {
    /// Parses the valid scopes into a set of [`ApiKeyScope`] enum variants.
    #[must_use]
    pub fn parsed_scopes(&self) -> HashSet<ApiKeyScope> {
        self.scopes
            .iter()
            .filter_map(|s| s.parse::<ApiKeyScope>().ok())
            .collect()
    }

    /// Computes the optional expiration timestamp from `expires_in_days`.
    #[must_use]
    pub fn compute_expires_at(&self) -> Option<DateTime<Utc>> {
        self.expires_in_days
            .map(|days| Utc::now() + Duration::days(i64::from(days)))
    }
}

/// Response returned when a new API key is created, containing the one-time plaintext key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyResponse {
    /// Unique identifier for the created API key.
    pub id: Uuid,
    /// Tenant identifier that owns the API key.
    pub tenant_id: Uuid,
    /// Descriptive name.
    pub name: String,
    /// Plaintext API key token (`wr_live_<64_hex_chars>`). Displayed only once upon creation.
    pub key: String,
    /// Safe public prefix for key identification.
    pub key_prefix: String,
    /// Granted permission scopes.
    pub scopes: Vec<String>,
    /// UTC timestamp of key creation.
    pub created_at: DateTime<Utc>,
    /// Optional UTC timestamp when the key will automatically expire.
    pub expires_at: Option<DateTime<Utc>>,
}

/// Summary metadata for an API key (never exposes the raw secret or hash).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeySummaryResponse {
    /// Unique identifier for the API key.
    pub id: Uuid,
    /// Tenant identifier that owns the API key.
    pub tenant_id: Uuid,
    /// Descriptive name.
    pub name: String,
    /// Safe public prefix for key identification.
    pub key_prefix: String,
    /// Granted permission scopes.
    pub scopes: Vec<String>,
    /// UTC timestamp of key creation.
    pub created_at: DateTime<Utc>,
    /// Optional UTC timestamp when the key will automatically expire.
    pub expires_at: Option<DateTime<Utc>>,
    /// Optional UTC timestamp when the key was last used in an API request.
    pub last_used_at: Option<DateTime<Utc>>,
    /// Whether the key is currently active (not revoked and not expired).
    pub is_active: bool,
}

impl From<ApiKeyRecord> for ApiKeySummaryResponse {
    fn from(record: ApiKeyRecord) -> Self {
        let is_active = record.is_currently_active();
        let mut scopes: Vec<String> = record.scopes.iter().map(ToString::to_string).collect();
        scopes.sort();

        Self {
            id: record.id,
            tenant_id: record.tenant_id,
            name: record.name,
            key_prefix: record.key_prefix,
            scopes,
            created_at: record.created_at,
            expires_at: record.expires_at,
            last_used_at: record.last_used_at,
            is_active,
        }
    }
}

/// Response containing a list of API key summaries for the caller's tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListApiKeysResponse {
    /// List of API key summary records.
    pub keys: Vec<ApiKeySummaryResponse>,
    /// Total count of keys returned.
    pub total: usize,
}

/// Response returned when an API key is revoked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeApiKeyResponse {
    /// Unique identifier of the revoked API key.
    pub id: Uuid,
    /// Whether the key was successfully revoked.
    pub revoked: bool,
    /// Informational message.
    pub message: String,
}

/// Path parameters for API key routes (`/auth/keys/:key_id`).
#[derive(Debug, Clone, Deserialize)]
pub struct ApiKeyPathParams {
    /// Target API key UUID.
    pub key_id: Uuid,
}

impl Validatable for ApiKeyPathParams {
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}
