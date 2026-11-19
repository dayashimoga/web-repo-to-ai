//! API key management route handlers.
//!
//! Provides endpoints to generate, list, and revoke API keys with strict tenant
//! boundary isolation and scope verification.

use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use web_repo_to_ai_core::auth::{ApiKeyScope, GeneratedApiKey};
use web_repo_to_ai_core::{CoreError, RequestId};

use crate::dto::{
    ApiKeyPathParams, ApiKeySummaryResponse, CreateApiKeyRequest, CreateApiKeyResponse,
    ListApiKeysResponse, RevokeApiKeyResponse,
};
use crate::error::{not_found_problem, to_problem_response};
use crate::extractors::{RequireAuth, ValidatedJson, ValidatedPath};
use crate::state::AppState;

/// Handles `POST /api/v1/auth/keys`.
///
/// Generates a new cryptographically secure API key for the caller's tenant.
/// Requires `Admin` permission scope (implicit for session callers).
/// Returns `201 Created` with the plaintext API key displayed once.
pub async fn create_api_key_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    RequireAuth(caller): RequireAuth,
    ValidatedJson(payload): ValidatedJson<CreateApiKeyRequest>,
) -> impl IntoResponse {
    if let Err(err) = caller.require_scope(&ApiKeyScope::Admin) {
        return to_problem_response(err, Some(request_id)).into_response();
    }

    let tenant_id = match caller.require_authenticated() {
        Ok(id) => id,
        Err(err) => return to_problem_response(err, Some(request_id)).into_response(),
    };

    let scopes = payload.parsed_scopes();
    let expires_at = payload.compute_expires_at();

    let generated = match GeneratedApiKey::generate(tenant_id, &payload.name, scopes, expires_at) {
        Ok(g) => g,
        Err(err) => return to_problem_response(err, Some(request_id)).into_response(),
    };

    match state.api_key_storage.save_key(&generated.record).await {
        Ok(saved) => {
            let mut scope_strings: Vec<String> =
                saved.scopes.iter().map(ToString::to_string).collect();
            scope_strings.sort();

            let response = CreateApiKeyResponse {
                id: saved.id,
                tenant_id: saved.tenant_id,
                name: saved.name,
                key: generated.plaintext,
                key_prefix: saved.key_prefix,
                scopes: scope_strings,
                created_at: saved.created_at,
                expires_at: saved.expires_at,
            };

            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(err) => to_problem_response(
            CoreError::Internal(format!("Failed to persist API key: {err}")),
            Some(request_id),
        )
        .into_response(),
    }
}

/// Handles `GET /api/v1/auth/keys`.
///
/// Lists all API keys belonging to the caller's tenant.
/// Never returns the raw secret or cryptographic hashes.
pub async fn list_api_keys_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    RequireAuth(caller): RequireAuth,
) -> impl IntoResponse {
    if let Err(err) = caller.require_scope(&ApiKeyScope::Admin) {
        return to_problem_response(err, Some(request_id)).into_response();
    }

    let tenant_id = match caller.require_authenticated() {
        Ok(id) => id,
        Err(err) => return to_problem_response(err, Some(request_id)).into_response(),
    };

    match state.api_key_storage.list_by_tenant(tenant_id).await {
        Ok(records) => {
            let keys: Vec<ApiKeySummaryResponse> = records
                .into_iter()
                .map(ApiKeySummaryResponse::from)
                .collect();
            let total = keys.len();

            (StatusCode::OK, Json(ListApiKeysResponse { keys, total })).into_response()
        }
        Err(err) => to_problem_response(
            CoreError::Internal(format!("Failed to list API keys: {err}")),
            Some(request_id),
        )
        .into_response(),
    }
}

/// Handles `DELETE /api/v1/auth/keys/:key_id`.
///
/// Revokes an API key belonging to the caller's tenant.
/// Enforces tenant ownership to prevent unauthorized revocation.
pub async fn revoke_api_key_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    RequireAuth(caller): RequireAuth,
    ValidatedPath(params): ValidatedPath<ApiKeyPathParams>,
) -> impl IntoResponse {
    if let Err(err) = caller.require_scope(&ApiKeyScope::Admin) {
        return to_problem_response(err, Some(request_id)).into_response();
    }

    let tenant_id = match caller.require_authenticated() {
        Ok(id) => id,
        Err(err) => return to_problem_response(err, Some(request_id)).into_response(),
    };

    let record = match state.api_key_storage.find_by_id(params.key_id).await {
        Ok(r) => r,
        Err(err) => {
            return to_problem_response(
                CoreError::Internal(format!("Failed to retrieve API key: {err}")),
                Some(request_id),
            )
            .into_response();
        }
    };

    match record {
        Some(r) if r.tenant_id == tenant_id => {
            if let Err(err) = state
                .api_key_storage
                .revoke_key(params.key_id, tenant_id)
                .await
            {
                return to_problem_response(
                    CoreError::Internal(format!("Failed to revoke API key: {err}")),
                    Some(request_id),
                )
                .into_response();
            }

            let response = RevokeApiKeyResponse {
                id: params.key_id,
                revoked: true,
                message: "API key successfully revoked".to_string(),
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        _ => not_found_problem("ApiKey", &params.key_id.to_string(), Some(request_id))
            .into_response(),
    }
}
