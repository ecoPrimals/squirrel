// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Capability-Based Discovery Constants
//!
//! **Philosophy**: TRUE PRIMAL - Primals discover others by capability at runtime,
//! not by hardcoded primal names. Use these constants when referring to OTHER
//! primals' capabilities.
//!
//! # Usage
//!
//! ```ignore
//! use universal_constants::capabilities;
//!
//! // Discover security provider (capability-based, not "beardog")
//! let security = discover_capability(capabilities::SECURITY_CAPABILITY).await?;
//!
//! // Discover service mesh (capability-based, not "songbird")
//! let mesh = discover_capability(capabilities::SERVICE_MESH_CAPABILITY).await?;
//! ```
//!
//! # Self Identity
//!
//! The squirrel primal's own identity - use `SELF_PRIMAL_NAME` only when this
//! primal refers to itself (CLI name, binary name, self-identification).

/// This primal's own identity (squirrel)
///
/// Use ONLY when referring to THIS primal - CLI name, binary name, self-identification.
/// Never use for discovering other primals.
pub const SELF_PRIMAL_NAME: &str = "squirrel";

/// Security capability (formerly beardog)
///
/// Discover providers with: `discover_capability(SECURITY_CAPABILITY)`
pub const SECURITY_CAPABILITY: &str = "security";

/// Service mesh / orchestration capability (formerly songbird)
///
/// Discover providers with: `discover_capability(SERVICE_MESH_CAPABILITY)`
pub const SERVICE_MESH_CAPABILITY: &str = "service-mesh";

/// Storage capability (formerly nestgate)
///
/// Discover providers with: `discover_capability(STORAGE_CAPABILITY)`
pub const STORAGE_CAPABILITY: &str = "storage";

/// Compute capability (formerly toadstool)
///
/// Discover providers with: `discover_capability(COMPUTE_CAPABILITY)`
pub const COMPUTE_CAPABILITY: &str = "compute";

/// Ecosystem / platform capability (formerly biomeOS)
///
/// Discover providers with: `discover_capability(ECOSYSTEM_CAPABILITY)`
pub const ECOSYSTEM_CAPABILITY: &str = "ecosystem";

/// Network capability (alias for service mesh)
pub const NETWORK_CAPABILITY: &str = "network";
