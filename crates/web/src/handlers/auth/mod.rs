use axum::Json;
use serde::{Deserialize, Serialize};
use crate::api::{ApiResponse, api_success};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub refresh_token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Handler for login requests
pub async fn login(
    Json(payload): Json<LoginRequest>,
) -> Json<ApiResponse<LoginResponse>> {
    // Mock implementation
    let user = UserInfo {
        id: "123".to_string(),
        username: payload.username,
        roles: vec!["user".to_string()],
    };
    
    let response = LoginResponse {
        token: "mock-jwt-token".to_string(),
        refresh_token: "mock-refresh-token".to_string(),
        user,
    };
    
    api_success(response)
}

/// Handler for token refresh
pub async fn refresh_token(
    Json(_payload): Json<RefreshTokenRequest>,
) -> Json<ApiResponse<LoginResponse>> {
    // Mock implementation
    let user = UserInfo {
        id: "123".to_string(),
        username: "user".to_string(),
        roles: vec!["user".to_string()],
    };
    
    let response = LoginResponse {
        token: "new-mock-jwt-token".to_string(),
        refresh_token: "new-mock-refresh-token".to_string(),
        user,
    };
    
    api_success(response)
} 