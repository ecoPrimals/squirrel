//! Capability-based service discovery and interaction
//!
//! TRUE PRIMAL pattern: Discover capabilities at runtime, NO hardcoded primal names.
//!
//! # Philosophy
//!
//! Each primal knows only itself and discovers other services by their capabilities,
//! not by their names. This enables:
//! - Dynamic ecosystem composition
//! - Zero vendor lock-in
//! - Deployment with zero knowledge (infant pattern)
//!
//! # Example
//!
//! ```no_run
//! use squirrel::capabilities::discover_capability;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Discover who provides crypto signing (don't care WHO, just WHAT)
//! let crypto = discover_capability("crypto.signing").await?;
//!
//! // Use the capability (via the discovered provider)
//! // We have NO IDEA if this is BearDog, or something else - we don't care!
//! # Ok(())
//! # }
//! ```

pub mod discovery;

// Re-exports for convenience
pub use discovery::{
    discover_all_capabilities, discover_capability, CapabilityProvider, DiscoveryError,
};
