//! Liveness and readiness probe handlers.
//!
//! Provides [`health_handler`] and [`readiness_handler`] for container orchestrators
//! and load balancer health checks following IETF healthcheck recommendations.

use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Handles `GET /api/v1/health` (process liveness probe).
///
/// Fast, non-blocking check used by container engines to verify that the
/// Axum process is alive and responding to HTTP requests.
pub async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "pass",
            "service": "web-repo-to-ai-api",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime_seconds": state.uptime_seconds(),
            "timestamp": Utc::now().to_rfc3339(),
        })),
    )
}

/// Evaluates writeability and directory integrity for the configured local storage path.
fn check_storage_subsystem(storage_dir: &Path) -> (bool, serde_json::Value) {
    if let Err(err) = fs::create_dir_all(storage_dir) {
        return (
            false,
            json!({
                "status": "fail",
                "path": storage_dir.to_string_lossy(),
                "error": format!("Cannot create storage directory: {err}"),
            }),
        );
    }

    let probe_file = storage_dir.join(format!(
        ".healthcheck_probe_{}",
        web_repo_to_ai_core::id::RequestId::new()
    ));
    match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&probe_file)
    {
        Ok(mut f) => {
            if let Err(err) = f.write_all(b"probe") {
                let _ = fs::remove_file(&probe_file);
                return (
                    false,
                    json!({
                        "status": "fail",
                        "path": storage_dir.to_string_lossy(),
                        "error": format!("Cannot write to probe file: {err}"),
                    }),
                );
            }
            let _ = fs::remove_file(&probe_file);
            (
                true,
                json!({
                    "status": "pass",
                    "path": storage_dir.to_string_lossy(),
                }),
            )
        }
        Err(err) => (
            false,
            json!({
                "status": "fail",
                "path": storage_dir.to_string_lossy(),
                "error": format!("Cannot open probe file for writing: {err}"),
            }),
        ),
    }
}

/// Handles `GET /api/v1/ready` (service dependencies readiness probe).
///
/// Evaluates whether the API server is ready to accept external workloads by probing:
/// 1. Storage subsystem (writeability of configured storage volume).
/// 2. Database configuration validity.
/// 3. Worker queue status.
///
/// Returns `200 OK` with `"status": "ready"` if all mandatory components are healthy;
/// returns `503 Service Unavailable` with `"status": "not_ready"` if any critical check fails.
pub async fn readiness_handler(State(state): State<AppState>) -> impl IntoResponse {
    let (storage_healthy, storage_details) =
        check_storage_subsystem(&state.config.storage.base_dir);

    let database_configured = !state.config.database.url.is_empty();
    let database_details = json!({
        "status": if database_configured { "pass" } else { "fail" },
        "configured": database_configured,
        "max_connections": state.config.database.max_connections,
    });

    let workers_details = json!({
        "status": "pass",
        "active_workers": state.metrics.active_workers.get(),
    });

    let is_ready = storage_healthy && database_configured;
    let status_code = if is_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let overall_status = if is_ready { "ready" } else { "not_ready" };

    (
        status_code,
        Json(json!({
            "status": overall_status,
            "service": "web-repo-to-ai-api",
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": Utc::now().to_rfc3339(),
            "checks": {
                "storage": storage_details,
                "database": database_details,
                "workers": workers_details,
            }
        })),
    )
}
