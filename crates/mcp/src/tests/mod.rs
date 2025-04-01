//! MCP Tests module
//!
//! This module contains tests for the MCP system.

#[cfg(all(test, feature = "di-tests"))]
pub mod adapter_tests;

#[cfg(test)]
mod security_tests {
    use crate::security::rbac::MockRBACManager;
    use crate::security::rbac::unified::RBACManager;

    #[tokio::test]
    async fn test_security_module() {
        // Simple verification test for security module
        let manager = MockRBACManager::new(false); // false = do not allow all operations
        
        // Test the trait method to get the name
        let name = manager.name();
        assert_eq!(name, "MockRBACManager");
    }
}
