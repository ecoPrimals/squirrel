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
    State(state): State<AppState>,
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
    State(state): State<AppState>,
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
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> impl IntoResponse {
    // TODO: Implement report retrieval
    Json(serde_json::json!({
        "job_id": job_id.to_string(),
        "message": "Report not implemented yet",
    }))
} 