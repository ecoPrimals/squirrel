//! Squirrel Universal AI Primal
//!
//! A universal AI coordination primal that implements the standardized ecosystem
//! patterns for dynamic primal evolution and integration with the ecoPrimals ecosystem.
//!
//! This primal follows the universal adapter patterns defined by Songbird and
//! implements the EcosystemServiceRegistration standard for seamless integration.

#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

// Core modules
pub mod biomeos_integration;
pub mod client;
pub mod communication;
pub mod core;
pub mod enhanced;
pub mod error;
pub mod infrastructure;
pub mod integration;
pub mod protocol;
pub mod session;
pub mod songbird;
pub mod tool;
pub mod transport;
pub mod web_integration;

// Universal adapter modules
pub mod ecosystem;
pub mod primal_provider;
pub mod universal;

// Specific re-exports to avoid ambiguity
pub use biomeos_integration::{
    AiIntelligence, ContextState, CoordinationSession as BiomeOSCoordinationSession,
    EcosystemClient, EcosystemServiceRegistration, HealthCheckConfig as BiomeOSHealthCheckConfig,
    HealthStatus as BiomeOSHealthStatus, McpIntegration, PrimalStatus as BiomeOSPrimalStatus,
    ResourceAllocation as BiomeOSResourceAllocation, SquirrelBiomeOSIntegration,
};

pub use songbird::{
    CoordinationSession as SongbirdCoordinationSession, HealthStatus as SongbirdHealthStatus,
    PrimalStatus as SongbirdPrimalStatus, ResourceAllocation as SongbirdResourceAllocation,
    SongbirdIntegration,
};

pub use error::PrimalError;
pub use protocol::types as protocol_types;

pub use ecosystem::*;
pub use primal_provider::*;
pub use universal::*;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build information for the SDK
#[derive(Debug, Clone)]
pub struct BuildInfo {
    pub version: String,
    pub build_time: String,
    pub git_hash: String,
    pub target: String,
    pub profile: String,
}

impl Default for BuildInfo {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_time: "unknown".to_string(),
            git_hash: "unknown".to_string(),
            target: std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string()),
            profile: std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string()),
        }
    }
}
