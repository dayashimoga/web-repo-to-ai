//! GitHub App authentication, installation revocation, and webhook synchronization handlers.

use axum::body::Bytes;
use axum::extract::{Extension, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use tracing::{info, warn};
use uuid::Uuid;
use web_repo_to_ai_core::{CoreError, RequestId};
use web_repo_to_ai_github_extractor::callback::{
    GitHubWebhookEvent, InstallationAction, WebhookEventParser,
};

use crate::dto::github::{
    InstallationPathParams, InstallationSummaryResponse, RevokeInstallationResponse,
    WebhookReceiptResponse,
};
use crate::error::{not_found_problem, to_problem_response};
use crate::extractors::ValidatedPath;
use crate::state::AppState;

/// Header name used for tenant session scoping.
pub const HEADER_SESSION_ID: &str = "x-session-id";

/// Extracts and parses the `X-Session-ID` header as a valid [`Uuid`].
fn extract_session_id(headers: &HeaderMap) -> Result<Uuid, CoreError> {
    let header_val = headers
        .get(HEADER_SESSION_ID)
        .or_else(|| headers.get("X-Session-ID"))
        .ok_or_else(|| CoreError::Validation {
            field: "X-Session-ID".to_string(),
            reason: "Missing mandatory X-Session-ID header for tenant credential isolation"
                .to_string(),
        })?;

    let s = header_val.to_str().map_err(|_| CoreError::Validation {
        field: "X-Session-ID".to_string(),
        reason: "X-Session-ID header must contain valid ASCII characters".to_string(),
    })?;

    Uuid::parse_str(s.trim()).map_err(|e| CoreError::Validation {
        field: "X-Session-ID".to_string(),
        reason: format!("Invalid UUID formatted string in X-Session-ID: {e}"),
    })
}

/// Lists all active authorized GitHub App installations for the calling session.
///
/// `GET /api/v1/auth/github/installations`
pub async fn list_installations_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    req_ext: Option<Extension<RequestId>>,
) -> Response {
    let request_id = req_ext.map(|Extension(r)| r);

    let session_id = match extract_session_id(&headers) {
        Ok(id) => id,
        Err(err) => return to_problem_response(err, request_id),
    };

    match state
        .token_storage
        .list_active_installations(session_id)
        .await
    {
        Ok(rows) => {
            let summaries: Vec<InstallationSummaryResponse> =
                rows.iter().map(InstallationSummaryResponse::from).collect();
            (StatusCode::OK, Json(summaries)).into_response()
        }
        Err(err) => to_problem_response(err, request_id),
    }
}

/// Explicitly revokes a GitHub App installation and its associated access token.
///
/// `DELETE /api/v1/auth/github/installations/:installation_id`
///
/// In accordance with Rule 05:
/// 1. Verifies the installation belongs to the authenticated session (`X-Session-ID`).
/// 2. If an active token exists, calls GitHub REST API `DELETE /installation/token` to invalidate upstream.
/// 3. Zeros out token memory and ciphertext in local storage.
/// 4. Returns confirmed revocation receipt.
pub async fn revoke_installation_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ValidatedPath(params): ValidatedPath<InstallationPathParams>,
    req_ext: Option<Extension<RequestId>>,
) -> Response {
    let request_id = req_ext.map(|Extension(r)| r);

    let session_id = match extract_session_id(&headers) {
        Ok(id) => id,
        Err(err) => return to_problem_response(err, request_id),
    };

    let install_id = params.installation_id;

    // 1. Verify existence and session ownership
    let installation = match state
        .token_storage
        .get_installation(session_id, install_id)
        .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return not_found_problem("GitHubInstallation", &install_id.to_string(), request_id);
        }
        Err(err) => return to_problem_response(err, request_id),
    };

    if installation.is_revoked() {
        return not_found_problem("GitHubInstallation", &install_id.to_string(), request_id);
    }

    // 2. Upstream token revocation if active
    let mut upstream_revoked = false;
    if let Ok(Some(plaintext_token)) = state
        .token_storage
        .get_active_token(session_id, install_id)
        .await
    {
        match state
            .github_client
            .revoke_installation_access_token(&plaintext_token)
            .await
        {
            Ok(()) => {
                upstream_revoked = true;
                info!(
                    "Upstream token revoked for installation {install_id} (session {session_id})"
                );
            }
            Err(e) => {
                warn!("Upstream GitHub token revocation returned non-fatal error: {e}");
            }
        }
    }

    // 3. Local storage revocation and zeroization
    match state
        .token_storage
        .revoke_installation(session_id, install_id)
        .await
    {
        Ok(true) => {
            info!("Local credentials revoked for installation {install_id} (session {session_id})");
            let resp = RevokeInstallationResponse {
                status: "revoked".to_string(),
                session_id,
                installation_id: install_id,
                revoked_at: Utc::now(),
                upstream_revoked,
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Ok(false) => not_found_problem("GitHubInstallation", &install_id.to_string(), request_id),
        Err(err) => to_problem_response(err, request_id),
    }
}

/// Receives, verifies, and processes GitHub App webhook events (`X-Hub-Signature-256`).
///
/// `POST /api/v1/auth/github/webhook`
///
/// Synchronizes installation uninstalls (`installation.deleted`) and suspensions (`installation.suspend`),
/// automatically purging credentials across all tenant records.
pub async fn github_webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    req_ext: Option<Extension<RequestId>>,
    body: Bytes,
) -> Response {
    let request_id = req_ext.map(|Extension(r)| r);

    // 1. Webhook secret check
    let secret = match &state.webhook_secret {
        Some(s) if !s.is_empty() => s.as_bytes(),
        _ => {
            warn!("Incoming GitHub webhook rejected: GITHUB_APP_WEBHOOK_SECRET is not configured");
            let err =
                CoreError::Config("Webhook verification secret is not configured".to_string());
            return to_problem_response(err, request_id);
        }
    };

    // 2. Extract mandatory verification headers
    let sig_header = match headers
        .get("x-hub-signature-256")
        .and_then(|h| h.to_str().ok())
    {
        Some(s) => s,
        None => {
            let err = CoreError::Validation {
                field: "X-Hub-Signature-256".to_string(),
                reason: "Missing mandatory HMAC-SHA256 signature header".to_string(),
            };
            return to_problem_response(err, request_id);
        }
    };

    let event_header = match headers.get("x-github-event").and_then(|h| h.to_str().ok()) {
        Some(e) => e,
        None => {
            let err = CoreError::Validation {
                field: "X-GitHub-Event".to_string(),
                reason: "Missing mandatory X-GitHub-Event header".to_string(),
            };
            return to_problem_response(err, request_id);
        }
    };

    let delivery_header = headers
        .get("x-github-delivery")
        .and_then(|h| h.to_str().ok());

    // 3. HMAC Verification & Replay Protection
    let parsed_event = {
        let mut tracker_guard = state.webhook_tracker.lock().await;
        WebhookEventParser::parse_webhook_event(
            secret,
            &body,
            sig_header,
            event_header,
            delivery_header,
            Some(&mut *tracker_guard),
        )
    };

    let event = match parsed_event {
        Ok(ev) => ev,
        Err(err) => {
            warn!("GitHub webhook verification failed: {err}");
            let core_err = CoreError::Security(format!("Webhook verification failed: {err}"));
            return to_problem_response(core_err, request_id);
        }
    };

    // 4. Lifecycle Event Synchronization
    match event {
        GitHubWebhookEvent::Installation {
            action,
            installation,
        } => {
            let inst_id = installation.id as i64;
            let action_str = format!("{action:?}").to_lowercase();

            match action {
                InstallationAction::Deleted | InstallationAction::Suspend => {
                    let revoked_count = match state
                        .token_storage
                        .revoke_all_by_installation_id(inst_id)
                        .await
                    {
                        Ok(c) => c,
                        Err(e) => return to_problem_response(e, request_id),
                    };

                    info!(
                        "Webhook lifecycle sync: installation {inst_id} action={action_str}; revoked {revoked_count} storage record(s)"
                    );

                    let receipt = WebhookReceiptResponse {
                        status: "processed".to_string(),
                        event: "installation".to_string(),
                        action: Some(action_str),
                        installation_id: Some(inst_id),
                        revoked_count,
                        received_at: Utc::now(),
                    };
                    (StatusCode::OK, Json(receipt)).into_response()
                }
                _ => {
                    let receipt = WebhookReceiptResponse {
                        status: "acknowledged".to_string(),
                        event: "installation".to_string(),
                        action: Some(action_str),
                        installation_id: Some(inst_id),
                        revoked_count: 0,
                        received_at: Utc::now(),
                    };
                    (StatusCode::OK, Json(receipt)).into_response()
                }
            }
        }
        GitHubWebhookEvent::Ping { hook_id, zen } => {
            info!("Received GitHub webhook ping for hook {hook_id}: {zen}");
            let receipt = WebhookReceiptResponse {
                status: "acknowledged".to_string(),
                event: "ping".to_string(),
                action: None,
                installation_id: None,
                revoked_count: 0,
                received_at: Utc::now(),
            };
            (StatusCode::OK, Json(receipt)).into_response()
        }
        _ => {
            let receipt = WebhookReceiptResponse {
                status: "ignored".to_string(),
                event: event_header.to_string(),
                action: None,
                installation_id: None,
                revoked_count: 0,
                received_at: Utc::now(),
            };
            (StatusCode::OK, Json(receipt)).into_response()
        }
    }
}
