/*!
 * Tests for the Galaxy adapter.
 * 
 * This module contains tests for the Galaxy adapter crate.
 */

// Import test modules
pub mod security_tests;

#[cfg(test)]
pub mod common {
    use crate::config::GalaxyConfig;
    use crate::security::{SecretString, SecureCredentials};
    
    /// Creates a test configuration for Galaxy
    pub fn create_test_config() -> GalaxyConfig {
        GalaxyConfig::new("http://localhost:8080/api")
            .with_api_key("test-api-key")
            .with_debug(true)
    }
    
    /// Creates test credentials for Galaxy
    pub fn create_test_credentials() -> SecureCredentials {
        SecureCredentials::with_api_key(SecretString::new("test-api-key"))
    }
} 