//! # Exhaustive Secrets Redaction, Credential Hygiene, and Dependency Audit (`services/api`)
//!
//! Validates:
//! - Zero plaintext API key persistence in storage (BLAKE3 one-way hashing only).
//! - AES-256-GCM envelope encryption nonce uniqueness and strict AAD session binding.
//! - Comprehensive credential redaction across logging, debug formatters, and telemetry.
//! - Strict `deny.toml` governance invariants, unmaintained advisory tracking, and license conformity.
//! - Workspace source tree audit verifying zero committed production credentials or private keys.

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use uuid::Uuid;
use web_repo_to_ai_core::auth::{ApiKeyScope, GeneratedApiKey};
use web_repo_to_ai_core::AppConfig;
use web_repo_to_ai_observability::{
    is_sensitive_key, redact_json_value, redact_sensitive_text, REDACTED_API_KEY, REDACTED_AWS_KEY,
    REDACTED_GITHUB_TOKEN, REDACTED_PRIVATE_KEY, REDACTED_SECRET, REDACTED_SIGNATURE,
    REDACTED_TOKEN,
};
use web_repo_to_ai_storage::{ApiKeyRepository, InMemoryApiKeyRepository};

// ==============================================================================
// 1. API KEY STORAGE & ZERO PLAINTEXT PERSISTENCE AUDIT
// ==============================================================================

#[tokio::test]
async fn test_audit_api_key_storage_zero_plaintext_leakage() {
    let storage = InMemoryApiKeyRepository::new();
    let tenant_id = Uuid::new_v4();

    let mut scopes = HashSet::new();
    scopes.insert(ApiKeyScope::Read);
    scopes.insert(ApiKeyScope::Write);

    // 1. Generate key
    let generated = GeneratedApiKey::generate(tenant_id, "Production Webhook Key", scopes, None)
        .expect("Generation must succeed");

    assert!(
        generated.plaintext.starts_with("wr_live_"),
        "Key must have canonical prefix"
    );
    assert_eq!(generated.plaintext.len(), 72); // "wr_live_" (8) + 64 hex chars

    // 2. Persist to storage
    let saved_record = storage
        .save_key(&generated.record)
        .await
        .expect("Persistence must succeed");

    // 3. Verify saved record DOES NOT contain plaintext key
    let plaintext = &generated.plaintext;
    let secret_entropy = &plaintext[8..]; // The 64 random hex characters

    assert_ne!(saved_record.key_hash, *plaintext);
    assert_ne!(saved_record.key_hash, secret_entropy);
    assert_eq!(
        saved_record.key_hash.len(),
        64,
        "Hash must be BLAKE3 256-bit hex"
    );
    assert!(
        saved_record.key_hash.chars().all(|c| c.is_ascii_hexdigit()),
        "Hash must be valid hexadecimal"
    );

    // 4. Verify lookup by ID does not reveal plaintext
    let fetched = storage
        .find_by_id(saved_record.id)
        .await
        .expect("Query must succeed")
        .expect("Key must exist");

    let record_debug = format!("{fetched:?}");
    assert!(
        !record_debug.contains(secret_entropy),
        "Record Debug representation must never contain secret entropy"
    );

    // 5. Verify lookup by BLAKE3 hash succeeds, but lookup by raw plaintext fails
    let found_by_hash = storage
        .find_by_hash(&saved_record.key_hash)
        .await
        .expect("Query must succeed");
    assert!(found_by_hash.is_some());

    let found_by_raw = storage
        .find_by_hash(plaintext)
        .await
        .expect("Query must succeed");
    assert!(
        found_by_raw.is_none(),
        "Querying directly by raw key plaintext must return None"
    );
}

// ==============================================================================
// 2. ENVELOPE ENCRYPTION NONCE UNIQUENESS & AAD BINDING AUDIT
// ==============================================================================

#[tokio::test]
async fn test_audit_envelope_encryption_nonce_uniqueness_and_aad_binding() {
    let config = AppConfig::default();
    let state = web_repo_to_ai_api::state::AppState::new(config);

    let session_id = Uuid::new_v4();
    let alt_session_id = Uuid::new_v4();
    let raw_token = "ghs_test_token_secret_value_1234567890abcdef";
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(2);

    let mut ciphertexts = HashSet::new();

    // 1. Save 25 distinct tokens and verify probabilistic encryption (fresh nonces)
    for i in 0..25 {
        let installation_id = 100_000 + i;
        let row = state
            .token_storage
            .save_installation_token(
                session_id,
                installation_id,
                "acme-inc",
                raw_token,
                expires_at,
            )
            .await
            .expect("Token save must succeed");

        // Verify stored row does not leak raw plaintext
        let row_debug = format!("{row:?}");
        assert!(
            !row_debug.contains(raw_token),
            "Stored row Debug format must never contain plaintext token"
        );

        // Verify ciphertext uniqueness (AES-256-GCM fresh 96-bit nonces)
        assert!(
            ciphertexts.insert(row.encrypted_token.clone()),
            "Duplicate ciphertext detected in consecutive encryptions (nonce reuse failure!)"
        );
    }

    // 2. Validate AAD session binding: Cross-session queries must return None
    let cross_session = state
        .token_storage
        .get_active_token(alt_session_id, 100_000)
        .await
        .expect("Query must not fail");
    assert_eq!(
        cross_session, None,
        "Decryption must fail or return None when caller session does not match AAD session"
    );

    // 3. Validate legitimate caller successfully decrypts token
    let legit_token = state
        .token_storage
        .get_active_token(session_id, 100_000)
        .await
        .expect("Query must succeed");
    assert_eq!(legit_token.as_deref(), Some(raw_token));
}

// ==============================================================================
// 3. COMPREHENSIVE SECRET REDACTION PATTERNS AUDIT
// ==============================================================================

#[test]
fn test_audit_observability_redaction_comprehensive_matrix() {
    // 1. Internal live API keys
    let internal_key = "wr_live_1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
    let red_internal = redact_sensitive_text(&format!("Caller authenticated with {internal_key}"));
    assert!(!red_internal.contains(internal_key));
    assert!(red_internal.contains(REDACTED_API_KEY));

    // 2. GitHub Tokens (ghp, ghs, gho, ghu, ghr, github_pat)
    let github_tokens = [
        "ghp_1234567890abcdef1234567890abcdef123456",
        "ghs_1234567890abcdef1234567890abcdef123456",
        "gho_1234567890abcdef1234567890abcdef123456",
        "ghu_1234567890abcdef1234567890abcdef123456",
        "ghr_1234567890abcdef1234567890abcdef123456",
        "github_pat_11AAAAAAA0123456789_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
    ];
    for tok in github_tokens {
        let res = redact_sensitive_text(&format!("GitHub auth: {tok}"));
        assert!(!res.contains(tok), "Token '{tok}' was not redacted!");
        assert!(res.contains(REDACTED_GITHUB_TOKEN));
    }

    // 3. External AI Provider Keys (OpenAI, Anthropic)
    let openai_key = "sk-proj-1234567890abcdefghijklmnopqrstuvwxyz12345";
    let anthropic_key = "sk-ant-api03-1234567890abcdefghijklmnopqrstuvwxyz12345";
    assert_eq!(
        redact_sensitive_text(&format!("key: {openai_key}")),
        format!("key: {REDACTED_API_KEY}")
    );
    assert_eq!(
        redact_sensitive_text(&format!("key: {anthropic_key}")),
        format!("key: {REDACTED_API_KEY}")
    );

    // 4. AWS Credentials
    let aws_key = "AKIAIOSFODNN7EXAMPLE";
    assert_eq!(
        redact_sensitive_text(&format!("aws key: {aws_key}")),
        format!("aws key: {REDACTED_AWS_KEY}")
    );

    // 5. Auth Headers & Basic Auth
    let bearer = "Bearer ya29.a0AfH6SMBabc12345xyz_token_value==";
    let basic = "Basic dXNlcjpwYXNzd29yZA==";
    assert_eq!(
        redact_sensitive_text(&format!("Header: {bearer}")),
        format!("Header: Bearer {REDACTED_TOKEN}")
    );
    assert_eq!(
        redact_sensitive_text(&format!("Header: {basic}")),
        format!("Header: Basic {REDACTED_TOKEN}")
    );

    // 6. URL Userinfo & Token Embedded in Host
    let url_userinfo = "https://admin:super_secret_pass@example.com/api/v1";
    let url_token = "https://ghp_1234567890abcdef1234567890abcdef123456@github.com/repo";
    assert_eq!(
        redact_sensitive_text(url_userinfo),
        "https://admin:[REDACTED]@example.com/api/v1"
    );
    assert_eq!(
        redact_sensitive_text(url_token),
        format!("https://{REDACTED_GITHUB_TOKEN}@github.com/repo")
    );

    // 7. URL Query Parameters
    let query_url =
        "https://api.example.com/sync?token=secret123&client_secret=topsecret456&code=123";
    let red_query = redact_sensitive_text(query_url);
    assert!(!red_query.contains("secret123"));
    assert!(!red_query.contains("topsecret456"));
    assert!(red_query.contains("?token=[REDACTED]"));
    assert!(red_query.contains("&client_secret=[REDACTED]"));

    // 8. Webhook HMAC Signatures
    let sig = "sha256=d3b07384d113edec49eaa6238ad5ff0000000000000000000000000000000000";
    assert_eq!(
        redact_sensitive_text(&format!("X-Hub-Signature-256: {sig}")),
        format!("X-Hub-Signature-256: sha256={REDACTED_SIGNATURE}")
    );

    // 9. Multi-format Private Keys
    let rsa_key =
        "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA0Y...\n-----END RSA PRIVATE KEY-----";
    let pkcs8_key = "-----BEGIN PRIVATE KEY-----\nMIIEvgIBADANBgkqhkiG9w0BAQEFAASC...\n-----END PRIVATE KEY-----";
    let ec_key = "-----BEGIN EC PRIVATE KEY-----\nMHcCAQEEII...\n-----END EC PRIVATE KEY-----";
    assert_eq!(redact_sensitive_text(rsa_key), REDACTED_PRIVATE_KEY);
    assert_eq!(redact_sensitive_text(pkcs8_key), REDACTED_PRIVATE_KEY);
    assert_eq!(redact_sensitive_text(ec_key), REDACTED_PRIVATE_KEY);
}

// ==============================================================================
// 4. JSON RECURSIVE SECRET REDACTION AUDIT
// ==============================================================================

#[test]
fn test_audit_json_redaction_recursive_fields() {
    let mut payload = serde_json::json!({
        "service": "api",
        "version": "1.0.0",
        "auth_payload": {
            "api_key": "raw_api_key_12345",
            "client_secret": "raw_oauth_secret_56789",
            "access_token": "ghs_token_abcde12345",
            "refresh_token": "ref_token_xyz987",
            "signing_key": "hmac_signing_key_443322",
            "webhook_secret": "wh_sec_112233",
            "user_password": "PlaintextPassword!",
            "private_key": "-----BEGIN PRIVATE KEY-----\nMIIE...\n-----END PRIVATE KEY-----",
            "encryption_key": "master_enc_key_hex"
        },
        "safe_data": {
            "name": "Public Repository",
            "description": "Safe text without secrets",
            "count": 42
        }
    });

    redact_json_value(&mut payload);

    let creds = &payload["auth_payload"];
    assert_eq!(creds["api_key"], REDACTED_SECRET);
    assert_eq!(creds["client_secret"], REDACTED_SECRET);
    assert_eq!(creds["access_token"], REDACTED_SECRET);
    assert_eq!(creds["refresh_token"], REDACTED_SECRET);
    assert_eq!(creds["signing_key"], REDACTED_SECRET);
    assert_eq!(creds["webhook_secret"], REDACTED_SECRET);
    assert_eq!(creds["user_password"], REDACTED_SECRET);
    assert_eq!(creds["private_key"], REDACTED_SECRET);
    assert_eq!(creds["encryption_key"], REDACTED_SECRET);

    // Verify top-level sensitive dictionary gets masked as a whole
    let mut top_level = serde_json::json!({
        "credentials": {
            "nested_secret": "super_secret_value"
        }
    });
    redact_json_value(&mut top_level);
    assert_eq!(top_level["credentials"], REDACTED_SECRET);

    let safe = &payload["safe_data"];
    assert_eq!(safe["name"], "Public Repository");
    assert_eq!(safe["count"], 42);

    // Verify sensitive key taxonomy completeness
    let sensitive_keys = [
        "password",
        "passwd",
        "secret",
        "token",
        "authorization",
        "api_key",
        "apikey",
        "private_key",
        "credential",
        "webhook_secret",
        "signing_key",
        "access_token",
        "refresh_token",
        "encryption_key",
    ];
    for key in sensitive_keys {
        assert!(
            is_sensitive_key(key),
            "Key '{key}' must be classified as sensitive"
        );
        assert!(
            is_sensitive_key(&format!("user_{key}_hash")),
            "Compound key must be classified as sensitive"
        );
    }
}

// ==============================================================================
// 5. DENY.TOML GOVERNANCE & LICENSE AUDIT
// ==============================================================================

#[test]
fn test_audit_deny_toml_governance_invariants() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let deny_path = workspace_root.join("deny.toml");
    let content = fs::read_to_string(&deny_path).expect("deny.toml must exist and be readable");

    // 1. Advisories configuration
    assert!(
        content.contains("version = 2"),
        "deny.toml must use advisories version 2"
    );
    assert!(
        content.contains("unmaintained = \"all\""),
        "Unmaintained dependencies must be enforced"
    );
    assert!(
        content.contains("unsound = \"all\""),
        "Unsound dependencies must be enforced"
    );

    // 2. Permitted licenses
    let required_licenses = [
        "MIT",
        "Apache-2.0",
        "BSD-2-Clause",
        "BSD-3-Clause",
        "ISC",
        "Unicode-3.0",
    ];
    for lic in required_licenses {
        assert!(
            content.contains(&format!("\"{lic}\"")),
            "deny.toml must allow standard permissive license '{lic}'"
        );
    }

    // 3. Bans & wildcards
    assert!(
        content.contains("wildcards = \"deny\""),
        "Wildcard dependencies must be denied"
    );

    // 4. Registry and Git sources
    assert!(
        content.contains("unknown-registry = \"deny\""),
        "Unknown registries must be denied"
    );
    assert!(
        content.contains("unknown-git = \"deny\""),
        "Unknown git repositories must be denied"
    );
}

// ==============================================================================
// 6. WORKSPACE SOURCE TREE ZERO-SECRET AUDIT
// ==============================================================================

#[test]
fn test_audit_workspace_no_committed_secrets() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let target_dirs = [
        workspace_root.join("crates"),
        workspace_root.join("services"),
        workspace_root.join("deploy"),
        workspace_root.join("scripts"),
    ];

    let mut scanned_files = 0;

    for dir in &target_dirs {
        scan_directory(dir, &mut scanned_files);
    }

    assert!(
        scanned_files > 50,
        "Audit must have scanned at least 50 source files (scanned {scanned_files})"
    );
}

fn scan_directory(dir: &Path, count: &mut usize) {
    if !dir.exists() {
        return;
    }
    let entries = fs::read_dir(dir).expect("Directory must be readable");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name != "target" && name != "node_modules" && name != ".git" {
                scan_directory(&path, count);
            }
        } else if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if matches!(
                ext,
                "rs" | "toml" | "json" | "yaml" | "yml" | "sh" | "ps1" | "sql"
            ) {
                *count += 1;
                inspect_file_for_live_credentials(&path);
            }
        }
    }
}

fn inspect_file_for_live_credentials(path: &Path) {
    let path_str = path.to_string_lossy().to_ascii_lowercase();

    // Skip test fixtures or mock modules that intentionally contain synthetic non-functional tokens
    if path_str.contains("test")
        || path_str.contains("mock")
        || path_str.contains("fixture")
        || path_str.contains("openapi")
        || path_str.contains("redaction.rs")
    {
        return;
    }

    let raw_content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return, // Skip non-UTF8 files
    };

    // For Rust files, only inspect the production code before any inline unit test module (#[cfg(test)])
    let content = if path_str.ends_with(".rs") {
        raw_content
            .split("#[cfg(test)]")
            .next()
            .unwrap_or(&raw_content)
    } else {
        &raw_content
    };

    // 1. Unencrypted RSA, EC, OpenSSH, or generic Private Keys with actual base64 key material
    static PRIVATE_KEY_BLOCK: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let priv_key_re = PRIVATE_KEY_BLOCK.get_or_init(|| {
        regex::Regex::new(
            r"(?s)-----BEGIN (?:[A-Z0-9_-]+ )?PRIVATE KEY-----\r?\n[a-zA-Z0-9+/=\r\n]{64,}-----END",
        )
        .unwrap()
    });
    if priv_key_re.is_match(content) {
        panic!(
            "Private key block with key data found in production file {:?}",
            path
        );
    }

    // 2. High-entropy live secret tokens
    static LIVE_PATTERNS: std::sync::OnceLock<Vec<regex::Regex>> = std::sync::OnceLock::new();
    let patterns = LIVE_PATTERNS.get_or_init(|| {
        vec![
            // AWS live access key
            regex::Regex::new(r"\b(AKIA|ABIA|ACCA|ASIA)[0-9A-Z]{16}\b").unwrap(),
            // GitHub live token (ghp, gho, ghu, ghs, ghr, github_pat)
            regex::Regex::new(r"\b(gh[psuor]_[a-zA-Z0-9]{36}|github_pat_[a-zA-Z0-9_]{22,})\b")
                .unwrap(),
            // Internal live API key
            regex::Regex::new(r"\bwr_live_[0-9a-fA-F]{64}\b").unwrap(),
            // Slack live tokens
            regex::Regex::new(r"\bxox[baprs]-[0-9a-zA-Z]{10,}\b").unwrap(),
            // Google API keys
            regex::Regex::new(r"\bAIza[0-9A-Za-z\-_]{35}\b").unwrap(),
        ]
    });

    for pat in patterns {
        if let Some(m) = pat.find(content) {
            panic!(
                "Live secret pattern '{}' detected in production file {:?}: {}",
                pat.as_str(),
                path,
                m.as_str()
            );
        }
    }
}
