//! # Callback & Webhook Security Integration Tests
//!
//! Comprehensive verification of OAuth callback validation, open-redirect defense,
//! installation setup callback handling, HMAC-SHA256 webhook signatures, delivery deduplication,
//! and live Axum webhook integration under Rule 05.

use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Router;
use bytes::Bytes;
use chrono::{Duration as ChronoDuration, Utc};
use serde_json::json;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

use web_repo_to_ai_github_extractor::{
    GitHubAccountSummary, GitHubExtractorError, GitHubInstallation, GitHubWebhookEvent,
    InMemoryOAuthStateStore, InMemoryWebhookDeliveryTracker, InstallationAction,
    InstallationCallbackParams, InstallationCallbackValidator, InstallationRepoAction,
    OAuthCallbackParams, OAuthCallbackValidator, OAuthState, OAuthStateStore, OpenRedirectGuard,
    RepositorySelection, SetupAction, WebhookDeliveryTracker, WebhookEventParser,
    WebhookSignatureVerifier,
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

fn sample_installation(id: u64) -> GitHubInstallation {
    let mut perms = HashMap::new();
    perms.insert("contents".to_string(), "read".to_string());
    perms.insert("metadata".to_string(), "read".to_string());

    GitHubInstallation {
        id,
        account: sample_account(),
        repository_selection: RepositorySelection::Selected,
        access_tokens_url: format!("https://api.github.com/app/installations/{id}/access_tokens"),
        repositories_url: "https://api.github.com/installation/repositories".to_string(),
        html_url: format!("https://github.com/settings/installations/{id}"),
        app_id: 42,
        app_slug: Some("web-repo-to-ai".to_string()),
        target_id: 1001,
        target_type: "Organization".to_string(),
        permissions: perms,
        events: vec![
            "installation".to_string(),
            "installation_repositories".to_string(),
        ],
        created_at: Utc::now() - ChronoDuration::days(5),
        updated_at: Utc::now() - ChronoDuration::hours(1),
        suspended_at: None,
        suspended_by: None,
    }
}

#[test]
fn test_oauth_callback_valid_code_and_state() {
    let mut store = InMemoryOAuthStateStore::new();
    let state = OAuthState::new()
        .with_session_id("sess-xyz-987")
        .with_tenant_id("tenant-omega")
        .with_redirect_uri("/settings/github");
    let nonce = state.nonce.clone();
    store.save_state(state).unwrap();

    let params = OAuthCallbackParams {
        code: Some("auth_code_alphanumeric_12345".to_string()),
        state: Some(nonce.clone()),
        error: None,
        error_description: None,
        error_uri: None,
    };

    let allowed_origins = ["https://app.example.com"];
    let validated = OAuthCallbackValidator::validate_callback(
        &params,
        &mut store,
        Some("sess-xyz-987"),
        &allowed_origins,
    )
    .unwrap();

    assert_eq!(validated.code, "auth_code_alphanumeric_12345");
    assert_eq!(validated.state.nonce, nonce);
    assert_eq!(validated.state.session_id.as_deref(), Some("sess-xyz-987"));
    assert_eq!(validated.state.tenant_id.as_deref(), Some("tenant-omega"));

    // State must be consumed on single use
    assert_eq!(store.active_count(), 0);
}

#[test]
fn test_oauth_callback_replay_prevention() {
    let mut store = InMemoryOAuthStateStore::new();
    let state = OAuthState::new().with_session_id("sess-single-use");
    let nonce = state.nonce.clone();
    store.save_state(state).unwrap();

    let params = OAuthCallbackParams {
        code: Some("code-1".to_string()),
        state: Some(nonce),
        error: None,
        error_description: None,
        error_uri: None,
    };

    // First consumption succeeds
    let first = OAuthCallbackValidator::validate_callback(
        &params,
        &mut store,
        Some("sess-single-use"),
        &[],
    );
    assert!(first.is_ok());

    // Second consumption (replay attack) fails
    let replay = OAuthCallbackValidator::validate_callback(
        &params,
        &mut store,
        Some("sess-single-use"),
        &[],
    );
    assert!(replay.is_err());
    let err_str = replay.unwrap_err().to_string();
    assert!(err_str.contains("not found or already consumed"));
}

#[test]
fn test_oauth_callback_session_id_mismatch() {
    let mut store = InMemoryOAuthStateStore::new();
    let state = OAuthState::new().with_session_id("legitimate-session");
    let nonce = state.nonce.clone();
    store.save_state(state).unwrap();

    let params = OAuthCallbackParams {
        code: Some("code-legit".to_string()),
        state: Some(nonce),
        error: None,
        error_description: None,
        error_uri: None,
    };

    let result = OAuthCallbackValidator::validate_callback(
        &params,
        &mut store,
        Some("attacker-session"),
        &[],
    );
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("session mismatch"));
}

#[test]
fn test_oauth_callback_missing_or_empty_parameters() {
    let mut store = InMemoryOAuthStateStore::new();

    // 1. Missing state
    let params_no_state = OAuthCallbackParams {
        code: Some("code-123".to_string()),
        state: None,
        error: None,
        error_description: None,
        error_uri: None,
    };
    assert!(
        OAuthCallbackValidator::validate_callback(&params_no_state, &mut store, None, &[]).is_err()
    );

    // 2. Missing code
    let params_no_code = OAuthCallbackParams {
        code: None,
        state: Some("some-state".to_string()),
        error: None,
        error_description: None,
        error_uri: None,
    };
    assert!(
        OAuthCallbackValidator::validate_callback(&params_no_code, &mut store, None, &[]).is_err()
    );

    // 3. Malformed code with invalid characters
    let params_bad_code = OAuthCallbackParams {
        code: Some("code with spaces and <script>".to_string()),
        state: Some("some-state".to_string()),
        error: None,
        error_description: None,
        error_uri: None,
    };
    assert!(
        OAuthCallbackValidator::validate_callback(&params_bad_code, &mut store, None, &[]).is_err()
    );
}

#[test]
fn test_oauth_callback_github_error_handling() {
    let mut store = InMemoryOAuthStateStore::new();

    let params = OAuthCallbackParams {
        code: None,
        state: Some("state-abc".to_string()),
        error: Some("access_denied".to_string()),
        error_description: Some("The user has denied your application access.".to_string()),
        error_uri: Some("https://docs.github.com/errors".to_string()),
    };

    let result = OAuthCallbackValidator::validate_callback(&params, &mut store, None, &[]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        GitHubExtractorError::CallbackSecurity(msg) => {
            assert!(msg.contains("access_denied"));
            assert!(msg.contains("The user has denied your application access."));
        }
        other => panic!("Expected CallbackSecurity error, got {other:?}"),
    }
}

#[test]
fn test_open_redirect_guard() {
    let allowed_origins = ["https://app.example.com", "http://localhost:5173"];

    // Safe relative paths
    assert!(OpenRedirectGuard::is_safe_relative_path("/dashboard"));
    assert!(OpenRedirectGuard::is_safe_relative_path(
        "/settings/integrations?tab=github"
    ));
    assert!(OpenRedirectGuard::is_safe_relative_path("/repos/acme/core"));

    // Dangerous protocol-relative and malicious paths
    assert!(!OpenRedirectGuard::is_safe_relative_path("//attacker.com"));
    assert!(!OpenRedirectGuard::is_safe_relative_path(
        "//attacker.com/evil"
    ));
    assert!(!OpenRedirectGuard::is_safe_relative_path("/\\attacker.com"));
    assert!(!OpenRedirectGuard::is_safe_relative_path("/\tattacker.com"));
    assert!(!OpenRedirectGuard::is_safe_relative_path(
        "not-starting-with-slash"
    ));
    assert!(!OpenRedirectGuard::is_safe_relative_path(""));

    // Absolute URLs matching allowed origins
    assert!(OpenRedirectGuard::is_safe_redirect(
        "https://app.example.com/onboarding",
        &allowed_origins
    ));
    assert!(OpenRedirectGuard::is_safe_redirect(
        "http://localhost:5173/settings",
        &allowed_origins
    ));

    // Absolute URLs to untrusted domains
    assert!(!OpenRedirectGuard::is_safe_redirect(
        "https://evil-phishing.com/steal-token",
        &allowed_origins
    ));
    assert!(!OpenRedirectGuard::is_safe_redirect(
        "javascript:alert(document.cookie)",
        &allowed_origins
    ));
    assert!(!OpenRedirectGuard::is_safe_redirect(
        "data:text/html,<script>evil()</script>",
        &allowed_origins
    ));
    assert!(!OpenRedirectGuard::is_safe_redirect(
        "https://app.example.com:8443/port-hijack",
        &allowed_origins
    ));

    // validate_redirect method
    assert!(OpenRedirectGuard::validate_redirect("/safe/path", &allowed_origins).is_ok());
    assert!(OpenRedirectGuard::validate_redirect("https://evil.com", &allowed_origins).is_err());
}

#[test]
fn test_installation_setup_callback_validation() {
    let mut store = InMemoryOAuthStateStore::new();
    let state = OAuthState::new().with_session_id("sess-install-123");
    let nonce = state.nonce.clone();
    store.save_state(state).unwrap();

    // 1. Valid install action
    let params_install = InstallationCallbackParams {
        installation_id: Some(12345),
        setup_action: Some("install".to_string()),
        code: Some("oauth_code_abc".to_string()),
        state: Some(nonce),
    };

    let validated = InstallationCallbackValidator::validate_installation_callback(
        &params_install,
        Some(&mut store),
        Some("sess-install-123"),
    )
    .unwrap();

    assert_eq!(validated.installation_id, 12345);
    assert_eq!(validated.setup_action, SetupAction::Install);
    assert_eq!(validated.code.as_deref(), Some("oauth_code_abc"));
    assert!(validated.state.is_some());

    // 2. Valid update action without state
    let params_update = InstallationCallbackParams {
        installation_id: Some(9999),
        setup_action: Some("update".to_string()),
        code: None,
        state: None,
    };
    let update_val =
        InstallationCallbackValidator::validate_installation_callback(&params_update, None, None)
            .unwrap();
    assert_eq!(update_val.installation_id, 9999);
    assert_eq!(update_val.setup_action, SetupAction::Update);

    // 3. Missing installation ID
    let params_no_id = InstallationCallbackParams {
        installation_id: None,
        setup_action: Some("install".to_string()),
        code: None,
        state: None,
    };
    assert!(
        InstallationCallbackValidator::validate_installation_callback(&params_no_id, None, None)
            .is_err()
    );

    // 4. Invalid setup action
    let params_bad_action = InstallationCallbackParams {
        installation_id: Some(123),
        setup_action: Some("unauthorized_action".to_string()),
        code: None,
        state: None,
    };
    assert!(
        InstallationCallbackValidator::validate_installation_callback(
            &params_bad_action,
            None,
            None
        )
        .is_err()
    );
}

#[test]
fn test_webhook_signature_verification_hmac_sha256() {
    let secret = b"super_secret_webhook_key_2026";
    let body = br#"{"action":"created","installation":{"id":55555}}"#;

    // Generate valid signature
    let header_val = WebhookSignatureVerifier::compute_signature_header(secret, body);
    assert!(header_val.starts_with("sha256="));
    assert_eq!(header_val.len(), 7 + 64);

    // Verify valid signature
    assert!(WebhookSignatureVerifier::verify_signature(
        secret,
        body,
        &header_val
    ));
    assert!(WebhookSignatureVerifier::verify_header(secret, body, &header_val).is_ok());

    // Tampered body must fail
    let tampered_body = br#"{"action":"deleted","installation":{"id":55555}}"#;
    assert!(!WebhookSignatureVerifier::verify_signature(
        secret,
        tampered_body,
        &header_val
    ));
    assert!(WebhookSignatureVerifier::verify_header(secret, tampered_body, &header_val).is_err());

    // Wrong secret must fail
    let wrong_secret = b"wrong_secret_key";
    assert!(!WebhookSignatureVerifier::verify_signature(
        wrong_secret,
        body,
        &header_val
    ));

    // Malformed header missing sha256= prefix
    assert!(!WebhookSignatureVerifier::verify_signature(
        secret,
        body,
        "abcdef0123456789"
    ));

    // Malformed hex length
    assert!(!WebhookSignatureVerifier::verify_signature(
        secret,
        body,
        "sha256=shorthex"
    ));
}

#[test]
fn test_webhook_delivery_tracker_replay_defense() {
    let mut tracker = InMemoryWebhookDeliveryTracker::new(ChronoDuration::minutes(30));
    let delivery_guid = "7b2d5a3c-9e1f-4d6a-8c2b-3a4e5f6a7b8c";

    assert!(!tracker.is_duplicate(delivery_guid));

    // First recording succeeds
    assert!(tracker.record_delivery(delivery_guid).is_ok());
    assert!(tracker.is_duplicate(delivery_guid));
    assert_eq!(tracker.active_count(), 1);

    // Second recording (duplicate delivery / replay attack) fails
    let replay_err = tracker.record_delivery(delivery_guid);
    assert!(replay_err.is_err());
    assert!(replay_err
        .unwrap_err()
        .to_string()
        .contains("Duplicate webhook delivery GUID"));

    // Different delivery GUID succeeds
    let new_guid = "11111111-2222-3333-4444-555555555555";
    assert!(tracker.record_delivery(new_guid).is_ok());
    assert_eq!(tracker.active_count(), 2);
}

#[test]
fn test_webhook_event_parser_ping_event() {
    let secret = b"test_secret";
    let body = br#"{"zen":"Design for failure.","hook_id":1234567}"#;
    let sig_header = WebhookSignatureVerifier::compute_signature_header(secret, body);

    let event = WebhookEventParser::parse_webhook_event(
        secret,
        body,
        &sig_header,
        "ping",
        Some("delivery-ping-1"),
        None,
    )
    .unwrap();

    match event {
        GitHubWebhookEvent::Ping { zen, hook_id } => {
            assert_eq!(zen, "Design for failure.");
            assert_eq!(hook_id, 1234567);
        }
        other => panic!("Expected Ping event, got: {other:?}"),
    }
}

#[test]
fn test_webhook_event_parser_installation_events() {
    let secret = b"test_secret";
    let installation = sample_installation(778899);

    // 1. Installation created
    let payload_created = json!({
        "action": "created",
        "installation": installation
    });
    let body_created = serde_json::to_vec(&payload_created).unwrap();
    let sig_created = WebhookSignatureVerifier::compute_signature_header(secret, &body_created);

    let event_created = WebhookEventParser::parse_webhook_event(
        secret,
        &body_created,
        &sig_created,
        "installation",
        Some("del-inst-1"),
        None,
    )
    .unwrap();

    match event_created {
        GitHubWebhookEvent::Installation {
            action,
            installation,
        } => {
            assert_eq!(action, InstallationAction::Created);
            assert_eq!(installation.id, 778899);
        }
        other => panic!("Expected Installation event, got: {other:?}"),
    }

    // 2. Installation suspended
    let payload_suspend = json!({
        "action": "suspend",
        "installation": installation
    });
    let body_suspend = serde_json::to_vec(&payload_suspend).unwrap();
    let sig_suspend = WebhookSignatureVerifier::compute_signature_header(secret, &body_suspend);

    let event_suspend = WebhookEventParser::parse_webhook_event(
        secret,
        &body_suspend,
        &sig_suspend,
        "installation",
        Some("del-inst-2"),
        None,
    )
    .unwrap();

    match event_suspend {
        GitHubWebhookEvent::Installation { action, .. } => {
            assert_eq!(action, InstallationAction::Suspend);
        }
        other => panic!("Expected Installation event, got: {other:?}"),
    }
}

#[test]
fn test_webhook_event_parser_installation_repositories_event() {
    let secret = b"test_secret";
    let installation = sample_installation(889900);

    let payload = json!({
        "action": "added",
        "installation": installation,
        "repositories_added": [
            {
                "id": 101,
                "node_id": "node_101",
                "name": "core-engine",
                "full_name": "acme-corp/core-engine",
                "private": true,
                "default_branch": "main",
                "html_url": "https://github.com/acme-corp/core-engine"
            }
        ],
        "repositories_removed": []
    });

    let body = serde_json::to_vec(&payload).unwrap();
    let sig = WebhookSignatureVerifier::compute_signature_header(secret, &body);

    let event = WebhookEventParser::parse_webhook_event(
        secret,
        &body,
        &sig,
        "installation_repositories",
        Some("del-repos-1"),
        None,
    )
    .unwrap();

    match event {
        GitHubWebhookEvent::InstallationRepositories {
            action,
            installation,
            repositories_added,
            repositories_removed,
        } => {
            assert_eq!(action, InstallationRepoAction::Added);
            assert_eq!(installation.id, 889900);
            assert_eq!(repositories_added.len(), 1);
            assert_eq!(repositories_added[0].full_name, "acme-corp/core-engine");
            assert!(repositories_removed.is_empty());
        }
        other => panic!("Expected InstallationRepositories event, got: {other:?}"),
    }
}

#[test]
fn test_webhook_event_parser_fails_without_valid_signature() {
    let secret = b"real_secret";
    let body = br#"{"action":"created"}"#;
    let forged_sig = "sha256=0000000000000000000000000000000000000000000000000000000000000000";

    let result = WebhookEventParser::parse_webhook_event(
        secret,
        body,
        forged_sig,
        "installation",
        None,
        None,
    );

    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("Invalid HMAC-SHA256 signature"));
}

#[tokio::test]
async fn test_live_axum_mock_webhook_endpoint_integration() {
    let secret: &'static [u8] = b"live_axum_webhook_secret_key";
    let tracker = Arc::new(Mutex::new(InMemoryWebhookDeliveryTracker::new(
        ChronoDuration::hours(1),
    )));

    let tracker_clone = Arc::clone(&tracker);
    let app = Router::new().route(
        "/api/v1/github/webhooks",
        post(move |headers: HeaderMap, body: Bytes| {
            let tracker_arc = Arc::clone(&tracker_clone);
            async move {
                let sig = match headers.get("x-hub-signature-256") {
                    Some(val) => val.to_str().unwrap_or(""),
                    None => return (StatusCode::UNAUTHORIZED, "Missing signature").into_response(),
                };

                let event_type = match headers.get("x-github-event") {
                    Some(val) => val.to_str().unwrap_or(""),
                    None => {
                        return (StatusCode::BAD_REQUEST, "Missing event header").into_response()
                    }
                };

                let delivery_id = headers
                    .get("x-github-delivery")
                    .and_then(|h| h.to_str().ok());

                let mut tr_lock = tracker_arc.lock().unwrap();
                match WebhookEventParser::parse_webhook_event(
                    secret,
                    &body,
                    sig,
                    event_type,
                    delivery_id,
                    Some(&mut *tr_lock),
                ) {
                    Ok(event) => match event {
                        GitHubWebhookEvent::Ping { .. } => {
                            (StatusCode::OK, "Ping acknowledged").into_response()
                        }
                        GitHubWebhookEvent::Installation { .. } => {
                            (StatusCode::OK, "Installation event handled").into_response()
                        }
                        _ => (StatusCode::OK, "Event handled").into_response(),
                    },
                    Err(GitHubExtractorError::CallbackSecurity(msg)) => {
                        if msg.contains("Duplicate webhook delivery GUID") {
                            (StatusCode::CONFLICT, "Duplicate delivery").into_response()
                        } else {
                            (StatusCode::UNAUTHORIZED, "Signature verification failed")
                                .into_response()
                        }
                    }
                    Err(_) => (StatusCode::BAD_REQUEST, "Payload decode error").into_response(),
                }
            }
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

    let client = reqwest::Client::new();
    let url = format!("http://{addr}/api/v1/github/webhooks");

    // 1. Valid signed ping request -> HTTP 200
    let ping_body = br#"{"zen":"Favor focus over features.","hook_id":987}"#;
    let valid_sig = WebhookSignatureVerifier::compute_signature_header(secret, ping_body);

    let resp_ping = client
        .post(&url)
        .header("x-hub-signature-256", &valid_sig)
        .header("x-github-event", "ping")
        .header("x-github-delivery", "del-live-1")
        .body(ping_body.to_vec())
        .send()
        .await
        .unwrap();

    assert_eq!(resp_ping.status(), StatusCode::OK);

    // 2. Tampered request -> HTTP 401 Unauthorized
    let tampered_body = br#"{"zen":"Tampered message!","hook_id":987}"#;
    let resp_tampered = client
        .post(&url)
        .header("x-hub-signature-256", &valid_sig) // signature computed for ping_body, not tampered_body
        .header("x-github-event", "ping")
        .header("x-github-delivery", "del-live-2")
        .body(tampered_body.to_vec())
        .send()
        .await
        .unwrap();

    assert_eq!(resp_tampered.status(), StatusCode::UNAUTHORIZED);

    // 3. Replay attack with duplicate delivery GUID -> HTTP 409 Conflict
    let resp_replay = client
        .post(&url)
        .header("x-hub-signature-256", &valid_sig)
        .header("x-github-event", "ping")
        .header("x-github-delivery", "del-live-1") // reuse del-live-1
        .body(ping_body.to_vec())
        .send()
        .await
        .unwrap();

    assert_eq!(resp_replay.status(), StatusCode::CONFLICT);

    server_handle.abort();
}
