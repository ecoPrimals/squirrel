//! Job management handlers for the API.

use axum::{
    extract::{Path, State, Extension},
    Json,
    routing::{get, post},
    Router,
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;
use serde::Serialize;
use serde_json::json;
use chrono::{DateTime, Utc};
#[cfg(feature = "db")]
use sqlx::sqlite::SqliteQueryResult;

use crate::{
    api::{
        CreateJobRequest, CreateJobResponse, JobStatus, JobState, error::AppError,
        api_success, api_success_with_pagination, ApiResponse
    },
    AppState,
    auth::Claims,
    auth::extractor::AuthClaims,
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
pub async fn create_job(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<CreateJobRequest>,
) -> Result<Json<ApiResponse<CreateJobResponse>>, AppError> {
    #[cfg(feature = "db")]
    if let Some(_) = state.db {
        let response = create_job_with_db(&state, &claims, &req).await?;
        return Ok(api_success(response));
    }
    
    // If we get here, either there's no DB connection or the feature is disabled
    
    // Create a new job ID
    let job_id = Uuid::new_v4().to_string();
    
    // In a real implementation, this would store the job in the database
    // TODO: Additional validation of job parameters would go here
    
    let response = CreateJobResponse {
        id: job_id,
        name: req.name.clone(),
        status: JobState::Queued,
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
        id: job_id.to_string(),
        name: "example-job".to_string(),
        status: JobState::Running,
        progress: 0.5,
        result: None,
        error: None,
        timestamps: crate::api::JobTimestamps {
            created_at: Utc::now().to_rfc3339(),
            started_at: Some(Utc::now().to_rfc3339()),
            completed_at: None,
        },
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
        id: job_id.to_string(),
        name: "example-job".to_string(),
        status: JobState::Running,
        progress: 0.5,
        result: None,
        error: None,
        timestamps: crate::api::JobTimestamps {
            created_at: Utc::now().to_rfc3339(),
            started_at: Some(Utc::now().to_rfc3339()),
            completed_at: None,
        },
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
        id: job_id.to_string(),
        name: "example-job".to_string(),
        status: JobState::Completed,
        progress: 1.0,
        result: Some(json!({
            "output": "Job completed successfully",
            "processingTime": 1234,
        })),
        error: None,
        timestamps: crate::api::JobTimestamps {
            created_at: Utc::now().to_rfc3339(),
            started_at: Some(Utc::now().to_rfc3339()),
            completed_at: Some(Utc::now().to_rfc3339()),
        },
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
        id: job_id.to_string(),
        name: "example-job".to_string(),
        status: JobState::Completed,
        progress: 1.0,
        result: Some(json!({
            "output": "Job completed successfully",
            "processingTime": 1234,
        })),
        error: None,
        timestamps: crate::api::JobTimestamps {
            created_at: Utc::now().to_rfc3339(),
            started_at: Some(Utc::now().to_rfc3339()),
            completed_at: Some(Utc::now().to_rfc3339()),
        },
    };
    
    Ok(api_success(status))
}

/// List all jobs for the current user
#[cfg(feature = "db")]
pub async fn list_jobs(
    State(_state): State<Arc<AppState>>,
    Extension(_claims): Extension<AuthClaims>,
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
            id: job.id.clone(),
            name: job.repository_url.clone(),
            status: parse_job_state(&job.status),
            progress: job.progress,
            error: job.error,
            result_url: job.result_url,
            timestamps: crate::api::JobTimestamps {
                created_at: job.created_at.to_rfc3339(),
                started_at: None,
                completed_at: None,
            },
        })
        .collect();
    
    let total_pages = ((total_count as f64) / (limit as f64)).ceil() as u32;
    
    Ok(api_success_with_pagination(
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
    let jobs = vec![
        JobStatus {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap().to_string(),
            name: "job1".to_string(),
            status: JobState::Running,
            progress: 0.7,
            result: None,
            error: None,
            timestamps: crate::api::JobTimestamps {
                created_at: Utc::now().to_rfc3339(),
                started_at: Some(Utc::now().to_rfc3339()),
                completed_at: None,
            },
        },
        JobStatus {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap().to_string(),
            name: "job2".to_string(),
            status: JobState::Completed,
            progress: 1.0,
            result: Some(json!({ "success": true })),
            error: None,
            timestamps: crate::api::JobTimestamps {
                created_at: Utc::now().to_rfc3339(),
                started_at: Some(Utc::now().to_rfc3339()),
                completed_at: Some(Utc::now().to_rfc3339()),
            },
        },
    ];
    
    let page = 1;
    let limit = 20;
    let total_count = jobs.len() as u64;
    let total_pages = 1;
    
    Ok(api_success_with_pagination(
        jobs,
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
        id: job_id.to_string(),
        name: job.repository_url.clone(),
        status: parse_job_state(&job.status),
        progress: job.progress,
        error: job.error,
        result_url: job.result_url,
        timestamps: crate::api::JobTimestamps {
            created_at: job.created_at.to_rfc3339(),
            started_at: None,
            completed_at: None,
        },
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
        id: job_id.to_string(),
        name: "example-job".to_string(),
        status: JobState::Running,
        progress: 0.7,
        result: None,
        error: None,
        timestamps: crate::api::JobTimestamps {
            created_at: Utc::now().to_rfc3339(),
            started_at: Some(Utc::now().to_rfc3339()),
            completed_at: None,
        },
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
    // Return an empty JSON object as stub response
    Json(json!({}))
}

/// Get the status of a job
pub async fn get_job_status(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<Json<ApiResponse<JobStatus>>, AppError> {
    // In a real implementation, this would fetch the job from the database
    let job_id = Uuid::parse_str(&job_id).unwrap_or_default();
    
    // Create a mock job status
    let status = JobStatus {
        id: job_id.to_string(),
        name: "example-job".to_string(),
        status: JobState::Running,
        progress: 0.5,
        result: None,
        error: None,
        timestamps: crate::api::JobTimestamps {
            created_at: Utc::now().to_rfc3339(),
            started_at: Some(Utc::now().to_rfc3339()),
            completed_at: None,
        },
    };
    
    Ok(api_success(status))
}

/// Get the result of a job
pub async fn get_job_result(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<Json<ApiResponse<JobStatus>>, AppError> {
    // In a real implementation, this would fetch the job result from the database or storage
    let job_id = Uuid::parse_str(&job_id).unwrap_or_default();
    
    // Create a mock job result
    let status = JobStatus {
        id: job_id.to_string(),
        name: "example-job".to_string(),
        status: JobState::Completed,
        progress: 1.0,
        result: Some(json!({
            "output": "Job completed successfully",
            "processingTime": 1234,
        })),
        error: None,
        timestamps: crate::api::JobTimestamps {
            created_at: Utc::now().to_rfc3339(),
            started_at: Some(Utc::now().to_rfc3339()),
            completed_at: Some(Utc::now().to_rfc3339()),
        },
    };
    
    Ok(api_success(status))
}

/// Cancel a job
pub async fn cancel_job(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<Json<ApiResponse<JobStatus>>, AppError> {
    // In a real implementation, this would cancel the job in the database
    let job_id = Uuid::parse_str(&job_id).unwrap_or_default();
    
    // Create a mock cancelled job
    let status = JobStatus {
        id: job_id.to_string(),
        name: "example-job".to_string(),
        status: JobState::Failed,
        progress: 0.3,
        result: None,
        error: Some("Job cancelled by user".to_string()),
        timestamps: crate::api::JobTimestamps {
            created_at: Utc::now().to_rfc3339(),
            started_at: Some(Utc::now().to_rfc3339()),
            completed_at: Some(Utc::now().to_rfc3339()),
        },
    };
    
    Ok(api_success(status))
}

/// Create a job
#[cfg(feature = "db")]
async fn create_job_with_db(
    state: &Arc<AppState>,
    claims: &Claims,
    req: &CreateJobRequest,
) -> Result<CreateJobResponse, AppError> {
    let job_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    // Store the job in the database
    sqlx::query!(
        r#"
        INSERT INTO jobs (
            id, user_id, name, parameters,
            status, progress, error, result_url,
            created_at, updated_at, started_at, completed_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        job_id,
        claims.sub.to_string(),
        req.name,
        serde_json::to_string(&req.parameters).unwrap(),
        "queued",
        0.0f32,
        Option::<String>::None,
        Option::<String>::None,
        now, now,
        Option::<DateTime<Utc>>::None,
        Option::<DateTime<Utc>>::None,
    )
    .execute(&state.db.as_ref().unwrap())
    .await
    .map_err(|e| AppError::Database(format!("Failed to create job: {}", e)))?;
    
    // Return the response
    let response = CreateJobResponse {
        id: job_id,
        name: req.name.clone(),
        status: JobState::Queued,
    };
    
    Ok(response)
}

async fn create_log_stub(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    "Create log endpoint"
}

async fn list_logs_stub(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    "List logs endpoint"
}

async fn download_logs_stub(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    "Download logs endpoint"
} 