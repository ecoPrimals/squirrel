//! Authentication and authorization for the web interface.

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
#[cfg(feature = "db")]
use bcrypt;
use std::sync::Arc;
use crate::{AppState, api::{ApiResponse, ApiError, ApiMeta}};

pub mod models;
pub mod routes;
pub mod middleware;
pub mod extractor;

use models::{User, Role, LoginRequest, RegisterRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration_minutes: i64,
    pub refresh_token_expiration_days: i64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "your-secret-key".to_string(),
            jwt_expiration_minutes: 60,
            refresh_token_expiration_days: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub role: Role,
    pub exp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // This method is called when extracting Claims from a request
        // Get the authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .ok_or(AuthError::MissingToken)?;

        // Convert to string and extract token
        let auth_str = auth_header.to_str().map_err(|_| AuthError::InvalidToken)?;
        let token = auth_str
            .strip_prefix("Bearer ")
            .ok_or(AuthError::InvalidToken)?;

        // Extract app state to access auth service
        let state = parts.extensions.get::<Arc<AppState>>()
            .ok_or_else(|| AuthError::InternalError("AppState not found in request".to_string()))?;
            
        // Verify token
        let claims = state.auth.verify_token(token).await?;
        
        Ok(claims)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing authentication token")]
    MissingToken,
    #[error("Invalid authentication token")]
    InvalidToken,
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Username already exists")]
    UsernameExists,
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "AUTH_MISSING_TOKEN", "Missing authentication token"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "AUTH_INVALID_TOKEN", "Invalid authentication token"),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "AUTH_USER_NOT_FOUND", "User not found"),
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "AUTH_INVALID_CREDENTIALS", "Invalid credentials"),
            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, "AUTH_UNAUTHORIZED", "Unauthorized"),
            AuthError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "AUTH_DATABASE_ERROR", "Database error"),
            AuthError::JwtError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "AUTH_JWT_ERROR", "JWT error"),
            AuthError::UserAlreadyExists => (StatusCode::CONFLICT, "AUTH_USER_EXISTS", "User already exists"),
            AuthError::UsernameExists => (StatusCode::CONFLICT, "AUTH_USERNAME_EXISTS", "Username already exists"),
            AuthError::Internal(ref e) => (StatusCode::INTERNAL_SERVER_ERROR, "AUTH_INTERNAL_ERROR", e.as_str()),
            AuthError::InternalError(ref e) => (StatusCode::INTERNAL_SERVER_ERROR, "AUTH_INTERNAL_ERROR", e.as_str()),
        };

        let response: ApiResponse<()> = ApiResponse {
            success: false,
            data: None,
            error: Some(ApiError {
                code: error_code.to_string(),
                message: message.to_string(),
                details: None,
            }),
            meta: ApiMeta {
                request_id: Uuid::new_v4().to_string(),
                timestamp: Utc::now().to_rfc3339(),
                pagination: None,
            },
        };

        (status, Json(response)).into_response()
    }
}

#[derive(Debug, Clone)]
pub struct AuthService {
    pub config: AuthConfig,
    pub pool: SqlitePool,
}

impl AuthService {
    pub fn new(config: AuthConfig, pool: SqlitePool) -> Self {
        Self { config, pool }
    }
    
    pub async fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_ref()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }
    
    pub async fn generate_refresh_token(&self, user_id: Uuid) -> Result<String, AuthError> {
        // In mock mode, just generate a token string
        #[cfg(feature = "mock-db")]
        {
            Ok(format!("mock_refresh_token_{}", user_id))
        }

        #[cfg(feature = "db")]
        {
            let now = Utc::now();
            let expiration = now + chrono::Duration::days(self.config.refresh_token_expiration_days);
            let token = Uuid::new_v4().to_string();
            let token_id = Uuid::new_v4();
            
            // Remove any existing refresh tokens for this user
            sqlx::query!(
                "DELETE FROM refresh_tokens WHERE user_id = ?",
                user_id.to_string()
            )
            .execute(&self.pool)
            .await?;
            
            // Store the refresh token
            sqlx::query!(
                "INSERT INTO refresh_tokens (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
                token_id.to_string(),
                user_id.to_string(),
                token,
                expiration,
                now
            )
            .execute(&self.pool)
            .await?;
            
            Ok(token)
        }
    }
    
    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<(Claims, String), AuthError> {
        #[cfg(feature = "mock-db")]
        {
            // In mock mode, only accept tokens with the correct format
            if refresh_token.starts_with("mock_refresh_token_") {
                let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
                let role = Role::User;
                
                // Generate new access token
                let access_token = self.generate_token(user_id, role).await?;
                
                // Create claims for response
                let now = Utc::now();
                let claims = Claims {
                    sub: user_id,
                    role,
                    exp: (now + chrono::Duration::minutes(self.config.jwt_expiration_minutes)).timestamp(),
                };
                
                Ok((claims, access_token))
            } else {
                Err(AuthError::InvalidToken)
            }
        }

        #[cfg(feature = "db")]
        {
            // Find the refresh token
            let token = sqlx::query_as!(
                RefreshToken,
                r#"
                SELECT 
                    id as "id: Uuid", 
                    user_id as "user_id: Uuid", 
                    token,
                    expires_at as "expires_at: DateTime<Utc>",
                    created_at as "created_at: DateTime<Utc>"
                FROM refresh_tokens 
                WHERE token = ?
                "#,
                refresh_token
            )
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AuthError::InvalidToken)?;
            
            // Check if token has expired
            let now = Utc::now();
            if token.expires_at < now {
                // Delete expired token
                sqlx::query!(
                    "DELETE FROM refresh_tokens WHERE id = ?",
                    token.id.to_string()
                )
                .execute(&self.pool)
                .await?;
                
                return Err(AuthError::InvalidToken);
            }
            
            // Get user
            let user = self.get_user(token.user_id).await?;
            
            // Generate new access token
            let access_token = self.generate_token(user.id, user.role).await?;
            
            // Generate new refresh token
            let new_refresh_token = self.generate_refresh_token(user.id).await?;
            
            // Create claims for response
            let claims = Claims {
                sub: user.id,
                role: user.role,
                exp: (now + chrono::Duration::minutes(self.config.jwt_expiration_minutes)).timestamp(),
            };
            
            Ok((claims, access_token))
        }
    }
    
    pub async fn generate_token(&self, user_id: Uuid, role: Role) -> Result<String, AuthError> {
        let now = Utc::now();
        let expiration = now + chrono::Duration::minutes(self.config.jwt_expiration_minutes);
        
        let claims = Claims {
            sub: user_id,
            role,
            exp: expiration.timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_ref()),
        )?;

        Ok(token)
    }
    
    pub async fn get_user(&self, user_id: Uuid) -> Result<User, AuthError> {
        #[cfg(feature = "mock-db")]
        {
            // In mock mode, we create a mock user
            if user_id == Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap() {
                Ok(User {
                    id: user_id,
                    username: "testuser".to_string(),
                    #[cfg(feature = "db")]
                    email: "".to_string(),
                    #[cfg(feature = "db")]
                    password_hash: "".to_string(),
                    #[cfg(feature = "db")]
                    role: Role::User,
                    #[cfg(feature = "db")]
                    created_at: Utc::now(),
                    #[cfg(feature = "db")]
                    updated_at: Utc::now(),
                })
            } else {
                Err(AuthError::UserNotFound)
            }
        }

        #[cfg(feature = "db")]
        {
            // Get user from database
            let user = sqlx::query_as!(
                User,
                r#"
                SELECT 
                    id as "id: Uuid", 
                    username, 
                    email, 
                    password_hash, 
                    role as "role: Role", 
                    created_at as "created_at: DateTime<Utc>", 
                    updated_at as "updated_at: DateTime<Utc>"
                FROM users 
                WHERE id = ?
                "#,
                user_id.to_string()
            )
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AuthError::UserNotFound)?;

            Ok(user)
        }
    }
    
    pub async fn login(&self, req: LoginRequest) -> Result<(User, String, String), AuthError> {
        #[cfg(feature = "mock-db")]
        {
            // In mock mode, only accept specific credentials
            if req.username == "testuser" && req.password == "password" {
                let user = User {
                    id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                    username: req.username,
                    #[cfg(feature = "db")]
                    email: "".to_string(),
                    #[cfg(feature = "db")]
                    password_hash: "".to_string(),
                    #[cfg(feature = "db")]
                    role: Role::User,
                    #[cfg(feature = "db")]
                    created_at: Utc::now(),
                    #[cfg(feature = "db")]
                    updated_at: Utc::now(),
                };
                
                // Generate JWT token
                let token = self.generate_token(user.id, Role::User).await?;
                
                // Generate refresh token
                let refresh_token = self.generate_refresh_token(user.id).await?;

                Ok((user, token, refresh_token))
            } else {
                Err(AuthError::InvalidCredentials)
            }
        }

        #[cfg(feature = "db")]
        {
            // Find user by username
            let user = sqlx::query_as!(
                User,
                r#"
                SELECT 
                    id as "id: Uuid", 
                    username, 
                    email, 
                    password_hash, 
                    role as "role: Role", 
                    created_at as "created_at: DateTime<Utc>", 
                    updated_at as "updated_at: DateTime<Utc>"
                FROM users 
                WHERE username = ? OR email = ?
                "#,
                req.username,
                req.username
            )
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

            // Verify password
            let is_valid = bcrypt::verify(&req.password, &user.password_hash)
                .map_err(|_| AuthError::InternalError("Failed to verify password".to_string()))?;

            if !is_valid {
                return Err(AuthError::InvalidCredentials);
            }

            // Generate JWT token
            let token = self.generate_token(user.id, user.role).await?;
            
            // Generate refresh token
            let refresh_token = self.generate_refresh_token(user.id).await?;

            Ok((user, token, refresh_token))
        }
    }
    
    pub async fn register(&self, req: RegisterRequest) -> Result<User, AuthError> {
        #[cfg(feature = "mock-db")]
        {
            // In mock mode, we just check if the username is one we want to reject
            if req.username == "existinguser" {
                return Err(AuthError::UserAlreadyExists);
            }

            // Create a mock user
            let user_id = Uuid::new_v4();
            
            // Return user
            Ok(User {
                id: user_id,
                username: req.username,
                #[cfg(feature = "db")]
                email: "".to_string(),
                #[cfg(feature = "db")]
                password_hash: "".to_string(),
                #[cfg(feature = "db")]
                role: Role::User,
                #[cfg(feature = "db")]
                created_at: Utc::now(),
                #[cfg(feature = "db")]
                updated_at: Utc::now(),
            })
        }

        #[cfg(feature = "db")]
        {
            // Check if user already exists
            let existing_user = sqlx::query!(
                "SELECT username FROM users WHERE username = ? OR email = ?",
                req.username,
                req.email
            )
            .fetch_optional(&self.pool)
            .await?;

            if existing_user.is_some() {
                return Err(AuthError::UserAlreadyExists);
            }

            // Hash the password
            let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
                .map_err(|_| AuthError::InternalError("Failed to hash password".to_string()))?;

            // Create new user
            let user_id = Uuid::new_v4();
            let now = Utc::now();

            // Insert user into database
            sqlx::query!(
                "INSERT INTO users (id, username, email, password_hash, role, created_at, updated_at) 
                VALUES (?, ?, ?, ?, ?, ?, ?)",
                user_id.to_string(),
                req.username,
                req.email,
                password_hash,
                "User", // Default role
                now,
                now
            )
            .execute(&self.pool)
            .await?;

            // Return user
            Ok(User {
                id: user_id,
                username: req.username,
                email: req.email,
                password_hash,
                role: Role::User,
                created_at: now,
                updated_at: now,
            })
        }
    }
}

#[cfg(feature = "db")]
pub async fn register_user(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<models::User, AuthError> {
    // Check if username already exists
    let user_exists = sqlx::query!(
        "SELECT username FROM users WHERE username = ?",
        username
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        AuthError::Internal("Database error".to_string())
    })?
    .is_some();

    if user_exists {
        return Err(AuthError::UsernameExists);
    }

    // Hash the password
    let hashed_password = bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|_| AuthError::Internal("Password hashing failed".to_string()))?;

    // Create a new user ID
    let user_id = Uuid::new_v4();

    // Insert new user
    sqlx::query!(
        "INSERT INTO users (id, username, password) VALUES (?, ?, ?)",
        user_id.to_string(),
        username,
        hashed_password
    )
    .execute(pool)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        AuthError::Internal("Database error".to_string())
    })?;

    // Return the new user
    Ok(models::User {
        id: user_id,
        username: username.to_string(),
        #[cfg(feature = "db")]
        email: "".to_string(),
        #[cfg(feature = "db")]
        password_hash: hashed_password,
        #[cfg(feature = "db")]
        role: models::Role::User,
        #[cfg(feature = "db")]
        created_at: Utc::now(),
        #[cfg(feature = "db")]
        updated_at: Utc::now(),
    })
}

#[cfg(feature = "mock-db")]
pub async fn register_user(
    _pool: &SqlitePool,
    username: &str,
    _password: &str,
) -> Result<models::User, AuthError> {
    // Mock implementation - always create a user with fixed ID
    if username == "existinguser" {
        return Err(AuthError::UsernameExists);
    }
    
    Ok(models::User {
        id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
        username: username.to_string(),
    })
}

#[cfg(feature = "db")]
pub async fn authenticate_user(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<models::User, AuthError> {
    // Fetch user from database
    let user = sqlx::query_as!(
        models::DatabaseUser,
        "SELECT id, username, password FROM users WHERE username = ?",
        username
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        AuthError::Internal("Database error".to_string())
    })?;

    // Check if user exists
    let user = user.ok_or(AuthError::InvalidCredentials)?;

    // Verify password
    let password_verified = bcrypt::verify(password, &user.password)
        .map_err(|_| AuthError::Internal("Password verification failed".to_string()))?;

    if !password_verified {
        return Err(AuthError::InvalidCredentials);
    }

    // Parse user ID
    let id = Uuid::parse_str(&user.id)
        .map_err(|_| AuthError::Internal("Invalid user ID".to_string()))?;

    Ok(models::User {
        id,
        username: user.username,
        #[cfg(feature = "db")]
        email: "".to_string(), // We don't have email in DatabaseUser
        #[cfg(feature = "db")]
        password_hash: user.password,
        #[cfg(feature = "db")]
        role: models::Role::User, // Default role
        #[cfg(feature = "db")]
        created_at: Utc::now(),
        #[cfg(feature = "db")]
        updated_at: Utc::now(),
    })
}

#[cfg(feature = "mock-db")]
pub async fn authenticate_user(
    _pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<models::User, AuthError> {
    // Mock implementation - only accept 'testuser' with 'password'
    if username == "testuser" && password == "password" {
        Ok(models::User {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            username: username.to_string(),
        })
    } else {
        Err(AuthError::InvalidCredentials)
    }
}

#[cfg(feature = "db")]
pub async fn store_refresh_token(
    pool: &SqlitePool,
    user_id: &Uuid,
    token: &str,
    expires_at: &DateTime<Utc>,
) -> Result<(), AuthError> {
    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (token, user_id, expires_at)
        VALUES (?, ?, ?)
        "#,
        token,
        user_id.to_string(),
        expires_at
    )
    .execute(pool)
    .await
    .map_err(|e| {
        eprintln!("Failed to store refresh token: {:?}", e);
        AuthError::Internal("Failed to store refresh token".to_string())
    })?;

    Ok(())
}

#[cfg(feature = "mock-db")]
pub async fn store_refresh_token(
    _pool: &SqlitePool,
    _user_id: &Uuid,
    _token: &str,
    _expires_at: &DateTime<Utc>,
) -> Result<(), AuthError> {
    // Mock implementation - always succeeds
    Ok(())
}

#[cfg(feature = "db")]
pub async fn validate_refresh_token(
    pool: &SqlitePool,
    token: &str,
) -> Result<Uuid, AuthError> {
    let result = sqlx::query!(
        r#"
        SELECT user_id FROM refresh_tokens
        WHERE token = ? AND expires_at > datetime('now')
        "#,
        token
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        eprintln!("Failed to validate refresh token: {:?}", e);
        AuthError::Internal("Failed to validate refresh token".to_string())
    })?;

    let user_id = result
        .ok_or(AuthError::InvalidToken)?
        .user_id;

    let user_id = Uuid::parse_str(&user_id)
        .map_err(|_| AuthError::Internal("Invalid user ID".to_string()))?;

    Ok(user_id)
}

#[cfg(feature = "mock-db")]
pub async fn validate_refresh_token(
    _pool: &SqlitePool,
    token: &str,
) -> Result<Uuid, AuthError> {
    // Mock implementation - only accept 'valid_refresh_token'
    if token == "valid_refresh_token" {
        Ok(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap())
    } else {
        Err(AuthError::InvalidToken)
    }
}

#[cfg(feature = "db")]
pub async fn revoke_refresh_token(
    pool: &SqlitePool,
    token: &str,
) -> Result<(), AuthError> {
    sqlx::query!(
        "DELETE FROM refresh_tokens WHERE token = ?",
        token
    )
    .execute(pool)
    .await
    .map_err(|e| {
        eprintln!("Failed to revoke refresh token: {:?}", e);
        AuthError::Internal("Failed to revoke refresh token".to_string())
    })?;

    Ok(())
}

#[cfg(feature = "mock-db")]
pub async fn revoke_refresh_token(
    _pool: &SqlitePool,
    _token: &str,
) -> Result<(), AuthError> {
    // Mock implementation - always succeeds
    Ok(())
}