//! MCP Tests module
//!
//! This module contains tests for the MCP system.

#[cfg(all(test, feature = "di-tests"))]
pub mod adapter_tests;

#[cfg(test)]
mod security_tests {
    use crate::security::rbac::RBACManager;

    #[test]
    fn test_security_module() {
        // Simple verification test for security module
        let manager = RBACManager::new();
        assert!(manager.get_role("admin").is_none());
    }
}
