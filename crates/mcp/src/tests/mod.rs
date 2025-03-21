//! MCP Tests module
//!
//! This module contains tests for the MCP system.

// Only include our DI adapter tests in di-tests mode
#[cfg(all(test, feature = "di-tests"))]
pub mod adapter_tests;

#[cfg(test)]
mod security_tests {
    use crate::security::*;
    
    #[test]
    fn test_security_module() {
        // Simple verification test for security module
        let manager = RBACManager::new();
        assert!(manager.get_role("admin").is_none());
    }
} 