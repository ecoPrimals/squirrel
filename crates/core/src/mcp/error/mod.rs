pub mod types;
pub mod context;

// Re-export specific types from types module
pub use types::{
    MCPError,
    ProtocolError,
    ConnectionError,
    SecurityError,
    PortErrorKind,
    MessageErrorKind,
    ToolErrorKind,
    ErrorContext,
};

// Re-export specific types from context module
pub use context::{
    ErrorHandlerError,
    ErrorSeverity,
    RecoveryStrategy,
    ErrorRecord,
    ErrorHandler,
};

pub type Result<T> = std::result::Result<T, MCPError>; 