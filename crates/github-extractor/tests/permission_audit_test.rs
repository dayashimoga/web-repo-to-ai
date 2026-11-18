//! # GitHub App Permission Audit & Least-Privilege Verification Tests (Subphase 6.4.4)
//!
//! Validates Rule 05 least-privilege enforcement, permission policy boundary checks,
//! excessive/forbidden scope rejection, installation suspension detection,
//! report rendering fidelity, and the Stage 6 security acceptance gate.

use axum::extract::{Path as AxumPath, Request};
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use chrono::{Duration, Utc};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;

use web_repo_to_ai_core::crypto::{Aes256GcmEngine, EncryptionKey};
use web_repo_to_ai_github_extractor::authorization::AuthorizedInstallationContext;
use web_repo_to_ai_github_extractor::client::GitHubApiClient;
use web_repo_to_ai_github_extractor::error::GitHubExtractorError;
use web_repo_to_ai_github_extractor::installation::{
    GitHubAccountSummary, GitHubInstallation, GitHubInstallationToken, InstallationTokenRequest,
    RepositorySelection,
};
use web_repo_to_ai_github_extractor::permissions::{
    PermissionAuditReport, PermissionAuditor, PermissionPolicy, PermissionViolation,
    DEFAULT_FORBIDDEN_CATEGORIES, DEFAULT_FORBIDDEN_LEVELS,
};
use web_repo_to_ai_github_extractor::private_extractor::PrivateRepositoryExtractor;

fn sample_account() -> GitHubAccountSummary {
    GitHubAccountSummary {
        id: 7001,
        login: "audit-org".to_string(),
        account_type: "Organization".to_string(),
        avatar_url: Some("https://github.com/images/error/octocat.gif".to_string()),
        html_url: Some("https://github.com/audit-org".to_string()),
    }
}

fn sample_installation(
    permissions: HashMap<String, String>,
    suspended: bool,
) -> GitHubInstallation {
    GitHubInstallation {
        id: 889900,
        account: sample_account(),
        repository_selection: RepositorySelection::Selected,
        access_tokens_url: "https://api.github.com/app/installations/889900/access_tokens"
            .to_string(),
        repositories_url: "https://api.github.com/installation/repositories".to_string(),
        html_url: "https://github.com/settings/installations/889900".to_string(),
        app_id: 4242,
        app_slug: Some("web-repo-to-ai".to_string()),
        target_id: 7001,
        target_type: "Organization".to_string(),
        permissions,
        events: vec!["installation".to_string()],
        created_at: Utc::now() - Duration::days(5),
        updated_at: Utc::now() - Duration::days(1),
        suspended_at: if suspended {
            Some(Utc::now() - Duration::hours(1))
        } else {
            None
        },
        suspended_by: None,
    }
}

fn sample_token(permissions: HashMap<String, String>) -> GitHubInstallationToken {
    GitHubInstallationToken {
        token: "ghs_AuditVerificationToken1234567890".to_string(),
        expires_at: Utc::now() + Duration::hours(1),
        permissions,
        repository_selection: RepositorySelection::Selected,
        repositories: None,
    }
}

/// Test Case 1: Strict minimal policy exact compliance verification.
#[test]
fn test_permission_audit_strict_minimal_exact_compliance() {
    let mut perms = HashMap::new();
    perms.insert("contents".to_string(), "read".to_string());
    perms.insert("metadata".to_string(), "read".to_string());

    let policy = PermissionPolicy::strict_minimal();
    let report = PermissionAuditor::audit_permissions(&perms, &policy);

    assert!(
        report.is_compliant(),
        "Expected exact minimal scopes to be compliant"
    );
    assert!(report.violations().is_empty());
    assert!(!report.has_excessive_level());
    assert!(!report.has_forbidden_category());
    assert!(!report.has_missing_required());
    assert!(!report.has_unexpected_scope());

    let summary = report.summary();
    assert!(summary.contains("Permission audit passed"));
    assert!(summary.contains("StrictMinimal"));
    assert!(summary.contains("all 2 granted scopes comply"));

    let md = report.to_markdown_report();
    assert!(md.contains("**COMPLIANT**"));
    assert!(md.contains("`PERMITTED`"));
    assert!(md.contains("| `contents` | `read` | `PERMITTED` |"));
    assert!(md.contains("| `metadata` | `read` | `PERMITTED` |"));

    let json_str = report.to_json().expect("JSON serialization succeeds");
    let deserialized: PermissionAuditReport =
        serde_json::from_str(&json_str).expect("JSON deserialization succeeds");
    assert_eq!(report, deserialized);

    assert!(PermissionAuditor::validate_permissions(&perms, &policy).is_ok());
}

/// Test Case 2: Reject missing required scopes (contents or metadata).
#[test]
fn test_permission_audit_rejects_missing_required_scopes() {
    let policy = PermissionPolicy::strict_minimal();

    // Missing contents
    let mut perms_missing_contents = HashMap::new();
    perms_missing_contents.insert("metadata".to_string(), "read".to_string());
    let report1 = PermissionAuditor::audit_permissions(&perms_missing_contents, &policy);
    assert!(!report1.is_compliant());
    assert!(report1.has_missing_required());
    assert_eq!(report1.violations().len(), 1);
    assert!(matches!(
        &report1.violations()[0],
        PermissionViolation::MissingRequired { scope, expected } if scope == "contents" && expected == "read"
    ));
    assert!(PermissionAuditor::validate_permissions(&perms_missing_contents, &policy).is_err());

    // Missing metadata
    let mut perms_missing_metadata = HashMap::new();
    perms_missing_metadata.insert("contents".to_string(), "read".to_string());
    let report2 = PermissionAuditor::audit_permissions(&perms_missing_metadata, &policy);
    assert!(!report2.is_compliant());
    assert!(report2.has_missing_required());
    assert_eq!(report2.violations().len(), 1);
    assert!(matches!(
        &report2.violations()[0],
        PermissionViolation::MissingRequired { scope, expected } if scope == "metadata" && expected == "read"
    ));

    // Completely empty
    let empty_perms = HashMap::new();
    let report3 = PermissionAuditor::audit_permissions(&empty_perms, &policy);
    assert!(!report3.is_compliant());
    assert_eq!(report3.violations().len(), 2);
    assert!(report3.has_missing_required());
}

/// Test Case 3: Exhaustively reject all write and admin permission levels across scopes.
#[test]
fn test_permission_audit_rejects_all_write_and_admin_levels() {
    let policy = PermissionPolicy::strict_minimal();

    let write_cases = [
        ("contents", "write"),
        ("contents", "admin"),
        ("metadata", "write"),
        ("metadata", "admin"),
        ("pull_requests", "write"),
        ("pull_requests", "admin"),
        ("issues", "write"),
        ("issues", "admin"),
        ("packages", "write"),
    ];

    for (scope, level) in write_cases {
        let mut perms = HashMap::new();
        perms.insert("contents".to_string(), "read".to_string());
        perms.insert("metadata".to_string(), "read".to_string());
        perms.insert(scope.to_string(), level.to_string());

        let report = PermissionAuditor::audit_permissions(&perms, &policy);
        assert!(
            !report.is_compliant(),
            "Scope '{scope}:{level}' should be rejected as excessive"
        );
        assert!(
            report.has_excessive_level(),
            "Expected has_excessive_level() for '{scope}:{level}'"
        );

        let err = PermissionAuditor::validate_permissions(&perms, &policy).unwrap_err();
        match err {
            GitHubExtractorError::PermissionViolation(msg) => {
                assert!(msg.contains("violation(s) detected"));
            }
            other => panic!("Expected PermissionViolation, got {other:?}"),
        }
    }
}

/// Test Case 4: Exhaustively reject all prohibited sensitive permission categories.
#[test]
fn test_permission_audit_rejects_all_prohibited_sensitive_categories() {
    let policy = PermissionPolicy::strict_minimal();

    for forbidden_category in DEFAULT_FORBIDDEN_CATEGORIES {
        // Even when granted with "read" level, each forbidden category MUST be rejected
        let mut perms = HashMap::new();
        perms.insert("contents".to_string(), "read".to_string());
        perms.insert("metadata".to_string(), "read".to_string());
        perms.insert((*forbidden_category).to_string(), "read".to_string());

        let report = PermissionAuditor::audit_permissions(&perms, &policy);
        assert!(
            !report.is_compliant(),
            "Forbidden category '{forbidden_category}:read' was not rejected!"
        );
        assert!(
            report.has_forbidden_category(),
            "Expected has_forbidden_category() for '{forbidden_category}'"
        );

        let found_violation = report.violations().iter().any(|v| match v {
            PermissionViolation::ForbiddenScope { scope, reason, .. } => {
                scope == forbidden_category && reason.contains("Rule 05")
            }
            _ => false,
        });
        assert!(
            found_violation,
            "Expected ForbiddenScope violation citing Rule 05 for '{forbidden_category}'"
        );
    }
}

/// Test Case 5: Reject unexpected additional scopes under StrictMinimal policy.
#[test]
fn test_permission_audit_rejects_unexpected_scopes_under_strict_minimal() {
    let policy = PermissionPolicy::strict_minimal();

    let benign_additional_scopes = [
        ("pull_requests", "read"),
        ("issues", "read"),
        ("discussions", "read"),
        ("pages", "read"),
        ("statuses", "read"),
        ("checks", "read"),
    ];

    for (scope, level) in benign_additional_scopes {
        let mut perms = HashMap::new();
        perms.insert("contents".to_string(), "read".to_string());
        perms.insert("metadata".to_string(), "read".to_string());
        perms.insert(scope.to_string(), level.to_string());

        let report = PermissionAuditor::audit_permissions(&perms, &policy);
        assert!(
            !report.is_compliant(),
            "StrictMinimal policy should reject unexpected scope '{scope}:{level}'"
        );
        assert!(report.has_unexpected_scope());

        let found_unexpected = report.violations().iter().any(|v| match v {
            PermissionViolation::UnexpectedScope { scope: s, granted } => {
                s == scope && granted == level
            }
            _ => false,
        });
        assert!(found_unexpected);
    }
}

/// Test Case 6: Standard policy permits benign read-only scopes while rejecting dangerous ones.
#[test]
fn test_permission_audit_standard_policy_permits_benign_read_scopes() {
    let policy = PermissionPolicy::standard();

    // Compliant case: contents: read + metadata: read + pull_requests: read + issues: read
    let mut compliant_perms = HashMap::new();
    compliant_perms.insert("contents".to_string(), "read".to_string());
    compliant_perms.insert("metadata".to_string(), "read".to_string());
    compliant_perms.insert("pull_requests".to_string(), "read".to_string());
    compliant_perms.insert("issues".to_string(), "read".to_string());

    let report = PermissionAuditor::audit_permissions(&compliant_perms, &policy);
    assert!(
        report.is_compliant(),
        "Standard policy should allow benign read-only scopes"
    );
    assert_eq!(report.violations().len(), 0);

    // Non-compliant case: standard policy STILL strictly rejects write levels
    let mut write_perms = compliant_perms.clone();
    write_perms.insert("pull_requests".to_string(), "write".to_string());
    let write_report = PermissionAuditor::audit_permissions(&write_perms, &policy);
    assert!(!write_report.is_compliant());
    assert!(write_report.has_excessive_level());

    // Non-compliant case: standard policy STILL strictly rejects prohibited categories
    let mut forbidden_perms = compliant_perms.clone();
    forbidden_perms.insert("secrets".to_string(), "read".to_string());
    let forbidden_report = PermissionAuditor::audit_permissions(&forbidden_perms, &policy);
    assert!(!forbidden_report.is_compliant());
    assert!(forbidden_report.has_forbidden_category());
}

/// Test Case 7: Custom PermissionPolicyBuilder with fine-grained rules.
#[test]
fn test_permission_audit_custom_policy_builder() {
    let custom_policy = PermissionPolicy::builder("CustomAuditPolicy")
        .require_scope("contents", "read")
        .require_scope("metadata", "read")
        .allow_additional_read_scopes(true)
        .allow_additional_scope("pull_requests")
        .forbid_category("telemetry")
        .forbid_level("maintain")
        .forbid_level("triage")
        .build();

    assert_eq!(custom_policy.name, "CustomAuditPolicy");
    assert!(custom_policy
        .forbidden_levels
        .contains(&"maintain".to_string()));
    assert!(custom_policy
        .forbidden_levels
        .contains(&"triage".to_string()));
    assert!(custom_policy
        .forbidden_categories
        .contains(&"telemetry".to_string()));

    // Permitted: contents, metadata, pull_requests (all read)
    let mut valid = HashMap::new();
    valid.insert("contents".to_string(), "read".to_string());
    valid.insert("metadata".to_string(), "read".to_string());
    valid.insert("pull_requests".to_string(), "read".to_string());
    assert!(PermissionAuditor::audit_permissions(&valid, &custom_policy).is_compliant());

    // Rejected: unwhitelisted additional scope "issues: read"
    let mut unwhitelisted = valid.clone();
    unwhitelisted.insert("issues".to_string(), "read".to_string());
    let report_unwhitelisted = PermissionAuditor::audit_permissions(&unwhitelisted, &custom_policy);
    assert!(!report_unwhitelisted.is_compliant());
    assert!(report_unwhitelisted.has_unexpected_scope());

    // Rejected: custom forbidden level "maintain"
    let mut maintain_perms = valid.clone();
    maintain_perms.insert("contents".to_string(), "maintain".to_string());
    let report_maintain = PermissionAuditor::audit_permissions(&maintain_perms, &custom_policy);
    assert!(!report_maintain.is_compliant());
    assert!(report_maintain.has_excessive_level());
}

/// Test Case 8: Installation suspension detection in audit and validation.
#[test]
fn test_permission_audit_installation_suspension_detection() {
    let mut perms = HashMap::new();
    perms.insert("contents".to_string(), "read".to_string());
    perms.insert("metadata".to_string(), "read".to_string());

    let policy = PermissionPolicy::strict_minimal();

    // Active installation -> compliant
    let active_inst = sample_installation(perms.clone(), false);
    let active_report = PermissionAuditor::audit_installation(&active_inst, &policy);
    assert!(active_report.is_compliant());
    assert!(active_inst.validate_permissions().is_ok());

    // Suspended installation -> non-compliant with SuspendedInstallation violation
    let suspended_inst = sample_installation(perms, true);
    let suspended_report = PermissionAuditor::audit_installation(&suspended_inst, &policy);
    assert!(!suspended_report.is_compliant());
    assert_eq!(suspended_report.violations().len(), 1);

    let is_suspended_violation = matches!(
        &suspended_report.violations()[0],
        PermissionViolation::SuspendedInstallation { installation_id } if *installation_id == 889900
    );
    assert!(is_suspended_violation);

    let err = PermissionAuditor::validate_installation(&suspended_inst, &policy).unwrap_err();
    match err {
        GitHubExtractorError::PermissionViolation(msg) => {
            assert!(msg.contains("suspended and cannot be accessed"));
        }
        other => panic!("Expected PermissionViolation, got {other:?}"),
    }

    // Direct helper method on GitHubInstallation
    assert!(suspended_inst.validate_permissions().is_err());
}

/// Test Case 9: Installation token audit and validation methods.
#[test]
fn test_permission_audit_installation_token_validation() {
    let policy = PermissionPolicy::strict_minimal();

    let mut valid_perms = HashMap::new();
    valid_perms.insert("contents".to_string(), "read".to_string());
    valid_perms.insert("metadata".to_string(), "read".to_string());
    let valid_token = sample_token(valid_perms);

    let valid_report = PermissionAuditor::audit_installation_token(&valid_token, &policy);
    assert!(valid_report.is_compliant());
    assert!(valid_token.validate_permissions().is_ok());

    let mut invalid_perms = HashMap::new();
    invalid_perms.insert("contents".to_string(), "write".to_string());
    invalid_perms.insert("metadata".to_string(), "read".to_string());
    let invalid_token = sample_token(invalid_perms);

    let invalid_report = PermissionAuditor::audit_installation_token(&invalid_token, &policy);
    assert!(!invalid_report.is_compliant());
    assert!(invalid_report.has_excessive_level());
    assert!(invalid_token.validate_permissions().is_err());
}

/// Test Case 10: AuthorizedInstallationContext audit and validation methods.
#[test]
fn test_permission_audit_authorized_installation_context_methods() {
    let mut valid_perms = HashMap::new();
    valid_perms.insert("contents".to_string(), "read".to_string());
    valid_perms.insert("metadata".to_string(), "read".to_string());

    let ctx = AuthorizedInstallationContext::new(
        "acme-org",
        "private-engine",
        112233,
        "acme-org",
        "ghs_ContextValidationToken12345",
        RepositorySelection::Selected,
    )
    .with_permissions(valid_perms);

    let report = ctx.audit_permissions();
    assert!(report.is_compliant());
    assert!(ctx.validate_permissions().is_ok());
    assert!(
        PermissionAuditor::audit_context(&ctx, &PermissionPolicy::strict_minimal()).is_compliant()
    );
    assert!(PermissionAuditor::validate_context(&ctx, &PermissionPolicy::strict_minimal()).is_ok());

    let mut invalid_perms = HashMap::new();
    invalid_perms.insert("contents".to_string(), "admin".to_string());
    invalid_perms.insert("metadata".to_string(), "read".to_string());

    let invalid_ctx = AuthorizedInstallationContext::new(
        "acme-org",
        "private-engine",
        112233,
        "acme-org",
        "ghs_ContextValidationToken12345",
        RepositorySelection::Selected,
    )
    .with_permissions(invalid_perms);

    let bad_report = invalid_ctx.audit_permissions();
    assert!(!bad_report.is_compliant());
    assert!(bad_report.has_excessive_level());
    assert!(invalid_ctx.validate_permissions().is_err());
    assert!(
        PermissionAuditor::validate_context(&invalid_ctx, &PermissionPolicy::strict_minimal())
            .is_err()
    );
}

/// Test Case 11: PrivateRepositoryExtractor fails closed immediately on permission violation before issuing HTTP calls.
#[tokio::test]
async fn test_permission_audit_private_extractor_fail_closed_on_policy_violation() {
    let requested = Arc::new(AtomicBool::new(false));
    let requested_flag = requested.clone();

    // In-process mock server that tracks if any request was sent
    let app = Router::new().fallback(move |_: Request| {
        let flag = requested_flag.clone();
        async move {
            flag.store(true, Ordering::SeqCst);
            (StatusCode::INTERNAL_SERVER_ERROR, "Should never be reached")
        }
    });

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = Arc::new(GitHubApiClient::new().with_base_url_override(format!("http://{addr}")));
    let extractor = PrivateRepositoryExtractor::new(client);

    // Violation: Context possesses "contents: write"
    let mut violating_perms = HashMap::new();
    violating_perms.insert("contents".to_string(), "write".to_string());
    violating_perms.insert("metadata".to_string(), "read".to_string());

    let ctx = AuthorizedInstallationContext::new(
        "acme",
        "protected-vault",
        999,
        "acme",
        "ghs_ViolatingToken123",
        RepositorySelection::Selected,
    )
    .with_permissions(violating_perms);

    let result = extractor.extract(&ctx, None).await;

    // Must fail closed with PermissionViolation
    assert!(result.is_err());
    match result.unwrap_err() {
        GitHubExtractorError::PermissionViolation(msg) => {
            assert!(msg.contains("violation(s) detected"));
        }
        other => panic!("Expected PermissionViolation, got {other:?}"),
    }

    // Zero network requests should have been transmitted
    assert!(
        !requested.load(Ordering::SeqCst),
        "Extraction issued network calls despite permission violation!"
    );
}

/// Test Case 12: Wire-level verification of read-only token minting & validation.
#[tokio::test]
async fn test_permission_audit_live_http_installation_token_request_read_only_preservation() {
    let token_req_verified = Arc::new(AtomicBool::new(false));
    let token_req_flag = token_req_verified.clone();

    let app = Router::new().route(
        "/app/installations/:id/access_tokens",
        post(move |AxumPath(_id): AxumPath<u64>, req: Request| {
            let flag = token_req_flag.clone();
            async move {
                let bytes = axum::body::to_bytes(req.into_body(), 1024 * 64)
                    .await
                    .unwrap();
                let json_body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

                // Assert that the request body specifies strictly read permissions
                let perms = json_body.get("permissions").unwrap();
                let contents_perm = perms.get("contents").and_then(|v| v.as_str());
                let metadata_perm = perms.get("metadata").and_then(|v| v.as_str());

                if contents_perm == Some("read") && metadata_perm == Some("read") {
                    flag.store(true, Ordering::SeqCst);
                }

                // Return a compliant read-only response
                let resp_body = json!({
                    "token": "ghs_LiveMintedCompliantToken12345",
                    "expires_at": (Utc::now() + Duration::hours(1)).to_rfc3339(),
                    "permissions": {
                        "contents": "read",
                        "metadata": "read"
                    },
                    "repository_selection": "selected"
                });

                (StatusCode::CREATED, Json(resp_body))
            }
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));
    let req = InstallationTokenRequest::minimal_read_only();

    let minted_token = client
        .create_scoped_installation_access_token(554433, "dummy-jwt", &req)
        .await
        .expect("Minting compliant token succeeds");

    assert!(token_req_verified.load(Ordering::SeqCst));
    assert_eq!(minted_token.token(), "ghs_LiveMintedCompliantToken12345");
    assert!(minted_token.validate_permissions().is_ok());
}

/// Test Case 13: Full Markdown table rendering, status badge formatting, and caution alerts.
#[test]
fn test_permission_audit_markdown_and_json_report_fidelity() {
    let mut mixed_perms = HashMap::new();
    mixed_perms.insert("contents".to_string(), "write".to_string()); // Excessive
    mixed_perms.insert("metadata".to_string(), "read".to_string()); // Permitted
    mixed_perms.insert("secrets".to_string(), "read".to_string()); // Forbidden category
    mixed_perms.insert("pull_requests".to_string(), "read".to_string()); // Unexpected under strict minimal

    let policy = PermissionPolicy::strict_minimal();
    let report = PermissionAuditor::audit_permissions(&mixed_perms, &policy);

    assert!(!report.is_compliant());
    assert_eq!(report.violations().len(), 4);

    let md = report.to_markdown_report();
    assert!(md.contains("### GitHub App Permission Audit Report"));
    assert!(md.contains("- **Compliance Status:** **NON-COMPLIANT** (Rule 05 violation)"));
    assert!(md.contains("> [!CAUTION]"));
    assert!(md.contains("> The following permission policy violations were detected:"));
    assert!(md.contains("| `contents` | `write` | `REJECTED` |"));
    assert!(md.contains("| `metadata` | `read` | `PERMITTED` |"));
    assert!(md.contains("| `secrets` | `read` | `REJECTED` |"));
    assert!(md.contains("| `pull_requests` | `read` | `REJECTED` |"));

    let json_val: serde_json::Value =
        serde_json::from_str(&report.to_json().unwrap()).expect("valid JSON");
    assert_eq!(json_val["compliant"], false);
    assert_eq!(json_val["policy_name"], "StrictMinimal");
    assert_eq!(json_val["violations"].as_array().unwrap().len(), 4);
}

/// Test Case 14: Holistic Stage 6 Security Acceptance Gate Verification.
///
/// Stage 6 Acceptance Gate: "Authorized private repositories can be extracted without exposing credentials,
/// leaking tokens, or permitting repository modifications."
#[test]
fn test_stage_6_security_acceptance_gate_invariants() {
    // Invariant 1: At-rest encryption with AES-256-GCM and session-scoped AAD binding
    let key = EncryptionKey::generate().expect("key generation succeeds");
    let secret_token = "ghs_AcceptanceGateSecretToken998877665544332211";
    let session_id = uuid::Uuid::new_v4();
    let installation_id = 998877;
    let aad = format!("tenant:{session_id}:installation:{installation_id}");

    let encrypted = Aes256GcmEngine::encrypt(&key, secret_token.as_bytes(), aad.as_bytes())
        .expect("encryption succeeds");
    assert_ne!(encrypted.ciphertext, secret_token.as_bytes());

    // Decryption under matching tenant succeeds
    let decrypted = Aes256GcmEngine::decrypt(&key, &encrypted, aad.as_bytes())
        .expect("decryption under valid tenant succeeds");
    assert_eq!(String::from_utf8(decrypted).unwrap(), secret_token);

    // Decryption under another tenant fails (strict cryptographic isolation)
    let cross_session = uuid::Uuid::new_v4();
    let wrong_aad = format!("tenant:{cross_session}:installation:{installation_id}");
    assert!(
        Aes256GcmEngine::decrypt(&key, &encrypted, wrong_aad.as_bytes()).is_err(),
        "Cross-tenant decryption MUST fail!"
    );

    // Invariant 2: Secret tokens are strictly masked in Debug and Display
    let ctx = AuthorizedInstallationContext::new(
        "acme-enterprise",
        "top-secret",
        installation_id,
        "acme-enterprise",
        secret_token,
        RepositorySelection::Selected,
    );
    let debug_str = format!("{ctx:?}");
    let display_str = format!("{ctx}");
    assert!(!debug_str.contains(secret_token));
    assert!(!display_str.contains(secret_token));
    assert!(debug_str.contains("***[REDACTED_TOKEN]***"));

    // Invariant 3: Zero repository modification permissions (Strict minimal read-only enforcement)
    let mut compliant_perms = HashMap::new();
    compliant_perms.insert("contents".to_string(), "read".to_string());
    compliant_perms.insert("metadata".to_string(), "read".to_string());

    let policy = PermissionPolicy::strict_minimal();
    assert!(
        PermissionAuditor::audit_permissions(&compliant_perms, &policy).is_compliant(),
        "Read-only contents and metadata MUST be compliant"
    );

    for forbidden_level in DEFAULT_FORBIDDEN_LEVELS {
        let mut mutating_perms = compliant_perms.clone();
        mutating_perms.insert("contents".to_string(), (*forbidden_level).to_string());
        assert!(
            !PermissionAuditor::audit_permissions(&mutating_perms, &policy).is_compliant(),
            "Permitting repository modification level '{forbidden_level}' violates Stage 6 Acceptance Gate!"
        );
    }
}
