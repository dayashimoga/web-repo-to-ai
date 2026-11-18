//! # GitHub App Manifest & Configuration Models
//!
//! Provides strongly-typed models, validation rules, 1-click manifest generation,
//! and runtime configuration loading for GitHub App integration.
//!
//! ## Security Invariants & Rule 05 Compliance
//! - **Strict Minimal Permissions:** Only `Contents: Read-only` and `Metadata: Read-only`
//!   permissions are permitted. Write, push, admin, or unnecessary scopes (e.g. issues,
//!   pull requests, workflows) are strictly prohibited and rejected during validation.
//! - **Secret Masking:** Runtime credentials (`client_secret`, `private_key_pem`,
//!   `webhook_secret`) are strictly redacted in [`fmt::Debug`] and [`fmt::Display`]
//!   implementations to prevent token and private key exposure in application logs.
//! - **Cryptographic Nonce & Verification:** Supports Webhook HMAC-SHA256 secrets and
//!   RSA private keys in standard PKCS#1 and PKCS#8 PEM formats.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use url::Url;

use crate::error::GitHubExtractorError;

/// Minimal permission level permitted for GitHub repository conversion.
///
/// Enforces Rule 05: only read access is permitted; write and admin scopes are rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PermissionLevel {
    /// Read-only permission.
    #[default]
    Read,
}

impl PermissionLevel {
    /// Returns the string representation expected by GitHub API / Manifest.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Read => "read",
        }
    }
}

impl fmt::Display for PermissionLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for PermissionLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for PermissionLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_ascii_lowercase().as_str() {
            "read" => Ok(Self::Read),
            "write" | "admin" => Err(serde::de::Error::custom(format!(
                "Permission level '{}' is forbidden by Rule 05; only 'read' is allowed",
                s
            ))),
            other => Err(serde::de::Error::custom(format!(
                "Unknown permission level '{}'; expected 'read'",
                other
            ))),
        }
    }
}

/// Minimal GitHub App permissions required for repository extraction.
///
/// Under Rule 05, this MUST remain strictly read-only (`Contents: Read`, `Metadata: Read`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubAppPermissions {
    /// Repository contents access level (strictly Read).
    pub contents: PermissionLevel,
    /// Repository metadata access level (strictly Read).
    pub metadata: PermissionLevel,
    /// Optional additional permissions (must NOT contain write/admin scopes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_permissions: Option<HashMap<String, String>>,
}

impl Default for GitHubAppPermissions {
    fn default() -> Self {
        Self {
            contents: PermissionLevel::Read,
            metadata: PermissionLevel::Read,
            additional_permissions: None,
        }
    }
}

impl GitHubAppPermissions {
    /// Creates a minimal read-only permissions specification.
    #[must_use]
    pub fn minimal_read_only() -> Self {
        Self::default()
    }

    /// Validates that the permissions adhere to the strict read-only policy.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError::AppConfiguration`] if write, admin, or unauthorized
    /// scopes are detected.
    pub fn validate(&self) -> Result<(), GitHubExtractorError> {
        if self.contents != PermissionLevel::Read {
            return Err(GitHubExtractorError::AppConfiguration(
                "Contents permission must be strictly 'read'".to_string(),
            ));
        }

        if self.metadata != PermissionLevel::Read {
            return Err(GitHubExtractorError::AppConfiguration(
                "Metadata permission must be strictly 'read'".to_string(),
            ));
        }

        if let Some(additional) = &self.additional_permissions {
            for (key, val) in additional {
                let val_lower = val.to_ascii_lowercase();
                if val_lower == "write" || val_lower == "admin" {
                    return Err(GitHubExtractorError::AppConfiguration(format!(
                        "Forbidden permission scope '{key}: {val}': write/admin permissions violate Rule 05"
                    )));
                }

                // Explicitly forbidden permission categories that present security or operational risk
                let forbidden_categories = [
                    "administration",
                    "actions",
                    "secrets",
                    "deployments",
                    "packages",
                    "security_events",
                ];
                if forbidden_categories.contains(&key.to_ascii_lowercase().as_str()) {
                    return Err(GitHubExtractorError::AppConfiguration(format!(
                        "Forbidden permission category '{key}': not permitted for repository converter"
                    )));
                }
            }
        }

        Ok(())
    }
}

/// Webhook configuration attributes for a GitHub App.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubHookAttributes {
    /// Target webhook listener URL.
    pub url: String,
    /// Whether the webhook is currently active.
    #[serde(default = "default_true")]
    pub active: bool,
}

const fn default_true() -> bool {
    true
}

impl GitHubHookAttributes {
    /// Creates a new active webhook attribute specification.
    #[must_use]
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            active: true,
        }
    }
}

/// Full GitHub App Manifest representation according to GitHub's manifest creation flow.
///
/// See: <https://docs.github.com/en/apps/sharing-github-apps/registering-a-github-app-from-a-manifest>
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubAppManifest {
    /// The unique public name of the GitHub App.
    pub name: String,
    /// The homepage URL for the application.
    pub url: String,
    /// Webhook configuration attributes.
    pub hook_attributes: GitHubHookAttributes,
    /// Redirect URL for OAuth user authorization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_url: Option<String>,
    /// Array of allowable callback URLs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub callback_urls: Vec<String>,
    /// Post-installation setup URL where GitHub redirects users after installation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup_url: Option<String>,
    /// Human-readable description of the application.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the app is public (installable by any account) or private (single-tenant).
    #[serde(default)]
    pub public: bool,
    /// Required permissions granted upon installation.
    pub default_permissions: GitHubAppPermissions,
    /// Webhook events subscribed to.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_events: Vec<String>,
}

impl GitHubAppManifest {
    /// Validates the manifest fields and permission constraints.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError::AppConfiguration`] if validation fails.
    pub fn validate(&self) -> Result<(), GitHubExtractorError> {
        if self.name.trim().is_empty() {
            return Err(GitHubExtractorError::AppConfiguration(
                "GitHub App name cannot be empty".to_string(),
            ));
        }

        Url::parse(&self.url).map_err(|e| {
            GitHubExtractorError::AppConfiguration(format!("Invalid App URL '{}': {e}", self.url))
        })?;

        if !self.hook_attributes.url.is_empty() {
            Url::parse(&self.hook_attributes.url).map_err(|e| {
                GitHubExtractorError::AppConfiguration(format!(
                    "Invalid webhook URL '{}': {e}",
                    self.hook_attributes.url
                ))
            })?;
        }

        if let Some(redirect) = &self.redirect_url {
            Url::parse(redirect).map_err(|e| {
                GitHubExtractorError::AppConfiguration(format!(
                    "Invalid redirect URL '{redirect}': {e}"
                ))
            })?;
        }

        for cb in &self.callback_urls {
            Url::parse(cb).map_err(|e| {
                GitHubExtractorError::AppConfiguration(format!("Invalid callback URL '{cb}': {e}"))
            })?;
        }

        if let Some(setup) = &self.setup_url {
            Url::parse(setup).map_err(|e| {
                GitHubExtractorError::AppConfiguration(format!("Invalid setup URL '{setup}': {e}"))
            })?;
        }

        self.default_permissions.validate()?;

        Ok(())
    }

    /// Serializes the manifest into a formatted JSON string.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError::AppConfiguration`] on serialization failure.
    pub fn to_json(&self) -> Result<String, GitHubExtractorError> {
        serde_json::to_string(self)
            .map_err(|e| GitHubExtractorError::AppConfiguration(e.to_string()))
    }

    /// Serializes the manifest into a pretty-printed JSON string.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError::AppConfiguration`] on serialization failure.
    pub fn to_json_pretty(&self) -> Result<String, GitHubExtractorError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| GitHubExtractorError::AppConfiguration(e.to_string()))
    }
}

/// Builder and generator for GitHub App Manifests.
#[derive(Debug, Clone)]
pub struct GitHubAppManifestGenerator {
    app_name: String,
    app_url: String,
    webhook_url: Option<String>,
    redirect_url: Option<String>,
    callback_urls: Vec<String>,
    setup_url: Option<String>,
    description: Option<String>,
    public: bool,
    events: Vec<String>,
}

impl GitHubAppManifestGenerator {
    /// Default webhook events required for tracking installations.
    pub const DEFAULT_EVENTS: [&'static str; 2] = ["installation", "installation_repositories"];

    /// Creates a new manifest generator with default settings.
    #[must_use]
    pub fn new(app_name: impl Into<String>, app_url: impl Into<String>) -> Self {
        let app_name = app_name.into();
        let app_url = app_url.into();
        let default_webhook = format!("{app_url}/api/v1/github/webhooks");
        let default_redirect = format!("{app_url}/api/v1/github/oauth/callback");
        let default_setup = format!("{app_url}/settings/integrations/github");

        Self {
            app_name,
            app_url,
            webhook_url: Some(default_webhook),
            redirect_url: Some(default_redirect.clone()),
            callback_urls: vec![default_redirect],
            setup_url: Some(default_setup),
            description: Some(
                "Universal URL & GitHub Repository -> AI-Ready Content Converter".to_string(),
            ),
            public: false,
            events: Self::DEFAULT_EVENTS
                .iter()
                .map(|&s| s.to_string())
                .collect(),
        }
    }

    /// Overrides the webhook listener URL.
    #[must_use]
    pub fn with_webhook_url(mut self, url: impl Into<String>) -> Self {
        self.webhook_url = Some(url.into());
        self
    }

    /// Disables the webhook URL if webhooks are not utilized.
    #[must_use]
    pub fn without_webhook(mut self) -> Self {
        self.webhook_url = None;
        self
    }

    /// Overrides the OAuth redirect URL.
    #[must_use]
    pub fn with_redirect_url(mut self, url: impl Into<String>) -> Self {
        let url_str = url.into();
        self.redirect_url = Some(url_str.clone());
        if !self.callback_urls.contains(&url_str) {
            self.callback_urls.push(url_str);
        }
        self
    }

    /// Adds an allowable callback URL.
    #[must_use]
    pub fn with_callback_url(mut self, url: impl Into<String>) -> Self {
        let url_str = url.into();
        if !self.callback_urls.contains(&url_str) {
            self.callback_urls.push(url_str);
        }
        self
    }

    /// Overrides the post-installation setup URL.
    #[must_use]
    pub fn with_setup_url(mut self, url: impl Into<String>) -> Self {
        self.setup_url = Some(url.into());
        self
    }

    /// Overrides the application description.
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Sets whether the app is public or single-tenant.
    #[must_use]
    pub fn with_public(mut self, public: bool) -> Self {
        self.public = public;
        self
    }

    /// Sets the subscribed webhook events.
    #[must_use]
    pub fn with_events(mut self, events: Vec<String>) -> Self {
        self.events = events;
        self
    }

    /// Generates and validates the [`GitHubAppManifest`].
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError::AppConfiguration`] if validation fails.
    pub fn generate(&self) -> Result<GitHubAppManifest, GitHubExtractorError> {
        let hook_url = self.webhook_url.clone().unwrap_or_default();
        let manifest = GitHubAppManifest {
            name: self.app_name.clone(),
            url: self.app_url.clone(),
            hook_attributes: GitHubHookAttributes {
                url: hook_url,
                active: self.webhook_url.is_some(),
            },
            redirect_url: self.redirect_url.clone(),
            callback_urls: self.callback_urls.clone(),
            setup_url: self.setup_url.clone(),
            description: self.description.clone(),
            public: self.public,
            default_permissions: GitHubAppPermissions::minimal_read_only(),
            default_events: self.events.clone(),
        };

        manifest.validate()?;
        Ok(manifest)
    }

    /// Produces the 1-click manifest creation URL for GitHub.
    ///
    /// Users navigating to this URL will be presented with GitHub's pre-filled
    /// "Register GitHub App" screen.
    ///
    /// # Arguments
    /// * `github_base` - Optional GitHub base URL (defaults to `"https://github.com"`).
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError::AppConfiguration`] if URL construction fails.
    pub fn to_manifest_create_url(
        &self,
        github_base: Option<&str>,
    ) -> Result<String, GitHubExtractorError> {
        let manifest = self.generate()?;
        let json_str = manifest.to_json()?;
        let encoded_manifest: String =
            url::form_urlencoded::byte_serialize(json_str.as_bytes()).collect();

        let base = github_base
            .unwrap_or("https://github.com")
            .trim_end_matches('/');
        Ok(format!(
            "{base}/settings/apps/new?manifest={encoded_manifest}"
        ))
    }
}

/// Runtime configuration for authenticating as a GitHub App.
///
/// Contains the necessary credentials to generate JWTs, exchange installation tokens,
/// and verify webhook signatures.
///
/// Sensitive fields ([`client_secret`], [`private_key_pem`], [`webhook_secret`])
/// are strictly redacted in [`fmt::Debug`] and [`fmt::Display`].
#[derive(Clone)]
pub struct GitHubAppConfig {
    /// Numeric GitHub App identifier.
    pub app_id: u64,
    /// Public Client ID of the GitHub App.
    pub client_id: String,
    /// OAuth Client Secret (sensitive).
    client_secret: String,
    /// RSA Private Key in PEM format (sensitive).
    private_key_pem: String,
    /// Webhook HMAC-SHA256 Secret (sensitive).
    webhook_secret: Option<String>,
    /// Human-readable application name.
    pub app_name: Option<String>,
    /// URL-friendly application slug.
    pub app_slug: Option<String>,
    /// Base GitHub API URL (defaults to `"https://api.github.com"`).
    pub base_api_url: String,
    /// Base GitHub HTML URL (defaults to `"https://github.com"`).
    pub html_url: String,
}

impl fmt::Debug for GitHubAppConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GitHubAppConfig")
            .field("app_id", &self.app_id)
            .field("client_id", &self.client_id)
            .field("client_secret", &"***[REDACTED_SECRET]***")
            .field("private_key_pem", &"***[REDACTED_RSA_KEY]***")
            .field(
                "webhook_secret",
                &self
                    .webhook_secret
                    .as_ref()
                    .map(|_| "***[REDACTED_SECRET]***"),
            )
            .field("app_name", &self.app_name)
            .field("app_slug", &self.app_slug)
            .field("base_api_url", &self.base_api_url)
            .field("html_url", &self.html_url)
            .finish()
    }
}

impl fmt::Display for GitHubAppConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GitHubAppConfig(app_id={}, client_id={}, name={:?}, base_api={})",
            self.app_id, self.client_id, self.app_name, self.base_api_url
        )
    }
}

impl GitHubAppConfig {
    /// Creates a new builder for [`GitHubAppConfig`].
    #[must_use]
    pub fn builder() -> GitHubAppConfigBuilder {
        GitHubAppConfigBuilder::default()
    }

    /// Accessor for the OAuth client secret.
    #[must_use]
    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }

    /// Accessor for the RSA private key in PEM format.
    #[must_use]
    pub fn private_key_pem(&self) -> &str {
        &self.private_key_pem
    }

    /// Accessor for the webhook secret, if configured.
    #[must_use]
    pub fn webhook_secret(&self) -> Option<&str> {
        self.webhook_secret.as_deref()
    }

    /// Returns non-sensitive public metadata safe for status reporting or health checks.
    #[must_use]
    pub fn public_metadata(&self) -> GitHubAppPublicMetadata {
        GitHubAppPublicMetadata {
            app_id: self.app_id,
            client_id: self.client_id.clone(),
            app_name: self.app_name.clone(),
            app_slug: self.app_slug.clone(),
            has_private_key: !self.private_key_pem.is_empty(),
            has_webhook_secret: self.webhook_secret.is_some(),
            base_api_url: self.base_api_url.clone(),
            html_url: self.html_url.clone(),
        }
    }

    /// Validates the configuration values and cryptographic key formatting.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError::AppConfiguration`] if any validation rule is violated.
    pub fn validate(&self) -> Result<(), GitHubExtractorError> {
        if self.app_id == 0 {
            return Err(GitHubExtractorError::AppConfiguration(
                "GitHub App ID must be a non-zero positive integer".to_string(),
            ));
        }

        if self.client_id.trim().is_empty() {
            return Err(GitHubExtractorError::AppConfiguration(
                "GitHub App Client ID cannot be empty".to_string(),
            ));
        }

        if self.client_secret.trim().is_empty() {
            return Err(GitHubExtractorError::AppConfiguration(
                "GitHub App Client Secret cannot be empty".to_string(),
            ));
        }

        // Validate RSA Private Key PEM syntax
        let pem = self.private_key_pem.trim();
        let valid_pkcs1 = pem.starts_with("-----BEGIN RSA PRIVATE KEY-----")
            && pem.ends_with("-----END RSA PRIVATE KEY-----");
        let valid_pkcs8 = pem.starts_with("-----BEGIN PRIVATE KEY-----")
            && pem.ends_with("-----END PRIVATE KEY-----");

        if !valid_pkcs1 && !valid_pkcs8 {
            return Err(GitHubExtractorError::AppConfiguration(
                "Invalid private key PEM format: must contain valid PKCS#1 ('-----BEGIN RSA PRIVATE KEY-----') or PKCS#8 ('-----BEGIN PRIVATE KEY-----') markers".to_string(),
            ));
        }

        if let Some(ws) = &self.webhook_secret {
            if ws.trim().is_empty() {
                return Err(GitHubExtractorError::AppConfiguration(
                    "Configured webhook secret cannot be empty string".to_string(),
                ));
            }
        }

        Url::parse(&self.base_api_url).map_err(|e| {
            GitHubExtractorError::AppConfiguration(format!(
                "Invalid base API URL '{}': {e}",
                self.base_api_url
            ))
        })?;

        Url::parse(&self.html_url).map_err(|e| {
            GitHubExtractorError::AppConfiguration(format!(
                "Invalid HTML URL '{}': {e}",
                self.html_url
            ))
        })?;

        Ok(())
    }

    /// Loads the configuration from standard environment variables:
    /// - `GITHUB_APP_ID`: Required (u64)
    /// - `GITHUB_APP_CLIENT_ID`: Required
    /// - `GITHUB_APP_CLIENT_SECRET`: Required
    /// - `GITHUB_APP_PRIVATE_KEY_PEM`: Required (PEM string or path to `.pem` file)
    /// - `GITHUB_APP_WEBHOOK_SECRET`: Optional
    /// - `GITHUB_APP_NAME`: Optional
    /// - `GITHUB_APP_SLUG`: Optional
    /// - `GITHUB_API_BASE_URL`: Optional (default `"https://api.github.com"`)
    /// - `GITHUB_HTML_URL`: Optional (default `"https://github.com"`)
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError::AppConfiguration`] if required variables are missing
    /// or validation fails.
    pub fn from_env() -> Result<Self, GitHubExtractorError> {
        let app_id_str = std::env::var("GITHUB_APP_ID").map_err(|_| {
            GitHubExtractorError::AppConfiguration(
                "Missing required environment variable 'GITHUB_APP_ID'".to_string(),
            )
        })?;

        let app_id = app_id_str.parse::<u64>().map_err(|e| {
            GitHubExtractorError::AppConfiguration(format!(
                "Invalid GITHUB_APP_ID '{app_id_str}': must be a positive integer ({e})"
            ))
        })?;

        let client_id = std::env::var("GITHUB_APP_CLIENT_ID").map_err(|_| {
            GitHubExtractorError::AppConfiguration(
                "Missing required environment variable 'GITHUB_APP_CLIENT_ID'".to_string(),
            )
        })?;

        let client_secret = std::env::var("GITHUB_APP_CLIENT_SECRET").map_err(|_| {
            GitHubExtractorError::AppConfiguration(
                "Missing required environment variable 'GITHUB_APP_CLIENT_SECRET'".to_string(),
            )
        })?;

        let raw_key = std::env::var("GITHUB_APP_PRIVATE_KEY_PEM").map_err(|_| {
            GitHubExtractorError::AppConfiguration(
                "Missing required environment variable 'GITHUB_APP_PRIVATE_KEY_PEM'".to_string(),
            )
        })?;

        // If the variable points to an existing file, read the contents from disk
        let private_key_pem = if Path::new(&raw_key).exists() && raw_key.ends_with(".pem") {
            std::fs::read_to_string(&raw_key).map_err(|e| {
                GitHubExtractorError::AppConfiguration(format!(
                    "Failed to read private key from file '{raw_key}': {e}"
                ))
            })?
        } else {
            raw_key
        };

        let webhook_secret = std::env::var("GITHUB_APP_WEBHOOK_SECRET").ok();
        let app_name = std::env::var("GITHUB_APP_NAME").ok();
        let app_slug = std::env::var("GITHUB_APP_SLUG").ok();

        let base_api_url = std::env::var("GITHUB_API_BASE_URL")
            .unwrap_or_else(|_| "https://api.github.com".to_string());
        let html_url =
            std::env::var("GITHUB_HTML_URL").unwrap_or_else(|_| "https://github.com".to_string());

        let config = Self {
            app_id,
            client_id,
            client_secret,
            private_key_pem,
            webhook_secret,
            app_name,
            app_slug,
            base_api_url,
            html_url,
        };

        config.validate()?;
        Ok(config)
    }
}

/// Fluent builder for [`GitHubAppConfig`].
#[derive(Debug, Clone, Default)]
pub struct GitHubAppConfigBuilder {
    app_id: Option<u64>,
    client_id: Option<String>,
    client_secret: Option<String>,
    private_key_pem: Option<String>,
    webhook_secret: Option<String>,
    app_name: Option<String>,
    app_slug: Option<String>,
    base_api_url: Option<String>,
    html_url: Option<String>,
}

impl GitHubAppConfigBuilder {
    /// Sets the GitHub App ID.
    #[must_use]
    pub fn app_id(mut self, id: u64) -> Self {
        self.app_id = Some(id);
        self
    }

    /// Sets the Client ID.
    #[must_use]
    pub fn client_id(mut self, id: impl Into<String>) -> Self {
        self.client_id = Some(id.into());
        self
    }

    /// Sets the Client Secret.
    #[must_use]
    pub fn client_secret(mut self, secret: impl Into<String>) -> Self {
        self.client_secret = Some(secret.into());
        self
    }

    /// Sets the RSA Private Key PEM.
    #[must_use]
    pub fn private_key_pem(mut self, pem: impl Into<String>) -> Self {
        self.private_key_pem = Some(pem.into());
        self
    }

    /// Sets the Webhook Secret.
    #[must_use]
    pub fn webhook_secret(mut self, secret: impl Into<String>) -> Self {
        self.webhook_secret = Some(secret.into());
        self
    }

    /// Sets the App Name.
    #[must_use]
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = Some(name.into());
        self
    }

    /// Sets the App Slug.
    #[must_use]
    pub fn app_slug(mut self, slug: impl Into<String>) -> Self {
        self.app_slug = Some(slug.into());
        self
    }

    /// Sets the base API URL.
    #[must_use]
    pub fn base_api_url(mut self, url: impl Into<String>) -> Self {
        self.base_api_url = Some(url.into());
        self
    }

    /// Sets the base HTML URL.
    #[must_use]
    pub fn html_url(mut self, url: impl Into<String>) -> Self {
        self.html_url = Some(url.into());
        self
    }

    /// Builds and validates the [`GitHubAppConfig`].
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError::AppConfiguration`] if required fields are missing
    /// or validation fails.
    pub fn build(self) -> Result<GitHubAppConfig, GitHubExtractorError> {
        let app_id = self.app_id.ok_or_else(|| {
            GitHubExtractorError::AppConfiguration("Missing required field 'app_id'".to_string())
        })?;

        let client_id = self.client_id.ok_or_else(|| {
            GitHubExtractorError::AppConfiguration("Missing required field 'client_id'".to_string())
        })?;

        let client_secret = self.client_secret.ok_or_else(|| {
            GitHubExtractorError::AppConfiguration(
                "Missing required field 'client_secret'".to_string(),
            )
        })?;

        let private_key_pem = self.private_key_pem.ok_or_else(|| {
            GitHubExtractorError::AppConfiguration(
                "Missing required field 'private_key_pem'".to_string(),
            )
        })?;

        let base_api_url = self
            .base_api_url
            .unwrap_or_else(|| "https://api.github.com".to_string());
        let html_url = self
            .html_url
            .unwrap_or_else(|| "https://github.com".to_string());

        let config = GitHubAppConfig {
            app_id,
            client_id,
            client_secret,
            private_key_pem,
            webhook_secret: self.webhook_secret,
            app_name: self.app_name,
            app_slug: self.app_slug,
            base_api_url,
            html_url,
        };

        config.validate()?;
        Ok(config)
    }
}

/// Public, unredacted metadata for status reporting and health endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubAppPublicMetadata {
    /// GitHub App ID.
    pub app_id: u64,
    /// Public Client ID.
    pub client_id: String,
    /// Optional application name.
    pub app_name: Option<String>,
    /// Optional application slug.
    pub app_slug: Option<String>,
    /// True if an RSA private key is configured.
    pub has_private_key: bool,
    /// True if a webhook secret is configured.
    pub has_webhook_secret: bool,
    /// Base API URL.
    pub base_api_url: String,
    /// Base HTML URL.
    pub html_url: String,
}
