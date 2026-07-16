// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Security module for MCP
//!
//! Provides security management, RBAC, audit, and related functionality.

pub mod audit;
#[cfg(feature = "local-crypto")]
pub mod crypto;
pub mod identity;
pub mod key_storage;
pub mod manager;
pub mod platform_secret_store;
pub mod rbac;
pub mod secret_store;
pub mod token;

pub use key_storage::KeyStorage;
pub use manager::SecurityManagerImpl;
pub use platform_secret_store::PlatformSecretStore;
pub use rbac::Permission;
pub use secret_store::SecretStore;
