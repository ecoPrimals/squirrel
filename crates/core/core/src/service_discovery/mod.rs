// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Service Discovery System
//!
//! This module implements a universal service discovery system that replaces
//! hardcoded primal endpoints with dynamic discovery and registration.
//!
//! ## Features
//!
//! - Dynamic service registration and discovery
//! - Health monitoring with configurable heartbeat intervals
//! - Capability-based service queries
//! - Service metadata and tags
//! - Automatic service cleanup on expiration
//! - Load balancing support
//! - Service versioning and compatibility checks
//!
//! ## Architecture
//!
//! The service discovery system is organized into several modules:
//!
//! - `types`: Core data structures and types
//! - `trait_`: Service discovery trait definition
//! - `memory`: In-memory implementation
//! - `client`: High-level client interface
//! - `registry`: Service lifecycle management
//! - `tests`: Comprehensive test suite
//!
//! ## Usage
//!
//! ```rust
//! use std::sync::Arc;
//! use squirrel_core::{CoreResult, InMemoryServiceDiscovery, ServiceDiscovery, ServiceDefinition, ServiceType};
//!
//! # async fn example() -> CoreResult<()> {
//! let discovery = Arc::new(InMemoryServiceDiscovery::new());
//!
//! // Register a service
//! let service = ServiceDefinition::new(
//!     "ai-service-1".to_string(),
//!     "AI Service".to_string(),
//!     ServiceType::AI,
//!     vec![],
//! );
//!
//! discovery.register_service(service).await?;
//!
//! // Query services
//! let services = discovery.get_active_services().await?;
//! println!("Found {} active services", services.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Examples
//!
//! ### Basic Service Registration
//!
//! ```rust
//! use squirrel_core::{CoreResult, InMemoryServiceDiscovery, ServiceDefinition, ServiceDiscovery, ServiceEndpoint, ServiceType};
//!
//! # async fn example() -> CoreResult<()> {
//! let discovery = InMemoryServiceDiscovery::new();
//!
//! let endpoint = ServiceEndpoint::new(
//!     "http://localhost:8080".to_string(),
//!     "http".to_string(),
//!     8080,
//! );
//!
//! let service = ServiceDefinition::new(
//!     "my-service".to_string(),
//!     "My Service".to_string(),
//!     ServiceType::AI,
//!     vec![endpoint],
//! )
//! .with_capability("chat".to_string())
//! .with_metadata("region".to_string(), "us-east-1".to_string());
//!
//! discovery.register_service(service).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Service Discovery with Query
//!
//! ```rust
//! use squirrel_core::service_discovery::ServiceHealthStatus;
//! use squirrel_core::{CoreResult, InMemoryServiceDiscovery, ServiceDiscovery, ServiceQuery, ServiceType};
//!
//! # async fn example() -> CoreResult<()> {
//! let discovery = InMemoryServiceDiscovery::new();
//!
//! let query = ServiceQuery::new()
//!     .with_service_type(ServiceType::AI)
//!     .with_capability("chat".to_string())
//!     .with_health_status(ServiceHealthStatus::Healthy)
//!     .limit(10);
//!
//! let services: Vec<_> = discovery.discover_services(query).await?;
//! println!("Found {} matching services", services.len());
//! # Ok(())
//! # }
//! ```
//!
//! ### Service Registry with Heartbeat
//!
//! ```rust
//! use std::sync::Arc;
//! use squirrel_core::{CoreResult, InMemoryServiceDiscovery, ServiceDefinition, ServiceRegistry, ServiceType};
//!
//! # async fn example() -> CoreResult<()> {
//! let discovery = Arc::new(InMemoryServiceDiscovery::new());
//! let registry = ServiceRegistry::new(discovery);
//!
//! let service = ServiceDefinition::new(
//!     "my-service".to_string(),
//!     "My Service".to_string(),
//!     ServiceType::AI,
//!     vec![],
//! );
//!
//! registry.register_local_service(service).await?;
//! registry.start_heartbeat_loop().await?;
//!
//! // Service will automatically send heartbeats
//! # Ok(())
//! # }
//! ```

// Module declarations
pub mod client;
pub mod memory;
pub mod registry;
pub mod trait_;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export the main types and traits for convenience
pub use client::*;
pub use memory::*;
pub use registry::*;
pub use trait_::*;
pub use types::*;

// Re-export commonly used external types
pub use async_trait::async_trait;
pub use std::sync::Arc;
