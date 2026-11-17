//! Custom JSON extractor enforcing schema deserialization and domain validation.

use axum::async_trait;
use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::de::DeserializeOwned;
use web_repo_to_ai_core::{CoreError, RequestId, Validatable};

/// Axum extractor that deserializes JSON request bodies and enforces [`Validatable`] constraints,
/// automatically generating RFC 7807 problem details responses on syntax or validation errors.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validatable,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let request_id = req.extensions().get::<RequestId>().map(ToString::to_string);

        let bytes = match Bytes::from_request(req, state).await {
            Ok(b) => b,
            Err(rejection) => {
                let err = CoreError::InvalidRequestSchema(format!(
                    "Failed to read request body: {rejection}"
                ));
                let status = StatusCode::BAD_REQUEST;
                return Err((status, Json(err.to_api_error(request_id))).into_response());
            }
        };

        let data: T = match serde_json::from_slice(&bytes) {
            Ok(parsed) => parsed,
            Err(err) => {
                let core_err =
                    CoreError::InvalidRequestSchema(format!("Malformed JSON payload: {err}"));
                let status = StatusCode::BAD_REQUEST;
                return Err((status, Json(core_err.to_api_error(request_id))).into_response());
            }
        };

        if let Err(val_err) = data.validate() {
            let status =
                StatusCode::from_u16(val_err.http_status()).unwrap_or(StatusCode::BAD_REQUEST);
            return Err((status, Json(val_err.to_api_error(request_id))).into_response());
        }

        Ok(ValidatedJson(data))
    }
}
