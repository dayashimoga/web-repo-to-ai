//! Custom Path parameter extractor enforcing domain validation.

use axum::async_trait;
use axum::extract::{FromRequestParts, Path};
use axum::http::request::Parts;
use axum::response::Response;
use serde::de::DeserializeOwned;
use web_repo_to_ai_core::{CoreError, RequestId, Validatable};

use crate::error::to_problem_response;

/// Axum extractor that parses URI path parameters and enforces [`Validatable`] constraints,
/// generating RFC 7807 problem details responses on syntax or validation errors.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedPath<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for ValidatedPath<T>
where
    T: DeserializeOwned + Validatable + Send + 'static,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let request_id = parts.extensions.get::<RequestId>().copied();

        let data: T = match Path::<T>::from_request_parts(parts, state).await {
            Ok(Path(val)) => val,
            Err(rejection) => {
                let core_err = CoreError::Validation {
                    field: "path".to_string(),
                    reason: rejection.to_string(),
                };
                return Err(to_problem_response(core_err, request_id));
            }
        };

        if let Err(val_err) = data.validate() {
            return Err(to_problem_response(val_err, request_id));
        }

        Ok(ValidatedPath(data))
    }
}
