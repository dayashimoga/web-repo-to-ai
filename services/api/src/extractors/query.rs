//! Custom Query parameter extractor enforcing domain validation.

use axum::async_trait;
use axum::extract::{FromRequestParts, Query};
use axum::http::request::Parts;
use axum::response::Response;
use serde::de::DeserializeOwned;
use web_repo_to_ai_core::{CoreError, RequestId, Validatable};

use crate::error::to_problem_response;

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
        let request_id = parts.extensions.get::<RequestId>().copied();

        let data: T = match Query::<T>::from_request_parts(parts, state).await {
            Ok(Query(val)) => val,
            Err(rejection) => {
                let core_err = CoreError::InvalidRequestSchema(format!(
                    "Invalid query parameters: {rejection}"
                ));
                return Err(to_problem_response(core_err, request_id));
            }
        };

        if let Err(val_err) = data.validate() {
            return Err(to_problem_response(val_err, request_id));
        }

        Ok(ValidatedQuery(data))
    }
}
