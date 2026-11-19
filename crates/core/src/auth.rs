//! Authentication & authorization models, API key lifecycle, and caller principal abstractions.
//!
//! Provides [`ApiKeyScope`], [`ApiKeyRecord`], [`GeneratedApiKey`], and [`AuthenticatedCaller`].

use chrono::{DateTime, Utc};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use uuid::Uuid;

use crate::error::CoreError;

/// Prefix assigned to all live generated API keys.
pub const API_KEY_PREFIX: &str = "wr_live_";

/// Granular capability scopes for API keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyScope {
    /// Read access: view jobs, list items, download artifacts, query preview, and inspect sitemaps.
    Read,
    /// Write access: create single conversion jobs, submit batch jobs, and cancel active jobs.
    Write,
    /// Administrative access: create and revoke API keys, manage installations, and inspect system telemetry.
    Admin,
}

impl ApiKeyScope {
    /// Returns the lowercase string identifier for this scope.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Admin => "admin",
        }
    }

    /// Parses an API key scope from a string slice.
    ///
    /// # Errors
    /// Returns [`CoreError::Validation`] if the scope string is not recognized.
    pub fn parse(s: &str) -> Result<Self, CoreError> {
        match s.trim().to_lowercase().as_str() {
            "read" => Ok(Self::Read),
            "write" => Ok(Self::Write),
            "admin" => Ok(Self::Admin),
            other => Err(CoreError::Validation {
                field: "scope".to_string(),
                reason: format!(
                    "Unknown API key scope: '{other}'. Expected 'read', 'write', or 'admin'"
                ),
            }),
        }
    }
}

impl fmt::Display for ApiKeyScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ApiKeyScope {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Persistent record representation of an API key.
///
/// Plaintext secrets are NEVER stored in this record or database; only the BLAKE3 `key_hash` is retained.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiKeyRecord {
    /// Unique identifier of the key record.
    pub id: Uuid,
    /// Tenant or account UUID that owns this key.
    pub tenant_id: Uuid,
    /// Human-readable label (e.g. "CLI Local Agent", "CI/CD Pipeline").
    pub name: String,
    /// Display-safe prefix of the key (e.g. "wr_live_ab12cd...").
    pub key_prefix: String,
    /// BLAKE3 cryptographic hash of the raw plaintext API key token.
    pub key_hash: String,
    /// Set of authorized permissions granted to this key.
    pub scopes: HashSet<ApiKeyScope>,
    /// UTC timestamp of key creation.
    pub created_at: DateTime<Utc>,
    /// Optional UTC timestamp after which the key is expired.
    pub expires_at: Option<DateTime<Utc>>,
    /// Optional UTC timestamp of most recent authenticated invocation.
    pub last_used_at: Option<DateTime<Utc>>,
    /// Optional UTC timestamp when the key was revoked.
    pub revoked_at: Option<DateTime<Utc>>,
}

impl ApiKeyRecord {
    /// Returns `true` if this key is currently active (not revoked and not expired).
    #[must_use]
    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        if self.revoked_at.is_some() {
            return false;
        }
        if let Some(exp) = self.expires_at {
            if now >= exp {
                return false;
            }
        }
        true
    }

    /// Returns `true` if this key is currently active at system UTC time.
    #[must_use]
    pub fn is_currently_active(&self) -> bool {
        self.is_active(Utc::now())
    }

    /// Returns `true` if this key holds the specified scope.
    ///
    /// The [`ApiKeyScope::Admin`] scope automatically satisfies all permission checks.
    #[must_use]
    pub fn has_scope(&self, scope: &ApiKeyScope) -> bool {
        self.scopes.contains(&ApiKeyScope::Admin) || self.scopes.contains(scope)
    }
}

/// Newly generated API key holding the ephemeral plaintext token (shown only once to the user).
#[derive(Debug, Clone)]
pub struct GeneratedApiKey {
    /// Raw bearer token string (`wr_live_<64_hex_chars>`). Must be securely copied by caller.
    pub plaintext: String,
    /// Metadata record to be stored in the database.
    pub record: ApiKeyRecord,
}

impl GeneratedApiKey {
    /// Generates a new cryptographically secure API key using 256 bits of OS entropy.
    ///
    /// # Errors
    /// Returns [`CoreError::Crypto`] if the OS random source fails.
    pub fn generate(
        tenant_id: Uuid,
        name: &str,
        scopes: HashSet<ApiKeyScope>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Self, CoreError> {
        let rng = SystemRandom::new();
        let mut random_bytes = [0u8; 32];
        rng.fill(&mut random_bytes).map_err(|e| {
            CoreError::Crypto(format!("Failed to generate secure random bytes: {e}"))
        })?;

        let hex_secret = hex::encode(random_bytes);
        let plaintext = format!("{API_KEY_PREFIX}{hex_secret}");

        // Compute BLAKE3 cryptographic hash of plaintext key
        let key_hash = blake3::hash(plaintext.as_bytes()).to_hex().to_string();

        let prefix_chars: String = hex_secret.chars().take(8).collect();
        let key_prefix = format!("{API_KEY_PREFIX}{prefix_chars}...");

        let record = ApiKeyRecord {
            id: Uuid::new_v4(),
            tenant_id,
            name: name.trim().to_string(),
            key_prefix,
            key_hash,
            scopes,
            created_at: Utc::now(),
            expires_at,
            last_used_at: None,
            revoked_at: None,
        };

        Ok(Self { plaintext, record })
    }
}

/// Resolved principal identity representing the calling actor in an HTTP request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthenticatedCaller {
    /// Authenticated via browser session header (`X-Session-ID`).
    Session {
        /// Session identifier of the tenant.
        session_id: Uuid,
    },
    /// Authenticated via Bearer API key token.
    ApiKey {
        /// Unique identifier of the API key record.
        key_id: Uuid,
        /// Tenant UUID that owns the API key.
        tenant_id: Uuid,
        /// Label of the key.
        name: String,
        /// Permissions granted to the key.
        scopes: HashSet<ApiKeyScope>,
    },
    /// Unauthenticated caller on public or probe endpoints.
    Anonymous,
}

impl AuthenticatedCaller {
    /// Returns the tenant identifier associated with this caller, if authenticated.
    #[must_use]
    pub fn tenant_id(&self) -> Option<Uuid> {
        match self {
            Self::Session { session_id } => Some(*session_id),
            Self::ApiKey { tenant_id, .. } => Some(*tenant_id),
            Self::Anonymous => None,
        }
    }

    /// Requires the caller to be authenticated, returning the tenant UUID.
    ///
    /// # Errors
    /// Returns [`CoreError::Unauthorized`] if the caller is anonymous.
    pub fn require_authenticated(&self) -> Result<Uuid, CoreError> {
        self.tenant_id().ok_or_else(|| {
            CoreError::Unauthorized(
                "Authentication required. Provide a valid Bearer API key or X-Session-ID header."
                    .to_string(),
            )
        })
    }

    /// Requires the caller to possess the specified permission scope.
    ///
    /// Interactive browser sessions have implicit full scope for their own tenant resources.
    /// API keys must have either the required scope or [`ApiKeyScope::Admin`].
    ///
    /// # Errors
    /// Returns [`CoreError::Unauthorized`] if anonymous, or [`CoreError::Forbidden`] if the API key lacks the scope.
    pub fn require_scope(&self, scope: &ApiKeyScope) -> Result<(), CoreError> {
        match self {
            Self::Session { .. } => Ok(()),
            Self::ApiKey { scopes, name, .. } => {
                if scopes.contains(&ApiKeyScope::Admin) || scopes.contains(scope) {
                    Ok(())
                } else {
                    Err(CoreError::Forbidden(format!(
                        "API key '{name}' lacks required '{scope}' scope. Granted scopes: {scopes:?}"
                    )))
                }
            }
            Self::Anonymous => Err(CoreError::Unauthorized(
                "Authentication required before scope validation.".to_string(),
            )),
        }
    }

    /// Enforces cross-tenant isolation.
    ///
    /// If the resource is associated with a tenant, the caller's tenant MUST match.
    ///
    /// # Errors
    /// Returns [`CoreError::NotFound`] (rather than Forbidden) to prevent tenant enumeration attacks.
    pub fn enforce_tenant_ownership(
        &self,
        resource_tenant_id: Option<Uuid>,
        resource_type: &'static str,
        resource_id: &str,
    ) -> Result<(), CoreError> {
        let expected_tenant = match resource_tenant_id {
            Some(t) => t,
            None => return Ok(()),
        };

        let caller_tenant = self.require_authenticated()?;
        if caller_tenant != expected_tenant {
            return Err(CoreError::NotFound {
                resource_type,
                id: resource_id.to_string(),
            });
        }

        Ok(())
    }
}

mod hex {
    use std::fmt::Write;

    pub fn encode<T: AsRef<[u8]>>(data: T) -> String {
        let bytes = data.as_ref();
        let mut s = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            let _ = write!(&mut s, "{b:02x}");
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_parsing_and_display() {
        assert_eq!(ApiKeyScope::parse("read").unwrap(), ApiKeyScope::Read);
        assert_eq!(ApiKeyScope::parse("WRITE").unwrap(), ApiKeyScope::Write);
        assert_eq!(ApiKeyScope::parse(" Admin ").unwrap(), ApiKeyScope::Admin);
        assert!(ApiKeyScope::parse("invalid_scope").is_err());

        assert_eq!(ApiKeyScope::Read.to_string(), "read");
        assert_eq!(ApiKeyScope::Write.to_string(), "write");
        assert_eq!(ApiKeyScope::Admin.to_string(), "admin");
    }

    #[test]
    fn test_generate_api_key_format_and_hashing() {
        let tenant_id = Uuid::new_v4();
        let mut scopes = HashSet::new();
        scopes.insert(ApiKeyScope::Read);
        scopes.insert(ApiKeyScope::Write);

        let generated = GeneratedApiKey::generate(tenant_id, "Test Key", scopes.clone(), None)
            .expect("Key generation succeeded");

        assert!(generated.plaintext.starts_with(API_KEY_PREFIX));
        assert_eq!(generated.plaintext.len(), API_KEY_PREFIX.len() + 64);
        assert_eq!(generated.record.tenant_id, tenant_id);
        assert_eq!(generated.record.name, "Test Key");
        assert_eq!(generated.record.scopes, scopes);
        assert!(generated.record.is_active(Utc::now()));

        // Cryptographic integrity check: key_hash must match blake3 of plaintext
        let expected_hash = blake3::hash(generated.plaintext.as_bytes())
            .to_hex()
            .to_string();
        assert_eq!(generated.record.key_hash, expected_hash);
    }

    #[test]
    fn test_api_key_expiration_and_revocation() {
        let tenant_id = Uuid::new_v4();
        let mut scopes = HashSet::new();
        scopes.insert(ApiKeyScope::Read);

        let now = Utc::now();
        let past = now - chrono::Duration::hours(1);
        let future = now + chrono::Duration::hours(1);

        let mut generated =
            GeneratedApiKey::generate(tenant_id, "Expiring Key", scopes, Some(past))
                .expect("Key generation succeeded");

        // Expired in past
        assert!(!generated.record.is_active(now));

        // Update expiry to future
        generated.record.expires_at = Some(future);
        assert!(generated.record.is_active(now));

        // Revoke
        generated.record.revoked_at = Some(now);
        assert!(!generated.record.is_active(now));
    }

    #[test]
    fn test_authenticated_caller_scope_and_tenant_enforcement() {
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();

        // 1. Session caller
        let session_caller = AuthenticatedCaller::Session {
            session_id: tenant_a,
        };
        assert_eq!(session_caller.tenant_id(), Some(tenant_a));
        assert!(session_caller.require_scope(&ApiKeyScope::Write).is_ok());
        assert!(session_caller
            .enforce_tenant_ownership(Some(tenant_a), "Job", "123")
            .is_ok());
        assert!(session_caller
            .enforce_tenant_ownership(Some(tenant_b), "Job", "123")
            .is_err());

        // 2. Read-only API Key caller
        let mut read_scopes = HashSet::new();
        read_scopes.insert(ApiKeyScope::Read);
        let key_caller = AuthenticatedCaller::ApiKey {
            key_id: Uuid::new_v4(),
            tenant_id: tenant_a,
            name: "Read Key".to_string(),
            scopes: read_scopes,
        };
        assert!(key_caller.require_scope(&ApiKeyScope::Read).is_ok());
        assert!(key_caller.require_scope(&ApiKeyScope::Write).is_err());

        // 3. Admin API Key caller
        let mut admin_scopes = HashSet::new();
        admin_scopes.insert(ApiKeyScope::Admin);
        let admin_caller = AuthenticatedCaller::ApiKey {
            key_id: Uuid::new_v4(),
            tenant_id: tenant_a,
            name: "Admin Key".to_string(),
            scopes: admin_scopes,
        };
        assert!(admin_caller.require_scope(&ApiKeyScope::Read).is_ok());
        assert!(admin_caller.require_scope(&ApiKeyScope::Write).is_ok());
        assert!(admin_caller.require_scope(&ApiKeyScope::Admin).is_ok());

        // 4. Anonymous caller
        let anon = AuthenticatedCaller::Anonymous;
        assert_eq!(anon.tenant_id(), None);
        assert!(anon.require_authenticated().is_err());
        assert!(anon.require_scope(&ApiKeyScope::Read).is_err());
    }
}
