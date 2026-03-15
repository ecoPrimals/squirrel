// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

/// Tool lifecycle management system (refactored)
/// 
/// This module provides enhanced lifecycle management capabilities for tools,
/// including state transitions, validation, and observability.

pub mod basic;
pub mod recovery;
// Security handled by BearDog framework
pub mod composite;
pub mod testing;

// Re-export public types and components for backward compatibility
pub use basic::BasicLifecycleHook;
pub use recovery::RecoveryLifecycleHook;
pub use composite::CompositeLifecycleHook; 