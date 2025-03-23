//! Commands module for handling command execution API endpoints
//!
//! This module contains handlers for the command execution API endpoints.

pub mod models;
pub mod routes;
pub mod service;

// Remove the missing module references
// pub mod v1;

// Import types directly from API module instead
use crate::api::commands::{
    CommandDefinition,
    CommandExecution,
    CommandStatus,
};

pub use routes::command_routes; 