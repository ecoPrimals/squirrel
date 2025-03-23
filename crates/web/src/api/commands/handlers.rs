use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize};
use std::sync::Arc;
use tracing::{info, error};

use crate::{
    api::{api_success, ApiResponse, ApiError, api_success_paginated},
    auth::extractor::AuthClaims,
    state::AppState,
};
use crate::api::commands::models::{
    CreateCommandRequest, CreateCommandResponse,
    CommandStatusResponse, CommandSummary,
};
use crate::api::commands::{
    AvailableCommand,
    CommandServiceError,
    CommandService
};
use crate::auth::Claims;

/// Pagination query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    10
}

/// Create a new command
pub async fn create_command(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AuthClaims>,
    Json(request): Json<CreateCommandRequest>,
) -> Result<Json<ApiResponse<CreateCommandResponse>>, CommandApiError> {
    info!(user_id = %claims.sub, command = %request.command, "Creating command");
    
    let response = state.command_service
        .execute_command(request, &claims.sub)
        .await?;
    
    Ok(api_success(response))
}

/// Get command status
pub async fn get_command_status(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AuthClaims>,
    Path(command_id): Path<String>,
) -> Result<Json<ApiResponse<CommandStatusResponse>>, CommandApiError> {
    info!(user_id = %claims.sub, command_id = %command_id, "Getting command status");
    
    let status = state.command_service
        .get_command_status(&command_id, &claims.sub)
        .await?;
    
    Ok(api_success(status))
}

/// List user commands
pub async fn list_user_commands(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AuthClaims>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<Vec<CommandSummary>>>, CommandApiError> {
    info!(
        user_id = %claims.sub, 
        page = %pagination.page, 
        limit = %pagination.limit, 
        "Listing user commands"
    );
    
    let (commands, total) = state.command_service
        .list_user_commands(&claims.sub, pagination.page, pagination.limit)
        .await?;
    
    Ok(api_success_paginated(
        commands,
        pagination.page,
        pagination.limit,
        total,
        ((total as f64) / (pagination.limit as f64)).ceil() as i64
    ))
}

/// List available commands
pub async fn list_available_commands(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<ApiResponse<Vec<AvailableCommand>>>, CommandApiError> {
    info!(user_id = %claims.sub, "Listing available commands");
    
    let commands = state.command_service
        .get_available_commands()
        .await?;
    
    Ok(api_success(commands))
}

/// Command API error
#[derive(Debug, thiserror::Error)]
pub enum CommandApiError {
    #[error("Command service error: {0}")]
    ServiceError(#[from] CommandServiceError),
}

impl IntoResponse for CommandApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_code, message) = match &self {
            CommandApiError::ServiceError(err) => match err {
                CommandServiceError::CommandNotFound(_) => (
                    StatusCode::NOT_FOUND,
                    "COMMAND_NOT_FOUND",
                    err.to_string(),
                ),
                CommandServiceError::InvalidParameters(_) => (
                    StatusCode::BAD_REQUEST,
                    "INVALID_PARAMETERS",
                    err.to_string(),
                ),
                CommandServiceError::ExecutionFailed(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "EXECUTION_FAILED",
                    err.to_string(),
                ),
                CommandServiceError::RepositoryError(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "An internal database error occurred".to_string(),
                ),
                CommandServiceError::McpError(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "MCP_ERROR",
                    "Error communicating with the command system".to_string(),
                ),
                CommandServiceError::InvalidExecutionId(_) => (
                    StatusCode::NOT_FOUND,
                    "EXECUTION_NOT_FOUND",
                    err.to_string(),
                ),
                CommandServiceError::Unauthorized(_) => (
                    StatusCode::FORBIDDEN,
                    "UNAUTHORIZED",
                    err.to_string(),
                ),
            },
        };
        
        // Log the error
        error!(
            error_code = %error_code,
            status_code = %status.as_u16(),
            error = %self,
            "Command API error"
        );
        
        let body = Json(ApiResponse::<()> {
            success: false,
            data: None,
            error: Some(ApiError {
                code: error_code.to_string(),
                message,
                details: None,
            }),
            meta: Default::default(),
        });
        
        (status, body).into_response()
    }
} 