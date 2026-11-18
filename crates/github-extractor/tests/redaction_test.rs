//! Integration tests for secret and token redaction (`crates/github-extractor`).

use chrono::{TimeZone, Utc};
use std::collections::HashMap;
use web_repo_to_ai_github_extractor::{
    redact_secrets, GitHubAppConfig, GitHubExtractorError, GitHubInstallationToken,
    GitHubOAuthTokenResponse, RepositorySelection, StoredTokenRecord,
};

#[test]
fn test_github_extractor_redact_secrets_all_variants() {
    let tokens = [
        ("ghs_0123456789abcdef0123456789abcdef0123", "installation"),
        ("ghu_0123456789abcdef0123456789abcdef0123", "user"),
        ("gho_0123456789abcdef0123456789abcdef0123", "oauth"),
        ("ghr_0123456789abcdef0123456789abcdef0123", "refresh"),
        ("ghp_0123456789abcdef0123456789abcdef0123", "pat"),
        (
            "github_pat_11AAAAAAA0123456789012345678901234567890123456789012345678901234567890123456789012",
            "fine-grained",
        ),
    ];

    for (token, kind) in tokens {
        let msg = format!("Failed to authenticate with {kind} token {token} on GitHub API");
        let sanitized = redact_secrets(&msg);
        assert!(
            !sanitized.contains(token),
            "Token {token} of kind {kind} was not redacted"
        );
        assert!(
            sanitized.contains("[REDACTED_GITHUB_TOKEN]"),
            "Expected [REDACTED_GITHUB_TOKEN] in '{sanitized}'"
        );
    }
}

#[test]
fn test_github_extractor_auth_headers_and_urls() {
    // Bearer header
    let bearer = "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.customjwt";
    assert_eq!(
        redact_secrets(bearer),
        "Authorization: Bearer [REDACTED_TOKEN]"
    );

    // Token header
    let token_hdr = "Authorization: token ghs_0123456789abcdef0123456789abcdef0123";
    assert_eq!(
        redact_secrets(token_hdr),
        "Authorization: token [REDACTED_GITHUB_TOKEN]"
    );

    // Basic auth header
    let basic_hdr = "Authorization: Basic dXNlcm5hbWU6cGFzc3dvcmQ=";
    assert_eq!(
        redact_secrets(basic_hdr),
        "Authorization: Basic [REDACTED_TOKEN]"
    );

    // URL with embedded token
    let url_token =
        "git clone https://ghs_0123456789abcdef0123456789abcdef0123@github.com/org/repo.git";
    assert_eq!(
        redact_secrets(url_token),
        "git clone https://[REDACTED_GITHUB_TOKEN]@github.com/org/repo.git"
    );

    // URL with user:pass
    let url_userpass = "https://admin:superSecret123@github.com/org/repo.git";
    assert_eq!(
        redact_secrets(url_userpass),
        "https://admin:[REDACTED]@github.com/org/repo.git"
    );

    // URL query string secret
    let url_query = "https://api.github.com/repos/org/repo?access_token=ghs_0123456789abcdef0123456789abcdef0123&page=1";
    assert_eq!(
        redact_secrets(url_query),
        "https://api.github.com/repos/org/repo?access_token=[REDACTED]&page=1"
    );
}

#[test]
fn test_github_extractor_private_keys_and_signatures() {
    let rsa =
        "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA0Y...\n-----END RSA PRIVATE KEY-----";
    assert_eq!(redact_secrets(rsa), "[REDACTED_PRIVATE_KEY]");

    let sig = "X-Hub-Signature-256: sha256=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
    assert_eq!(
        redact_secrets(sig),
        "X-Hub-Signature-256: sha256=[REDACTED_SIGNATURE]"
    );
}

#[test]
fn test_github_extractor_error_sanitization_methods() {
    let raw_token = "ghs_9876543210fedcba9876543210fedcba9876";

    // RequestFailed
    let err = GitHubExtractorError::RequestFailed(format!(
        "Connection refused to https://api.github.com with token {raw_token}"
    ));
    let sanitized_err = err.sanitized();
    assert!(!sanitized_err.to_string().contains(raw_token));
    assert!(sanitized_err
        .to_string()
        .contains("[REDACTED_GITHUB_TOKEN]"));
    assert!(!err.to_sanitized_string().contains(raw_token));

    // HttpError
    let http_err = GitHubExtractorError::HttpError {
        status: 500,
        message: format!("Upstream crashed while validating {raw_token}"),
    };
    assert!(!http_err.sanitized().to_string().contains(raw_token));
    assert!(http_err
        .sanitized()
        .to_string()
        .contains("[REDACTED_GITHUB_TOKEN]"));

    // ResourceNotFound
    let not_found_err = GitHubExtractorError::ResourceNotFound {
        url: format!("https://api.github.com/repos?token={raw_token}"),
        message: format!("Not found with bearer {raw_token}"),
    };
    assert!(!not_found_err.sanitized().to_string().contains(raw_token));

    // RateLimitExceeded
    let rl_err = GitHubExtractorError::RateLimitExceeded {
        limit: 5000,
        reset_at: Utc.timestamp_opt(1700000000, 0).unwrap(),
        message: format!("Rate limit hit for key {raw_token}"),
    };
    assert!(!rl_err.sanitized().to_string().contains(raw_token));
}

#[test]
fn test_stored_token_record_masks_token_in_debug_and_display() {
    let secret = "ghs_SUPER_SECRET_INSTALLATION_TOKEN_VALUE_12345";
    let record =
        StoredTokenRecord::new(secret, Utc.timestamp_opt(1800000000, 0).unwrap(), "octocat");

    let debug_str = format!("{record:?}");
    let display_str = format!("{record}");

    assert!(!debug_str.contains(secret));
    assert!(debug_str.contains("***[REDACTED_TOKEN]***"));
    assert!(!display_str.contains(secret));
    assert!(display_str.contains("octocat"));
}

#[test]
fn test_installation_token_struct_masks_in_debug() {
    let secret = "ghs_SECRET_INSTALLATION_ACCESS_TOKEN_45678";
    let token = GitHubInstallationToken {
        token: secret.to_string(),
        expires_at: Utc.timestamp_opt(1800000000, 0).unwrap(),
        permissions: HashMap::new(),
        repository_selection: RepositorySelection::Selected,
        repositories: None,
    };

    let debug_str = format!("{token:?}");
    assert!(!debug_str.contains(secret));
    assert!(debug_str.contains("***[REDACTED_TOKEN]***"));
}

#[test]
fn test_oauth_token_response_masks_in_debug_and_display() {
    let access = "gho_SECRET_OAUTH_ACCESS_TOKEN_ABCDEF";
    let refresh = "ghr_SECRET_OAUTH_REFRESH_TOKEN_XYZ123";

    let resp = GitHubOAuthTokenResponse {
        access_token: access.to_string(),
        token_type: "bearer".to_string(),
        scope: "read:user".to_string(),
        expires_in: Some(28800),
        refresh_token: Some(refresh.to_string()),
        refresh_token_expires_in: Some(15811200),
    };

    let debug_str = format!("{resp:?}");
    let display_str = format!("{resp}");

    assert!(!debug_str.contains(access));
    assert!(!debug_str.contains(refresh));
    assert!(debug_str.contains("***[REDACTED_TOKEN]***"));

    assert!(!display_str.contains(access));
    assert!(!display_str.contains(refresh));
    assert!(display_str.contains("bearer"));
}

#[test]
fn test_app_config_masks_secrets_in_debug_and_display() {
    let config = GitHubAppConfig::builder()
        .app_id(12345)
        .client_id("Iv1.test_client_id")
        .client_secret("super_sensitive_client_secret_xyz")
        .private_key_pem(
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA0Y...\n-----END RSA PRIVATE KEY-----",
        )
        .webhook_secret("super_sensitive_webhook_hmac_secret")
        .build()
        .expect("Valid config");

    let debug_str = format!("{config:?}");
    let display_str = format!("{config}");

    assert!(!debug_str.contains("super_sensitive_client_secret_xyz"));
    assert!(!debug_str.contains("super_sensitive_webhook_hmac_secret"));
    assert!(!debug_str.contains("MIIEowIBAAKCAQEA0Y"));

    assert!(debug_str.contains("***[REDACTED_SECRET]***"));
    assert!(debug_str.contains("***[REDACTED_RSA_KEY]***"));

    assert!(!display_str.contains("super_sensitive_client_secret_xyz"));
    assert!(!display_str.contains("super_sensitive_webhook_hmac_secret"));
    assert!(!display_str.contains("MIIEowIBAAKCAQEA0Y"));
}
