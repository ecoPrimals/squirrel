//! Job management handlers for the API.

use axum::{
    extract::{Path, State},
    Json,
    routing::{get, post},
    Router,
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
#[cfg(feature = "db")]
use sqlx::sqlite::SqliteQueryResult;

use crate::{
    api::{
        CreateJobRequest, CreateJobResponse, JobStatus, JobState, error::AppError,
        api_success, api_success_paginated, ApiResponse
    },
    AppState,
    auth::Claims,
};

/// Database Job model
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
#[allow(dead_code)]
struct DbJob {
    id: String,
    user_id: String,
    repository_url: String,
    git_ref: String,
    config: String,
    status: String,
    progress: f32,
    error: Option<String>,
    result_url: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Job routes
pub fn job_routes() -> Router {
    Router::new()
        .route("/", post(create_job_stub))
        .route("/", get(list_jobs_stub))
        .route("/:id", get(get_job_stub))
        .route("/:id/status", get(status_stub))
        .route("/:id/report", get(report_stub))
}

/// Create a new job
#[cfg(feature = "db")]
pub async fn create_job(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Json(req): Json<CreateJobRequest>,
) -> Result<Json<ApiResponse<CreateJobResponse>>, AppError> {
    let job_id = Uuid::new_v4();
    let now = Utc::now();
    
    // Store job in database
    sqlx::query!(
        r#"
        INSERT INTO jobs (
            id, user_id, repository_url, git_ref, config, 
            status, progress, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        job_id.to_string(),
        claims.sub.to_string(),
        req.repository_url,
        req.git_ref,
        req.config.to_string(),
        "Queued",
        0.0,
        now,
        now
    )
    .execute(&state.db)
    .await?;
    
    // Notify MCP of job creation (if configured)
    if let Some(mcp_client) = &state.mcp {
        let _ = mcp_client.send_message(&format!(
            "New job created: {}", job_id
        ));
    }
    
    let response = CreateJobResponse {
        job_id,
        status_url: format!("/api/jobs/{}/status", job_id),
    };
    
    Ok(api_success(response))
}

/// Create a new job (mock implementation)
#[cfg(feature = "mock-db")]
pub async fn create_job(
    State(state): State<Arc<AppState>>,
    _claims: Claims,
    Json(req): Json<CreateJobRequest>,
) -> Result<Json<ApiResponse<CreateJobResponse>>, AppError> {
    let job_id = Uuid::new_v4();
    
    // Queue job with MCP client (still want to test this part)
    if let Some(mcp) = &state.mcp {
        let _ = mcp.send_message(&format!(
            "New job: {} for repository: {}", 
            job_id, 
            req.repository_url
        ));
    }
    
    let response = CreateJobResponse {
        job_id,
        status_url: format!("/api/jobs/{}/status", job_id),
    };
    
    Ok(api_success(response))
}

/// Get job status
#[cfg(feature = "db")]
pub async fn status(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(job_id): Path<Uuid>,
) -> Result<Json<ApiResponse<JobStatus>>, AppError> {
    // Get job from database
    let job = get_job_from_db(&state.db, &job_id.to_string(), &claims.sub.to_string()).await?;
    
    // Convert to JobStatus
    let status = JobStatus {
        job_id,
        status: parse_job_state(&job.status),
        progress: job.progress,
        error: job.error,
        result_url: job.result_url,
    };
    
    Ok(api_success(status))
}

/// Get job status (mock implementation)
#[cfg(feature = "mock-db")]
pub async fn status(
    _state: State<Arc<AppState>>,
    _claims: Claims,
    Path(job_id): Path<Uuid>,
) -> Result<Json<ApiResponse<JobStatus>>, AppError> {
    // Create a mock job status
    let status = JobStatus {
        job_id,
        status: JobState::Running,
        progress: 0.5,
        error: None,
        result_url: None,
    };
    
    Ok(api_success(status))
}

/// Get job report
#[cfg(feature = "db")]
pub async fn report(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(job_id): Path<Uuid>,
) -> Result<Json<ApiResponse<JobStatus>>, AppError> {
    // Get job from database
    let job = get_job_from_db(&state.db, &job_id.to_string(), &claims.sub.to_string()).await?;
    
    // TODO: Generate a more detailed report
    
    // For now, return the same as status
    let status = JobStatus {
        job_id,
        status: parse_job_state(&job.status),
        progress: job.progress,
        error: job.error,
        result_url: job.result_url,
    };
    
    Ok(api_success(status))
}

/// Get job report (mock implementation)
#[cfg(feature = "mock-db")]
pub async fn report(
    _state: State<Arc<AppState>>,
    _claims: Claims,
    Path(job_id): Path<Uuid>,
) -> Result<Json<ApiResponse<JobStatus>>, AppError> {
    // Create a mock job report
    let status = JobStatus {
        job_id,
        status: JobState::Running,
        progress: 0.5,
        error: None,
        result_url: None,
    };
    
    Ok(api_success(status))
}

/// List all jobs for the current user
#[cfg(feature = "db")]
pub async fn list_jobs(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<ApiResponse<Vec<JobStatus>>>, AppError> {
    // Get total count for pagination
    let total_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM jobs WHERE user_id = ?",
        claims.sub.to_string()
    )
    .fetch_one(&state.db)
    .await?
    .count as u64;
    
    // Get jobs from database with pagination
    let page = 1;
    let limit = 20;
    let offset = (page - 1) * limit;
    
    let jobs = sqlx::query_as!(
        DbJob,
        r#"
        SELECT * FROM jobs 
        WHERE user_id = ? 
        ORDER BY created_at DESC
        LIMIT ? OFFSET ?
        "#,
        claims.sub.to_string(),
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await?;
    
    // Convert to JobStatus objects
    let job_statuses = jobs.into_iter()
        .map(|job| JobStatus {
            job_id: Uuid::parse_str(&job.id).unwrap_or_default(),
            status: parse_job_state(&job.status),
            progress: job.progress,
            error: job.error,
            result_url: job.result_url,
        })
        .collect();
    
    let total_pages = ((total_count as f64) / (limit as f64)).ceil() as u32;
    
    Ok(api_success_paginated(
        job_statuses,
        page,
        limit,
        total_count,
        total_pages
    ))
}

/// List all jobs for the current user (mock implementation)
#[cfg(feature = "mock-db")]
pub async fn list_jobs(
    _state: State<Arc<AppState>>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<JobStatus>>>, AppError> {
    // Create mock job statuses
    let job_statuses = vec![
        JobStatus {
            job_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            status: JobState::Running,
            progress: 0.3,
            error: None,
            result_url: None,
        },
        JobStatus {
            job_id: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
            status: JobState::Completed,
            progress: 1.0,
            error: None,
            result_url: Some("https://example.com/results/job2".to_string()),
        },
    ];
    
    let page = 1;
    let limit = 20;
    let total_count = job_statuses.len() as u64;
    let total_pages = 1;
    
    Ok(api_success_paginated(
        job_statuses,
        page,
        limit,
        total_count,
        total_pages
    ))
}

/// Get a job by ID
#[cfg(feature = "db")]
pub async fn get_job(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(job_id): Path<Uuid>,
) -> Result<Json<ApiResponse<JobStatus>>, AppError> {
    // Get job from database
    let job = get_job_from_db(&state.db, &job_id.to_string(), &claims.sub.to_string()).await?;
    
    // Convert to JobStatus
    let status = JobStatus {
        job_id,
        status: parse_job_state(&job.status),
        progress: job.progress,
        error: job.error,
        result_url: job.result_url,
    };
    
    Ok(api_success(status))
}

/// Get a job by ID (mock implementation)
#[cfg(feature = "mock-db")]
pub async fn get_job(
    _state: State<Arc<AppState>>,
    _claims: Claims,
    Path(job_id): Path<Uuid>,
) -> Result<Json<ApiResponse<JobStatus>>, AppError> {
    // Create a mock job
    let status = JobStatus {
        job_id,
        status: JobState::Running,
        progress: 0.7,
        error: None,
        result_url: None,
    };
    
    Ok(api_success(status))
}

/// Helper function to get a job from the database
#[cfg(feature = "db")]
async fn get_job_from_db(
    db: &sqlx::SqlitePool, 
    job_id: &str, 
    user_id: &str
) -> Result<DbJob, AppError> {
    let job = sqlx::query_as!(
        DbJob,
        r#"
        SELECT * FROM jobs 
        WHERE id = ? AND user_id = ?
        "#,
        job_id,
        user_id
    )
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound("Job not found".to_string()))?;
    
    Ok(job)
}

/// Helper function to parse job state
#[allow(dead_code)]
fn parse_job_state(status: &str) -> JobState {
    match status {
        "Queued" => JobState::Queued,
        "Running" => JobState::Running,
        "Completed" => JobState::Completed,
        "Failed" => JobState::Failed,
        _ => JobState::Queued,
    }
}

/// Helper function to update job status
#[allow(dead_code)]
#[cfg(feature = "db")]
async fn update_job_status(
    db: &sqlx::SqlitePool,
    job_id: &str,
    status: JobState,
    progress: f32,
    error: Option<String>,
    result_url: Option<String>,
) -> Result<SqliteQueryResult, sqlx::Error> {
    let now = Utc::now();
    let status_str = match status {
        JobState::Queued => "Queued",
        JobState::Running => "Running",
        JobState::Completed => "Completed",
        JobState::Failed => "Failed",
    };
    
    sqlx::query!(
        r#"
        UPDATE jobs 
        SET status = ?, progress = ?, error = ?, result_url = ?, updated_at = ?
        WHERE id = ?
        "#,
        status_str,
        progress,
        error,
        result_url,
        now,
        job_id
    )
    .execute(db)
    .await
}

// Simple handlers for job routes
async fn create_job_stub() -> impl IntoResponse {
    "Create job endpoint"
}

async fn list_jobs_stub() -> impl IntoResponse {
    "List jobs endpoint"
}

async fn get_job_stub() -> impl IntoResponse {
    "Get job endpoint"
}

async fn status_stub() -> impl IntoResponse {
    "Job status endpoint"
}

async fn report_stub() -> impl IntoResponse {
    "Job report endpoint"
} 