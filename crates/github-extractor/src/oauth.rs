//! # GitHub App OAuth & State Management
//!
//! Provides cryptographic OAuth state generation and verification (CSRF mitigation),
//! authorization URL generators, OAuth code exchange models, and RS256 JWT generation
//! for authenticating as a GitHub App.
//!
//! ## Rule 05 Compliance
//! - Tokens (`access_token`, `refresh_token`) and private keys are strictly redacted
//!   in all [`fmt::Debug`] and [`fmt::Display`] implementations to prevent credential
//!   leakage in logs.
//! - Nonces are cryptographically random, tied to user/session parameters, and strictly
//!   single-use with automated expiration.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use url::Url;
use uuid::Uuid;

use crate::app_manifest::GitHubAppConfig;
use crate::error::GitHubExtractorError;
use crate::source::decode_base64;

/// Encodes raw bytes into URL-safe, unpadded base64 (RFC 7515 / RFC 4648).
#[must_use]
pub fn base64url_encode(data: &[u8]) -> String {
    const URL_SAFE_CHARSET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity((data.len() * 4).div_ceil(3));
    let mut chunks = data.chunks_exact(3);
    for chunk in chunks.by_ref() {
        let n = ((u32::from(chunk[0])) << 16) | ((u32::from(chunk[1])) << 8) | u32::from(chunk[2]);
        out.push(URL_SAFE_CHARSET[((n >> 18) & 63) as usize] as char);
        out.push(URL_SAFE_CHARSET[((n >> 12) & 63) as usize] as char);
        out.push(URL_SAFE_CHARSET[((n >> 6) & 63) as usize] as char);
        out.push(URL_SAFE_CHARSET[(n & 63) as usize] as char);
    }
    let rem = chunks.remainder();
    if rem.len() == 1 {
        let n = (u32::from(rem[0])) << 16;
        out.push(URL_SAFE_CHARSET[((n >> 18) & 63) as usize] as char);
        out.push(URL_SAFE_CHARSET[((n >> 12) & 63) as usize] as char);
    } else if rem.len() == 2 {
        let n = ((u32::from(rem[0])) << 16) | ((u32::from(rem[1])) << 8);
        out.push(URL_SAFE_CHARSET[((n >> 18) & 63) as usize] as char);
        out.push(URL_SAFE_CHARSET[((n >> 12) & 63) as usize] as char);
        out.push(URL_SAFE_CHARSET[((n >> 6) & 63) as usize] as char);
    }
    out
}

/// Helper encoding an ASN.1 DER length field.
fn encode_asn1_length(len: usize, out: &mut Vec<u8>) {
    if len < 0x80 {
        out.push(len as u8);
    } else if len <= 0xff {
        out.push(0x81);
        out.push(len as u8);
    } else if len <= 0xffff {
        out.push(0x82);
        out.push((len >> 8) as u8);
        out.push((len & 0xff) as u8);
    } else if len <= 0xff_ffff {
        out.push(0x83);
        out.push((len >> 16) as u8);
        out.push(((len >> 8) & 0xff) as u8);
        out.push((len & 0xff) as u8);
    } else {
        out.push(0x84);
        out.push((len >> 24) as u8);
        out.push(((len >> 16) & 0xff) as u8);
        out.push(((len >> 8) & 0xff) as u8);
        out.push((len & 0xff) as u8);
    }
}

/// Wraps PKCS#1 RSA private key DER into a PKCS#8 PrivateKeyInfo structure.
#[must_use]
pub fn pkcs1_to_pkcs8_der(pkcs1_der: &[u8]) -> Vec<u8> {
    // AlgorithmIdentifier for rsaEncryption: SEQUENCE { OID 1.2.840.113549.1.1.1, NULL }
    let alg_id: [u8; 15] = [
        0x30, 0x0d, 0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01, 0x05, 0x00,
    ];
    let version: [u8; 3] = [0x02, 0x01, 0x00]; // Version 0 (v1)

    let mut octet_string = Vec::new();
    octet_string.push(0x04); // OCTET STRING
    encode_asn1_length(pkcs1_der.len(), &mut octet_string);
    octet_string.extend_from_slice(pkcs1_der);

    let total_len = version.len() + alg_id.len() + octet_string.len();
    let mut pkcs8 = Vec::new();
    pkcs8.push(0x30); // SEQUENCE
    encode_asn1_length(total_len, &mut pkcs8);
    pkcs8.extend_from_slice(&version);
    pkcs8.extend_from_slice(&alg_id);
    pkcs8.extend_from_slice(&octet_string);
    pkcs8
}

/// Parses a PEM-encoded RSA private key (PKCS#1 or PKCS#8) into DER bytes suitable for `ring`.
///
/// # Errors
/// Returns [`GitHubExtractorError::AppConfiguration`] if PEM delimiters are missing
/// or base64 decoding fails.
pub fn parse_rsa_pem_to_pkcs8_der(pem: &str) -> Result<Vec<u8>, GitHubExtractorError> {
    let is_pkcs1 = pem.contains("BEGIN RSA PRIVATE KEY");
    let is_pkcs8 = pem.contains("BEGIN PRIVATE KEY");

    if !is_pkcs1 && !is_pkcs8 {
        return Err(GitHubExtractorError::AppConfiguration(
            "Private key PEM must contain 'BEGIN RSA PRIVATE KEY' or 'BEGIN PRIVATE KEY'"
                .to_string(),
        ));
    }

    let mut in_block = false;
    let mut b64 = String::new();
    for line in pem.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("-----BEGIN") {
            in_block = true;
            continue;
        }
        if trimmed.starts_with("-----END") {
            break;
        }
        if in_block {
            b64.push_str(trimmed);
        }
    }

    if b64.is_empty() {
        return Err(GitHubExtractorError::AppConfiguration(
            "No PEM payload found between private key delimiters".to_string(),
        ));
    }

    let der = decode_base64(&b64).map_err(|e| {
        GitHubExtractorError::AppConfiguration(format!("Failed to base64 decode private key: {e}"))
    })?;

    if is_pkcs1 {
        Ok(pkcs1_to_pkcs8_der(&der))
    } else {
        Ok(der)
    }
}

/// Generates an RS256 JWT for GitHub App authentication (RFC 7519).
///
/// GitHub Apps authenticate with a JWT signed using their RSA private key:
/// - Algorithm: RS256
/// - `iss`: App ID
/// - `iat`: now - 60 seconds (clock skew tolerance)
/// - `exp`: now + 600 seconds (10 minutes ceiling)
///
/// # Errors
/// Returns [`GitHubExtractorError::AppConfiguration`] if the private key is invalid
/// or signing fails.
pub fn generate_app_jwt(config: &GitHubAppConfig) -> Result<String, GitHubExtractorError> {
    generate_app_jwt_at_time(config.app_id, config.private_key_pem(), Utc::now())
}

/// Generates an RS256 JWT given explicit timestamp (for deterministic testing).
///
/// # Errors
/// Returns [`GitHubExtractorError::AppConfiguration`] if signing fails.
pub fn generate_app_jwt_at_time(
    app_id: u64,
    private_key_pem: &str,
    now: DateTime<Utc>,
) -> Result<String, GitHubExtractorError> {
    let pkcs8_der = parse_rsa_pem_to_pkcs8_der(private_key_pem)?;

    let key_pair = ring::signature::RsaKeyPair::from_pkcs8(&pkcs8_der).map_err(|e| {
        GitHubExtractorError::AppConfiguration(format!("Invalid RSA private key PKCS#8 DER: {e}"))
    })?;

    let header = serde_json::json!({
        "alg": "RS256",
        "typ": "JWT"
    });
    let header_json = serde_json::to_string(&header)
        .map_err(|e| GitHubExtractorError::AppConfiguration(e.to_string()))?;
    let header_b64 = base64url_encode(header_json.as_bytes());

    let iat = now.timestamp() - 60;
    let exp = now.timestamp() + 600;
    let claims = serde_json::json!({
        "iat": iat,
        "exp": exp,
        "iss": app_id.to_string()
    });
    let claims_json = serde_json::to_string(&claims)
        .map_err(|e| GitHubExtractorError::AppConfiguration(e.to_string()))?;
    let claims_b64 = base64url_encode(claims_json.as_bytes());

    let signing_input = format!("{header_b64}.{claims_b64}");

    let mut signature = vec![0u8; key_pair.public().modulus_len()];
    let rng = ring::rand::SystemRandom::new();
    key_pair
        .sign(
            &ring::signature::RSA_PKCS1_SHA256,
            &rng,
            signing_input.as_bytes(),
            &mut signature,
        )
        .map_err(|e| {
            GitHubExtractorError::AppConfiguration(format!("RSA signature failed: {e}"))
        })?;

    let sig_b64 = base64url_encode(&signature);
    Ok(format!("{signing_input}.{sig_b64}"))
}

/// Constant-time slice equality comparison to mitigate timing attacks.
#[must_use]
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (&x, &y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Cryptographically secure OAuth state parameter for CSRF defense.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OAuthState {
    /// Cryptographic nonce string.
    pub nonce: String,
    /// Nonce generation timestamp.
    pub created_at: DateTime<Utc>,
    /// Nonce expiration timestamp.
    pub expires_at: DateTime<Utc>,
    /// Optional user or session identifier bound to this nonce.
    pub session_id: Option<String>,
    /// Optional tenant identifier bound to this nonce.
    pub tenant_id: Option<String>,
    /// Optional expected redirect URI.
    pub redirect_uri: Option<String>,
}

impl Default for OAuthState {
    fn default() -> Self {
        Self::new()
    }
}

impl OAuthState {
    /// Default time-to-live for an OAuth state nonce (10 minutes).
    pub const DEFAULT_TTL: Duration = Duration::minutes(10);

    /// Generates a new cryptographically random OAuth state with standard 10-minute TTL.
    #[must_use]
    pub fn new() -> Self {
        Self::with_ttl(Self::DEFAULT_TTL)
    }

    /// Generates an OAuth state with a customized TTL.
    #[must_use]
    pub fn with_ttl(ttl: Duration) -> Self {
        let now = Utc::now();
        let u1 = Uuid::new_v4();
        let u2 = Uuid::new_v4();
        let seed = format!("{}:{}:{}", u1, u2, now.timestamp_nanos_opt().unwrap_or(0));
        let hash = blake3::hash(seed.as_bytes());
        let nonce = hash.to_hex().to_string();

        Self {
            nonce,
            created_at: now,
            expires_at: now + ttl,
            session_id: None,
            tenant_id: None,
            redirect_uri: None,
        }
    }

    /// Binds this state to a user or session identifier.
    #[must_use]
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Binds this state to a tenant identifier.
    #[must_use]
    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// Binds this state to an expected redirect URI.
    #[must_use]
    pub fn with_redirect_uri(mut self, redirect_uri: impl Into<String>) -> Self {
        self.redirect_uri = Some(redirect_uri.into());
        self
    }

    /// Returns true if this state nonce has expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Performs constant-time comparison against an untrusted candidate nonce.
    #[must_use]
    pub fn constant_time_matches(&self, candidate: &str) -> bool {
        constant_time_eq(self.nonce.as_bytes(), candidate.as_bytes())
    }
}

/// Abstract storage and single-use validation interface for OAuth states.
pub trait OAuthStateStore: Send + Sync {
    /// Persists a newly created state nonce.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError::AppConfiguration`] on storage failure.
    fn save_state(&mut self, state: OAuthState) -> Result<(), GitHubExtractorError>;

    /// Validates an incoming callback nonce, verifying expiration, session match,
    /// and single-use consumption.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError::AppConfiguration`] if the nonce is missing,
    /// expired, or session mismatch occurs.
    fn validate_and_consume(
        &mut self,
        nonce: &str,
        expected_session_id: Option<&str>,
    ) -> Result<OAuthState, GitHubExtractorError>;
}

/// In-memory implementation of [`OAuthStateStore`] with automatic eviction of expired nonces.
#[derive(Debug, Clone, Default)]
pub struct InMemoryOAuthStateStore {
    states: HashMap<String, OAuthState>,
}

impl InMemoryOAuthStateStore {
    /// Creates a new in-memory state store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    /// Removes all expired states from storage.
    pub fn prune_expired(&mut self) {
        let now = Utc::now();
        self.states.retain(|_, s| s.expires_at > now);
    }

    /// Returns the count of currently active states in memory.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.states.len()
    }
}

impl OAuthStateStore for InMemoryOAuthStateStore {
    fn save_state(&mut self, state: OAuthState) -> Result<(), GitHubExtractorError> {
        self.prune_expired();
        self.states.insert(state.nonce.clone(), state);
        Ok(())
    }

    fn validate_and_consume(
        &mut self,
        nonce: &str,
        expected_session_id: Option<&str>,
    ) -> Result<OAuthState, GitHubExtractorError> {
        self.prune_expired();

        let state = self.states.remove(nonce).ok_or_else(|| {
            GitHubExtractorError::AppConfiguration(
                "Invalid or expired OAuth state nonce: state parameter not found or already consumed"
                    .to_string(),
            )
        })?;

        if state.is_expired() {
            return Err(GitHubExtractorError::AppConfiguration(
                "OAuth state nonce has expired".to_string(),
            ));
        }

        if let (Some(expected), Some(actual)) = (expected_session_id, &state.session_id) {
            if !constant_time_eq(expected.as_bytes(), actual.as_bytes()) {
                return Err(GitHubExtractorError::AppConfiguration(
                    "OAuth state session mismatch: state does not belong to the current session"
                        .to_string(),
                ));
            }
        }

        Ok(state)
    }
}

/// Constructs the GitHub OAuth authorization URL directing the user to grant access.
///
/// # Arguments
/// * `client_id` - The public Client ID of the GitHub App.
/// * `redirect_uri` - Optional explicit redirect URI matching the App's configuration.
/// * `state` - The cryptographically generated state nonce.
/// * `allow_signup` - Whether unauthenticated users can sign up during flow (defaults to false).
/// * `base_html_url` - Optional GitHub base URL (defaults to `"https://github.com"`).
///
/// # Errors
/// Returns [`GitHubExtractorError::AppConfiguration`] if URL parsing fails.
pub fn build_oauth_authorize_url(
    client_id: &str,
    redirect_uri: Option<&str>,
    state: &str,
    allow_signup: bool,
    base_html_url: Option<&str>,
) -> Result<String, GitHubExtractorError> {
    if client_id.trim().is_empty() {
        return Err(GitHubExtractorError::AppConfiguration(
            "client_id cannot be empty".to_string(),
        ));
    }
    if state.trim().is_empty() {
        return Err(GitHubExtractorError::AppConfiguration(
            "state parameter cannot be empty".to_string(),
        ));
    }

    let base = base_html_url
        .unwrap_or("https://github.com")
        .trim_end_matches('/');
    let mut url = Url::parse(&format!("{base}/login/oauth/authorize")).map_err(|e| {
        GitHubExtractorError::AppConfiguration(format!("Failed to parse authorize URL: {e}"))
    })?;

    url.query_pairs_mut().append_pair("client_id", client_id);
    url.query_pairs_mut().append_pair("state", state);
    if let Some(uri) = redirect_uri {
        url.query_pairs_mut().append_pair("redirect_uri", uri);
    }
    if !allow_signup {
        url.query_pairs_mut().append_pair("allow_signup", "false");
    }

    Ok(url.to_string())
}

/// Constructs the GitHub App installation URL directing an admin to install the app.
///
/// # Arguments
/// * `app_slug` - The URL slug of the GitHub App.
/// * `state` - Optional cryptographic state nonce.
/// * `base_html_url` - Optional GitHub base URL (defaults to `"https://github.com"`).
///
/// # Errors
/// Returns [`GitHubExtractorError::AppConfiguration`] if URL parsing fails.
pub fn build_app_installation_url(
    app_slug: &str,
    state: Option<&str>,
    base_html_url: Option<&str>,
) -> Result<String, GitHubExtractorError> {
    if app_slug.trim().is_empty() {
        return Err(GitHubExtractorError::AppConfiguration(
            "app_slug cannot be empty".to_string(),
        ));
    }

    let base = base_html_url
        .unwrap_or("https://github.com")
        .trim_end_matches('/');
    let mut url =
        Url::parse(&format!("{base}/apps/{app_slug}/installations/new")).map_err(|e| {
            GitHubExtractorError::AppConfiguration(format!("Failed to parse installation URL: {e}"))
        })?;

    if let Some(s) = state {
        url.query_pairs_mut().append_pair("state", s);
    }

    Ok(url.to_string())
}

/// Successful response from GitHub's OAuth code exchange endpoint (`POST /login/oauth/access_token`).
///
/// Sensitive token fields are strictly masked in [`fmt::Debug`] and [`fmt::Display`].
#[derive(Clone, Serialize, Deserialize)]
pub struct GitHubOAuthTokenResponse {
    /// OAuth user access token (sensitive).
    pub access_token: String,
    /// Token type (typically `"bearer"`).
    pub token_type: String,
    /// Granted permission scopes string (e.g. `""` for GitHub App or space-separated list).
    #[serde(default)]
    pub scope: String,
    /// Seconds until token expires (if expiring user access tokens are enabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
    /// Refresh token for renewing access (sensitive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Seconds until refresh token expires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token_expires_in: Option<u64>,
}

impl fmt::Debug for GitHubOAuthTokenResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GitHubOAuthTokenResponse")
            .field("access_token", &"***[REDACTED_TOKEN]***")
            .field("token_type", &self.token_type)
            .field("scope", &self.scope)
            .field("expires_in", &self.expires_in)
            .field(
                "refresh_token",
                &self
                    .refresh_token
                    .as_ref()
                    .map(|_| "***[REDACTED_TOKEN]***"),
            )
            .field("refresh_token_expires_in", &self.refresh_token_expires_in)
            .finish()
    }
}

impl fmt::Display for GitHubOAuthTokenResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GitHubOAuthTokenResponse(type={}, scope={:?}, expires_in={:?})",
            self.token_type, self.scope, self.expires_in
        )
    }
}

impl GitHubOAuthTokenResponse {
    /// Accessor for the access token string.
    #[must_use]
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    /// Accessor for the refresh token string, if present.
    #[must_use]
    pub fn refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_deref()
    }
}

/// Error response returned by GitHub's OAuth endpoint when exchange fails.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubOAuthErrorResponse {
    /// Error code string (e.g. `"bad_verification_code"`, `"redirect_uri_mismatch"`).
    pub error: String,
    /// Human-readable explanation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
    /// Documentation URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_uri: Option<String>,
}

impl fmt::Display for GitHubOAuthErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OAuth error '{}'", self.error)?;
        if let Some(desc) = &self.error_description {
            write!(f, ": {desc}")?;
        }
        Ok(())
    }
}
