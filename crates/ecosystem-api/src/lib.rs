//! Ecosystem API - Shared types and traits for ecoPrimals ecosystem integration
//!
//! This crate provides the standardized API types and traits that all primals
//! in the ecoPrimals ecosystem must implement for seamless integration through
//! the Songbird service mesh.
//!
//! ## Key Traits
//!
//! - [`EcosystemIntegration`] - Core trait for ecosystem communication
//! - [`UniversalPrimalProvider`] - Universal primal provider interface
//! - [`ServiceMeshClient`] - Client for Songbird service mesh
//!
//! ## Core Types
//!
//! - [`EcosystemRequest`]/[`EcosystemResponse`] - Standard request/response format
//! - [`PrimalCapability`] - Capability system for all primals
//! - [`UniversalConfig`] - Configuration management
//!
//! ## Usage
//!

#![deny(unsafe_code)] // ✅ ENFORCED: No unsafe code allowed
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
//! ```rust
//! use ecosystem_api::*;
//! use async_trait::async_trait;
//!
//! struct MyPrimal {
//!     config: UniversalConfig,
//! }
//!
//! #[async_trait]
//! impl UniversalPrimalProvider for MyPrimal {
//!     fn primal_type(&self) -> PrimalType {
//!         PrimalType::Squirrel
//!     }
//!
//!     async fn handle_primal_request(&self, request: PrimalRequest) -> UniversalResult<PrimalResponse> {
//!         // Handle request
//!         Ok(PrimalResponse::default())
//!     }
//!
//!     // ... other required methods
//! }
//! ```

#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
#![warn(missing_docs)]

pub mod client;
pub mod config;
pub mod defaults;
pub mod error;
pub mod traits;
pub mod types;

// Re-export commonly used types and traits
pub use client::*;
pub use config::*;
pub use error::*;
pub use traits::*;
pub use types::*;

/// Ecosystem API version
pub const ECOSYSTEM_API_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the ecosystem API
///
/// This function sets up logging and validates the API configuration.
/// It should be called once at application startup.
pub fn init() -> Result<(), EcosystemError> {
    tracing::info!("Initializing Ecosystem API v{}", ECOSYSTEM_API_VERSION);
    Ok(())
}
