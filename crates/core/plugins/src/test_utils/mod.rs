// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Test utilities plugin module
//!
//! This module provides functionality for test utilities plugins.

use std::fmt::Debug;
use anyhow::Result;
use serde_json::Value;

use crate::plugin::Plugin;

/// Test scenario
#[derive(Clone, Debug)]
pub struct TestScenario {
    /// Scenario ID
    pub id: String,
    
    /// Scenario name
    pub name: String,
    
    /// Scenario description
    pub description: String,
    
    /// Input data
    pub input: Value,
    
    /// Expected output
    pub expected_output: Value,
}

/// Test plugin trait
#[expect(async_fn_in_trait, reason = "internal trait — all impls are Send + Sync")]
pub trait TestUtilsPlugin: Plugin + Send + Sync {
    /// Get available test scenarios
    fn get_test_scenarios(&self) -> Vec<TestScenario>;
    
    /// Run a test scenario
    async fn run_test_scenario(&self, scenario_id: &str) -> Result<Value>;
    
    /// Validate test results
    fn validate_test_results(&self, scenario_id: &str, actual_output: &Value) -> Result<bool>;
    
    /// Create a mock object
    async fn create_mock(&self, mock_type: &str, config: Value) -> Result<Value>;
    
    /// Check if the plugin supports a test scenario
    fn supports_test_scenario(&self, scenario_id: &str) -> bool {
        self.get_test_scenarios().iter().any(|ts| ts.id == scenario_id)
    }
    
    /// Get plugin capabilities
    fn get_capabilities(&self) -> Vec<String> {
        self.metadata().capabilities.clone()
    }
} 