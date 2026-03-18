// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Security module for MCP
//!
//! Provides security management, RBAC, audit, and related functionality.

pub mod audit;
pub mod crypto;
pub mod identity;
pub mod key_storage;
pub mod manager;
pub mod rbac;
pub mod token;

pub use manager::SecurityManagerImpl;
pub use rbac::Permission;
