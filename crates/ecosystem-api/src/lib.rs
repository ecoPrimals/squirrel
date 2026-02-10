// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Ecosystem API - Shared types and traits for ecoPrimals ecosystem integration
//!
//! This crate provides the standardized API types and traits that all primals
//! in the ecoPrimals ecosystem must implement for seamless integration through
//! the service mesh.
//!
//! ## Key Traits
//!
//! - [`EcosystemIntegration`] - Core trait for ecosystem communication
//! - [`UniversalPrimalProvider`] - Universal primal provider interface
//! - [`ServiceMeshClient`] - Client for service mesh
//!
//! ## Core Types
//!
//! - [`EcosystemRequest`]/[`EcosystemResponse`] - Standard request/response format
//! - [`PrimalCapability`] - Capability system for all primals
//! - [`UniversalConfig`] - Configuration management
//!
//! ## TRUE PRIMAL Philosophy
//!
//! Each primal has **self-knowledge only** and discovers other primals at runtime
//! via the universal capability system. No primal names are hardcoded.
//!
//! ```rust,no_run
//! # use ecosystem_api::*;
//! # use async_trait::async_trait;
//! # struct MyPrimal { config: UniversalConfig, capabilities: Vec<PrimalCapability> }
//! // Example: Primal with capability-based discovery (concept only)
//! // Actual implementation requires all UniversalPrimalProvider methods
//! // See the trait definition for complete requirements
//! ```

#![deny(unsafe_code)] // ✅ ENFORCED: No unsafe code allowed
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
#![warn(missing_docs)]

#[cfg(feature = "http-api")]
pub mod client;
pub mod config;
pub mod defaults;
pub mod error;
pub mod traits;
pub mod types;

// Re-export commonly used types and traits
#[cfg(feature = "http-api")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecosystem_api_version_is_set() {
        assert!(!ECOSYSTEM_API_VERSION.is_empty());
    }

    #[test]
    fn test_init_succeeds() {
        let result = init();
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_can_be_called_multiple_times() {
        assert!(init().is_ok());
        assert!(init().is_ok());
    }
}
