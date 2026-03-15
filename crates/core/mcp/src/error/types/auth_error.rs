// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Authentication and authorization error types.

/// Errors related to Authentication and Authorization
#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials provided.")]
    InvalidCredentials,
    #[error("Authentication token is invalid or expired.")]
    InvalidToken,
    #[error("User account is locked or inactive.")]
    AccountLocked,
    #[error("Permission denied for action on resource '{0}'.")]
    PermissionDenied(String), // Holds permission identifier
    #[error("Authorization context is missing or invalid.")]
    MissingContext,
    #[error("External authentication provider error: {0}")]
    ProviderError(String),
    #[error("An internal authentication error occurred: {0}")]
    InternalError(String),
}
