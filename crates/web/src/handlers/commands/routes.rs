use std::{str::FromStr, sync::Arc};

use axum::{
    extract::{Path, Query, State, Extension},
    routing::{get, post},
    Router,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    api::{
        commands::models::{CommandStatus, CreateCommandRequest, CreateCommandResponse},
        ApiResponse,
        api_success,
    },
    AppState,
    auth::extractor::AuthClaims,
    api::error::AppError,
};

/// Command routes
pub fn command_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_command))
        .route("/", get(list_commands))
        .route("/:id", get(get_command_status))
        .route("/:id/cancel", post(cancel_command))
        .route("/history", get(get_command_history))
}

/// Custom response types to replace the missing ones from the API module
#[derive(Debug, Serialize)]
pub struct CommandListResponse {
    pub commands: Vec<crate::api::commands::models::AvailableCommand>,
}

#[derive(Debug, Serialize)]
pub struct CommandStatusResponse {
    pub id: String,
    pub command: String,
    pub status: CommandStatus,
    pub progress: f32,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CommandHistoryResponse {
    pub executions: Vec<CommandStatusResponse>,
}

/// Create a new command
async fn create_command(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthClaims>,
    Json(payload): Json<CreateCommandRequest>,
) -> Result<Json<ApiResponse<CreateCommandResponse>>, AppError> {
    let command_service = state.get_command_service();
    
    // Map the legacy-style request to the new API style
    let request = crate::api::commands::models::CreateCommandRequest {
        command: payload.command,
        parameters: payload.parameters,
    };
    
    // Call the execute_command method from the real service
    let response = command_service.execute_command(request, &user.sub).await
        .map_err(|e| AppError::Internal(format!("Command execution failed: {}", e)))?;
    
    Ok(api_success(response))
}

/// List available commands
async fn list_commands(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthClaims>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ApiResponse<CommandListResponse>>, AppError> {
    let command_service = state.get_command_service();
    
    // Get all available commands without pagination (API doesn't support it yet)
    let commands = command_service.get_available_commands().await
        .map_err(|e| AppError::Internal(format!("Failed to get available commands: {}", e)))?;
    
    // Apply pagination in memory
    let page = params.page.unwrap_or(1) as usize;
    let limit = params.limit.unwrap_or(20) as usize;
    let start = (page - 1) * limit;
    let end = start + limit;
    
    let total_items = commands.len();
    let commands_slice = if start < total_items {
        commands[start..total_items.min(end)].to_vec()
    } else {
        Vec::new()
    };
    
    let response = CommandListResponse {
        commands: commands_slice,
    };
    
    Ok(api_success(response))
}

/// Get command status
async fn get_command_status(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthClaims>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<CommandStatusResponse>>, AppError> {
    let command_service = state.get_command_service();
    
    // Call the get_command_status method from the real service
    let command = command_service.get_command_status(&id, &user.sub).await
        .map_err(|e| AppError::Internal(format!("Failed to get command status: {}", e)))?;
    
    // Convert response with proper datetime formatting
    let response = CommandStatusResponse {
        id: command.id,
        command: command.command,
        status: command.status,
        progress: command.progress,
        result: command.result,
        error: command.error,
        started_at: command.started_at.map(|dt| dt.to_rfc3339()),
        completed_at: command.completed_at.map(|dt| dt.to_rfc3339()),
        created_at: Some(command.created_at.to_rfc3339()),
        updated_at: Some(command.updated_at.to_rfc3339()),
    };
    
    Ok(api_success(response))
}

/// Cancel command execution
async fn cancel_command(
    State(_state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthClaims>,
    Path(_id): Path<String>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    // Note: The real CommandService doesn't have a cancel_command method yet
    // For now, we'll just return success
    Ok(api_success(()))
}

/// Helper function to convert string to CommandStatus
fn parse_command_status(status_str: &str) -> Option<CommandStatus> {
    match status_str.to_lowercase().as_str() {
        "queued" => Some(CommandStatus::Queued),
        "running" => Some(CommandStatus::Running),
        "completed" => Some(CommandStatus::Completed),
        "failed" => Some(CommandStatus::Failed),
        "cancelled" => Some(CommandStatus::Cancelled),
        _ => None,
    }
}

/// Get command history
async fn get_command_history(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthClaims>,
    Query(params): Query<CommandHistoryParams>,
) -> Result<Json<ApiResponse<CommandHistoryResponse>>, AppError> {
    let command_service = state.get_command_service();
    
    let page = params.page.unwrap_or(1) as i64;
    let limit = params.limit.unwrap_or(20) as i64;
    let _status = params.status.as_deref()
        .map(|s| CommandStatus::from_str(s).unwrap_or(CommandStatus::Failed));
    
    // Get command history for user
    let (commands, _) = command_service.list_user_commands(&user.sub, page, limit).await
        .map_err(|e| AppError::Internal(format!("Failed to get command history: {}", e)))?;
    
    // Filter by status and command if requested
    let status_filter = params.status.as_deref().and_then(parse_command_status);
    let command_filter = params.command.as_deref();
    
    let filtered_commands = commands.into_iter()
        .filter(|cmd| {
            let status_match = match status_filter {
                None => true,
                Some(s) => cmd.status == s,
            };
            let command_match = match command_filter {
                None => true,
                Some(c) => cmd.command == *c,
            };
            status_match && command_match
        })
        .collect::<Vec<_>>();
    
    // Convert to response format
    let executions = filtered_commands.into_iter().map(|cmd| {
        CommandStatusResponse {
            id: cmd.id,
            command: cmd.command,
            status: cmd.status,
            progress: 1.0, // Not available in summary
            result: None,  // Not available in summary
            error: None,   // Not available in summary
            started_at: cmd.started_at.map(|dt| dt.to_rfc3339()),
            completed_at: cmd.completed_at.map(|dt| dt.to_rfc3339()),
            created_at: Some(cmd.created_at.to_rfc3339()),
            updated_at: None, // Not available in summary
        }
    }).collect();
    
    let response = CommandHistoryResponse {
        executions,
    };
    
    Ok(api_success(response))
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