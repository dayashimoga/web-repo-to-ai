//! Custom Query parameter extractor enforcing domain validation.

use axum::async_trait;
use axum::extract::{FromRequestParts, Query};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::de::DeserializeOwned;
use web_repo_to_ai_core::{CoreError, RequestId, Validatable};

/// Axum extractor that parses query parameters and enforces [`Validatable`] constraints,
/// generating RFC 7807 problem details responses on syntax or validation errors.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedQuery<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validatable,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let request_id = parts.extensions.get::<RequestId>().map(ToString::to_string);

        let data: T = match Query::<T>::from_request_parts(parts, state).await {
            Ok(Query(val)) => val,
            Err(rejection) => {
                let core_err = CoreError::InvalidRequestSchema(format!(
                    "Invalid query parameters: {rejection}"
                ));
                let status = StatusCode::BAD_REQUEST;
                return Err((status, Json(core_err.to_api_error(request_id))).into_response());
            }
        };

        if let Err(val_err) = data.validate() {
            let status =
                StatusCode::from_u16(val_err.http_status()).unwrap_or(StatusCode::BAD_REQUEST);
            return Err((status, Json(val_err.to_api_error(request_id))).into_response());
        }

        Ok(ValidatedQuery(data))
    }
}
