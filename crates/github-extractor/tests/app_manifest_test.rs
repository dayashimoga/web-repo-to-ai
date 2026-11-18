//! # GitHub App Manifest & Configuration Tests
//!
//! Validates:
//! - Strict least-privilege permission validation (Rule 05: Contents & Metadata Read-only).
//! - Rejection of write and administrative permission scopes.
//! - 1-Click Manifest generation and URL encoding.
//! - Runtime configuration validation and RSA private key PEM syntax checking.
//! - Strict secret masking in Debug and Display implementations.
//! - Safe public metadata exposition.

use std::collections::HashMap;
use web_repo_to_ai_github_extractor::{
    GitHubAppConfig, GitHubAppManifestGenerator, GitHubAppPermissions, PermissionLevel,
};

// Sample dummy RSA keys for testing PEM validation (PKCS#1 and PKCS#8)
const DUMMY_PKCS1_PEM: &str = "-----BEGIN RSA PRIVATE KEY-----\n\
MIIEowIBAAKCAQEA0YqLpQ1234567890abcdefghijklmnopqrstuvwxyzABCDEF\n\
GHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/\n\
-----END RSA PRIVATE KEY-----";

const DUMMY_PKCS8_PEM: &str = "-----BEGIN PRIVATE KEY-----\n\
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDRiou123456789\n\
abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/\n\
-----END PRIVATE KEY-----";

#[test]
fn test_manifest_default_permissions_strict_read_only() {
    let perms = GitHubAppPermissions::default();
    assert_eq!(perms.contents, PermissionLevel::Read);
    assert_eq!(perms.metadata, PermissionLevel::Read);
    assert!(perms.additional_permissions.is_none());

    // Validation must succeed for minimal read-only
    assert!(perms.validate().is_ok());

    let minimal = GitHubAppPermissions::minimal_read_only();
    assert_eq!(minimal, perms);
    assert!(minimal.validate().is_ok());
}

#[test]
fn test_permission_level_serialization_and_deserialization() {
    let read_level = PermissionLevel::Read;
    assert_eq!(read_level.as_str(), "read");

    let json_serialized = serde_json::to_string(&read_level).expect("Serialization failed");
    assert_eq!(json_serialized, "\"read\"");

    let deserialized: PermissionLevel =
        serde_json::from_str("\"read\"").expect("Deserialization failed");
    assert_eq!(deserialized, PermissionLevel::Read);

    // Write and admin must be rejected by deserializer
    let write_res: Result<PermissionLevel, _> = serde_json::from_str("\"write\"");
    assert!(
        write_res.is_err(),
        "Expected 'write' permission to be rejected"
    );
    let err_msg = write_res.unwrap_err().to_string();
    assert!(
        err_msg.contains("forbidden by Rule 05"),
        "Unexpected error: {err_msg}"
    );

    let admin_res: Result<PermissionLevel, _> = serde_json::from_str("\"admin\"");
    assert!(
        admin_res.is_err(),
        "Expected 'admin' permission to be rejected"
    );

    let unknown_res: Result<PermissionLevel, _> = serde_json::from_str("\"invalid_scope\"");
    assert!(unknown_res.is_err());
}

#[test]
fn test_permissions_validation_rejects_forbidden_scopes() {
    let mut perms = GitHubAppPermissions::minimal_read_only();
    let mut additional = HashMap::new();
    additional.insert("issues".to_string(), "write".to_string());
    perms.additional_permissions = Some(additional);

    let val_err = perms
        .validate()
        .expect_err("Write permission must fail validation");
    let err_str = val_err.to_string();
    assert!(
        err_str.contains("violate Rule 05"),
        "Unexpected error: {err_str}"
    );

    // Forbidden administration scope even if read
    let mut admin_perms = GitHubAppPermissions::minimal_read_only();
    let mut admin_additional = HashMap::new();
    admin_additional.insert("administration".to_string(), "read".to_string());
    admin_perms.additional_permissions = Some(admin_additional);

    let admin_err = admin_perms
        .validate()
        .expect_err("Administration scope must be rejected");
    assert!(admin_err
        .to_string()
        .contains("Forbidden permission category"));
}

#[test]
fn test_manifest_generator_fluent_builder() {
    let generator =
        GitHubAppManifestGenerator::new("web-repo-to-ai-test", "https://app.example.com")
            .with_description("Test App Description")
            .with_public(true)
            .with_webhook_url("https://app.example.com/custom/webhook")
            .with_redirect_url("https://app.example.com/custom/oauth")
            .with_setup_url("https://app.example.com/custom/setup");

    let manifest = generator.generate().expect("Failed to generate manifest");
    assert_eq!(manifest.name, "web-repo-to-ai-test");
    assert_eq!(manifest.url, "https://app.example.com");
    assert_eq!(
        manifest.description.as_deref(),
        Some("Test App Description")
    );
    assert!(manifest.public);
    assert_eq!(
        manifest.hook_attributes.url,
        "https://app.example.com/custom/webhook"
    );
    assert!(manifest.hook_attributes.active);
    assert_eq!(
        manifest.redirect_url.as_deref(),
        Some("https://app.example.com/custom/oauth")
    );
    assert!(manifest
        .callback_urls
        .contains(&"https://app.example.com/custom/oauth".to_string()));
    assert_eq!(
        manifest.setup_url.as_deref(),
        Some("https://app.example.com/custom/setup")
    );
    assert_eq!(manifest.default_permissions.contents, PermissionLevel::Read);
    assert_eq!(manifest.default_permissions.metadata, PermissionLevel::Read);
    assert_eq!(
        manifest.default_events,
        vec![
            "installation".to_string(),
            "installation_repositories".to_string()
        ]
    );
}

#[test]
fn test_manifest_json_serialization_roundtrip() {
    let generator = GitHubAppManifestGenerator::new("web-repo-to-ai", "https://example.com");
    let manifest = generator.generate().expect("Generation failed");

    let json_compact = manifest
        .to_json()
        .expect("Compact JSON serialization failed");
    let json_pretty = manifest
        .to_json_pretty()
        .expect("Pretty JSON serialization failed");

    assert!(json_compact.contains("\"name\":\"web-repo-to-ai\""));
    assert!(json_compact.contains("\"contents\":\"read\""));
    assert!(json_compact.contains("\"metadata\":\"read\""));

    let deserialized: web_repo_to_ai_github_extractor::GitHubAppManifest =
        serde_json::from_str(&json_compact).expect("Deserialization failed");
    assert_eq!(deserialized, manifest);

    let deserialized_pretty: web_repo_to_ai_github_extractor::GitHubAppManifest =
        serde_json::from_str(&json_pretty).expect("Deserialization pretty failed");
    assert_eq!(deserialized_pretty, manifest);
}

#[test]
fn test_manifest_1click_url_generation() {
    let generator = GitHubAppManifestGenerator::new("web-repo-to-ai", "https://myhost.example.com");

    // Default GitHub.com creation URL
    let default_url = generator
        .to_manifest_create_url(None)
        .expect("URL generation failed");
    assert!(default_url.starts_with("https://github.com/settings/apps/new?manifest="));
    assert!(default_url.contains("myhost.example.com"));

    // Custom GitHub Enterprise Server creation URL
    let ghe_url = generator
        .to_manifest_create_url(Some("https://github.corp.internal/"))
        .expect("GHE URL generation failed");
    assert!(ghe_url.starts_with("https://github.corp.internal/settings/apps/new?manifest="));
}

#[test]
fn test_manifest_validation_rejects_invalid_inputs() {
    // Empty app name
    let empty_name = GitHubAppManifestGenerator::new("", "https://example.com");
    assert!(empty_name.generate().is_err());

    // Invalid app URL
    let invalid_url = GitHubAppManifestGenerator::new("app", "not-a-url");
    assert!(invalid_url.generate().is_err());

    // Invalid webhook URL
    let invalid_webhook = GitHubAppManifestGenerator::new("app", "https://example.com")
        .with_webhook_url("invalid-webhook");
    assert!(invalid_webhook.generate().is_err());
}

#[test]
fn test_app_config_builder_and_validation() {
    // Valid configuration with PKCS#1 key
    let config = GitHubAppConfig::builder()
        .app_id(123456)
        .client_id("Iv1.testclientid")
        .client_secret("super_secret_oauth_client_secret_9988")
        .private_key_pem(DUMMY_PKCS1_PEM)
        .webhook_secret("my_webhook_secret_hmac_sha256")
        .app_name("Web Repo AI")
        .app_slug("web-repo-ai")
        .build()
        .expect("Valid PKCS#1 config failed to build");

    assert_eq!(config.app_id, 123456);
    assert_eq!(config.client_id, "Iv1.testclientid");
    assert_eq!(
        config.client_secret(),
        "super_secret_oauth_client_secret_9988"
    );
    assert_eq!(config.private_key_pem(), DUMMY_PKCS1_PEM);
    assert_eq!(
        config.webhook_secret(),
        Some("my_webhook_secret_hmac_sha256")
    );
    assert_eq!(config.app_name.as_deref(), Some("Web Repo AI"));
    assert_eq!(config.app_slug.as_deref(), Some("web-repo-ai"));
    assert_eq!(config.base_api_url, "https://api.github.com");
    assert_eq!(config.html_url, "https://github.com");

    // Valid configuration with PKCS#8 key
    let pkcs8_config = GitHubAppConfig::builder()
        .app_id(654321)
        .client_id("Iv2.pkcs8client")
        .client_secret("secret")
        .private_key_pem(DUMMY_PKCS8_PEM)
        .build()
        .expect("Valid PKCS#8 config failed to build");
    assert_eq!(pkcs8_config.app_id, 654321);

    // Rejection on zero app_id
    let zero_id = GitHubAppConfig::builder()
        .app_id(0)
        .client_id("id")
        .client_secret("secret")
        .private_key_pem(DUMMY_PKCS1_PEM)
        .build();
    assert!(zero_id.is_err(), "Zero app_id must be rejected");

    // Rejection on empty client_id
    let empty_client = GitHubAppConfig::builder()
        .app_id(100)
        .client_id("   ")
        .client_secret("secret")
        .private_key_pem(DUMMY_PKCS1_PEM)
        .build();
    assert!(empty_client.is_err(), "Empty client_id must be rejected");

    // Rejection on empty client_secret
    let empty_secret = GitHubAppConfig::builder()
        .app_id(100)
        .client_id("id")
        .client_secret("")
        .private_key_pem(DUMMY_PKCS1_PEM)
        .build();
    assert!(
        empty_secret.is_err(),
        "Empty client_secret must be rejected"
    );

    // Rejection on invalid PEM headers
    let invalid_pem = GitHubAppConfig::builder()
        .app_id(100)
        .client_id("id")
        .client_secret("secret")
        .private_key_pem("NOT A PEM KEY AT ALL")
        .build();
    assert!(invalid_pem.is_err(), "Invalid PEM must be rejected");
}

#[test]
fn test_app_config_secret_masking_in_debug_and_display() {
    let secret = "VERY_CONFIDENTIAL_CLIENT_SECRET_987654";
    let key = DUMMY_PKCS1_PEM;
    let webhook_secret = "CONFIDENTIAL_WEBHOOK_SECRET_4321";

    let config = GitHubAppConfig::builder()
        .app_id(99999)
        .client_id("Iv1.visible_client_id")
        .client_secret(secret)
        .private_key_pem(key)
        .webhook_secret(webhook_secret)
        .build()
        .expect("Build failed");

    let debug_output = format!("{config:?}");
    let display_output = format!("{config}");

    // Assert that Debug representation masks secrets
    assert!(
        !debug_output.contains(secret),
        "Debug leaked client secret: {debug_output}"
    );
    assert!(
        !debug_output.contains("MIIEowIBAAKCAQEA0YqLpQ"),
        "Debug leaked private key content: {debug_output}"
    );
    assert!(
        !debug_output.contains(webhook_secret),
        "Debug leaked webhook secret: {debug_output}"
    );

    assert!(debug_output.contains("***[REDACTED_SECRET]***"));
    assert!(debug_output.contains("***[REDACTED_RSA_KEY]***"));
    assert!(debug_output.contains("Iv1.visible_client_id"));

    // Assert that Display representation does not leak secrets
    assert!(!display_output.contains(secret));
    assert!(!display_output.contains("MIIEowIBAAKCAQEA0YqLpQ"));
    assert!(!display_output.contains(webhook_secret));
}

#[test]
fn test_app_config_public_metadata() {
    let config = GitHubAppConfig::builder()
        .app_id(112233)
        .client_id("Iv1.testmeta")
        .client_secret("secret123")
        .private_key_pem(DUMMY_PKCS1_PEM)
        .webhook_secret("webhook123")
        .app_name("Status App")
        .app_slug("status-app")
        .build()
        .expect("Build failed");

    let meta = config.public_metadata();
    assert_eq!(meta.app_id, 112233);
    assert_eq!(meta.client_id, "Iv1.testmeta");
    assert_eq!(meta.app_name.as_deref(), Some("Status App"));
    assert_eq!(meta.app_slug.as_deref(), Some("status-app"));
    assert!(meta.has_private_key);
    assert!(meta.has_webhook_secret);
    assert_eq!(meta.base_api_url, "https://api.github.com");
    assert_eq!(meta.html_url, "https://github.com");

    // Serialization check
    let meta_json = serde_json::to_string(&meta).expect("Serialization failed");
    assert!(meta_json.contains("\"has_private_key\":true"));
    assert!(meta_json.contains("\"has_webhook_secret\":true"));
    assert!(!meta_json.contains("secret123"));
}
