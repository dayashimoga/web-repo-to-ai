//! Job management route handlers.

use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use web_repo_to_ai_core::{JobId, RequestId};

use crate::dto::{
    ArtifactPathParams, CreateJobRequest, JobPathParams, JobResponse, ListJobsQuery,
    PaginatedResponse,
};
use crate::error::not_found_problem;
use crate::extractors::{ValidatedJson, ValidatedPath, ValidatedQuery};
use crate::state::AppState;

/// Handles `POST /api/v1/jobs`.
///
/// Accepts a validated job request, initializes a strongly-typed UUID v7 [`JobId`],
/// records submission metrics, and returns `202 Accepted` with initial job descriptor.
pub async fn create_job_handler(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateJobRequest>,
) -> impl IntoResponse {
    let job_id = JobId::new();
    let default_model = &state.config.optimization.default_model;
    let default_budget = state.config.optimization.default_token_budget;

    let response = JobResponse::new_queued(job_id, payload, default_model, default_budget);
    state.metrics.jobs_total.inc("submitted");

    (StatusCode::ACCEPTED, Json(response))
}

/// Handles `GET /api/v1/jobs`.
///
/// Lists conversion jobs with validated pagination and filter criteria.
pub async fn list_jobs_handler(
    State(_state): State<AppState>,
    ValidatedQuery(query): ValidatedQuery<ListJobsQuery>,
) -> impl IntoResponse {
    let page = query.page();
    let limit = query.limit();

    // In Phase 2.2, persistence is not yet wired (Phase 2.3).
    // Returns an empty validated paginated envelope.
    let response = PaginatedResponse::<JobResponse>::new(vec![], page, limit, 0);
    (StatusCode::OK, Json(response))
}

/// Handles `GET /api/v1/jobs/{job_id}`.
///
/// Retrieves conversion job status by UUID v7 identifier.
pub async fn get_job_handler(
    State(_state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    ValidatedPath(params): ValidatedPath<JobPathParams>,
) -> impl IntoResponse {
    // Until persistence is wired in Phase 2.3, returns typed 404 Problem Details
    not_found_problem("Job", &params.job_id.to_string(), Some(request_id))
}

/// Handles `GET /api/v1/jobs/{job_id}/artifacts/{format}`.
///
/// Streams or downloads an exported artifact by format.
pub async fn get_artifact_handler(
    State(_state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    ValidatedPath(params): ValidatedPath<ArtifactPathParams>,
) -> impl IntoResponse {
    // Until storage is wired in Phase 2.4, returns typed 404 Problem Details
    not_found_problem(
        "Artifact",
        &format!("{}:{}", params.job_id, params.format),
        Some(request_id),
    )
}
