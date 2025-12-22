//! Capability-Based Architecture
//!
//! This module provides capability discovery and registration for the Squirrel ecosystem.

pub mod discovery;

pub use discovery::{
    Capability, CapabilityDiscovery, DiscoveredEndpoint, DiscoveryConfig, DiscoveryError,
};
