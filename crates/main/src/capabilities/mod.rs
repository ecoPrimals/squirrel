//! # 🎯 Capability Traits - Universal Primal Interface
//!
//! This module defines capability-based traits that replace hardcoded primal dependencies.
//!
//! ## Philosophy
//!
//! Instead of depending on specific primals (Songbird, BearDog, Toadstool), we define
//! **capabilities** that any primal can provide. Discovery happens at runtime.
//!
//! ```rust,ignore
//! // ❌ OLD: Hardcoded primal dependency
//! let songbird = SongbirdClient::connect("http://localhost:9090").await?;
//! let response = songbird.infer(prompt).await?;
//!
//! // ✅ NEW: Capability-based discovery
//! let ai_provider = adapter.connect_capability("ai.inference").await?;
//! let response = ai_provider.execute_capability(request).await?;
//! ```

pub mod ai;
pub mod compute;
pub mod federation;
pub mod monitoring;
pub mod security;
pub mod storage;

// Re-export common capability traits
pub use ai::{AiInferenceCapability, EmbeddingsCapability};
pub use compute::ComputeCapability;
pub use federation::FederationCapability;
pub use monitoring::MonitoringCapability;
pub use security::{AuthenticationCapability, AuthorizationCapability};
pub use storage::StorageCapability;
