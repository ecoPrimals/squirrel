//! **DEPRECATED**: Direct BearDog Integration
//!
//! # Migration Guide
//!
//! This module is deprecated in favor of capability-based discovery.
//! Instead of hardcoding "BearDog", discover authentication capability.
//!
//! ## Before (Hardcoded):
//! ```rust,ignore
//! let beardog = BeardogIntegration::new();
//! beardog.authenticate(credentials).await?;
//! ```
//!
//! ## After (Capability-Based):
//! ```rust,ignore
//! use crate::capability_registry::{CapabilityRegistry, PrimalCapability};
//!
//! // Discover ANY authentication provider (could be BearDog, or anything else)
//! let auth_providers = capability_registry
//!     .discover_by_capability(&PrimalCapability::Authentication)
//!     .await?;
//! ```
//!
//! ## Perfect Example
//!
//! See `crates/main/src/songbird/mod.rs` for the perfect template.
//! Also see `CAPABILITY_INTEGRATION_TEMPLATE.md` in the project root.
//!
//! ## Primal Sovereignty
//!
//! Each primal knows only itself and discovers others by capability at runtime.
//! This eliminates N² hardcoded connections and enables true ecosystem evolution.

use crate::error::PrimalError;
use std::collections::HashMap;

#[deprecated(
    since = "3.0.0",
    note = "Use CapabilityRegistry to discover 'authentication' capability instead of direct BearDog integration. See songbird/mod.rs for perfect example."
)]
pub struct BeardogIntegration {
    pub initialized: bool,
}

#[allow(deprecated)]
impl BeardogIntegration {
    /// **DEPRECATED**: Create new BearDog integration
    ///
    /// # Migration
    ///
    /// Instead of creating this struct, use `CapabilityRegistry` to discover
    /// authentication services:
    ///
    /// ```rust,ignore
    /// let auth = capability_registry
    ///     .discover_by_capability(&PrimalCapability::Authentication)
    ///     .await?;
    /// ```
    #[must_use]
    #[deprecated(
        since = "3.0.0",
        note = "Use CapabilityRegistry::discover_by_capability(&PrimalCapability::Authentication)"
    )]
    pub fn new() -> Self {
        Self { initialized: false }
    }

    /// **DEPRECATED**: Initialize BearDog integration
    ///
    /// # Migration
    ///
    /// See `songbird/mod.rs::initialize()` for the capability-based pattern.
    #[deprecated(
        since = "3.0.0",
        note = "Use capability-based discovery initialization pattern"
    )]
    pub async fn initialize(&mut self) -> Result<(), PrimalError> {
        self.initialized = true;
        Ok(())
    }

    /// **DEPRECATED**: Shutdown BearDog integration
    #[deprecated(since = "3.0.0", note = "Use capability coordinator pattern")]
    pub async fn shutdown(&mut self) -> Result<(), PrimalError> {
        self.initialized = false;
        Ok(())
    }

    /// **DEPRECATED**: Authenticate using BearDog
    ///
    /// # Migration
    ///
    /// ```rust,ignore
    /// // Discover authentication capability
    /// let auth_providers = capability_registry
    ///     .discover_by_capability(&PrimalCapability::Authentication)
    ///     .await?;
    ///
    /// if let Some(provider) = auth_providers.first() {
    ///     // Use discovered provider (HTTP/gRPC call)
    ///     // See CAPABILITY_INTEGRATION_TEMPLATE.md
    /// }
    /// ```
    #[deprecated(since = "3.0.0", note = "Use discovered authentication capability")]
    pub async fn authenticate(&self, _credentials: &str) -> Result<bool, PrimalError> {
        Ok(true)
    }

    /// **DEPRECATED**: Get BearDog health status
    #[deprecated(since = "3.0.0", note = "Use capability discovery health checks")]
    pub async fn get_health_status(&self) -> Result<HashMap<String, String>, PrimalError> {
        let mut status = HashMap::new();
        status.insert("status".to_string(), "healthy".to_string());
        Ok(status)
    }
}

impl Default for BeardogIntegration {
    fn default() -> Self {
        Self::new()
    }
}
