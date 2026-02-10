// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Legacy comprehensive cleanup module.
//!
//! This module provides backward compatibility for the comprehensive cleanup system.
//! The actual implementation has been moved to the `comprehensive` module with
//! better organization and structure.
//!
//! For new code, please use the `comprehensive` module directly:
//! ```rust
//! use crate::tool::cleanup::comprehensive::ComprehensiveCleanupHook;
//! ```
//!
//! This module will be deprecated in future versions.

// Re-export everything from the new comprehensive module for backward compatibility
pub use super::comprehensive::*;

// Legacy alias for the main hook
pub use super::comprehensive::ComprehensiveCleanupHook;

#[deprecated(note = "Use comprehensive::ComprehensiveCleanupHook instead")]
pub type LegacyComprehensiveCleanupHook = ComprehensiveCleanupHook;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_legacy_comprehensive_cleanup_hook() {
        // Test that the legacy interface still works
        let hook = ComprehensiveCleanupHook::new();
        
        // Register a resource
        let resource_id = hook
            .register_resource(
                "test-tool",
                ResourceType::Memory,
                "test-memory",
                1024,
                HashMap::new(),
            )
            .await;
        
        // Check if resource exists
        let resources = hook.get_active_resources("test-tool").await;
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].id.name, "test-memory");
        assert_eq!(resources[0].id.resource_type, ResourceType::Memory);
        assert_eq!(resources[0].size, 1024);
        assert!(resources[0].is_active);
        
        // Deactivate the resource
        hook.deactivate_resource(&resource_id).await.unwrap();
        
        // Should now have no active resources
        let active = hook.get_active_resources("test-tool").await;
        assert_eq!(active.len(), 0);
    }
} 