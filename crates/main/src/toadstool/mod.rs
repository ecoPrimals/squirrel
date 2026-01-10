//! `ToadStool` Integration for Squirrel AI Primal
//!
//! This module provides integration with the `ToadStool` compute primal for
//! intensive AI operations, distributed computing, and resource management.
//!
//! ## Module Organization
//!
//! - **`config`**: Configuration for `ToadStool` integration
//! - **`state`**: Compute state management
//! - **`job`**: Job types and specifications
//! - **`resource`**: Resource allocation and tracking
//! - **`node`**: Compute node management
//! - **`health`**: Health monitoring
//! - **`messages`**: Request/response types
//! - **`integration`**: Main integration implementation

// Public modules
pub mod config;
pub mod health;
pub mod integration;
pub mod job;
pub mod messages;
pub mod node;
pub mod resource;
pub mod state;

// Re-export commonly used types
pub use config::{ResourceLimits, ToadStoolConfig};
pub use health::HealthStatus;
pub use integration::ToadStoolIntegration;
pub use job::{
    ComputeJob, ComputeJobType, JobPriority, JobStatus, QueuedJob, ResourceRequirements,
};
pub use messages::{ComputeJobRequest, ComputeJobResponse, JobResultResponse};
pub use node::{ComputeNode, NodeHealth, NodeType};
pub use resource::{AllocatedResources, AllocationStatus, ResourceAllocation, ResourceUsage};
pub use state::ComputeState;
