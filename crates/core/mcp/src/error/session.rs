use thiserror::Error;

/// Errors related to MCP session operations
///
/// This enum represents errors that can occur when working with MCP sessions,
/// including authentication and authorization failures, timeouts, and validation issues.
#[derive(Debug, Clone, Error)]
pub enum SessionError {
    /// Error that occurs when session authentication fails
    ///
    /// This typically happens when credentials cannot be verified
    /// or the authentication process fails for any reason.
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Error that occurs when session authorization fails
    ///
    /// This typically happens when a user lacks the necessary permissions
    /// to perform a requested operation.
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    /// Error that occurs when a session times out
    ///
    /// This can happen when a session exceeds its maximum allowed
    /// duration or when there is no activity for a specified period.
    #[error("Session timeout: {0}")]
    Timeout(String),

    /// Error that occurs when a session is invalid
    ///
    /// This can happen when a session is malformed, corrupted,
    /// or doesn't meet the required constraints.
    #[error("Invalid session: {0}")]
    InvalidSession(String),

    /// Error that occurs when a session cannot be found
    ///
    /// This typically happens when trying to access a session
    /// that doesn't exist or has been removed.
    #[error("Session not found: {0}")]
    NotFound(String),

    /// Error that occurs during session validation
    ///
    /// This can happen when session data fails validation checks
    /// or doesn't meet the required constraints.
    #[error("Session validation error: {0}")]
    Validation(String),

    /// General internal error within the session management system
    ///
    /// This is used for errors that don't fit into other specific
    /// categories but occur within the session subsystem.
    #[error("Internal session error: {0}")]
    InternalError(String),

    /// Error reported by the remote peer
    #[error("Remote session error: {0}")]
    RemoteError(String),
}
