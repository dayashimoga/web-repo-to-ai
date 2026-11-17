//! Unified RFC 7807 problem details error mapping and response formatting.

use axum::http::header::{HeaderValue, CONTENT_TYPE};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use web_repo_to_ai_core::{ApiErrorEnvelope, CoreError, RequestId};

use crate::middleware::HEADER_REQUEST_ID;

/// Standard RFC 7807 problem details media type.
pub const PROBLEM_JSON_CONTENT_TYPE: &str = "application/problem+json";

/// Wrapper around [`CoreError`] implementing Axum [`IntoResponse`] with RFC 7807 formatting.
#[derive(Debug)]
pub struct ApiProblem {
    /// Domain error payload.
    pub error: CoreError,
    /// Correlated request identifier, if present.
    pub request_id: Option<RequestId>,
}

impl ApiProblem {
    /// Creates a new RFC 7807 problem descriptor.
    #[must_use]
    pub const fn new(error: CoreError, request_id: Option<RequestId>) -> Self {
        Self { error, request_id }
    }

    /// Converts this problem into an RFC 7807 [`ApiErrorEnvelope`].
    #[must_use]
    pub fn to_envelope(&self) -> ApiErrorEnvelope {
        self.error
            .to_api_error(self.request_id.map(|r| r.to_string()))
    }
}

impl IntoResponse for ApiProblem {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.error.http_status())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let envelope = self.to_envelope();
        let body_str = serde_json::to_string(&envelope).unwrap_or_else(|_| {
            r#"{"error":{"code":"INTERNAL_SERVER_ERROR","message":"Error serialization failed"}}"#
                .to_string()
        });

        let mut response = (status, body_str).into_response();

        response.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static(PROBLEM_JSON_CONTENT_TYPE),
        );

        if let Some(req_id) = self.request_id {
            if let Ok(val) = HeaderValue::from_str(&req_id.to_string()) {
                response
                    .headers_mut()
                    .insert(HEADER_REQUEST_ID.clone(), val);
            }
        }

        response
    }
}

/// Converts a [`CoreError`] and optional [`RequestId`] into an RFC 7807 HTTP response.
#[must_use]
pub fn to_problem_response(error: CoreError, request_id: Option<RequestId>) -> Response {
    ApiProblem::new(error, request_id).into_response()
}

/// Generates a standardized 404 Resource Not Found RFC 7807 problem response.
#[must_use]
pub fn not_found_problem(
    resource_type: &'static str,
    id: &str,
    request_id: Option<RequestId>,
) -> Response {
    let err = CoreError::NotFound {
        resource_type,
        id: id.to_string(),
    };
    to_problem_response(err, request_id)
}

/// Generates a standardized 400 Validation Error RFC 7807 problem response.
#[must_use]
pub fn validation_problem(field: &str, reason: &str, request_id: Option<RequestId>) -> Response {
    let err = CoreError::Validation {
        field: field.to_string(),
        reason: reason.to_string(),
    };
    to_problem_response(err, request_id)
}

/// Generates a standardized 400 Invalid Request Schema RFC 7807 problem response.
#[must_use]
pub fn schema_problem(reason: &str, request_id: Option<RequestId>) -> Response {
    let err = CoreError::InvalidRequestSchema(reason.to_string());
    to_problem_response(err, request_id)
}

/// Generates a standardized 413 Payload Too Large RFC 7807 problem response.
#[must_use]
pub fn payload_too_large_problem(
    actual_bytes: u64,
    max_bytes: u64,
    request_id: Option<RequestId>,
) -> Response {
    let err = CoreError::PayloadTooLarge {
        actual_bytes,
        max_bytes,
    };
    to_problem_response(err, request_id)
}

/// Generates a standardized 405 Method Not Allowed RFC 7807 problem response.
#[must_use]
pub fn method_not_allowed_problem(
    method: &str,
    path: &str,
    request_id: Option<RequestId>,
) -> Response {
    let err = CoreError::MethodNotAllowed {
        method: method.to_string(),
        path: path.to_string(),
    };
    to_problem_response(err, request_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_problem_envelope_creation() {
        let req_id = RequestId::new();
        let problem = ApiProblem::new(
            CoreError::Validation {
                field: "email".to_string(),
                reason: "invalid syntax".to_string(),
            },
            Some(req_id),
        );
        let envelope = problem.to_envelope();
        assert_eq!(envelope.error.code, "VALIDATION_ERROR");
        assert_eq!(envelope.error.request_id, Some(req_id.to_string()));
    }
}
