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
    ResourceExhausted(String),

    /// Invalid argument error
    InvalidArgument(String),

    /// Not found error
    NotFound(String),

    /// Internal server error
    Internal(String),

    /// Authentication error
    Authentication(String),

    /// Authorization error
    Authorization(String),

    /// Rate limit exceeded
    RateLimit(String),

    /// Timeout error
    Timeout(String),

    /// Configuration error
    Configuration(String),

    /// Validation error
    Validation(String),

    /// Invalid state error
    InvalidState(String),

    /// Invalid operation error
    InvalidOperation(String),

    /// Network error
    Network(String),

    /// IO error
    Io(String),

    /// JSON parsing error
    Json(String),

    /// General error
    General(String),

    /// Generic error (alias for General)
    Generic(String),

    /// Message router error
    MessageRouter(String),

    /// Serialization error
    Serialization(String),

    /// Deserialization error
    Deserialization(String),

    /// Invalid message error
    InvalidMessage(String),

    /// State error
    State(String),

    /// Unsupported operation error
    UnsupportedOperation(String),

    /// Circuit breaker error
    CircuitBreaker(String),

    /// Security error
    Security(String),

    /// Resource error
    Resource(String),

    /// Lifecycle error
    Lifecycle(String),

    /// Wire format error
    WireFormat(String),

    /// Not initialized error
    NotInitialized(String),

    /// Already in progress error
    AlreadyInProgress(String),

    /// Monitoring error
    Monitoring(String),

    /// Not connected error
    NotConnected(String),

    /// Remote error
    Remote(String),

    /// Unexpected error
    Unexpected(String),

    /// Version mismatch error
    VersionMismatch(String),

    /// Unsupported error
    Unsupported(String),

    /// Not implemented error
    NotImplemented(String),

    /// Not authorized error
    NotAuthorized(String),

    /// Internal error
    InternalError(String),

    /// Sync error
    Sync(String),

    /// Already exists error
    AlreadyExists(String),

    /// Invalid request error
    InvalidRequest(String),

    /// Database error
    Database(String),

    /// Operation failed error
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
            ) => ErrorSeverity::High,

            Self::Protocol(ProtocolError::InvalidVersion(_)) => ErrorSeverity::High,
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
    pub fn code_str(&self) -> &'static str {
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
    pub fn category_str(&self) -> &'static str {
        match self {
            MCPError::Transport(_) => "TRANSPORT",
            MCPError::Protocol(_) => "PROTOCOL",
            MCPError::Connection(_) => "CONNECTION",
            MCPError::Session(_) => "SESSION",
            MCPError::Context(_) => "CONTEXT",
            MCPError::Client(_) => "CLIENT",
            MCPError::MessageRouter(_) => "MESSAGE_ROUTER",
            MCPError::Serialization(_) => "SERIALIZATION",
            MCPError::Deserialization(_) => "DESERIALIZATION",
            MCPError::InvalidMessage(_) => "INVALID_MESSAGE",
            MCPError::State(_) => "STATE",
            MCPError::Authorization(_) => "AUTHORIZATION",
            MCPError::UnsupportedOperation(_) => "UNSUPPORTED_OPERATION",
            MCPError::CircuitBreaker(_) => "CIRCUIT_BREAKER",
            MCPError::Io(_) => "IO",
            MCPError::Json(_) => "JSON",
            MCPError::Task(_) => "TASK",
            MCPError::Handler(_) => "HANDLER",
            MCPError::Plugin(_) => "PLUGIN",
            MCPError::Security(_) => "SECURITY",
            MCPError::Resource(_) => "RESOURCE",
            MCPError::Validation(_) => "VALIDATION",
            MCPError::Lifecycle(_) => "LIFECYCLE",
            MCPError::Tool(_) => "TOOL",
            MCPError::WireFormat(_) => "WIRE_FORMAT",
            MCPError::NotInitialized(_) => "NOT_INITIALIZED",
            MCPError::General(_) => "GENERAL",
            MCPError::AlreadyInProgress(_) => "ALREADY_IN_PROGRESS",
            MCPError::Monitoring(_) => "MONITORING",
            MCPError::NotConnected(_) => "NOT_CONNECTED",
            MCPError::Timeout(_) => "TIMEOUT",
            MCPError::Remote(_) => "REMOTE",
            MCPError::Unexpected(_) => "UNEXPECTED",
            MCPError::VersionMismatch(_) => "VERSION_MISMATCH",
            MCPError::Unsupported(_) => "UNSUPPORTED",
            MCPError::NotImplemented(_) => "NOT_IMPLEMENTED",
            MCPError::NotAuthorized(_) => "NOT_AUTHORIZED",
            MCPError::InvalidState(_) => "INVALID_STATE",
            MCPError::InvalidOperation(_) => "INVALID_OPERATION",
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

// Add the Display trait implementation for MCPError
impl std::fmt::Display for MCPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(e) => write!(f, "Transport error: {e}"),
            Self::Protocol(e) => write!(f, "Protocol error: {e}"),
            Self::Connection(e) => write!(f, "Connection error: {e}"),
            Self::Session(e) => write!(f, "Session error: {e}"),
            Self::Context(e) => write!(f, "Context error: {e}"),
            Self::Client(e) => write!(f, "Client error: {e}"),
            Self::Alert(e) => write!(f, "Alert error: {e}"),
            _ => write!(f, "{}", self.category_str()),
        }
    }
}
