//! # Capability-Based Discovery System
//!
//! **Philosophy**: Zero hardcoding through pure capability-based discovery
//!
//! Following Songbird's proven patterns for runtime service discovery.
//!
//! ## Core Principles
//! 1. **Self-Knowledge Only**: Each primal knows only itself
//! 2. **Runtime Discovery**: All external services discovered at runtime
//! 3. **Capability-Based**: Request by capability, not by name
//! 4. **Zero Hardcoding**: No primal names, vendor names, or endpoints in code
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────┐
//! │   Application   │
//! │  "I need AI"    │
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────────────┐
//! │ Capability Resolver     │
//! │  discovers providers... │
//! └────────┬────────────────┘
//!          │
//!   ┌──────┼──────┬──────┬─────────┐
//!   │      │      │      │         │
//!   ▼      ▼      ▼      ▼         ▼
//! ENV   mDNS  DNS-SD  Registry  P2P
//! Vars  (local) (auto) (central) (peer)
//! ```
//!
//! ## Usage
//!
//! ### Instead of Hardcoding:
//! ```rust,ignore
//! // ❌ OLD WAY: Hardcoded endpoint
//! let client = SquirrelClient::connect("http://localhost:9200")?;
//! ```
//!
//! ### Use Capability-Based Discovery:
//! ```rust,ignore
//! use squirrel::discovery::{RuntimeDiscoveryEngine, CapabilityRequest};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // ✅ NEW WAY: Discover by capability at runtime
//! let discovery = RuntimeDiscoveryEngine::new();
//! let provider = discovery.discover_capability("ai").await?;
//!
//! // Use whatever provider was discovered
//! println!("Found AI provider: {} at {}", provider.name, provider.endpoint);
//! # Ok(())
//! # }
//! ```

pub mod capability_resolver;
pub mod mechanisms;
pub mod runtime_engine;
pub mod self_knowledge;
pub mod types;

#[cfg(test)]
mod capability_resolver_tests;

// Re-exports
pub use capability_resolver::CapabilityResolver;
pub use runtime_engine::RuntimeDiscoveryEngine;
pub use self_knowledge::PrimalSelfKnowledge;
pub use types::{CapabilityRequest, DiscoveredService, DiscoveryError, DiscoveryResult};
