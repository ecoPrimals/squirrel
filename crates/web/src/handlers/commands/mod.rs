//! Commands module for handling command execution API endpoints
//!
//! This module contains handlers for the command execution API endpoints.

pub mod service;

// Re-export the service, conditionally re-export DbCommandService
pub use service::CommandService;
#[cfg(feature = "db")]
pub use service::DbCommandService;
pub use service::MockCommandService;

mod routes;

pub use routes::command_routes; 