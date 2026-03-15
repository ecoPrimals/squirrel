// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Test Fixtures for Integration Tests
//!
//! Provides reusable test data and configuration for integration tests.

use serde_json::json;

/// Common test user data
pub struct TestUser {
    pub user_id: String,
    pub username: String,
    pub email: String,
}

impl TestUser {
    pub fn alice() -> Self {
        Self {
            user_id: "test-user-alice".to_string(),
            username: "alice".to_string(),
            email: "alice@test.example.com".to_string(),
        }
    }
    
    pub fn bob() -> Self {
        Self {
            user_id: "test-user-bob".to_string(),
            username: "bob".to_string(),
            email: "bob@test.example.com".to_string(),
        }
    }
    
    pub fn admin() -> Self {
        Self {
            user_id: "test-admin".to_string(),
            username: "test_admin".to_string(),
            email: "admin@test.example.com".to_string(),
        }
    }
}

/// Common test MCP messages
pub mod messages {
    use super::*;
    
    pub fn ping_message() -> serde_json::Value {
        json!({
            "type": "ping",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })
    }
    
    pub fn capability_query() -> serde_json::Value {
        json!({
            "type": "capability_query",
            "requested_capabilities": ["ai_coordination", "service_discovery"],
        })
    }
    
    pub fn service_registration(service_name: &str) -> serde_json::Value {
        json!({
            "type": "service_registration",
            "service_name": service_name,
            "capabilities": ["test_capability"],
            "endpoint": format!("http://localhost:50000/{}", service_name),
        })
    }
}

/// Common test configurations
pub mod configs {
    use super::*;
    
    pub fn minimal_squirrel_config() -> serde_json::Value {
        json!({
            "server": {
                "host": "127.0.0.1",
                "port": 50000,
            },
            "mcp": {
                "enable": true,
                "transport": "http",
            },
            "security": {
                "enable_auth": false,
            },
        })
    }
    
    pub fn full_ecosystem_config() -> serde_json::Value {
        json!({
            "server": {
                "host": "127.0.0.1",
                "port": 50000,
            },
            "mcp": {
                "enable": true,
                "transport": "http",
            },
            "security": {
                "enable_auth": true,
                "beardog_url": "http://localhost:50001",
            },
            "discovery": {
                "songbird_url": "http://localhost:50002",
            },
        })
    }
}

