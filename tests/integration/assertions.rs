// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Custom Assertions for Integration Tests
//!
//! Provides domain-specific assertions for testing primal interactions.

use super::TestError;

/// Assert that a service responds to health checks
pub async fn assert_service_healthy(health_url: &str) -> Result<(), TestError> {
    let client = reqwest::Client::new();
    let response = client.get(health_url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| TestError::AssertionFailed(format!("Health check failed: {}", e)))?;
    
    if response.status().is_success() {
        Ok(())
    } else {
        Err(TestError::AssertionFailed(
            format!("Service not healthy: status {}", response.status())
        ))
    }
}

/// Assert that MCP message exchange succeeds
pub async fn assert_mcp_exchange_succeeds(
    from_url: &str,
    to_url: &str,
    message: serde_json::Value,
) -> Result<serde_json::Value, TestError> {
    let client = reqwest::Client::new();
    let response = client.post(format!("{}/mcp/send", from_url))
        .json(&json!({
            "target": to_url,
            "message": message,
        }))
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| TestError::CommunicationError(format!("MCP send failed: {}", e)))?;
    
    if !response.status().is_success() {
        return Err(TestError::AssertionFailed(
            format!("MCP exchange failed with status: {}", response.status())
        ));
    }
    
    response.json().await
        .map_err(|e| TestError::CommunicationError(format!("Response parse failed: {}", e)))
}

/// Assert that service discovery finds a capability
pub async fn assert_capability_discovered(
    discovery_url: &str,
    capability: &str,
) -> Result<Vec<serde_json::Value>, TestError> {
    let client = reqwest::Client::new();
    let response = client.get(format!("{}/discover", discovery_url))
        .query(&[("capability", capability)])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| TestError::CommunicationError(format!("Discovery failed: {}", e)))?;
    
    if !response.status().is_success() {
        return Err(TestError::AssertionFailed(
            format!("Discovery failed with status: {}", response.status())
        ));
    }
    
    let services: Vec<serde_json::Value> = response.json().await
        .map_err(|e| TestError::CommunicationError(format!("Response parse failed: {}", e)))?;
    
    if services.is_empty() {
        return Err(TestError::AssertionFailed(
            format!("No services found with capability: {}", capability)
        ));
    }
    
    Ok(services)
}

use serde_json::json;

/// Assert that an HTTP response is successful (2xx status code)
pub fn assert_response_successful(response: &reqwest::Response) {
    assert!(
        response.status().is_success(),
        "Expected successful response, got: {}",
        response.status()
    );
}

