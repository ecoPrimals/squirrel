//! Discovery mechanism implementations
//!
//! Each module implements a different discovery mechanism:
//! - Environment variables (highest priority)
//! - mDNS (local network)
//! - DNS-SD (network-wide)
//! - Service registry (central coordination)

pub mod dnssd;
pub mod env_vars;
pub mod mdns;
pub mod registry;
pub mod registry_trait;

// Re-export for convenience
pub use dnssd::DnssdDiscovery;
pub use env_vars::{discover_all_from_env, discover_from_env};
pub use mdns::MdnsDiscovery;
#[allow(deprecated)]
pub use registry::{RegistryDiscovery, RegistryType};
pub use registry_trait::{auto_detect_registry, ServiceRegistryProvider};
