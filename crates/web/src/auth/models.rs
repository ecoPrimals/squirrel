//! User models for authentication.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "db")]
use sqlx::{Decode, Encode, Sqlite, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Role {
    User,
    Admin,
}

#[cfg(feature = "db")]
impl Type<Sqlite> for Role {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <String as Type<Sqlite>>::type_info()
    }
}

#[cfg(feature = "db")]
impl<'r> Decode<'r, Sqlite> for Role {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <String as Decode<Sqlite>>::decode(value)?;
        match value.as_str() {
            "User" => Ok(Role::User),
            "Admin" => Ok(Role::Admin),
            _ => Err("Invalid role".into()),
        }
    }
}

#[cfg(feature = "db")]
impl<'q> Encode<'q, Sqlite> for Role {
    fn encode_by_ref(&self, args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>) -> sqlx::encode::IsNull {
        let s = match self {
            Role::User => "User",
            Role::Admin => "Admin",
        };
        args.push(sqlx::sqlite::SqliteArgumentValue::Text(std::borrow::Cow::Owned(s.to_string())));
        sqlx::encode::IsNull::No
    }
}

/// Database user model with all fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID
    pub id: Uuid,
    /// Username
    pub username: String,
    #[cfg(feature = "db")]
    /// Email address
    pub email: String,
    #[cfg(feature = "db")]
    /// Password hash
    pub password_hash: String,
    #[cfg(feature = "db")]
    /// User role
    pub role: Role,
    #[cfg(feature = "db")]
    /// Account creation time
    pub created_at: DateTime<Utc>,
    #[cfg(feature = "db")]
    /// Last update time
    pub updated_at: DateTime<Utc>,
}

/// Database user model used for query_as
#[derive(Debug, Clone)]
pub struct DatabaseUser {
    pub id: String,
    pub username: String,
    pub password: String,
}

/// User registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    /// Username
    pub username: String,
    /// Password
    pub password: String,
    /// Email address
    pub email: String,
}

/// User login request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    /// Username or email
    pub username: String,
    /// Password
    pub password: String,
}

/// Authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    /// Access token
    pub token: String,
    /// Token type
    pub token_type: String,
    /// Token expiration in seconds
    pub expires_in: i64,
    /// User ID
    pub user_id: Uuid,
    /// User role
    pub role: Role,
    /// Refresh token for getting a new access token
    pub refresh_token: String,
}

/// User profile response
#[derive(Debug, Serialize)]
pub struct UserProfile {
    /// User ID
    pub id: Uuid,
    /// Username
    pub username: String,
    /// Email address
    pub email: String,
    /// User role
    pub role: Role,
    /// Account creation time
    pub created_at: DateTime<Utc>,
}

#[cfg(feature = "db")]
impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            created_at: user.created_at,
        }
    }
}

#[cfg(feature = "mock-db")]
impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: "mock@example.com".to_string(),
            role: Role::User,
            created_at: Utc::now(),
        }
    }
} 