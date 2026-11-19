//! Authentication and tenant authorization extractors.
//!
//! Provides [`AuthCaller`] and [`RequireAuth`] extractors supporting Bearer API keys
//! and browser sessions (`X-Session-ID`), enforcing scope restrictions and tenant boundaries.

use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::Response;
use uuid::Uuid;
use web_repo_to_ai_core::auth::AuthenticatedCaller;
use web_repo_to_ai_core::{CoreError, RequestId};

use crate::error::to_problem_response;
use crate::state::AppState;

/// Optional authentication extractor resolving the calling principal identity.
///
/// Checks `Authorization` (`Bearer <token>` or `ApiKey <token>`), followed by
/// `X-Session-ID`. If neither is provided, resolves to [`AuthenticatedCaller::Anonymous`].
/// Returns RFC 7807 problem response if the provided token is malformed, invalid, or expired.
#[derive(Debug, Clone)]
pub struct AuthCaller(pub AuthenticatedCaller);

#[async_trait]
impl FromRequestParts<AppState> for AuthCaller {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let request_id = parts.extensions.get::<RequestId>().copied();

        // 1. Check for Authorization header
        if let Some(auth_val) = parts.headers.get(axum::http::header::AUTHORIZATION) {
            let auth_str = match auth_val.to_str() {
                Ok(s) => s.trim(),
                Err(_) => {
                    return Err(to_problem_response(
                        CoreError::Unauthorized(
                            "Authorization header must contain valid ASCII characters".into(),
                        ),
                        request_id,
                    ));
                }
            };

            let token = if let Some(stripped) = auth_str.strip_prefix("Bearer ") {
                stripped.trim()
            } else if let Some(stripped) = auth_str.strip_prefix("bearer ") {
                stripped.trim()
            } else if let Some(stripped) = auth_str.strip_prefix("ApiKey ") {
                stripped.trim()
            } else if let Some(stripped) = auth_str.strip_prefix("apikey ") {
                stripped.trim()
            } else {
                return Err(to_problem_response(
                    CoreError::Unauthorized(
                        "Invalid Authorization header format. Expected 'Bearer <token>' or 'ApiKey <token>'".into(),
                    ),
                    request_id,
                ));
            };

            if token.is_empty() {
                return Err(to_problem_response(
                    CoreError::Unauthorized("Empty authentication token provided".into()),
                    request_id,
                ));
            }

            // Hash the token with BLAKE3
            let hash = blake3::hash(token.as_bytes()).to_hex().to_string();

            match state.api_key_storage.find_by_hash(&hash).await {
                Ok(Some(record)) => {
                    if !record.is_currently_active() {
                        return Err(to_problem_response(
                            CoreError::Unauthorized(
                                "API key has expired or has been revoked".into(),
                            ),
                            request_id,
                        ));
                    }

                    // Asynchronously record usage (best-effort)
                    let _ = state.api_key_storage.record_usage(record.id).await;

                    return Ok(Self(AuthenticatedCaller::ApiKey {
                        key_id: record.id,
                        tenant_id: record.tenant_id,
                        name: record.name,
                        scopes: record.scopes,
                    }));
                }
                Ok(None) => {
                    return Err(to_problem_response(
                        CoreError::Unauthorized("Invalid API key token provided".into()),
                        request_id,
                    ));
                }
                Err(err) => {
                    return Err(to_problem_response(
                        CoreError::Internal(format!("API key verification failed: {err}")),
                        request_id,
                    ));
                }
            }
        }

        // 2. Check for X-Session-ID header
        if let Some(session_val) = parts
            .headers
            .get("X-Session-ID")
            .or_else(|| parts.headers.get("x-session-id"))
        {
            let session_str = match session_val.to_str() {
                Ok(s) => s.trim(),
                Err(_) => {
                    return Err(to_problem_response(
                        CoreError::InvalidRequestSchema(
                            "Invalid X-Session-ID header characters".into(),
                        ),
                        request_id,
                    ));
                }
            };

            match Uuid::parse_str(session_str) {
                Ok(session_id) => {
                    return Ok(Self(AuthenticatedCaller::Session { session_id }));
                }
                Err(_) => {
                    return Err(to_problem_response(
                        CoreError::InvalidRequestSchema(
                            "Invalid X-Session-ID header format; UUID expected".into(),
                        ),
                        request_id,
                    ));
                }
            }
        }

        // 3. Fallback to Anonymous
        Ok(Self(AuthenticatedCaller::Anonymous))
    }
}

/// Mandatory authentication extractor requiring a valid API key or browser session.
///
/// Rejects unauthenticated (anonymous) callers with `401 Unauthorized`.
#[derive(Debug, Clone)]
pub struct RequireAuth(pub AuthenticatedCaller);

#[async_trait]
impl FromRequestParts<AppState> for RequireAuth {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let request_id = parts.extensions.get::<RequestId>().copied();
        let AuthCaller(caller) = AuthCaller::from_request_parts(parts, state).await?;

        if let AuthenticatedCaller::Anonymous = caller {
            return Err(to_problem_response(
                CoreError::Unauthorized(
                    "Authentication required. Provide 'Authorization: Bearer <token>' or 'X-Session-ID: <uuid>'".into(),
                ),
                request_id,
            ));
        }

        Ok(Self(caller))
    }
}
