//! API module for the web interface.

// This module will contain API-specific functionality and utilities.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use axum::Json;

pub mod error;

/// API Response envelope for standardized responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Indicates if the request was successful
    pub success: bool,
    /// Response data (null if error)
    pub data: Option<T>,
    /// Error information (null if success)
    pub error: Option<ApiError>,
    /// Response metadata
    pub meta: ApiMeta,
}

/// API Error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error code
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional error details
    pub details: Option<serde_json::Value>,
}

/// API Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMeta {
    /// Unique request ID
    pub request_id: String,
    /// Response timestamp
    pub timestamp: String,
    /// Pagination information if applicable
    pub pagination: Option<PaginationMeta>,
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    /// Current page
    pub page: u32,
    /// Items per page
    pub limit: u32,
    /// Total number of items
    pub total_items: u64,
    /// Total number of pages
    pub total_pages: u32,
}

/// Helper function to create a successful API response
pub fn api_success<T>(data: T) -> Json<ApiResponse<T>> {
    let request_id = Uuid::new_v4().to_string();
    let timestamp = Utc::now().to_rfc3339();
    
    Json(ApiResponse {
        success: true,
        data: Some(data),
        error: None,
        meta: ApiMeta {
            request_id,
            timestamp,
            pagination: None,
        },
    })
}

/// Helper function to create a successful API response with pagination
pub fn api_success_paginated<T>(
    data: T, 
    page: u32,
    limit: u32,
    total_items: u64,
    total_pages: u32,
) -> Json<ApiResponse<T>> {
    let request_id = Uuid::new_v4().to_string();
    let timestamp = Utc::now().to_rfc3339();
    
    Json(ApiResponse {
        success: true,
        data: Some(data),
        error: None,
        meta: ApiMeta {
            request_id,
            timestamp,
            pagination: Some(PaginationMeta {
                page,
                limit,
                total_items,
                total_pages,
            }),
        },
    })
}

/// Request to create a new job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobRequest {
    /// Repository URL 
    pub repository_url: String,
    /// Branch or commit
    pub git_ref: String,
    /// Configuration
    pub config: serde_json::Value,
}

/// Response for a created job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobResponse {
    /// Job ID
    pub job_id: Uuid,
    /// Status URL to check job progress
    pub status_url: String,
}

/// Status of a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {
    /// Job ID
    pub job_id: Uuid,
    /// Current status
    pub status: JobState,
    /// Progress percentage
    pub progress: f32,
    /// Error message if any
    pub error: Option<String>,
    /// Result URL if completed
    pub result_url: Option<String>,
}

/// State of a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobState {
    Queued,
    Running,
    Completed,
    Failed,
} 