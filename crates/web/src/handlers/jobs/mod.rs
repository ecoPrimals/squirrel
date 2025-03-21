//! Job management handlers for the API.

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use std::sync::Arc;

use crate::{
    AppState,
    CreateJobRequest,
    CreateJobResponse,
    JobStatus,
    JobState,
};
use crate::api::error::AppError;

/// Create a new job
pub async fn create_job(
    State(_state): State<Arc<AppState>>,
    Json(_request): Json<CreateJobRequest>,
) -> Result<Json<CreateJobResponse>, AppError> {
    let job_id = Uuid::new_v4();
    Ok(Json(CreateJobResponse {
        job_id,
        status_url: format!("/api/jobs/{}", job_id),
    }))
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
    State(_state): State<Arc<AppState>>,
    Path(_job_id): Path<String>,
) -> Result<Json<JobStatus>, AppError> {
    let job_id = Uuid::new_v4();
    Ok(Json(JobStatus {
        job_id,
        status: JobState::Queued,
        progress: 0.0,
        error: None,
        result_url: None,
    }))
}

pub async fn list_jobs(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // Implementation...
}

/// Get a job by ID
pub async fn get_job(
    State(_state): State<Arc<AppState>>,
    Path(_job_id): Path<String>,
) -> Result<Json<JobStatus>, AppError> {
    let job_id = Uuid::new_v4();
    Ok(Json(JobStatus {
        job_id,
        status: JobState::Queued,
        progress: 0.0,
        error: None,
        result_url: None,
    }))
} 