//! API module for the web interface.

// This module will contain API-specific functionality and utilities.

use serde::{Deserialize, Serialize};
// Remove unused imports
// use uuid::Uuid;
// use chrono::Utc;

pub mod error;
// We need to decide: either keep commands.rs or commands/mod.rs - not both
// For now, explicitly use the directory-based module
pub mod commands;
pub mod router;

pub use router::api_router;

/// Standard API response envelope
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    #[serde(default)]
    pub meta: ApiMeta,
}

/// API error details
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// API response metadata
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ApiMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationMeta>,
}

/// Pagination metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
}

/// Helper function to create a successful API response
pub fn api_success<T>(data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        success: true,
        data: Some(data),
        error: None,
        meta: Default::default(),
    })
}

/// Helper function to create a successful API response with pagination
pub fn api_success_paginated<T>(
    data: T,
    page: i64,
    per_page: i64,
    total: i64,
    total_pages: i64,
) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        success: true,
        data: Some(data),
        error: None,
        meta: ApiMeta {
            pagination: Some(PaginationMeta {
                page,
                per_page,
                total,
                total_pages,
            }),
        },
    })
}

/// Request model for creating a new job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobRequest {
    /// Name of the job to execute
    pub name: String,
    /// Job parameters
    pub parameters: serde_json::Value,
}

/// Response model for a created job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobResponse {
    /// Job ID
    pub id: String,
    /// Job name
    pub name: String,
    /// Job status
    pub status: JobState,
}

/// Job status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {
    /// Job ID
    pub id: String,
    /// Job name
    pub name: String,
    /// Job status
    pub status: JobState,
    /// Job progress (0.0 to 1.0)
    pub progress: f32,
    /// Job result data (if completed)
    pub result: Option<serde_json::Value>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Timestamps for job lifecycle events
    pub timestamps: JobTimestamps,
}

/// Job lifecycle timestamps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobTimestamps {
    /// When the job was created
    pub created_at: String,
    /// When the job started execution
    pub started_at: Option<String>,
    /// When the job completed or failed
    pub completed_at: Option<String>,
}

/// Job state enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobState {
    /// Job is queued for execution
    Queued,
    /// Job is currently running
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed
    Failed,
}

// Import Json only once from axum
use axum::Json; 