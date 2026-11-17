//! Custom JSON extractor enforcing schema deserialization and domain validation.

use axum::async_trait;
use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::response::Response;
use serde::de::DeserializeOwned;
use web_repo_to_ai_core::{CoreError, RequestId, Validatable};

use crate::error::to_problem_response;

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
        let request_id = req.extensions().get::<RequestId>().copied();

        let bytes = match Bytes::from_request(req, state).await {
            Ok(b) => b,
            Err(rejection) => {
                let debug_str = format!("{rejection:?}");
                let is_length_limit = debug_str.contains("LengthLimitError")
                    || debug_str.to_ascii_lowercase().contains("length limit")
                    || debug_str.to_ascii_lowercase().contains("too large");

                let err = if is_length_limit {
                    CoreError::PayloadTooLarge {
                        actual_bytes: 0,
                        max_bytes: 10 * 1024 * 1024,
                    }
                } else {
                    CoreError::InvalidRequestSchema(format!(
                        "Failed to read request body: {rejection}"
                    ))
                };
                return Err(to_problem_response(err, request_id));
            }
        };

        let data: T = match serde_json::from_slice(&bytes) {
            Ok(parsed) => parsed,
            Err(err) => {
                let core_err =
                    CoreError::InvalidRequestSchema(format!("Malformed JSON payload: {err}"));
                return Err(to_problem_response(core_err, request_id));
            }
        };

        if let Err(val_err) = data.validate() {
            return Err(to_problem_response(val_err, request_id));
        }

        Ok(ValidatedJson(data))
    }
}
