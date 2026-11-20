//! # GitHub App Callback & Webhook Security
//!
//! Provides cryptographic OAuth callback verification, open-redirect defense,
//! GitHub App installation setup callback validation, HMAC-SHA256 webhook signature
//! verification (`X-Hub-Signature-256`), delivery GUID deduplication (replay defense),
//! and strongly-typed lifecycle event routing under Rule 05.
//!
//! ## Rule 05 Compliance & Security Invariants
//!
//! 1. **Timing-Safe CSRF Verification:** OAuth callback `state` parameters are matched
//!    in constant time, bound to session/tenant identifiers, and strictly consumed
//!    on first use to prevent replay and CSRF injection attacks.
//! 2. **Open-Redirect Defense:** Redirect targets are strictly validated: relative paths
//!    must start with a single `/` (and never protocol-relative `//` or `/\`), and
//!    absolute URLs must match explicitly whitelisted base origins.
//! 3. **Fail-Closed Webhook Verification:** Incoming webhook request bodies are never
//!    deserialized or processed until the `X-Hub-Signature-256` HMAC-SHA256 signature
//!    is verified using constant-time MAC comparison.
//! 4. **Delivery Replay Defense:** Webhook delivery GUIDs (`X-GitHub-Delivery`) are tracked
//!    in a sliding window to detect and reject replayed payloads.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use url::Url;

use crate::error::GitHubExtractorError;
use crate::installation::{GitHubInstallation, InstalledRepositorySummary};
use crate::oauth::{OAuthState, OAuthStateStore};

/// Decodes a 64-character hex string into a 32-byte SHA-256 digest.
fn decode_hex_sha256(hex_str: &str) -> Option<[u8; 32]> {
    if hex_str.len() != 64 {
        return None;
    }
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        let byte_str = &hex_str[i * 2..i * 2 + 2];
        bytes[i] = u8::from_str_radix(byte_str, 16).ok()?;
    }
    Some(bytes)
}

/// Query parameters received from GitHub at the OAuth redirect URI (`/api/v1/github/oauth/callback`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OAuthCallbackParams {
    /// Temporary authorization code issued by GitHub on successful authorization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Cryptographic state nonce passed in the authorization request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Error code if the user denied authorization or an error occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Human-readable description of the error from GitHub.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
    /// Web page providing additional details on the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_uri: Option<String>,
}

/// Successfully validated OAuth callback data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedOAuthCallback {
    /// The sanitized authorization code ready for token exchange.
    pub code: String,
    /// The verified, consumed [`OAuthState`] associated with this callback.
    pub state: OAuthState,
}

/// Helper decoding percent-encoded path bytes. Returns None if encoding is malformed.
fn percent_decode_path(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len() {
                return None;
            }
            let hex_sub = &s[i + 1..i + 3];
            let byte = u8::from_str_radix(hex_sub, 16).ok()?;
            out.push(byte);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// Open redirect vulnerability defense.
pub struct OpenRedirectGuard;

impl OpenRedirectGuard {
    /// Returns true if the target path is a safe relative redirect path.
    ///
    /// Rejects protocol-relative URLs (`//attacker.com`), backslash variants (`/\`, `\\`),
    /// control characters, and percent-encoded obfuscation sequences (`%2f%2f`, `%5c`, `%00`, `%0d%0a`).
    #[must_use]
    pub fn is_safe_relative_path(path: &str) -> bool {
        let trimmed = path.trim();
        if !trimmed.starts_with('/') {
            return false;
        }

        // Protocol-relative URLs (e.g. "//evil.com" or "/\evil.com")
        if trimmed.starts_with("//") || trimmed.starts_with("/\\") || trimmed.starts_with("/\t") {
            return false;
        }

        // Check for control characters, backslashes, or path traversal segments
        if trimmed.chars().any(|c| c.is_control() || c == '\\') || trimmed.contains("..") {
            return false;
        }

        // Percent-decoded evaluation: blocks %2f%2f (//), %5c (\), %00 (null byte), %0d%0a (CRLF), %2e%2e (..)
        if trimmed.contains('%') {
            let decoded = match percent_decode_path(trimmed) {
                Some(d) => d,
                None => return false,
            };
            let dec_trimmed = decoded.trim();
            if !dec_trimmed.starts_with('/') {
                return false;
            }
            if dec_trimmed.starts_with("//")
                || dec_trimmed.starts_with("/\\")
                || dec_trimmed.starts_with("/\t")
            {
                return false;
            }
            if dec_trimmed.chars().any(|c| c.is_control() || c == '\\')
                || dec_trimmed.contains("..")
            {
                return false;
            }
        }

        true
    }

    /// Validates whether a redirect URL is safe to follow.
    ///
    /// # Arguments
    /// * `target_url` - The untrusted target redirect URI.
    /// * `allowed_base_urls` - Whitelisted base origins (e.g. `["https://app.example.com"]`).
    #[must_use]
    pub fn is_safe_redirect(target_url: &str, allowed_base_urls: &[&str]) -> bool {
        let trimmed = target_url.trim();

        // 1. Check if safe relative path
        if Self::is_safe_relative_path(trimmed) {
            return true;
        }

        // 2. Otherwise parse as absolute URL
        let parsed = match Url::parse(trimmed) {
            Ok(u) => u,
            Err(_) => return false,
        };

        // Reject embedded userinfo (e.g. https://attacker:pass@app.example.com)
        if !parsed.username().is_empty() || parsed.password().is_some() {
            return false;
        }

        // Reject non-http(s) schemes (e.g. javascript:, data:, vbscript:)
        let scheme = parsed.scheme();
        if scheme != "http" && scheme != "https" {
            return false;
        }

        // Must match one of the allowed base URLs
        for allowed in allowed_base_urls {
            if let Ok(allowed_parsed) = Url::parse(allowed) {
                if parsed.scheme() == allowed_parsed.scheme()
                    && parsed.host() == allowed_parsed.host()
                    && parsed.port_or_known_default() == allowed_parsed.port_or_known_default()
                {
                    return true;
                }
            }
        }

        false
    }

    /// Validates a redirect URI, returning [`GitHubExtractorError::CallbackSecurity`] on violation.
    ///
    /// # Errors
    /// Returns error if the redirect target is unsafe or points to an unauthorized origin.
    pub fn validate_redirect(
        target_url: &str,
        allowed_base_urls: &[&str],
    ) -> Result<(), GitHubExtractorError> {
        if Self::is_safe_redirect(target_url, allowed_base_urls) {
            Ok(())
        } else {
            Err(GitHubExtractorError::CallbackSecurity(format!(
                "Open redirect attempt rejected: redirect URI '{target_url}' is not a safe relative path or whitelisted origin"
            )))
        }
    }
}

/// Validator for OAuth redirect callbacks.
pub struct OAuthCallbackValidator;

impl OAuthCallbackValidator {
    /// Validates an incoming OAuth callback request.
    ///
    /// # Errors
    /// - Returns [`GitHubExtractorError::CallbackSecurity`] if an error was returned from GitHub
    ///   (e.g. `access_denied`), if required parameters are missing, or if the code/state is invalid.
    /// - Consumes the state nonce from `store` ensuring single-use replay protection.
    pub fn validate_callback(
        params: &OAuthCallbackParams,
        store: &mut dyn OAuthStateStore,
        expected_session_id: Option<&str>,
        allowed_redirect_origins: &[&str],
    ) -> Result<ValidatedOAuthCallback, GitHubExtractorError> {
        // 1. Handle error response from GitHub
        if let Some(err) = &params.error {
            let desc = params
                .error_description
                .as_deref()
                .unwrap_or("no error description provided");
            return Err(GitHubExtractorError::CallbackSecurity(format!(
                "GitHub OAuth authorization failed with error '{err}': {desc}"
            )));
        }

        // 2. Validate state presence
        let state_param = params.state.as_deref().unwrap_or("").trim();
        if state_param.is_empty() {
            return Err(GitHubExtractorError::CallbackSecurity(
                "Missing or empty 'state' parameter in OAuth callback".to_string(),
            ));
        }

        // 3. Validate code presence and format
        let code_param = params.code.as_deref().unwrap_or("").trim();
        if code_param.is_empty() {
            return Err(GitHubExtractorError::CallbackSecurity(
                "Missing or empty 'code' parameter in OAuth callback".to_string(),
            ));
        }

        // Code sanity check: 1 to 256 characters, ASCII alphanumeric, dash, underscore
        if code_param.len() > 256
            || !code_param
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(GitHubExtractorError::CallbackSecurity(
                "Malformed 'code' parameter in OAuth callback: invalid characters or excessive length"
                    .to_string(),
            ));
        }

        // 4. Validate and consume state from store (timing-safe, single-use, session matching)
        let state = store.validate_and_consume(state_param, expected_session_id)?;

        // 5. If state contains a redirect URI, check for open redirect
        if let Some(uri) = &state.redirect_uri {
            OpenRedirectGuard::validate_redirect(uri, allowed_redirect_origins)?;
        }

        Ok(ValidatedOAuthCallback {
            code: code_param.to_string(),
            state,
        })
    }
}

/// Action taken during GitHub App installation setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SetupAction {
    /// Newly installed application.
    Install,
    /// Existing installation configuration updated.
    Update,
    /// Access request submitted.
    Request,
}

impl SetupAction {
    /// Parses a setup action string from GitHub.
    #[must_use]
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "install" => Some(Self::Install),
            "update" => Some(Self::Update),
            "request" => Some(Self::Request),
            _ => None,
        }
    }

    /// Returns the string representation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Install => "install",
            Self::Update => "update",
            Self::Request => "request",
        }
    }
}

impl fmt::Display for SetupAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Query parameters received at the Post-Installation setup URL (`/settings/integrations/github`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InstallationCallbackParams {
    /// Numeric installation ID assigned by GitHub.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installation_id: Option<u64>,
    /// Setup action string (`"install"`, `"update"`, or `"request"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup_action: Option<String>,
    /// Optional OAuth authorization code if requested during installation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Optional state parameter passed in the installation URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

/// Successfully validated installation callback data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedInstallationCallback {
    /// Verified numeric installation ID.
    pub installation_id: u64,
    /// Verified setup action.
    pub setup_action: SetupAction,
    /// Optional OAuth authorization code.
    pub code: Option<String>,
    /// Optional consumed state if state parameter was present.
    pub state: Option<OAuthState>,
}

/// Validator for GitHub App post-installation setup callbacks.
pub struct InstallationCallbackValidator;

impl InstallationCallbackValidator {
    /// Validates an incoming installation callback request.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError::CallbackSecurity`] if the installation ID is missing,
    /// zero, or if the setup action is invalid or state verification fails.
    pub fn validate_installation_callback(
        params: &InstallationCallbackParams,
        mut store: Option<&mut dyn OAuthStateStore>,
        expected_session_id: Option<&str>,
    ) -> Result<ValidatedInstallationCallback, GitHubExtractorError> {
        // 1. Validate installation_id
        let installation_id = match params.installation_id {
            Some(id) if id > 0 => id,
            _ => {
                return Err(GitHubExtractorError::CallbackSecurity(
                    "Missing or invalid 'installation_id' in setup callback (must be a positive integer)"
                        .to_string(),
                ));
            }
        };

        // 2. Validate setup_action
        let action_str = params.setup_action.as_deref().unwrap_or("install");
        let setup_action = SetupAction::from_str_loose(action_str).ok_or_else(|| {
            GitHubExtractorError::CallbackSecurity(format!(
                "Invalid 'setup_action' in installation callback: '{action_str}'; expected 'install', 'update', or 'request'"
            ))
        })?;

        // 3. Optional code sanitization
        let code = if let Some(c) = &params.code {
            let trimmed = c.trim();
            if trimmed.is_empty() {
                None
            } else if trimmed.len() > 256
                || !trimmed
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.')
            {
                return Err(GitHubExtractorError::CallbackSecurity(
                    "Malformed 'code' parameter in installation callback".to_string(),
                ));
            } else {
                Some(trimmed.to_string())
            }
        } else {
            None
        };

        // 4. Optional state validation if state was passed
        let state = if let Some(s) = &params.state {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else if let Some(ref mut st) = store {
                Some(st.validate_and_consume(trimmed, expected_session_id)?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(ValidatedInstallationCallback {
            installation_id,
            setup_action,
            code,
            state,
        })
    }
}

/// Webhook signature verification engine using HMAC-SHA256 (`X-Hub-Signature-256`).
pub struct WebhookSignatureVerifier;

impl WebhookSignatureVerifier {
    /// Computes the HMAC-SHA256 digest of `body` using `secret`.
    #[must_use]
    pub fn compute_signature(secret: &[u8], body: &[u8]) -> [u8; 32] {
        let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, secret);
        let tag = ring::hmac::sign(&key, body);
        let mut out = [0u8; 32];
        out.copy_from_slice(tag.as_ref());
        out
    }

    /// Computes the formatted header value (e.g. `"sha256=<hex>"`).
    #[must_use]
    pub fn compute_signature_header(secret: &[u8], body: &[u8]) -> String {
        let digest = Self::compute_signature(secret, body);
        let hex_str: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        format!("sha256={hex_str}")
    }

    /// Verifies the signature in the `X-Hub-Signature-256` header in constant time.
    ///
    /// # Arguments
    /// * `secret` - The shared webhook secret configured on GitHub.
    /// * `body` - The raw, unparsed HTTP request body bytes.
    /// * `signature_header` - Value of the `X-Hub-Signature-256` HTTP header.
    #[must_use]
    pub fn verify_signature(secret: &[u8], body: &[u8], signature_header: &str) -> bool {
        let hex_sig = match signature_header.strip_prefix("sha256=") {
            Some(hex) => hex.trim(),
            None => return false,
        };

        let decoded_sig = match decode_hex_sha256(hex_sig) {
            Some(bytes) => bytes,
            None => return false,
        };

        let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, secret);
        ring::hmac::verify(&key, body, &decoded_sig).is_ok()
    }

    /// Verifies the webhook signature, returning [`GitHubExtractorError::CallbackSecurity`] on failure.
    ///
    /// # Errors
    /// Returns error if the signature is missing, malformed, or invalid.
    pub fn verify_header(
        secret: &[u8],
        body: &[u8],
        signature_header: &str,
    ) -> Result<(), GitHubExtractorError> {
        if secret.is_empty() {
            return Err(GitHubExtractorError::CallbackSecurity(
                "Webhook secret is not configured".to_string(),
            ));
        }

        if Self::verify_signature(secret, body, signature_header) {
            Ok(())
        } else {
            Err(GitHubExtractorError::CallbackSecurity(
                "Invalid HMAC-SHA256 signature in X-Hub-Signature-256: webhook payload verification failed"
                    .to_string(),
            ))
        }
    }
}

/// Abstract storage and duplicate detection interface for webhook delivery GUIDs (`X-GitHub-Delivery`).
pub trait WebhookDeliveryTracker: Send + Sync {
    /// Returns true if this delivery ID has already been recorded within the sliding window.
    fn is_duplicate(&self, delivery_id: &str) -> bool;

    /// Records a new delivery ID. Returns error if it is a duplicate.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError::CallbackSecurity`] if `delivery_id` is a duplicate replay.
    fn record_delivery(&mut self, delivery_id: &str) -> Result<(), GitHubExtractorError>;

    /// Evicts expired delivery records.
    fn prune_expired(&mut self);
}

/// In-memory implementation of [`WebhookDeliveryTracker`] with sliding-window TTL.
#[derive(Debug, Clone)]
pub struct InMemoryWebhookDeliveryTracker {
    deliveries: HashMap<String, DateTime<Utc>>,
    ttl: Duration,
}

impl Default for InMemoryWebhookDeliveryTracker {
    fn default() -> Self {
        Self::new(Duration::hours(1))
    }
}

impl InMemoryWebhookDeliveryTracker {
    /// Creates a new delivery tracker with the given sliding-window TTL.
    #[must_use]
    pub fn new(ttl: Duration) -> Self {
        Self {
            deliveries: HashMap::new(),
            ttl,
        }
    }

    /// Returns the number of active delivery records in memory.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.deliveries.len()
    }
}

impl WebhookDeliveryTracker for InMemoryWebhookDeliveryTracker {
    fn is_duplicate(&self, delivery_id: &str) -> bool {
        let now = Utc::now();
        if let Some(&expires_at) = self.deliveries.get(delivery_id) {
            expires_at > now
        } else {
            false
        }
    }

    fn record_delivery(&mut self, delivery_id: &str) -> Result<(), GitHubExtractorError> {
        self.prune_expired();

        if self.is_duplicate(delivery_id) {
            return Err(GitHubExtractorError::CallbackSecurity(format!(
                "Duplicate webhook delivery GUID '{delivery_id}' detected: possible replay attack"
            )));
        }

        let expires_at = Utc::now() + self.ttl;
        self.deliveries.insert(delivery_id.to_string(), expires_at);
        Ok(())
    }

    fn prune_expired(&mut self) {
        let now = Utc::now();
        self.deliveries.retain(|_, &mut exp| exp > now);
    }
}

/// Action taken in an `installation` webhook event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallationAction {
    /// App installation created on user/org account.
    Created,
    /// App installation deleted.
    Deleted,
    /// App installation temporarily suspended.
    Suspend,
    /// Suspended installation restored.
    Unsuspend,
    /// New permissions accepted by organization admin.
    NewPermissionsAccepted,
    /// Other action.
    #[serde(untagged)]
    Other(String),
}

/// Action taken in an `installation_repositories` webhook event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallationRepoAction {
    /// Repositories added to the installation.
    Added,
    /// Repositories removed from the installation.
    Removed,
    /// Other action.
    #[serde(untagged)]
    Other(String),
}

/// Strongly-typed GitHub App lifecycle webhook events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum GitHubWebhookEvent {
    /// Ping event sent when a webhook is first configured.
    Ping {
        /// Friendly zen philosophy string from GitHub.
        zen: String,
        /// Unique numeric webhook hook ID.
        hook_id: u64,
    },
    /// Installation lifecycle event (`action: created`, `deleted`, `suspend`, `unsuspend`).
    Installation {
        /// Event action.
        action: InstallationAction,
        /// Detailed installation entity.
        installation: GitHubInstallation,
    },
    /// Repository access lifecycle event (`action: added`, `removed`).
    InstallationRepositories {
        /// Event action.
        action: InstallationRepoAction,
        /// Associated installation.
        installation: GitHubInstallation,
        /// Repositories newly added to access list.
        #[serde(default)]
        repositories_added: Vec<InstalledRepositorySummary>,
        /// Repositories removed from access list.
        #[serde(default)]
        repositories_removed: Vec<InstalledRepositorySummary>,
    },
    /// Other unhandled event type.
    Other {
        /// Name of the event from the `X-GitHub-Event` header.
        event_name: String,
        /// Raw payload value.
        payload: serde_json::Value,
    },
}

/// High-level parser for verified GitHub webhook requests.
pub struct WebhookEventParser;

impl WebhookEventParser {
    /// Validates HMAC signature, checks delivery deduplication, and parses event payload.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError::CallbackSecurity`] if signature verification fails,
    /// delivery is a duplicate replay, or payload JSON parsing fails.
    pub fn parse_webhook_event(
        secret: &[u8],
        body: &[u8],
        signature_header: &str,
        event_header: &str,
        delivery_id_header: Option<&str>,
        mut tracker: Option<&mut dyn WebhookDeliveryTracker>,
    ) -> Result<GitHubWebhookEvent, GitHubExtractorError> {
        // 1. Mandatory HMAC verification before reading any body contents
        WebhookSignatureVerifier::verify_header(secret, body, signature_header)?;

        // 2. Replay detection on delivery GUID
        if let Some(del_id) = delivery_id_header {
            let del_id_trimmed = del_id.trim();
            if !del_id_trimmed.is_empty() {
                if let Some(ref mut tr) = tracker {
                    tr.record_delivery(del_id_trimmed)?;
                }
            }
        }

        // 3. Deserialization based on X-GitHub-Event
        let event_lower = event_header.trim().to_ascii_lowercase();
        match event_lower.as_str() {
            "ping" => {
                #[derive(Deserialize)]
                struct PingPayload {
                    zen: Option<String>,
                    hook_id: Option<u64>,
                }
                let parsed: PingPayload = serde_json::from_slice(body).map_err(|e| {
                    GitHubExtractorError::JsonDecodeError(format!(
                        "Failed to decode ping webhook payload: {e}"
                    ))
                })?;
                Ok(GitHubWebhookEvent::Ping {
                    zen: parsed
                        .zen
                        .unwrap_or_else(|| "Keep it logically awesome.".to_string()),
                    hook_id: parsed.hook_id.unwrap_or(0),
                })
            }
            "installation" => {
                #[derive(Deserialize)]
                struct InstallationPayload {
                    action: InstallationAction,
                    installation: GitHubInstallation,
                }
                let parsed: InstallationPayload = serde_json::from_slice(body).map_err(|e| {
                    GitHubExtractorError::JsonDecodeError(format!(
                        "Failed to decode installation webhook payload: {e}"
                    ))
                })?;
                Ok(GitHubWebhookEvent::Installation {
                    action: parsed.action,
                    installation: parsed.installation,
                })
            }
            "installation_repositories" => {
                #[derive(Deserialize)]
                struct InstallationReposPayload {
                    action: InstallationRepoAction,
                    installation: GitHubInstallation,
                    #[serde(default)]
                    repositories_added: Vec<InstalledRepositorySummary>,
                    #[serde(default)]
                    repositories_removed: Vec<InstalledRepositorySummary>,
                }
                let parsed: InstallationReposPayload =
                    serde_json::from_slice(body).map_err(|e| {
                        GitHubExtractorError::JsonDecodeError(format!(
                            "Failed to decode installation_repositories webhook payload: {e}"
                        ))
                    })?;
                Ok(GitHubWebhookEvent::InstallationRepositories {
                    action: parsed.action,
                    installation: parsed.installation,
                    repositories_added: parsed.repositories_added,
                    repositories_removed: parsed.repositories_removed,
                })
            }
            other => {
                let payload: serde_json::Value = serde_json::from_slice(body).map_err(|e| {
                    GitHubExtractorError::JsonDecodeError(format!(
                        "Failed to decode '{other}' webhook payload: {e}"
                    ))
                })?;
                Ok(GitHubWebhookEvent::Other {
                    event_name: other.to_string(),
                    payload,
                })
            }
        }
    }
}
