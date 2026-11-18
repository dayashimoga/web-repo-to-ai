//! Integration test suite for comprehensive secret and token leakage prevention (Subphase 6.4.3).
//!
//! Validates that no sensitive tokens, private keys, API secrets, or credentials leak
//! into telemetry, logs, JSON objects, or writer streams across any representation.

use std::io::Write;
use web_repo_to_ai_observability::{
    is_sensitive_key, redact_json_value, redact_sensitive_text, RedactingWriter, RedactionEngine,
    REDACTED_API_KEY, REDACTED_AWS_KEY, REDACTED_GITHUB_TOKEN, REDACTED_PRIVATE_KEY,
    REDACTED_SECRET, REDACTED_SIGNATURE, REDACTED_TOKEN,
};

/// Test Case 1: Comprehensive redaction of all known GitHub token formats.
///
/// Validates personal, installation, user, oauth, refresh, and fine-grained PAT tokens.
#[test]
fn test_all_github_token_formats_leakage_prevention() {
    let test_tokens = [
        ("ghp_1234567890abcdefghijklmnopqrstuvwxyz", "Personal Access Token (classic)"),
        ("ghs_ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789", "App Installation Access Token"),
        ("ghu_9876543210zyxwvutsrqponmlkjihgfedcba", "User-to-Server Token"),
        ("gho_012345678901234567890123456789012345", "OAuth Access Token"),
        ("ghr_abcdef0123456789abcdef0123456789abcd", "App Refresh Token"),
        (
            "github_pat_11AAAAAAA0123456789012345678901234567890123456789012345678901234567890123456789012",
            "Fine-Grained Personal Access Token",
        ),
    ];

    for (token, description) in test_tokens {
        // Plain text embedding
        let plain_msg = format!("Authenticating operation with {description}: {token}");
        let sanitized = redact_sensitive_text(&plain_msg);
        assert!(
            !sanitized.contains(token),
            "Token {token} ({description}) was leaked in sanitized text!"
        );
        assert!(
            sanitized.contains(REDACTED_GITHUB_TOKEN),
            "Expected {REDACTED_GITHUB_TOKEN} in '{sanitized}'"
        );

        // URL userinfo embedding (https://token@github.com)
        let url_msg = format!("Fetching git tree from https://{token}@github.com/org/repo.git");
        let sanitized_url = redact_sensitive_text(&url_msg);
        assert!(!sanitized_url.contains(token));
        assert!(sanitized_url.contains(REDACTED_GITHUB_TOKEN));

        // URL query parameter embedding (?access_token=...)
        let query_msg = format!("GET /repos/org/repo?access_token={token}&page=1");
        let sanitized_query = redact_sensitive_text(&query_msg);
        assert!(!sanitized_query.contains(token));
        assert!(sanitized_query.contains("[REDACTED]"));
    }
}

/// Test Case 2: Multi-line and single-line RSA, EC, and PKCS#8 private keys.
///
/// Verifies that PEM private keys are completely replaced with REDACTED_PRIVATE_KEY.
#[test]
fn test_private_key_leakage_prevention() {
    let rsa_key = "\
-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEA0Yxyz123456789abcdefghijklmnopqrstuvwxyz
...confidential...private...key...bytes...
-----END RSA PRIVATE KEY-----";

    let pkcs8_key = "\
-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQC5
-----END PRIVATE KEY-----";

    let ec_key = "\
-----BEGIN EC PRIVATE KEY-----
MHcCAQEEII85r30eD123456789abcdef
-----END EC PRIVATE KEY-----";

    for key_pem in [rsa_key, pkcs8_key, ec_key] {
        let text =
            format!("Loaded certificate pair with key:\n{key_pem}\nKey initialization done.");
        let sanitized = redact_sensitive_text(&text);
        assert!(
            !sanitized.contains("BEGIN"),
            "Private key PEM delimiter was not redacted!"
        );
        assert!(
            !sanitized.contains("END"),
            "Private key PEM delimiter was not redacted!"
        );
        assert!(
            sanitized.contains(REDACTED_PRIVATE_KEY),
            "Expected {REDACTED_PRIVATE_KEY} in '{sanitized}'"
        );
    }
}

/// Test Case 3: HTTP Authorization headers, HMAC signatures, and provider API keys.
#[test]
fn test_headers_signatures_and_provider_keys_leakage_prevention() {
    let bearer = "Authorization: Bearer secret_bearer_token_string_12345";
    assert_eq!(
        redact_sensitive_text(bearer),
        format!("Authorization: Bearer {REDACTED_TOKEN}")
    );

    let token_hdr = "Authorization: token ghs_1234567890abcdef1234567890abcdef";
    assert_eq!(
        redact_sensitive_text(token_hdr),
        format!("Authorization: token {REDACTED_GITHUB_TOKEN}")
    );

    let basic = "Authorization: Basic dXNlcm5hbWU6cGFzc3dvcmRfZXhhbXBsZQ==";
    assert_eq!(
        redact_sensitive_text(basic),
        format!("Authorization: Basic {REDACTED_TOKEN}")
    );

    let sig = "X-Hub-Signature-256: sha256=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
    assert_eq!(
        redact_sensitive_text(sig),
        format!("X-Hub-Signature-256: sha256={REDACTED_SIGNATURE}")
    );

    let openai = "OpenAI key: sk-abcdefghijklmnopqrstuvwxyz1234567890";
    assert!(redact_sensitive_text(openai).contains(REDACTED_API_KEY));

    let aws = "AWS Access Key: AKIAIOSFODNN7EXAMPLE";
    assert!(redact_sensitive_text(aws).contains(REDACTED_AWS_KEY));
}

/// Test Case 4: Recursive deep JSON redaction across sensitive object keys and nested strings.
#[test]
fn test_recursive_json_value_leakage_prevention() {
    let mut payload = serde_json::json!({
        "status": "success",
        "tenant_id": "tenant-42",
        "config": {
            "client_secret": "raw_oauth_client_secret_xyz123",
            "webhook_secret": "raw_webhook_hmac_secret_456",
            "password": "super_secret_db_password"
        },
        "session": {
            "credentials": {
                "raw_blob": "super_secret_blob_data"
            },
            "auth_profile": {
                "access_token": "ghs_SecretInstallationToken1234567890",
                "refresh_token": "ghr_SecretRefreshToken12345678901234"
            }
        },
        "logs": [
            "Calling API endpoint https://ghs_SecretInstallationToken1234567890@api.github.com/repos",
            "Authenticated with Bearer secret_jwt_token_payload_xyz"
        ]
    });

    redact_json_value(&mut payload);

    // Assert sensitive object keys are replaced with REDACTED_SECRET
    assert_eq!(payload["config"]["client_secret"], REDACTED_SECRET);
    assert_eq!(payload["config"]["webhook_secret"], REDACTED_SECRET);
    assert_eq!(payload["config"]["password"], REDACTED_SECRET);
    assert_eq!(payload["session"]["credentials"], REDACTED_SECRET);
    assert_eq!(
        payload["session"]["auth_profile"]["access_token"],
        REDACTED_SECRET
    );
    assert_eq!(
        payload["session"]["auth_profile"]["refresh_token"],
        REDACTED_SECRET
    );

    // Non-sensitive keys remain intact
    assert_eq!(payload["status"], "success");
    assert_eq!(payload["tenant_id"], "tenant-42");

    // Nested array string entries are scrubbed
    let log0 = payload["logs"][0].as_str().unwrap();
    assert!(!log0.contains("ghs_SecretInstallationToken1234567890"));
    assert!(log0.contains(REDACTED_GITHUB_TOKEN));

    let log1 = payload["logs"][1].as_str().unwrap();
    assert!(!log1.contains("secret_jwt_token_payload_xyz"));
    assert!(log1.contains(REDACTED_TOKEN));
}

/// Test Case 5: Sensitive key detection heuristics.
#[test]
fn test_is_sensitive_key_comprehensive_taxonomy() {
    let positive_keys = [
        "token",
        "access_token",
        "refresh_token",
        "github_token",
        "bearer_token",
        "api_key",
        "apikey",
        "secret",
        "client_secret",
        "webhook_secret",
        "password",
        "user_password",
        "passwd",
        "signing_key",
        "private_key",
        "credential",
        "credentials",
        "authorization",
        "encryption_key",
    ];

    for key in positive_keys {
        assert!(
            is_sensitive_key(key),
            "Key '{key}' should be classified as sensitive"
        );
        assert!(
            is_sensitive_key(&key.to_uppercase()),
            "Key '{key}' in uppercase should be classified as sensitive"
        );
    }

    let negative_keys = [
        "username",
        "user_id",
        "email",
        "status",
        "created_at",
        "installation_id",
        "repository_name",
        "full_name",
        "branch",
        "sha",
        "tree_sha",
        "url",
    ];

    for key in negative_keys {
        assert!(
            !is_sensitive_key(key),
            "Key '{key}' should NOT be classified as sensitive"
        );
    }
}

/// Test Case 6: Stream-based RedactingWriter handling chunked and buffered writes.
#[test]
fn test_redacting_writer_stream_handling() {
    let mut sink = Vec::new();
    {
        let mut writer = RedactingWriter::new(&mut sink);

        // Write line containing GitHub installation token
        writeln!(
            writer,
            "INFO: Acquired installation token: ghs_SecretStreamToken1234567890"
        )
        .unwrap();

        // Write line with embedded URL token
        writeln!(
            writer,
            "DEBUG: Fetching https://ghs_UrlStreamToken1234567890@github.com/repo.git"
        )
        .unwrap();

        writer.flush().unwrap();
    }

    let output = String::from_utf8(sink).expect("valid utf-8 output");
    assert!(
        !output.contains("ghs_SecretStreamToken1234567890"),
        "Stream output leaked ghs_SecretStreamToken1234567890!"
    );
    assert!(
        !output.contains("ghs_UrlStreamToken1234567890"),
        "Stream output leaked ghs_UrlStreamToken1234567890!"
    );
    assert!(output.contains(REDACTED_GITHUB_TOKEN));
}

/// Test Case 7: RedactionEngine with registered custom secret literals and regexes.
#[test]
fn test_custom_redaction_engine_literals_and_regexes() {
    let engine = RedactionEngine::new()
        .with_literal("super_confidential_custom_shared_passphrase_999")
        .with_regex(r"\bCUSTOM_SECRET_[0-9A-F]{16}\b")
        .expect("valid custom regex");

    let text = "Operation succeeded with custom secret: super_confidential_custom_shared_passphrase_999 \
                and payload token CUSTOM_SECRET_0123456789ABCDEF and standard ghs_StandardToken1234567890";

    let redacted = engine.redact(text);

    assert!(!redacted.contains("super_confidential_custom_shared_passphrase_999"));
    assert!(!redacted.contains("CUSTOM_SECRET_0123456789ABCDEF"));
    assert!(!redacted.contains("ghs_StandardToken1234567890"));

    assert!(redacted.contains(REDACTED_SECRET));
    assert!(redacted.contains(REDACTED_GITHUB_TOKEN));
}
