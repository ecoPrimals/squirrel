//! Authentication middleware for protecting routes.

use axum::{
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use super::{AuthError, Role};
use crate::AppState;

/// Require authentication for a route
pub async fn require_auth<B>(
    State(state): State<Arc<AppState>>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, AuthError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(AuthError::InvalidCredentials)?;

    let claims = state.auth.verify_token(auth_header).await?;
    req.extensions_mut().insert(claims);
    
    Ok(next.run(req).await)
}

/// Require admin role for a route
pub async fn require_admin<B>(
    State(state): State<Arc<AppState>>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, AuthError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(AuthError::InvalidCredentials)?;

    let claims = state.auth.verify_token(auth_header).await?;

    if claims.role != Role::Admin {
        return Err(AuthError::Unauthorized);
    }

    req.extensions_mut().insert(claims);
    
    Ok(next.run(req).await)
} 