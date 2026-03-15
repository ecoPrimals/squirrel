// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Integration Test Framework Helpers
//!
//! Provides utilities for setting up and running integration tests.

use super::*;

/// Builder for integration test environments
pub struct TestEnvironmentBuilder {
    test_name: String,
    config: TestConfig,
    services_to_start: Vec<ServiceType>,
}

impl TestEnvironmentBuilder {
    pub fn new(test_name: impl Into<String>) -> Self {
        Self {
            test_name: test_name.into(),
            config: TestConfig::default(),
            services_to_start: Vec::new(),
        }
    }
    
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }
    
    pub fn verbose(mut self) -> Self {
        self.config.verbose = true;
        self
    }
    
    pub fn with_service(mut self, service: ServiceType) -> Self {
        self.services_to_start.push(service);
        self
    }
    
    pub fn with_chaos(mut self) -> Self {
        self.config.enable_chaos = true;
        self
    }
    
    pub fn with_user(mut self, _user: fixtures::TestUser) -> Self {
        // Store test user for authentication tests
        // Implementation will be added when test auth system is ready
        self
    }
    
    pub async fn build(self) -> Result<IntegrationTestEnvironment, TestError> {
        let mut env = IntegrationTestEnvironment::new(self.test_name).await;
        env.config = self.config;
        
        // Start requested services
        for service_type in self.services_to_start {
            let service_id = env.start_service(service_type).await?;
            env.wait_for_service(&service_id).await?;
        }
        
        Ok(env)
    }
}

/// Macro for creating integration tests with automatic cleanup
#[macro_export]
macro_rules! integration_test {
    ($test_name:ident, $test_fn:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let env = IntegrationTestEnvironment::new(stringify!($test_name)).await;
            let result = $test_fn(&env).await;
            env.cleanup().await.expect("Cleanup failed");
            result.expect("Test failed");
        }
    };
}

