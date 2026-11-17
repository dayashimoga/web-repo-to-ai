//! Source detection and classification route handlers.

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;

use crate::dto::{classify_source_url, DetectSourcesRequest, DetectSourcesResponse};
use crate::extractors::ValidatedJson;
use crate::state::AppState;

/// Handles `POST /api/v1/sources/detect`.
///
/// Validates and classifies one or more input URLs into WebPage, PublicGitHub, PrivateGitHub, or BatchList.
pub async fn detect_sources_handler(
    State(_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<DetectSourcesRequest>,
) -> impl IntoResponse {
    let sources: Vec<_> = payload
        .urls
        .iter()
        .map(|url| classify_source_url(url))
        .collect();

    let total = sources.len();
    Json(DetectSourcesResponse { sources, total })
}
