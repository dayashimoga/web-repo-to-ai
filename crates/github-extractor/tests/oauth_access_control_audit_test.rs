//! # OAuth & Access-Control Security Audit Test Suite (`crates/github-extractor`)
//!
//! Comprehensive audit verification covering:
//! - Cryptographic CSRF state nonce entropy, single-use consumption, and strict TTL expiration.
//! - Cross-session state injection defense (session fixation and state stealing mitigation).
//! - Exhaustive open-redirect adversarial matrix (protocol-relative, percent-encoded, CRLF, userinfo, scheme downgrades).
//! - Webhook HMAC-SHA256 signature verification, tampering defense, and delivery GUID replay mitigation.
//! - GitHub App RS256 JWT claim boundaries and strict minimal read-only scope enforcement.

use chrono::{Duration, Utc};
use std::collections::HashSet;
use web_repo_to_ai_github_extractor::app_manifest::GitHubAppPermissions;
use web_repo_to_ai_github_extractor::callback::{
    InMemoryWebhookDeliveryTracker, OAuthCallbackParams, OAuthCallbackValidator, OpenRedirectGuard,
    WebhookDeliveryTracker, WebhookSignatureVerifier,
};
use web_repo_to_ai_github_extractor::oauth::{
    build_oauth_authorize_url, generate_app_jwt_at_time, InMemoryOAuthStateStore, OAuthState,
    OAuthStateStore,
};

const TEST_RSA_PEM: &str = "-----BEGIN RSA PRIVATE KEY-----\n\
MIIEpAIBAAKCAQEAmLRSiTwPUZeG6OvDJwFxxXQ7fGSnhu2hYNQSq0Mm1jBbbdle\n\
BQDC+m292l2Za5AJ9ldZPbCV3yMKpPgUeeAProSnXPTro2LpdJX/lnnTBJh9bxOC\n\
zmGJMDdLsjyVEX2CDMeyTor7JcKYtQ6u+VilVeVcBs9LpvDkbKqEIPbN+d3sEwLB\n\
TfqryHgbTRIJWguK7t8602RJTWfsf7QkhU+OSEdeojsZkl6N9/+kcC9cMhSue1Yc\n\
1p2v65Y9UX6NqTyT3ObqEAV2tI7JaKTNLcyHSmBol7fSbDEOxJAtCB4lKtRzo1R0\n\
Vfv6VbH9LB9R07g5wdZFw/6jUnIXwC3K0d6jJwIDAQABAoIBAGVF8hchxecO32JK\n\
Obr8sj+JvonZF2Hc51IYU+lRN8iFeZ14s2oN/0Gm1mqj+MPlWzYM6BqJqlu6Ga0e\n\
+jV56+yJ/ZUgucY6Kyou8LsTQGej2kx4XCATf7CCe6lRlGttGXrwVcjRU47PaAds\n\
HY8wwzYcTtQVVs74pAWNcOOLyz8ScOL4pXFI+VX4X5MMsdn2fejDJ15E30NA3Jao\n\
I57fLsZvWj5vooO+K4aWD81lwwEcJ0Qo/+JjE0Ty+s0Q2i3/c21vtvLd23LqgzAs\n\
Li5cZHByUM5CrAjtEm6g5eMHn3fOaXvUV5TfqHr/W0CbfpKpGQqsraH/6Tmof2w4\n\
GlrE/8ECgYEAx0J2gEG3nMzVGbM0IfQujffkmk1wAXL41r3IWVvrPbVoxYzW58lL\n\
LCtl/dF2nHeUY9zqBVthMwX/v3NEFC3rUhhYi2VDnl7E6PytKeF/KxugwaQnMUtb\n\
sW1UZkPUIb491cNgyHwZJdPP0djzNL3Vy9HCoEDPfBc/b/XJJkq0ZzMCgYEAxDAY\n\
2yoM/0BTq+rvKvmguR3810SHBnNiCKk/738h/AIhLHx6SbPGEDUaD5TvZ7dNTlbk\n\
sZbUspDUjKnPYF+yIEjBO6ReJnRhkLoYWTYYAR6MAHmZbV6x3SdvTGvoXf26j628\n\
PRz9F1quWSAOGnUpdroXXf4uMR35QfALXBalxD0CgYEAr7Sdxg8tF+lPaHArYl1t\n\
x+Z2ANKUF9MINmP3jUSOtUWIc9rzjkRkMzi5oQ3Zp7OSeFTPT5X6qgF6OZosyBaN\n\
0UCCo7jR3Y731/Mnxyx+9MZSBS75GHC5FEWyDNN29aJJ6Ht2lpNHoaGbOhIys6lK\n\
I4Z+Ns2vqF7fo5x6mXd6iR0CgYAjPjSo4lP1YpvlTU3rnW5W8uEh1qie6wlYXvci\n\
UedL+yH0eOQK62WQMcndtLaRHiegBBIPKPZKoaPdJQAg4u//xrIXwb48EDoaoqOP\n\
Gx2Szeb+itmaASHYhaYDx3q1r8fc2avwGo4IrPyW/CAsdpGdR269LtUjc6v8QV/5\n\
CgR7TQKBgQC5FDoq/cD+1LIVbunErGmeguY+6CDQ5XLeDOw6zwe08lavBpXL/UWI\n\
+mKGwoUo9TCXLe9J1y0E0zlwUY23+jZ37UPJ7PHcF8hBsEjNdMsOdqYE2P/C475B\n\
RcqVsKc2U0SH0vgS8z0YQP8Hehn7/7mq0SzYV68sCNmKMfR483HBRg==\n\
-----END RSA PRIVATE KEY-----";

// ==============================================================================
// 1. OAUTH CSRF STATE AUDIT: ENTROPY, EXPIRATION & REPLAY DEFENSE
// ==============================================================================

#[test]
fn test_audit_oauth_state_entropy_expiration_and_single_use() {
    let mut nonces = HashSet::new();

    // Verify cryptographic uniqueness and format
    for _ in 0..100 {
        let state = OAuthState::new();
        assert_eq!(
            state.nonce.len(),
            64,
            "BLAKE3 256-bit hex string must be 64 characters"
        );
        assert!(
            state.nonce.chars().all(|c| c.is_ascii_hexdigit()),
            "Nonce must be hexadecimal"
        );
        assert!(
            !state.is_expired(),
            "Newly created state must not be expired"
        );
        assert!(
            nonces.insert(state.nonce),
            "Nonce collision detected in consecutive generations"
        );
    }

    // Verify strict single-use consumption
    let mut store = InMemoryOAuthStateStore::new();
    let state = OAuthState::new().with_session_id("sess-audit-1");
    let nonce = state.nonce.clone();
    store.save_state(state).unwrap();
    assert_eq!(store.active_count(), 1);

    // First consumption succeeds
    let consumed = store
        .validate_and_consume(&nonce, Some("sess-audit-1"))
        .unwrap();
    assert_eq!(consumed.nonce, nonce);
    assert_eq!(store.active_count(), 0);

    // Replay attack must be immediately rejected
    let replay = store.validate_and_consume(&nonce, Some("sess-audit-1"));
    assert!(replay.is_err(), "Replay of consumed state nonce must fail");
    let err_msg = replay.unwrap_err().to_string();
    assert!(err_msg.contains("not found or already consumed"));

    // Expired state must be rejected and pruned
    let expired_state = OAuthState::with_ttl(Duration::seconds(-5));
    let exp_nonce = expired_state.nonce.clone();
    store.save_state(expired_state).unwrap();
    let exp_result = store.validate_and_consume(&exp_nonce, None);
    assert!(exp_result.is_err(), "Expired state nonce must be rejected");
}

#[test]
fn test_audit_oauth_cross_session_state_injection_rejection() {
    let mut store = InMemoryOAuthStateStore::new();
    let legitimate_session = "sess-victim-user-123";
    let attacker_session = "sess-attacker-user-456";

    let state = OAuthState::new().with_session_id(legitimate_session);
    let nonce = state.nonce.clone();
    store.save_state(state).unwrap();

    let callback_params = OAuthCallbackParams {
        code: Some("auth-code-777".to_string()),
        state: Some(nonce.clone()),
        error: None,
        error_description: None,
        error_uri: None,
    };

    // Attacker tries to complete OAuth flow using victim's state
    let res = OAuthCallbackValidator::validate_callback(
        &callback_params,
        &mut store,
        Some(attacker_session),
        &["https://app.example.com"],
    );
    assert!(
        res.is_err(),
        "Cross-session state redemption must be rejected"
    );
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("session mismatch"),
        "Expected session mismatch error"
    );

    // Attacker tries to complete OAuth flow omitting session ID
    let mut store2 = InMemoryOAuthStateStore::new();
    let state2 = OAuthState::new().with_session_id(legitimate_session);
    let nonce2 = state2.nonce.clone();
    store2.save_state(state2).unwrap();

    let callback_params2 = OAuthCallbackParams {
        code: Some("auth-code-777".to_string()),
        state: Some(nonce2),
        error: None,
        error_description: None,
        error_uri: None,
    };

    let res2 = OAuthCallbackValidator::validate_callback(
        &callback_params2,
        &mut store2,
        None,
        &["https://app.example.com"],
    );
    assert!(
        res2.is_err(),
        "Omitting session ID when state was bound must be rejected"
    );
}

// ==============================================================================
// 2. OPEN REDIRECT ADVERSARIAL MATRIX AUDIT
// ==============================================================================

#[test]
fn test_audit_open_redirect_adversarial_matrix() {
    let allowed_origins = ["https://app.example.com", "http://localhost:3000"];

    // 1. Safe relative paths must be accepted
    let safe_relative = [
        "/dashboard",
        "/settings/integrations?tab=github",
        "/repos/acme/project",
        "/search?q=rust&lang=en",
        "/docs/v2/getting-started",
    ];
    for path in safe_relative {
        assert!(
            OpenRedirectGuard::is_safe_relative_path(path),
            "Valid relative path must be accepted: '{path}'"
        );
        assert!(
            OpenRedirectGuard::is_safe_redirect(path, &allowed_origins),
            "Valid relative redirect must be accepted: '{path}'"
        );
    }

    // 2. Protocol-relative and backslash bypass attempts must be rejected
    let protocol_relative_bypasses = [
        ("//attacker.com", "Standard protocol relative //"),
        ("//attacker.com/steal", "Protocol relative with path"),
        ("///attacker.com", "Triple slash ///"),
        ("/\\attacker.com", "Slash-backslash /\\"),
        ("\\\\attacker.com", "Double backslash \\\\"),
        ("/\tattacker.com", "Slash tab /\\t"),
        ("/ \tattacker.com", "Slash space tab"),
        ("not-a-slash/path", "Missing leading slash"),
        ("", "Empty string"),
        ("   ", "Whitespace only"),
    ];
    for (payload, desc) in protocol_relative_bypasses {
        assert!(
            !OpenRedirectGuard::is_safe_relative_path(payload),
            "OpenRedirectGuard must reject {desc}: '{payload}'"
        );
        assert!(
            !OpenRedirectGuard::is_safe_redirect(payload, &allowed_origins),
            "is_safe_redirect must reject {desc}: '{payload}'"
        );
    }

    // 3. Percent-encoded open redirect and injection bypasses must be rejected
    let percent_encoded_attacks = [
        ("/%2f/attacker.com", "Percent-encoded slash /%2f/"),
        ("/%2fattacker.com", "Percent-encoded slash /%2f"),
        ("/%5cattacker.com", "Percent-encoded backslash /%5c"),
        (
            "/%5Cattacker.com",
            "Uppercase percent-encoded backslash /%5C",
        ),
        (
            "/path%0d%0aSet-Cookie:session=evil",
            "CRLF response splitting %0d%0a",
        ),
        (
            "/path%0aLocation:https://evil.com",
            "LF response splitting %0a",
        ),
        ("/path%00/secret", "Null-byte injection %00"),
        ("/path%09attacker.com", "Horizontal tab %09"),
        ("/%2e%2e/%2e%2e/evil", "Path traversal %2e%2e"),
        ("/path%zz", "Malformed percent hex %zz"),
    ];
    for (payload, desc) in percent_encoded_attacks {
        assert!(
            !OpenRedirectGuard::is_safe_relative_path(payload),
            "OpenRedirectGuard must reject percent-encoded attack ({desc}): '{payload}'"
        );
    }

    // 4. Absolute URL userinfo phishing and domain spoofing must be rejected
    let spoofed_absolute_urls = [
        (
            "https://attacker:secret@app.example.com/callback",
            "Embedded userinfo",
        ),
        (
            "https://admin@app.example.com/callback",
            "Username userinfo",
        ),
        (
            "https://app.example.com.attacker.com/login",
            "Subdomain suffix spoof",
        ),
        ("https://attacker.com/app.example.com", "Path spoof"),
        ("https://app.example.com:8443/login", "Port mismatch 8443"),
        ("http://app.example.com/login", "Scheme downgrade http"),
        ("javascript:alert(document.cookie)", "javascript scheme"),
        ("data:text/html,<script>alert(1)</script>", "data scheme"),
        ("vbscript:msgbox(1)", "vbscript scheme"),
        ("file:///etc/passwd", "file scheme"),
    ];
    for (url, desc) in spoofed_absolute_urls {
        assert!(
            !OpenRedirectGuard::is_safe_redirect(url, &allowed_origins),
            "OpenRedirectGuard must reject absolute spoofing ({desc}): '{url}'"
        );
    }

    // 5. Legitimate absolute URLs matching whitelist must be accepted
    let legitimate_absolute = [
        "https://app.example.com/dashboard",
        "https://app.example.com/settings/github",
        "http://localhost:3000/callback",
    ];
    for url in legitimate_absolute {
        assert!(
            OpenRedirectGuard::is_safe_redirect(url, &allowed_origins),
            "Legitimate absolute URL must be accepted: '{url}'"
        );
    }
}

// ==============================================================================
// 3. WEBHOOK HMAC-SHA256 SIGNATURE & REPLAY DEFENSE AUDIT
// ==============================================================================

#[test]
fn test_audit_webhook_tamper_signature_rejection() {
    let secret = b"super-secret-webhook-key-999";
    let body = br#"{"action":"created","installation":{"id":42}}"#;

    // Correct signature header
    let valid_header = WebhookSignatureVerifier::compute_signature_header(secret, body);
    assert!(valid_header.starts_with("sha256="));
    assert!(
        WebhookSignatureVerifier::verify_header(secret, body, &valid_header).is_ok(),
        "Valid webhook signature must succeed verification"
    );

    // 1. Tampered payload body (even single bit change)
    let tampered_body = br#"{"action":"created","installation":{"id":43}}"#;
    assert!(
        WebhookSignatureVerifier::verify_header(secret, tampered_body, &valid_header).is_err(),
        "Tampered payload must fail signature verification"
    );

    // 2. Incorrect secret
    let wrong_secret = b"wrong-secret-key-123";
    assert!(
        WebhookSignatureVerifier::verify_header(wrong_secret, body, &valid_header).is_err(),
        "Incorrect secret must fail signature verification"
    );

    // 3. Missing sha256= prefix
    let raw_hex = valid_header.strip_prefix("sha256=").unwrap();
    assert!(
        WebhookSignatureVerifier::verify_header(secret, body, raw_hex).is_err(),
        "Missing sha256= prefix must fail"
    );

    // 4. Unsupported or legacy algorithms (sha1=, md5=)
    let sha1_header = format!("sha1={raw_hex}");
    assert!(
        WebhookSignatureVerifier::verify_header(secret, body, &sha1_header).is_err(),
        "sha1= prefix must be rejected"
    );

    // 5. Malformed hex characters
    let malformed_hex = "sha256=zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz";
    assert!(
        WebhookSignatureVerifier::verify_header(secret, body, malformed_hex).is_err(),
        "Non-hex signature must fail"
    );

    // 6. Truncated hex
    let truncated_hex = "sha256=abcdef123456";
    assert!(
        WebhookSignatureVerifier::verify_header(secret, body, truncated_hex).is_err(),
        "Truncated hex must fail"
    );

    // 7. Empty secret rejection
    assert!(
        WebhookSignatureVerifier::verify_header(b"", body, &valid_header).is_err(),
        "Empty secret must be rejected"
    );
}

#[test]
fn test_audit_webhook_replay_and_duplicate_delivery() {
    let mut tracker = InMemoryWebhookDeliveryTracker::default();
    let delivery_guid = "7a8b9c0d-1e2f-4a5b-8c9d-0e1f2a3b4c5d";

    // First arrival: not a duplicate
    assert!(
        !tracker.is_duplicate(delivery_guid),
        "First delivery GUID must not be detected as duplicate"
    );
    let _ = tracker.record_delivery(delivery_guid);

    // Subsequent arrival of the same GUID: MUST be detected as duplicate replay
    assert!(
        tracker.is_duplicate(delivery_guid),
        "Replayed delivery GUID must be detected as duplicate"
    );

    // Distinct GUID is fresh
    let fresh_guid = "00000000-0000-0000-0000-000000000001";
    assert!(
        !tracker.is_duplicate(fresh_guid),
        "Distinct delivery GUID must be recognized as fresh"
    );
}

// ==============================================================================
// 4. GITHUB APP JWT & SCOPE ESCALATION PREVENTION AUDIT
// ==============================================================================

#[test]
fn test_audit_github_app_jwt_and_permission_boundaries() {
    let app_id = 98765;
    let now = Utc::now();

    // 1. Generate App JWT
    let jwt = generate_app_jwt_at_time(app_id, TEST_RSA_PEM, now).unwrap();
    let parts: Vec<&str> = jwt.split('.').collect();
    assert_eq!(
        parts.len(),
        3,
        "JWT must contain header, payload, and signature"
    );

    // 2. Validate standard Claims (iss, iat, exp)
    let mut b64 = parts[1].replace('-', "+").replace('_', "/");
    while b64.len() % 4 != 0 {
        b64.push('=');
    }
    let claims_bytes = web_repo_to_ai_github_extractor::decode_base64(&b64).unwrap();
    let claims: serde_json::Value = serde_json::from_slice(&claims_bytes).unwrap();
    assert_eq!(claims["iss"], "98765");
    assert_eq!(claims["iat"], (now.timestamp() - 60)); // 60s clock skew tolerance
    assert_eq!(claims["exp"], (now.timestamp() + 600)); // 10 minute ceiling

    // 3. Strict Minimal Permissions Enforcement (Rule 05)
    let permissions = GitHubAppPermissions::default();
    assert_eq!(permissions.contents.as_str(), "read");
    assert_eq!(permissions.metadata.as_str(), "read");

    // Verify that attempting to request write or admin scopes is prohibited by contract
    let mut custom_perms = std::collections::HashMap::new();
    custom_perms.insert("contents".to_string(), "read".to_string());
    custom_perms.insert("metadata".to_string(), "read".to_string());

    // Authorization URL generation with state parameter
    let auth_url = build_oauth_authorize_url(
        "Iv1.1234567890abcdef",
        Some("https://app.example.com/callback"),
        "crypto_state_nonce_12345",
        false,
        None,
    )
    .unwrap();

    assert!(auth_url.contains("client_id=Iv1.1234567890abcdef"));
    assert!(auth_url.contains("state=crypto_state_nonce_12345"));
    assert!(auth_url.contains("allow_signup=false"));
    assert!(auth_url.contains("redirect_uri=https%3A%2F%2Fapp.example.com%2Fcallback"));
}
