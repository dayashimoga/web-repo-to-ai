//! Integration tests for secret, credential, and token redaction.

use std::io::Write;
use web_repo_to_ai_observability::{
    is_sensitive_key, redact_json_value, redact_sensitive_text, RedactingWriter, RedactionEngine,
    REDACTED_API_KEY, REDACTED_AWS_KEY, REDACTED_GITHUB_TOKEN, REDACTED_PRIVATE_KEY,
    REDACTED_SECRET, REDACTED_SIGNATURE, REDACTED_TOKEN,
};

#[test]
fn test_all_github_token_variants_redacted() {
    let tokens = [
        (
            "ghp_123456789012345678901234567890123456",
            "personal access token",
        ),
        (
            "ghs_abcdefghijklmnopqrstuvwxyz0123456789",
            "installation access token",
        ),
        (
            "ghu_9876543210zyxwvutsrqponmlkjihgfedcba",
            "user-to-server token",
        ),
        (
            "gho_000011112222333344445555666677778888",
            "oauth access token",
        ),
        (
            "ghr_fedcba9876543210fedcba98765432101234",
            "app refresh token",
        ),
        (
            "github_pat_11ABCDEFG0123456789012345678901234567890123456789012345678901234567890123456789012",
            "fine-grained personal access token",
        ),
    ];

    for (token, description) in tokens {
        let text = format!("Received {description}: {token} for operation");
        let sanitized = redact_sensitive_text(&text);
        assert!(
            !sanitized.contains(token),
            "Failed to redact token {token} ({description})"
        );
        assert!(
            sanitized.contains(REDACTED_GITHUB_TOKEN),
            "Sanitized text does not contain REDACTED_GITHUB_TOKEN: {sanitized}"
        );
    }
}

#[test]
fn test_multiple_secrets_in_single_payload() {
    let raw = "Log: token=ghs_123456789012345678901234567890123456; \
               openai=sk-abcdefghijklmnopqrstuvwxyz123456; \
               auth=Bearer eyJhbGciOiJIUzI1NiJ9.customjwt; \
               key=AKIAIOSFODNN7EXAMPLE; \
               sig=sha256=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; \
               url=https://deployer:secretPass456@git.internal.net/repo?token=ghp_999999999999999999999999999999999999&limit=10";

    let sanitized = redact_sensitive_text(raw);

    assert!(!sanitized.contains("ghs_123456789012345678901234567890123456"));
    assert!(!sanitized.contains("sk-abcdefghijklmnopqrstuvwxyz123456"));
    assert!(!sanitized.contains("AKIAIOSFODNN7EXAMPLE"));
    assert!(!sanitized.contains("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"));
    assert!(!sanitized.contains("secretPass456"));
    assert!(!sanitized.contains("ghp_999999999999999999999999999999999999"));

    assert!(sanitized.contains(REDACTED_GITHUB_TOKEN));
    assert!(sanitized.contains(REDACTED_API_KEY));
    assert!(sanitized.contains(REDACTED_AWS_KEY));
    assert!(sanitized.contains(REDACTED_SIGNATURE));
    assert!(sanitized.contains("[REDACTED]@"));
}

#[test]
fn test_multiline_pem_private_keys() {
    let rsa_pem = "\
-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEA0Yxyz...
...multi...line...base64...content...
-----END RSA PRIVATE KEY-----";

    let pkcs8_pem = "\
-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQC5
-----END PRIVATE KEY-----";

    let ec_pem = "\
-----BEGIN EC PRIVATE KEY-----
MHcCAQEEII85r30eD
-----END EC PRIVATE KEY-----";

    assert_eq!(redact_sensitive_text(rsa_pem), REDACTED_PRIVATE_KEY);
    assert_eq!(redact_sensitive_text(pkcs8_pem), REDACTED_PRIVATE_KEY);
    assert_eq!(redact_sensitive_text(ec_pem), REDACTED_PRIVATE_KEY);
}

#[test]
fn test_json_redaction_recursive() {
    let mut val = serde_json::json!({
        "status": "ok",
        "auth": {
            "installation_id": 42,
            "access_token": "ghs_111122223333444455556666777788889999",
            "refresh_token": "ghr_000011112222333344445555666677778888",
            "signing_key": "raw_symmetric_signing_secret"
        },
        "repositories": [
            {
                "name": "project",
                "clone_url": "https://ghs_111122223333444455556666777788889999@github.com/org/project.git"
            }
        ]
    });

    redact_json_value(&mut val);

    assert_eq!(val["auth"]["access_token"], REDACTED_SECRET);
    assert_eq!(val["auth"]["refresh_token"], REDACTED_SECRET);
    assert_eq!(val["auth"]["signing_key"], REDACTED_SECRET);
    assert_eq!(val["auth"]["installation_id"], 42);

    let clone_url = val["repositories"][0]["clone_url"].as_str().unwrap();
    assert!(!clone_url.contains("ghs_111122223333444455556666777788889999"));
    assert!(clone_url.contains(REDACTED_GITHUB_TOKEN));
}

#[test]
fn test_is_sensitive_key_comprehensive() {
    let sensitive_keys = [
        "token",
        "access_token",
        "refresh_token",
        "api_key",
        "apikey",
        "password",
        "user_password",
        "client_secret",
        "secret",
        "webhook_secret",
        "signing_key",
        "private_key",
        "credential",
        "credentials",
        "authorization",
        "encryption_key",
    ];

    for key in sensitive_keys {
        assert!(
            is_sensitive_key(key),
            "Key '{key}' should be recognized as sensitive"
        );
        assert!(
            is_sensitive_key(&key.to_uppercase()),
            "Uppercase key '{key}' should be recognized as sensitive"
        );
    }

    let non_sensitive_keys = [
        "id",
        "username",
        "login",
        "status",
        "created_at",
        "updated_at",
        "repo_name",
        "branch",
        "limit",
        "page",
    ];

    for key in non_sensitive_keys {
        assert!(
            !is_sensitive_key(key),
            "Key '{key}' should not be sensitive"
        );
    }
}

#[test]
fn test_redaction_engine_builder_and_dynamic_registration() {
    let mut engine = RedactionEngine::new()
        .with_literal("session_token_xyz_998877")
        .with_regex(r"\bNONCE-[A-F0-9]{8}\b")
        .expect("Valid regex");

    engine.add_literal("tenant_salt_secret_1234");

    let text = "User logged in with session_token_xyz_998877 and NONCE-A1B2C3D4 with salt tenant_salt_secret_1234";
    let redacted = engine.redact(text);

    assert!(!redacted.contains("session_token_xyz_998877"));
    assert!(!redacted.contains("NONCE-A1B2C3D4"));
    assert!(!redacted.contains("tenant_salt_secret_1234"));
    assert!(redacted.contains(REDACTED_SECRET));
}

#[test]
fn test_redacting_writer_stream_buffering() {
    let mut target = Vec::new();
    {
        let mut writer = RedactingWriter::new(&mut target);

        // Write partial line chunks
        write!(writer, "Chunk 1: using ").unwrap();
        write!(writer, "ghs_123456789012345678901234567890123456 in ").unwrap();
        writeln!(writer, "production.").unwrap();

        // Write a second line with bearer token
        writeln!(writer, "Header: Bearer eyJhbGciOiJIUzI1NiJ9.test").unwrap();
        writer.flush().unwrap();
    }

    let output = String::from_utf8(target).unwrap();
    assert!(!output.contains("ghs_123456789012345678901234567890123456"));
    assert!(!output.contains("eyJhbGciOiJIUzI1NiJ9.test"));
    assert!(output.contains(REDACTED_GITHUB_TOKEN));
    assert!(output.contains(REDACTED_TOKEN));
}
