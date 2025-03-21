//! Job management handlers for the API.

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{AppState, CreateJobRequest, CreateJobResponse, JobStatus};

/// Create a new job
pub async fn create(
    State(_state): State<AppState>,
    Json(request): Json<CreateJobRequest>,
) -> impl IntoResponse {
    // TODO: Implement job creation
    let job_id = Uuid::new_v4();
    
    Json(CreateJobResponse {
        job_id,
        status_url: format!("/api/jobs/{}", job_id),
    })
}

/// Get job status
pub async fn status(
    State(_state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> impl IntoResponse {
    // TODO: Implement job status retrieval
    Json(JobStatus {
        job_id,
        status: crate::JobState::Queued,
        progress: 0.0,
        error: None,
        result_url: None,
    })
}

/// Get job report
pub async fn report(
    State(_state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> impl IntoResponse {
    // TODO: Implement report retrieval
    Json(serde_json::json!({
        "job_id": job_id.to_string(),
        "message": "Report not implemented yet",
    }))
}

pub async fn create_job(
    State(_state): State<AppState>,
    Json(_request): Json<CreateJobRequest>,
) -> impl IntoResponse {
    // Implementation...
}

pub async fn list_jobs(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // Implementation...
}

pub async fn get_job(
    State(_state): State<AppState>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    // Implementation...
} 