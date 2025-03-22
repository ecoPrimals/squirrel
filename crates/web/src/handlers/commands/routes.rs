use axum::{
    Router,
    routing::{get, post},
    extract::{Path, Query, State, Extension},
    Json,
};
use std::sync::Arc;
use crate::state::AppState;
use crate::auth::extractor::AuthClaims;
use crate::api::{
    api_success,
    commands::{
        CommandListResponse, CommandHistoryResponse, CommandStatusResponse, 
        CreateCommandRequest, CreateCommandResponse, CommandStatus
    },
    error::AppError,
    ApiResponse,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Command routes
pub fn command_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_command))
        .route("/", get(list_commands))
        .route("/:id", get(get_command_status))
        .route("/:id/cancel", post(cancel_command))
        .route("/history", get(get_command_history))
}

/// Create a new command
async fn create_command(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthClaims>,
    Json(payload): Json<CreateCommandRequest>,
) -> Result<Json<ApiResponse<CreateCommandResponse>>, AppError> {
    let command_service = state.get_command_service()?;
    let id = command_service.create_command(
        &user.sub,
        &payload.command,
        &payload.parameters,
    ).await?;

    let response = CreateCommandResponse {
        id: id.clone(),
        status_url: format!("/api/commands/{}/status", id),
    };

    Ok(api_success(response))
}

/// List available commands
async fn list_commands(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthClaims>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ApiResponse<CommandListResponse>>, AppError> {
    let command_service = state.get_command_service()?;
    
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);
    
    let (commands, _total_items, _total_pages) = command_service.get_available_commands(
        &user.sub,
        page,
        limit,
    ).await?;
    
    let response = CommandListResponse {
        commands,
    };
    
    Ok(api_success(response))
}

/// Get command status
async fn get_command_status(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthClaims>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<CommandStatusResponse>>, AppError> {
    let command_service = state.get_command_service()?;
    
    let command = command_service.get_command_status(
        &user.sub,
        &id,
    ).await?;
    
    // Convert CommandExecution to CommandStatusResponse
    let response = CommandStatusResponse {
        id: command.id,
        command: command.command_name,
        status: command.status,
        progress: command.progress,
        result: command.result,
        error: command.error,
        started_at: command.started_at.map(|t| t.to_rfc3339()),
        completed_at: command.completed_at.map(|t| t.to_rfc3339()),
        elapsed: format_elapsed(command.created_at),
    };
    
    Ok(api_success(response))
}

/// Cancel command execution
async fn cancel_command(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthClaims>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let command_service = state.get_command_service()?;
    
    command_service.cancel_command(
        &user.sub,
        &id,
    ).await?;
    
    Ok(api_success(()))
}

/// Get command history
async fn get_command_history(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthClaims>,
    Query(params): Query<CommandHistoryParams>,
) -> Result<Json<ApiResponse<CommandHistoryResponse>>, AppError> {
    let command_service = state.get_command_service()?;
    
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);
    let status = params.status.as_deref().map(CommandStatus::from_str);
    
    let (executions, _total_items, _total_pages) = command_service.get_command_history(
        &user.sub,
        page,
        limit,
        status,
        params.command.as_deref(),
    ).await?;
    
    // Convert CommandExecution to CommandStatusResponse
    let executions = executions.into_iter().map(|execution| {
        CommandStatusResponse {
            id: execution.id,
            command: execution.command_name,
            status: execution.status,
            progress: execution.progress,
            result: execution.result,
            error: execution.error,
            started_at: execution.started_at.map(|t| t.to_rfc3339()),
            completed_at: execution.completed_at.map(|t| t.to_rfc3339()),
            elapsed: format_elapsed(execution.created_at),
        }
    }).collect();
    
    let response = CommandHistoryResponse {
        executions,
    };
    
    Ok(api_success(response))
}

/// Helper function to format elapsed time
fn format_elapsed(created_at: DateTime<Utc>) -> String {
    let elapsed = Utc::now().signed_duration_since(created_at);
    if elapsed.num_days() > 0 {
        format!("{}d {}h", elapsed.num_days(), elapsed.num_hours() % 24)
    } else if elapsed.num_hours() > 0 {
        format!("{}h {}m", elapsed.num_hours(), elapsed.num_minutes() % 60)
    } else if elapsed.num_minutes() > 0 {
        format!("{}m {}s", elapsed.num_minutes(), elapsed.num_seconds() % 60)
    } else {
        format!("{}s", elapsed.num_seconds())
    }
}

/// Pagination parameters
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// Command history query parameters
#[derive(Debug, Deserialize)]
pub struct CommandHistoryParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub status: Option<String>,
    pub command: Option<String>,
}

/// Pagination information
#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub total_items: usize,
    pub total_pages: usize,
    pub current_page: usize,
    pub per_page: usize,
} 