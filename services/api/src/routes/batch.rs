//! Multi-URL batch conversion route handlers.

use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use web_repo_to_ai_core::{BatchId, JobId, RequestId};

use crate::dto::{
    BatchItemStatus, BatchPathParams, BatchStatus, CreateBatchJobRequest, CreateBatchJobResponse,
    JobStatus, DEFAULT_BATCH_PARALLELISM,
};
use crate::error::not_found_problem;
use crate::extractors::{ValidatedJson, ValidatedPath};
use crate::state::AppState;

/// Handles `POST /api/v1/jobs/batch`.
///
/// Accepts a validated multi-URL batch request, initializes a strongly-typed [`BatchId`],
/// allocates strongly-typed UUID v7 [`JobId`]s for each distinct URL member,
/// records submission metrics, and returns `202 Accepted` with initial batch descriptor.
pub async fn create_batch_job_handler(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateBatchJobRequest>,
) -> impl IntoResponse {
    let batch_id = BatchId::new();
    let effective_items = payload.effective_items();
    let now = Utc::now();

    let mut item_statuses = Vec::with_capacity(effective_items.len());
    for (idx, item) in effective_items.iter().enumerate() {
        item_statuses.push(BatchItemStatus {
            item_index: idx,
            job_id: JobId::new(),
            url: item.url.clone(),
            custom_id: item.custom_id.clone(),
            status: JobStatus::Queued,
            created_at: now,
        });
    }

    let total_items = item_statuses.len();
    let max_parallelism = payload.max_parallelism.unwrap_or(DEFAULT_BATCH_PARALLELISM);

    let response = CreateBatchJobResponse {
        batch_id,
        name: payload.name,
        status: BatchStatus::Queued,
        total_items,
        queued_items: total_items,
        max_parallelism,
        items: item_statuses,
        created_at: now,
    };

    state.metrics.jobs_total.inc("batch_submitted");

    (StatusCode::ACCEPTED, Json(response))
}

/// Handles `GET /api/v1/jobs/batch/{batch_id}`.
///
/// Retrieves multi-URL batch job execution progress and itemized statuses.
pub async fn get_batch_job_handler(
    State(_state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    ValidatedPath(params): ValidatedPath<BatchPathParams>,
) -> impl IntoResponse {
    // In Subphase 8.1.1, persistence for batch state will be wired in Subphase 8.1.2/8.1.3.
    // Returns typed RFC 7807 404 Problem Details if batch is not found.
    not_found_problem("BatchJob", &params.batch_id.to_string(), Some(request_id))
}
