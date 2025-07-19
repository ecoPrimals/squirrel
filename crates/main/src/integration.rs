//! Integration module for backward compatibility
//!
//! This module provides backward compatibility for test imports
//! by re-exporting the McpIntegration as SimpleMCPIntegration.

pub use crate::biomeos_integration::McpIntegration as SimpleMCPIntegration;
