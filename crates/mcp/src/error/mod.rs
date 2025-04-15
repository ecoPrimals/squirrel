// Error module definitions
// Re-exports all error-related modules

// Export the protocol error module
pub mod protocol_err;

// Export other error modules
pub mod types;
pub mod context;
pub mod transport;
pub mod port;
pub mod rbac;
pub mod security_err;
pub mod session;
pub mod tool;
pub mod alert;
pub mod config;
pub mod connection;
pub mod context_err;
pub mod handler;
pub mod plugin;
pub mod client;

// Re-export the main Error and ProtocolError types
pub use crate::error::types::*;
pub use crate::error::protocol_err::ProtocolError;
pub use crate::error::transport::TransportError;
pub use crate::error::client::ClientError;
pub use crate::error::session::SessionError;
pub use crate::error::security_err::SecurityError;
pub use crate::error::rbac::RBACError;

// Export test utilities
#[cfg(test)]
pub mod tests; 