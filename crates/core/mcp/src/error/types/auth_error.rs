// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Authentication and authorization error types.

/// Errors related to Authentication and Authorization
#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthError {
    /// Returned when credentials are invalid
    #[error("Invalid credentials provided.")]
    InvalidCredentials,
    /// Returned when the authentication token is invalid or expired
    #[error("Authentication token is invalid or expired.")]
    InvalidToken,
    /// Returned when the user account is locked or inactive
    #[error("User account is locked or inactive.")]
    AccountLocked,
    /// Returned when permission is denied for the requested action
    #[error("Permission denied for action on resource '{0}'.")]
    PermissionDenied(String),
    /// Returned when authorization context is missing or invalid
    #[error("Authorization context is missing or invalid.")]
    MissingContext,
    /// Returned when the external authentication provider fails
    #[error("External authentication provider error: {0}")]
    ProviderError(String),
    /// Returned for internal authentication system errors
    #[error("An internal authentication error occurred: {0}")]
    InternalError(String),
}
