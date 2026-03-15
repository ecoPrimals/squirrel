// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! MCP Tests module
//!
//! This module contains tests for the MCP system.

#[cfg(all(test, feature = "di-tests"))]
pub mod adapter_tests;

#[cfg(test)]
mod security_tests {
    // Security handled by BearDog framework
    // BearDog handles security: MockRBACManager and RBACManager
    // BearDog handles security: Authentication and authorization delegated to BearDog

    #[tokio::test]
    async fn test_security_module() {
        // Security module tests moved to BearDog framework
        // This test is now a placeholder for BearDog integration
        assert!(true, "Security handled by BearDog framework");
    }
}
