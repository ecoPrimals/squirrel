// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Main error type for MCP operations.

use crate::error::{
    alert::AlertError, client::ClientError, config::ConfigError, connection::ConnectionError,
    context_err::ContextError as ContextErr, handler::HandlerError, integration::IntegrationError,
    plugin::PluginError, port::PortErrorKind, protocol_err::ProtocolError, rbac::RBACError,
    registry::RegistryError, session::SessionError, task::TaskError, tool::ToolError,
    transport::TransportError,
};

use super::ErrorSeverity;

/// Main error type for MCP operations.
///
/// This enum represents all possible errors that can occur during MCP operations.
/// It categorizes errors into different types based on their source and nature,
/// providing detailed information about what went wrong.
///
/// # Examples
///
/// ```
/// use squirrel_mcp::error::{MCPError, Result};
///
/// fn handle_error() -> Result<()> {
///     // Operation that might fail
///     if false {
///         return Err(MCPError::General("Something went wrong".to_string()));
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum MCPError {
    // === Core Layer Errors ===
    /// Error originating from the MCP transport layer
    #[error(transparent)]
    Transport(#[from] TransportError),

    /// Error originating from the MCP protocol
    #[error(transparent)]
    Protocol(#[from] ProtocolError),

    /// Error originating from connection management
    #[error(transparent)]
    Connection(#[from] ConnectionError),

    // === Application Layer Errors ===
    /// Error originating from context management
    #[error(transparent)]
    Context(#[from] ContextErr),

    /// Error originating from session management
    #[error(transparent)]
    Session(#[from] SessionError),

    /// Error originating from client operations
    #[error(transparent)]
    Client(#[from] ClientError),

    /// Plugin-related errors
    #[error(transparent)]
    Plugin(#[from] PluginError),

    /// Tool execution errors
    #[error(transparent)]
    Tool(#[from] ToolError),

    /// Service registry errors
    #[error(transparent)]
    Registry(#[from] RegistryError),

    /// Task management errors
    #[error(transparent)]
    Task(#[from] TaskError),

    /// Request handler errors
    #[error(transparent)]
    Handler(#[from] HandlerError),

    // === Infrastructure Layer Errors ===
    /// Configuration errors
    #[error(transparent)]
    Config(#[from] ConfigError),

    /// Integration errors
    #[error(transparent)]
    Integration(#[from] IntegrationError),

    /// Port allocation errors
    #[error(transparent)]
    Port(#[from] PortErrorKind),

    // === Security Layer Errors ===
    /// Role-based access control errors
    #[error(transparent)]
    RBAC(#[from] RBACError),

    // === Monitoring Layer Errors ===
    /// Alert system errors
    #[error(transparent)]
    Alert(#[from] AlertError),

    /// Resource exhausted error
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    /// Invalid argument error
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// Internal server error
    #[error("Internal: {0}")]
    Internal(String),

    /// Authentication error
    #[error("Authentication: {0}")]
    Authentication(String),

    /// Authorization error
    #[error("Authorization: {0}")]
    Authorization(String),

    /// Rate limit exceeded
    #[error("Rate limit: {0}")]
    RateLimit(String),

    /// Timeout error
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Configuration error
    #[error("Configuration: {0}")]
    Configuration(String),

    /// Validation error
    #[error("Validation: {0}")]
    Validation(String),

    /// Invalid state error
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Invalid operation error
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Network error
    #[error("Network: {0}")]
    Network(String),

    /// IO error
    #[error("IO: {0}")]
    Io(String),

    /// JSON parsing error
    #[error("JSON: {0}")]
    Json(String),

    /// General error
    #[error("{0}")]
    General(String),

    /// Generic error (alias for General)
    #[error("{0}")]
    Generic(String),

    /// Message router error
    #[error("Message router: {0}")]
    MessageRouter(String),

    /// Serialization error
    #[error("Serialization: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization: {0}")]
    Deserialization(String),

    /// Invalid message error
    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    /// State error
    #[error("State: {0}")]
    State(String),

    /// Unsupported operation error
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    /// Circuit breaker error
    #[error("Circuit breaker: {0}")]
    CircuitBreaker(String),

    /// Security error
    #[error("Security: {0}")]
    Security(String),

    /// Resource error
    #[error("Resource: {0}")]
    Resource(String),

    /// Lifecycle error
    #[error("Lifecycle: {0}")]
    Lifecycle(String),

    /// Wire format error
    #[error("Wire format: {0}")]
    WireFormat(String),

    /// Not initialized error
    #[error("Not initialized: {0}")]
    NotInitialized(String),

    /// Already in progress error
    #[error("Already in progress: {0}")]
    AlreadyInProgress(String),

    /// Monitoring error
    #[error("Monitoring: {0}")]
    Monitoring(String),

    /// Not connected error
    #[error("Not connected: {0}")]
    NotConnected(String),

    /// Remote error
    #[error("Remote: {0}")]
    Remote(String),

    /// Unexpected error
    #[error("Unexpected: {0}")]
    Unexpected(String),

    /// Version mismatch error
    #[error("Version mismatch: {0}")]
    VersionMismatch(String),

    /// Unsupported error
    #[error("Unsupported: {0}")]
    Unsupported(String),

    /// Not implemented error
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Not authorized error
    #[error("Not authorized: {0}")]
    NotAuthorized(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Sync error
    #[error("Sync: {0}")]
    Sync(String),

    /// Already exists error
    #[error("Already exists: {0}")]
    AlreadyExists(String),

    /// Invalid request error
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Database error
    #[error("Database: {0}")]
    Database(String),

    /// Operation failed error
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

impl MCPError {
    /// Creates an error from an MCP error message
    #[must_use]
    pub fn from_message(message: &str) -> Self {
        Self::Generic(message.to_string())
    }

    /// Determines if this error is recoverable
    ///
    /// Recoverable errors can be retried with a reasonable expectation of success.
    /// Examples include temporary network issues, authentication failures that can
    /// be resolved by obtaining new credentials, or timeout errors that might succeed
    /// on a subsequent attempt.
    ///
    /// # Returns
    ///
    /// * `true` - If the error is recoverable
    /// * `false` - If the error is not recoverable
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        match self {
            // Recoverable errors
            Self::Connection(ConnectionError::Timeout(_) | ConnectionError::Reset)
            | Self::UnsupportedOperation(_) => true,

            // Default case - all other errors are non-recoverable
            _ => false,
        }
    }

    /// Determines the severity level of this error
    ///
    /// Severity levels help prioritize error handling and reporting.
    /// Higher severity errors require more immediate attention or
    /// may have a greater impact on system functionality.
    ///
    /// # Returns
    ///
    /// The severity level of the error as an `ErrorSeverity` enum value
    #[must_use]
    pub const fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Connection(
                ConnectionError::ConnectionFailed(_)
                | ConnectionError::Closed(_)
                | ConnectionError::Reset
                | ConnectionError::Refused
                | ConnectionError::Unreachable
                | ConnectionError::TooManyConnections
                | ConnectionError::LimitReached(_),
            )
            | Self::Protocol(ProtocolError::InvalidVersion(_)) => ErrorSeverity::High,
            Self::UnsupportedOperation(_) => ErrorSeverity::Medium,

            // All other errors are low severity
            _ => ErrorSeverity::Low,
        }
    }

    /// Returns a string error code for this error.
    ///
    /// The error code consists of a category prefix and a numeric code, e.g., "MCP-001".
    /// This can be used for error tracking and reporting.
    #[must_use]
    pub const fn code_str(&self) -> &'static str {
        match self {
            Self::Transport(_) => "MCP-001",
            Self::Protocol(_) => "MCP-002",
            Self::Connection(_) => "MCP-004",
            Self::Session(_) => "MCP-005",
            Self::Context(_) => "MCP-006",
            Self::Client(_) => "MCP-007",
            Self::MessageRouter(_) => "MCP-008",
            Self::Serialization(_) => "MCP-009",
            Self::Deserialization(_) => "MCP-010",
            Self::InvalidMessage(_) => "MCP-011",
            Self::State(_) => "MCP-012",
            Self::Authorization(_) => "MCP-013",
            Self::UnsupportedOperation(_) => "MCP-014",
            Self::CircuitBreaker(_) => "MCP-015",
            Self::Io(_) => "MCP-016",
            Self::Json(_) => "MCP-017",
            Self::Task(_) => "MCP-018",
            Self::Handler(_) => "MCP-019",
            Self::Plugin(_) => "MCP-020",
            Self::Security(_) => "MCP-021",
            Self::Resource(_) => "MCP-022",
            Self::Validation(_) => "MCP-023",
            Self::Lifecycle(_) => "MCP-024",
            Self::Tool(_) => "MCP-025",
            Self::WireFormat(_) => "MCP-027",
            Self::NotInitialized(_) => "MCP-028",
            Self::General(_) => "MCP-029",
            Self::AlreadyInProgress(_) => "MCP-030",
            Self::Monitoring(_) => "MCP-031",
            Self::NotConnected(_) => "MCP-032",
            Self::Timeout(_) => "MCP-033",
            Self::Remote(_) => "MCP-034",
            Self::Unexpected(_) => "MCP-035",
            Self::VersionMismatch(_) => "MCP-036",
            Self::Unsupported(_) => "MCP-037",
            Self::InvalidArgument(_) => "MCP-038",
            Self::NotFound(_) => "MCP-039",
            Self::NotImplemented(_) => "MCP-040",
            Self::NotAuthorized(_) => "MCP-041",
            Self::InvalidState(_) => "MCP-042",
            Self::InvalidOperation(_) => "MCP-043",
            Self::InternalError(_) => "MCP-044",
            Self::Sync(_) => "MCP-045",
            Self::AlreadyExists(_) => "MCP-046",
            Self::InvalidRequest(_) => "MCP-047",
            Self::Database(_) => "MCP-048",
            Self::OperationFailed(_) => "MCP-049",
            Self::Generic(_) => "MCP-050",
            Self::Configuration(_) => "MCP-051",
            Self::Alert(_) => "MCP-052",
            Self::Network(_) => "MCP-053",
            Self::ResourceExhausted(_) => "MCP-056",
            Self::Internal(_) => "MCP-057",
            Self::Authentication(_) => "MCP-058",
            Self::RateLimit(_) => "MCP-059",
            Self::Registry(_) => "MCP-060",
            Self::Config(_) => "MCP-061",
            Self::Integration(_) => "MCP-062",
            Self::Port(_) => "MCP-063",
            Self::RBAC(_) => "MCP-064",
        }
    }

    /// For backwards compatibility with existing code
    #[must_use]
    pub fn error_code(&self) -> String {
        self.code_str().to_string()
    }

    /// Returns a string representation of the general error category.
    pub const fn category_str(&self) -> &'static str {
        match self {
            Self::Transport(_) => "TRANSPORT",
            Self::Protocol(_) => "PROTOCOL",
            Self::Connection(_) => "CONNECTION",
            Self::Session(_) => "SESSION",
            Self::Context(_) => "CONTEXT",
            Self::Client(_) => "CLIENT",
            Self::MessageRouter(_) => "MESSAGE_ROUTER",
            Self::Serialization(_) => "SERIALIZATION",
            Self::Deserialization(_) => "DESERIALIZATION",
            Self::InvalidMessage(_) => "INVALID_MESSAGE",
            Self::State(_) => "STATE",
            Self::Authorization(_) => "AUTHORIZATION",
            Self::UnsupportedOperation(_) => "UNSUPPORTED_OPERATION",
            Self::CircuitBreaker(_) => "CIRCUIT_BREAKER",
            Self::Io(_) => "IO",
            Self::Json(_) => "JSON",
            Self::Task(_) => "TASK",
            Self::Handler(_) => "HANDLER",
            Self::Plugin(_) => "PLUGIN",
            Self::Security(_) => "SECURITY",
            Self::Resource(_) => "RESOURCE",
            Self::Validation(_) => "VALIDATION",
            Self::Lifecycle(_) => "LIFECYCLE",
            Self::Tool(_) => "TOOL",
            Self::WireFormat(_) => "WIRE_FORMAT",
            Self::NotInitialized(_) => "NOT_INITIALIZED",
            Self::General(_) => "GENERAL",
            Self::AlreadyInProgress(_) => "ALREADY_IN_PROGRESS",
            Self::Monitoring(_) => "MONITORING",
            Self::NotConnected(_) => "NOT_CONNECTED",
            Self::Timeout(_) => "TIMEOUT",
            Self::Remote(_) => "REMOTE",
            Self::Unexpected(_) => "UNEXPECTED",
            Self::VersionMismatch(_) => "VERSION_MISMATCH",
            Self::Unsupported(_) => "UNSUPPORTED",
            Self::NotImplemented(_) => "NOT_IMPLEMENTED",
            Self::NotAuthorized(_) => "NOT_AUTHORIZED",
            Self::InvalidState(_) => "INVALID_STATE",
            Self::InvalidOperation(_) => "INVALID_OPERATION",
            Self::InternalError(_) => "INTERNAL_ERROR",
            Self::Sync(_) => "SYNC",
            Self::AlreadyExists(_) => "ALREADY_EXISTS",
            Self::InvalidRequest(_) => "INVALID_REQUEST",
            Self::Database(_) => "DATABASE",
            Self::OperationFailed(_) => "OPERATION_FAILED",
            Self::Generic(_) => "GENERIC",
            Self::Configuration(_) => "CONFIGURATION",
            Self::Alert(_) => "ALERT",
            Self::Network(_) => "NETWORK",
            Self::InvalidArgument(_) => "INVALID_ARGUMENT",
            Self::NotFound(_) => "NOT_FOUND",
            Self::ResourceExhausted(_) => "RESOURCE_EXHAUSTED",
            Self::Internal(_) => "INTERNAL",
            Self::Authentication(_) => "AUTHENTICATION",
            Self::RateLimit(_) => "RATE_LIMIT",
            Self::Registry(_) => "REGISTRY",
            Self::Config(_) => "CONFIG",
            Self::Integration(_) => "INTEGRATION",
            Self::Port(_) => "PORT",
            Self::RBAC(_) => "RBAC",
        }
    }
}
