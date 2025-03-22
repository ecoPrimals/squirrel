use std::fmt;
use axum::{
    async_trait,
    extract::FromRequest,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

/// JWT Claims for authentication
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Issued at
    pub iat: i64,
    /// Expires at
    pub exp: i64,
    /// Roles assigned to the user
    #[serde(default)]
    pub roles: Vec<String>,
}

impl fmt::Display for AuthClaims {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AuthClaims(sub={}, roles={})", self.sub, self.roles.join(","))
    }
}

#[async_trait]
impl<S, B> FromRequest<S, B> for AuthClaims
where
    B: Send + 'static,
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(_req: axum::http::Request<B>, _state: &S) -> Result<Self, Self::Rejection> {
        // In a real implementation, this would extract and validate a JWT token
        // For now, we'll just return an error
        Err((StatusCode::UNAUTHORIZED, "Not authorized"))
    }
} 