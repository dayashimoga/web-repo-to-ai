//! Multi-URL batch conversion route handlers.

use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use web_repo_to_ai_core::{BatchId, JobId, RequestId};
use web_repo_to_ai_storage::batch::{
    BatchItemRecord, BatchItemStatusKind, BatchJobStatus, BatchRecord,
};

use crate::dto::{
    BatchItemPathParams, BatchItemStatus, BatchItemsListResponse, BatchJobResponse,
    BatchPathParams, BatchProgress, BatchStatus, BatchSummaryItem, CreateBatchJobRequest,
    CreateBatchJobResponse, JobStatus, ListBatchesResponse, DEFAULT_BATCH_PARALLELISM,
};
use crate::error::not_found_problem;
use crate::extractors::{ValidatedJson, ValidatedPath};
use crate::state::AppState;

/// Helper to convert a storage [`BatchRecord`] and slice of [`BatchItemRecord`]s into [`BatchJobResponse`].
fn build_batch_job_response(record: BatchRecord, items: Vec<BatchItemRecord>) -> BatchJobResponse {
    let item_statuses: Vec<BatchItemStatus> = items.into_iter().map(Into::into).collect();
    let status = match record.status {
        BatchJobStatus::Queued => BatchStatus::Queued,
        BatchJobStatus::Processing => BatchStatus::Processing,
        BatchJobStatus::Completed => BatchStatus::Completed,
        BatchJobStatus::PartialFailure => BatchStatus::PartialFailure,
        BatchJobStatus::Failed => BatchStatus::Failed,
        BatchJobStatus::Cancelled => BatchStatus::Cancelled,
    };
    let progress = BatchProgress::from_items(&item_statuses);

    BatchJobResponse {
        batch_id: record.id,
        name: record.name,
        status,
        progress,
        max_parallelism: record.max_parallelism,
        items: item_statuses,
        created_at: record.created_at,
        updated_at: record.updated_at,
        completed_at: record.completed_at,
    }
}

/// Handles `POST /api/v1/jobs/batch`.
///
/// Accepts a validated multi-URL batch request, initializes a strongly-typed [`BatchId`],
/// allocates strongly-typed UUID v7 [`JobId`]s for each distinct URL member,
/// persists initial batch and item records to storage, records submission metrics,
/// and returns `202 Accepted` with initial batch descriptor.
pub async fn create_batch_job_handler(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateBatchJobRequest>,
) -> impl IntoResponse {
    let batch_id = BatchId::new();
    let effective_items = payload.effective_items();
    let now = Utc::now();
    let total_items = effective_items.len();
    let max_parallelism = payload.max_parallelism.unwrap_or(DEFAULT_BATCH_PARALLELISM);

    let mut item_records = Vec::with_capacity(total_items);
    let mut item_statuses = Vec::with_capacity(total_items);

    for (idx, item) in effective_items.iter().enumerate() {
        let job_id = JobId::new();
        item_records.push(BatchItemRecord {
            batch_id,
            item_index: idx,
            job_id,
            url: item.url.clone(),
            custom_id: item.custom_id.clone(),
            status: BatchItemStatusKind::Queued,
            error_code: None,
            error_message: None,
            duration_ms: None,
            artifact_count: 0,
            created_at: now,
            updated_at: now,
            completed_at: None,
        });

        item_statuses.push(BatchItemStatus {
            item_index: idx,
            job_id,
            url: item.url.clone(),
            custom_id: item.custom_id.clone(),
            status: JobStatus::Queued,
            error_code: None,
            error_message: None,
            duration_ms: None,
            created_at: now,
            completed_at: None,
        });
    }

    let batch_record = BatchRecord {
        id: batch_id,
        name: payload.name.clone(),
        status: BatchJobStatus::Queued,
        total_items,
        queued_items: total_items,
        processing_items: 0,
        completed_items: 0,
        failed_items: 0,
        max_parallelism,
        owner_session_id: None,
        created_at: now,
        updated_at: now,
        completed_at: None,
    };

    if let Err(e) = state
        .batch_storage
        .create_batch(batch_record, item_records)
        .await
    {
        tracing::error!("Failed to persist batch {batch_id}: {e}");
    }

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
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    ValidatedPath(params): ValidatedPath<BatchPathParams>,
) -> impl IntoResponse {
    let batch = match state.batch_storage.get_batch(params.batch_id).await {
        Ok(Some(b)) => b,
        Ok(None) => {
            return not_found_problem("BatchJob", &params.batch_id.to_string(), Some(request_id))
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to query batch {}: {e}", params.batch_id);
            return not_found_problem("BatchJob", &params.batch_id.to_string(), Some(request_id))
                .into_response();
        }
    };

    let items = state
        .batch_storage
        .get_batch_items(params.batch_id)
        .await
        .unwrap_or_default();

    let response = build_batch_job_response(batch, items);
    (StatusCode::OK, Json(response)).into_response()
}

/// Handles `GET /api/v1/jobs/batch/{batch_id}/items`.
///
/// Retrieves all individual item statuses for a given batch conversion job.
pub async fn get_batch_items_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    ValidatedPath(params): ValidatedPath<BatchPathParams>,
) -> impl IntoResponse {
    let exists = state.batch_storage.get_batch(params.batch_id).await;
    match exists {
        Ok(Some(_)) => {
            let items = state
                .batch_storage
                .get_batch_items(params.batch_id)
                .await
                .unwrap_or_default();
            let total_items = items.len();
            let item_statuses = items.into_iter().map(Into::into).collect();

            (
                StatusCode::OK,
                Json(BatchItemsListResponse {
                    batch_id: params.batch_id,
                    total_items,
                    items: item_statuses,
                }),
            )
                .into_response()
        }
        _ => not_found_problem("BatchJob", &params.batch_id.to_string(), Some(request_id))
            .into_response(),
    }
}

/// Handles `GET /api/v1/jobs/batch/{batch_id}/items/{item_index}`.
///
/// Retrieves detailed execution status of a single URL item within a batch.
pub async fn get_batch_item_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    ValidatedPath(params): ValidatedPath<BatchItemPathParams>,
) -> impl IntoResponse {
    match state
        .batch_storage
        .get_batch_item(params.batch_id, params.item_index)
        .await
    {
        Ok(Some(item)) => {
            let status: BatchItemStatus = item.into();
            (StatusCode::OK, Json(status)).into_response()
        }
        _ => not_found_problem(
            "BatchItem",
            &format!("{}:{}", params.batch_id, params.item_index),
            Some(request_id),
        )
        .into_response(),
    }
}

/// Handles `DELETE /api/v1/jobs/batch/{batch_id}` and `POST /api/v1/jobs/batch/{batch_id}/cancel`.
///
/// Cancels all remaining queued and processing items in a batch.
pub async fn cancel_batch_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    ValidatedPath(params): ValidatedPath<BatchPathParams>,
) -> impl IntoResponse {
    match state.batch_storage.cancel_batch(params.batch_id).await {
        Ok(Some(cancelled_batch)) => {
            let items = state
                .batch_storage
                .get_batch_items(params.batch_id)
                .await
                .unwrap_or_default();
            let response = build_batch_job_response(cancelled_batch, items);
            (StatusCode::OK, Json(response)).into_response()
        }
        _ => not_found_problem("BatchJob", &params.batch_id.to_string(), Some(request_id))
            .into_response(),
    }
}

/// Handles `GET /api/v1/jobs/batch`.
///
/// Lists multi-URL batch conversion jobs.
pub async fn list_batches_handler(State(state): State<AppState>) -> impl IntoResponse {
    let list = state
        .batch_storage
        .list_batches(None, 50, 0)
        .await
        .unwrap_or_default();

    let count = list.len();
    let batches = list.into_iter().map(BatchSummaryItem::from).collect();

    (StatusCode::OK, Json(ListBatchesResponse { batches, count })).into_response()
}
