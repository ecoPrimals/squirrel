//! Authentication routes for user management.

use axum::{
    routing::{post, get},
    Router, 
    extract::State,
    Json,
    response::IntoResponse,
};
use std::sync::Arc;
use serde::Deserialize;

use crate::{
    AppState,
    auth::{
        models::{LoginRequest, RegisterRequest, UserProfile, AuthResponse},
        AuthError, Claims,
    },
    api::{api_success, ApiResponse},
};

#[cfg(feature = "mock-db")]
use crate::auth::models::Role;

/// Refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    /// Refresh token
    pub refresh_token: String,
}

/// Simple handlers for route registration
async fn register_stub() -> impl IntoResponse {
    "Register endpoint"
}

async fn login_stub() -> impl IntoResponse {
    "Login endpoint"
}

async fn get_profile_stub() -> impl IntoResponse {
    "Profile endpoint"
}

/// Simple refresh handler for route registration
async fn refresh_stub() -> impl IntoResponse {
    "Refresh token endpoint"
}

/// Authentication routes
pub fn auth_routes() -> Router {
    Router::new()
        .route("/register", post(register_stub))
        .route("/login", post(login_stub))
        .route("/refresh", post(refresh_stub))
        .route("/profile", get(get_profile_stub))
}

/// Register a new user
#[cfg(feature = "db")]
async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AuthError> {
    let user = state.auth.register(req).await?;
    let token = state.auth.generate_token(user.id, user.role).await?;
    let refresh_token = state.auth.generate_refresh_token(user.id).await?;
    
    let response = AuthResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: state.auth.config.jwt_expiration_minutes * 60,
        user_id: user.id,
        role: user.role,
        refresh_token,
    };
    
    Ok(api_success(response))
}

/// Register a new user (mock implementation)
#[cfg(feature = "mock-db")]
async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AuthError> {
    let user = state.auth.register(req).await?;
    let token = state.auth.generate_token(user.id, Role::User).await?;
    let refresh_token = state.auth.generate_refresh_token(user.id).await?;
    
    let response = AuthResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: state.auth.config.jwt_expiration_minutes * 60,
        user_id: user.id,
        role: Role::User,
        refresh_token,
    };
    
    Ok(api_success(response))
}

/// Login an existing user
#[cfg(feature = "db")]
async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AuthError> {
    let (user, token, refresh_token) = state.auth.login(req).await?;
    
    let response = AuthResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: state.auth.config.jwt_expiration_minutes * 60,
        user_id: user.id,
        role: user.role,
        refresh_token,
    };
    
    Ok(api_success(response))
}

/// Login an existing user (mock implementation)
#[cfg(feature = "mock-db")]
async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AuthError> {
    let (user, token, refresh_token) = state.auth.login(req).await?;
    
    let response = AuthResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: state.auth.config.jwt_expiration_minutes * 60,
        user_id: user.id,
        role: Role::User,
        refresh_token,
    };
    
    Ok(api_success(response))
}

/// Refresh authentication token
async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AuthError> {
    let (claims, access_token) = state.auth.refresh_access_token(&req.refresh_token).await?;
    let refresh_token = state.auth.generate_refresh_token(claims.sub).await?;
    
    let response = AuthResponse {
        token: access_token,
        token_type: "Bearer".to_string(),
        expires_in: state.auth.config.jwt_expiration_minutes * 60,
        user_id: claims.sub,
        role: claims.role,
        refresh_token,
    };
    
    Ok(api_success(response))
}

/// Get authenticated user's profile
#[cfg(feature = "db")]
async fn get_profile(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<ApiResponse<UserProfile>>, AuthError> {
    let user = state.auth.get_user(claims.sub).await?;
    let profile = UserProfile::from(user);
    
    Ok(api_success(profile))
}

/// Get authenticated user's profile (mock implementation)
#[cfg(feature = "mock-db")]
async fn get_profile(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<ApiResponse<UserProfile>>, AuthError> {
    let user = state.auth.get_user(claims.sub).await?;
    
    // For mock-db, create a simplified profile
    let profile = UserProfile {
        id: user.id,
        username: user.username,
        email: "mock@example.com".to_string(),
        role: Role::User,
        created_at: chrono::Utc::now(),
    };
    
    Ok(api_success(profile))
} 