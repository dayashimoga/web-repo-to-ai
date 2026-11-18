//! # Minimum Permissions & Rule 05 Integration Tests
//!
//! Comprehensive verification of runtime permission validation, scope policy enforcement,
//! excessive/forbidden scope detection, and Markdown audit report generation.

use axum::extract::Path as AxumPath;
use axum::routing::post;
use axum::{Json, Router};
use chrono::{Duration as ChronoDuration, Utc};
use serde_json::json;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use web_repo_to_ai_github_extractor::{
    GitHubAccountSummary, GitHubApiClient, GitHubExtractorError, GitHubInstallation,
    GitHubInstallationToken, InstallationTokenRequest, PermissionAuditReport, PermissionAuditor,
    PermissionPolicy, PermissionViolation, RepositorySelection,
};

fn sample_account() -> GitHubAccountSummary {
    GitHubAccountSummary {
        id: 1001,
        login: "acme-corp".to_string(),
        account_type: "Organization".to_string(),
        avatar_url: Some("https://github.com/images/error/octocat_happy.gif".to_string()),
        html_url: Some("https://github.com/acme-corp".to_string()),
    }
}

fn sample_installation(
    permissions: HashMap<String, String>,
    suspended: bool,
) -> GitHubInstallation {
    GitHubInstallation {
        id: 998877,
        account: sample_account(),
        repository_selection: RepositorySelection::Selected,
        access_tokens_url: "https://api.github.com/app/installations/998877/access_tokens"
            .to_string(),
        repositories_url: "https://api.github.com/installation/repositories".to_string(),
        html_url: "https://github.com/settings/installations/998877".to_string(),
        app_id: 42,
        app_slug: Some("web-repo-to-ai".to_string()),
        target_id: 1001,
        target_type: "Organization".to_string(),
        permissions,
        events: vec![
            "installation".to_string(),
            "installation_repositories".to_string(),
        ],
        created_at: Utc::now() - ChronoDuration::days(10),
        updated_at: Utc::now() - ChronoDuration::days(1),
        suspended_at: if suspended {
            Some(Utc::now() - ChronoDuration::hours(2))
        } else {
            None
        },
        suspended_by: None,
    }
}

#[test]
fn test_compliant_minimal_permissions_passes() {
    let mut perms = HashMap::new();
    perms.insert("contents".to_string(), "read".to_string());
    perms.insert("metadata".to_string(), "read".to_string());

    let policy = PermissionPolicy::strict_minimal();
    let report = PermissionAuditor::audit_permissions(&perms, &policy);

    assert!(report.is_compliant());
    assert!(report.violations().is_empty());
    assert_eq!(report.policy_name, "StrictMinimal");
    assert_eq!(report.granted_permissions.len(), 2);

    let summary = report.summary();
    assert!(summary.contains("passed"));
    assert!(summary.contains("StrictMinimal"));

    let markdown = report.to_markdown_report();
    assert!(markdown.contains("COMPLIANT"));
    assert!(markdown.contains("`contents`"));
    assert!(markdown.contains("`metadata`"));
    assert!(markdown.contains("`PERMITTED`"));

    assert!(PermissionAuditor::validate_permissions(&perms, &policy).is_ok());
}

#[test]
fn test_missing_required_contents_fails() {
    let mut perms = HashMap::new();
    perms.insert("metadata".to_string(), "read".to_string());

    let policy = PermissionPolicy::strict_minimal();
    let report = PermissionAuditor::audit_permissions(&perms, &policy);

    assert!(!report.is_compliant());
    assert_eq!(report.violations().len(), 1);
    assert_eq!(
        report.violations()[0],
        PermissionViolation::MissingRequired {
            scope: "contents".to_string(),
            expected: "read".to_string(),
        }
    );

    let err = PermissionAuditor::validate_permissions(&perms, &policy).unwrap_err();
    match err {
        GitHubExtractorError::PermissionViolation(msg) => {
            assert!(msg.contains("Missing required permission scope 'contents'"));
        }
        other => panic!("Unexpected error variant: {other:?}"),
    }
}

#[test]
fn test_missing_required_metadata_fails() {
    let mut perms = HashMap::new();
    perms.insert("contents".to_string(), "read".to_string());

    let policy = PermissionPolicy::strict_minimal();
    let report = PermissionAuditor::audit_permissions(&perms, &policy);

    assert!(!report.is_compliant());
    assert_eq!(report.violations().len(), 1);
    assert_eq!(
        report.violations()[0],
        PermissionViolation::MissingRequired {
            scope: "metadata".to_string(),
            expected: "read".to_string(),
        }
    );

    let err = PermissionAuditor::validate_permissions(&perms, &policy).unwrap_err();
    assert!(err
        .to_string()
        .contains("Missing required permission scope 'metadata'"));
}

#[test]
fn test_excessive_contents_write_and_admin_fails() {
    let mut perms_write = HashMap::new();
    perms_write.insert("contents".to_string(), "write".to_string());
    perms_write.insert("metadata".to_string(), "read".to_string());

    let policy = PermissionPolicy::strict_minimal();
    let report_write = PermissionAuditor::audit_permissions(&perms_write, &policy);

    assert!(!report_write.is_compliant());
    assert!(report_write.violations().iter().any(|v| matches!(
        v,
        PermissionViolation::ExcessiveLevel { scope, granted, .. } if scope == "contents" && granted == "write"
    )));

    let mut perms_admin = HashMap::new();
    perms_admin.insert("contents".to_string(), "admin".to_string());
    perms_admin.insert("metadata".to_string(), "read".to_string());

    let report_admin = PermissionAuditor::audit_permissions(&perms_admin, &policy);
    assert!(!report_admin.is_compliant());
    assert!(report_admin.violations().iter().any(|v| matches!(
        v,
        PermissionViolation::ExcessiveLevel { scope, granted, .. } if scope == "contents" && granted == "admin"
    )));
}

#[test]
fn test_forbidden_categories_rejected() {
    let forbidden_cases = [
        ("secrets", "read"),
        ("actions", "read"),
        ("administration", "read"),
        ("deployments", "read"),
        ("packages", "read"),
        ("security_events", "read"),
        ("members", "read"),
        ("organization_administration", "read"),
        ("workflows", "write"),
    ];

    let policy = PermissionPolicy::standard();

    for (scope, level) in forbidden_cases {
        let mut perms = HashMap::new();
        perms.insert("contents".to_string(), "read".to_string());
        perms.insert("metadata".to_string(), "read".to_string());
        perms.insert(scope.to_string(), level.to_string());

        let report = PermissionAuditor::audit_permissions(&perms, &policy);
        assert!(
            !report.is_compliant(),
            "Scope '{scope}' ('{level}') must be flagged as non-compliant"
        );
        assert!(
            report.violations().iter().any(|v| matches!(
                v,
                PermissionViolation::ForbiddenScope { scope: s, .. } if s == scope
            )),
            "Expected ForbiddenScope violation for '{scope}'"
        );
    }
}

#[test]
fn test_strict_minimal_vs_standard_policy() {
    let mut perms = HashMap::new();
    perms.insert("contents".to_string(), "read".to_string());
    perms.insert("metadata".to_string(), "read".to_string());
    perms.insert("pull_requests".to_string(), "read".to_string());

    // 1. StrictMinimal rejects pull_requests because additional scopes are not allowed
    let strict_policy = PermissionPolicy::strict_minimal();
    let strict_report = PermissionAuditor::audit_permissions(&perms, &strict_policy);
    assert!(!strict_report.is_compliant());
    assert!(strict_report.violations().iter().any(|v| matches!(
        v,
        PermissionViolation::UnexpectedScope { scope, .. } if scope == "pull_requests"
    )));

    // 2. Standard permits benign read scopes like pull_requests: read
    let standard_policy = PermissionPolicy::standard();
    let standard_report = PermissionAuditor::audit_permissions(&perms, &standard_policy);
    assert!(standard_report.is_compliant());
    assert!(standard_report.violations().is_empty());
}

#[test]
fn test_custom_policy_builder() {
    let custom_policy = PermissionPolicy::builder("CustomAudit")
        .require_scope("contents", "read")
        .require_scope("metadata", "read")
        .require_scope("issues", "read")
        .forbid_category("billing")
        .forbid_level("maintain")
        .allow_additional_read_scopes(true)
        .allow_additional_scope("pull_requests")
        .build();

    assert_eq!(custom_policy.name, "CustomAudit");
    assert_eq!(custom_policy.required_scopes.len(), 3);
    assert!(custom_policy
        .forbidden_categories
        .contains(&"billing".to_string()));
    assert!(custom_policy
        .forbidden_levels
        .contains(&"maintain".to_string()));

    let mut perms = HashMap::new();
    perms.insert("contents".to_string(), "read".to_string());
    perms.insert("metadata".to_string(), "read".to_string());
    perms.insert("issues".to_string(), "read".to_string());
    perms.insert("pull_requests".to_string(), "read".to_string());

    let report = PermissionAuditor::audit_permissions(&perms, &custom_policy);
    assert!(report.is_compliant());

    // Unexpected scope not in whitelist
    perms.insert("discussions".to_string(), "read".to_string());
    let report_discussions = PermissionAuditor::audit_permissions(&perms, &custom_policy);
    assert!(!report_discussions.is_compliant());
}

#[test]
fn test_installation_model_methods() {
    let mut perms = HashMap::new();
    perms.insert("contents".to_string(), "read".to_string());
    perms.insert("metadata".to_string(), "read".to_string());

    let installation = sample_installation(perms, false);

    assert!(installation.validate_permissions().is_ok());
    let report = installation.audit_permissions();
    assert!(report.is_compliant());

    // Non-compliant installation with write permission
    let mut bad_perms = HashMap::new();
    bad_perms.insert("contents".to_string(), "write".to_string());
    bad_perms.insert("metadata".to_string(), "read".to_string());

    let bad_installation = sample_installation(bad_perms, false);
    assert!(bad_installation.validate_permissions().is_err());
}

#[test]
fn test_suspended_installation_rejected() {
    let mut perms = HashMap::new();
    perms.insert("contents".to_string(), "read".to_string());
    perms.insert("metadata".to_string(), "read".to_string());

    let suspended_inst = sample_installation(perms, true);
    assert!(suspended_inst.is_suspended());

    let report = suspended_inst.audit_permissions();
    assert!(!report.is_compliant());
    assert!(report.violations().iter().any(|v| matches!(
        v,
        PermissionViolation::SuspendedInstallation { installation_id } if *installation_id == 998877
    )));

    let err = suspended_inst.validate_permissions().unwrap_err();
    assert!(err.to_string().contains("is currently suspended"));
}

#[test]
fn test_installation_token_model_methods() {
    let mut perms = HashMap::new();
    perms.insert("contents".to_string(), "read".to_string());
    perms.insert("metadata".to_string(), "read".to_string());

    let token = GitHubInstallationToken {
        token: "ghs_test_123456789".to_string(),
        expires_at: Utc::now() + ChronoDuration::minutes(50),
        permissions: perms,
        repository_selection: RepositorySelection::Selected,
        repositories: None,
    };

    assert!(!token.is_expired());
    assert!(!token.is_expiring_soon(300));
    assert!(token.is_expiring_soon(3600)); // expires within 60 mins

    assert!(token.validate_permissions().is_ok());
    let report = token.audit_permissions();
    assert!(report.is_compliant());
}

#[test]
fn test_installation_token_request_serialization() {
    let req = InstallationTokenRequest::minimal_read_only();
    assert!(req.repositories.is_none());
    assert!(req.permissions.is_some());

    let perms = req.permissions.as_ref().unwrap();
    assert_eq!(perms.contents.as_str(), "read");
    assert_eq!(perms.metadata.as_str(), "read");

    let scoped_req = InstallationTokenRequest::for_repositories(["repo-1", "repo-2"])
        .with_repository_ids(vec![101, 102]);
    assert_eq!(
        scoped_req.repositories.as_deref(),
        Some(&["repo-1".to_string(), "repo-2".to_string()][..])
    );
    assert_eq!(scoped_req.repository_ids.as_deref(), Some(&[101, 102][..]));

    let json_val = serde_json::to_value(&scoped_req).unwrap();
    assert_eq!(json_val["repositories"][0], "repo-1");
    assert_eq!(json_val["repository_ids"][0], 101);
    assert_eq!(json_val["permissions"]["contents"], "read");
}

#[test]
fn test_markdown_and_json_report_generation() {
    let mut perms = HashMap::new();
    perms.insert("contents".to_string(), "read".to_string());
    perms.insert("metadata".to_string(), "read".to_string());
    perms.insert("secrets".to_string(), "read".to_string());
    perms.insert("issues".to_string(), "write".to_string());

    let policy = PermissionPolicy::strict_minimal();
    let report = PermissionAuditor::audit_permissions(&perms, &policy);

    assert!(!report.is_compliant());
    let md = report.to_markdown_report();
    assert!(md.contains("GitHub App Permission Audit Report"));
    assert!(md.contains("NON-COMPLIANT"));
    assert!(md.contains("[!CAUTION]"));
    assert!(md.contains("`REJECTED`"));

    let json_str = report.to_json().unwrap();
    let deserialized: PermissionAuditReport = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.policy_name, "StrictMinimal");
    assert!(!deserialized.compliant);
    assert_eq!(deserialized.violations.len(), report.violations.len());
}

#[tokio::test]
async fn test_scoped_installation_access_token_mock_http_live_exchange() {
    let app = Router::new().route(
        "/app/installations/:id/access_tokens",
        post(
            |AxumPath(id): AxumPath<u64>, Json(body): Json<serde_json::Value>| async move {
                assert_eq!(id, 4567);
                // Verify client sent scoped permissions
                assert_eq!(body["permissions"]["contents"], "read");
                assert_eq!(body["permissions"]["metadata"], "read");
                assert_eq!(body["repositories"][0], "secret-repo");

                Json(json!({
                    "token": "ghs_live_scoped_secret_token",
                    "expires_at": (Utc::now() + ChronoDuration::hours(1)).to_rfc3339(),
                    "permissions": {
                        "contents": "read",
                        "metadata": "read"
                    },
                    "repository_selection": "selected"
                }))
            },
        ),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_handle = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    let mock_base = format!("http://{addr}");
    let client = GitHubApiClient::new().with_base_url_override(mock_base);

    let req = InstallationTokenRequest::for_repositories(["secret-repo"]);
    let token = client
        .create_scoped_installation_access_token(4567, "test-jwt", &req)
        .await
        .unwrap();

    assert_eq!(token.token(), "ghs_live_scoped_secret_token");
    assert_eq!(
        token.permissions.get("contents").map(|s| s.as_str()),
        Some("read")
    );
    assert!(token.validate_permissions().is_ok());

    server_handle.abort();
}

#[tokio::test]
async fn test_client_rejects_over_permissioned_token_response() {
    let app = Router::new().route(
        "/app/installations/:id/access_tokens",
        post(|AxumPath(_): AxumPath<u64>| async move {
            // Simulated malicious or misconfigured upstream GitHub returning write access
            Json(json!({
                "token": "ghs_over_permissioned_token",
                "expires_at": (Utc::now() + ChronoDuration::hours(1)).to_rfc3339(),
                "permissions": {
                    "contents": "write",
                    "metadata": "read"
                },
                "repository_selection": "all"
            }))
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_handle = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    let mock_base = format!("http://{addr}");
    let client = GitHubApiClient::new().with_base_url_override(mock_base);

    let req = InstallationTokenRequest::minimal_read_only();
    let result = client
        .create_scoped_installation_access_token(9999, "test-jwt", &req)
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        GitHubExtractorError::PermissionViolation(msg) => {
            assert!(msg.contains("Excessive permission level for scope 'contents'"));
        }
        other => panic!("Expected PermissionViolation error, got: {other:?}"),
    }

    server_handle.abort();
}
