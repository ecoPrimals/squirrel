// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Test utilities and mock providers for AI API testing
//! 
//! Provides comprehensive mock implementations for testing AI orchestration,
//! ecosystem discovery, and error handling scenarios.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use async_trait::async_trait;

/// Mock ecosystem manager for testing
#[derive(Clone)]
pub struct MockEcosystemManager {
    providers: Arc<Mutex<HashMap<String, MockProvider>>>,
    config: MockConfig,
}

#[derive(Clone)]
pub struct MockConfig {
    pub simulate_slow: bool,
    pub simulate_errors: bool,
    pub max_capacity: usize,
}

impl Default for MockConfig {
    fn default() -> Self {
        Self {
            simulate_slow: false,
            simulate_errors: false,
            max_capacity: 100,
        }
    }
}

#[derive(Clone)]
pub struct MockProvider {
    pub name: String,
    pub capabilities: Vec<String>,
    pub healthy: bool,
    pub priority: u8,
    pub response_time: Duration,
    pub error_mode: Option<ErrorMode>,
}

#[derive(Clone)]
pub enum ErrorMode {
    NetworkError(String),
    Timeout,
    InvalidResponse,
    AuthFailure,
    RateLimit,
    Crash,
}

impl MockEcosystemManager {
    pub fn new() -> Self {
        Self {
            providers: Arc::new(Mutex::new(HashMap::new())),
            config: MockConfig::default(),
        }
    }

    pub fn new_empty() -> Self {
        Self::new()
    }

    pub fn new_with_limited_capacity(capacity: usize) -> Self {
        Self {
            providers: Arc::new(Mutex::new(HashMap::new())),
            config: MockConfig {
                max_capacity: capacity,
                ..Default::default()
            },
        }
    }

    pub fn add_healthy_provider(&self, name: &str) {
        let mut providers = self.providers.lock().unwrap();
        providers.insert(
            name.to_string(),
            MockProvider {
                name: name.to_string(),
                capabilities: vec!["text-generation".to_string()],
                healthy: true,
                priority: 5,
                response_time: Duration::from_millis(10),
                error_mode: None,
            },
        );
    }

    pub fn add_slow_provider(&self, name: &str, delay: Duration) {
        let mut providers = self.providers.lock().unwrap();
        providers.insert(
            name.to_string(),
            MockProvider {
                name: name.to_string(),
                capabilities: vec!["text-generation".to_string()],
                healthy: true,
                priority: 5,
                response_time: delay,
                error_mode: None,
            },
        );
    }

    pub fn add_network_error_provider(&self, name: &str, error: &str) {
        let mut providers = self.providers.lock().unwrap();
        providers.insert(
            name.to_string(),
            MockProvider {
                name: name.to_string(),
                capabilities: vec!["text-generation".to_string()],
                healthy: false,
                priority: 5,
                response_time: Duration::from_millis(10),
                error_mode: Some(ErrorMode::NetworkError(error.to_string())),
            },
        );
    }

    pub fn add_invalid_provider(&self, name: &str) {
        let mut providers = self.providers.lock().unwrap();
        providers.insert(
            name.to_string(),
            MockProvider {
                name: name.to_string(),
                capabilities: vec!["text-generation".to_string()],
                healthy: true,
                priority: 5,
                response_time: Duration::from_millis(10),
                error_mode: Some(ErrorMode::InvalidResponse),
            },
        );
    }

    pub fn add_provider_with_capabilities(&self, name: &str, capabilities: Vec<&str>) {
        let mut providers = self.providers.lock().unwrap();
        providers.insert(
            name.to_string(),
            MockProvider {
                name: name.to_string(),
                capabilities: capabilities.iter().map(|s| s.to_string()).collect(),
                healthy: true,
                priority: 5,
                response_time: Duration::from_millis(10),
                error_mode: None,
            },
        );
    }

    pub fn add_rate_limited_provider(&self, name: &str, _limit: usize) -> MockProvider {
        let provider = MockProvider {
            name: name.to_string(),
            capabilities: vec!["text-generation".to_string()],
            healthy: true,
            priority: 5,
            response_time: Duration::from_millis(10),
            error_mode: Some(ErrorMode::RateLimit),
        };
        
        let mut providers = self.providers.lock().unwrap();
        providers.insert(name.to_string(), provider.clone());
        provider
    }

    pub fn add_streaming_provider_with_disconnect(&self, name: &str, _disconnect_at: usize) {
        let mut providers = self.providers.lock().unwrap();
        providers.insert(
            name.to_string(),
            MockProvider {
                name: name.to_string(),
                capabilities: vec!["text-generation".to_string()],
                healthy: true,
                priority: 5,
                response_time: Duration::from_millis(10),
                error_mode: Some(ErrorMode::NetworkError("Stream disconnected".to_string())),
            },
        );
    }

    pub fn add_provider_requiring_auth(&self, name: &str, _authenticated: bool) {
        let mut providers = self.providers.lock().unwrap();
        providers.insert(
            name.to_string(),
            MockProvider {
                name: name.to_string(),
                capabilities: vec!["text-generation".to_string()],
                healthy: true,
                priority: 5,
                response_time: Duration::from_millis(10),
                error_mode: Some(ErrorMode::AuthFailure),
            },
        );
    }

    pub fn add_crash_provider(&self, name: &str) {
        let mut providers = self.providers.lock().unwrap();
        providers.insert(
            name.to_string(),
            MockProvider {
                name: name.to_string(),
                capabilities: vec!["text-generation".to_string()],
                healthy: false,
                priority: 1,
                response_time: Duration::from_millis(10),
                error_mode: Some(ErrorMode::Crash),
            },
        );
    }

    pub fn add_provider_with_invalid_schema(&self, name: &str) {
        let mut providers = self.providers.lock().unwrap();
        providers.insert(
            name.to_string(),
            MockProvider {
                name: name.to_string(),
                capabilities: vec!["text-generation".to_string()],
                healthy: true,
                priority: 5,
                response_time: Duration::from_millis(10),
                error_mode: Some(ErrorMode::InvalidResponse),
            },
        );
    }

    pub fn add_redirect_provider(&self, name: &str, _redirect_to: &str) {
        let mut providers = self.providers.lock().unwrap();
        providers.insert(
            name.to_string(),
            MockProvider {
                name: name.to_string(),
                capabilities: vec!["text-generation".to_string()],
                healthy: true,
                priority: 5,
                response_time: Duration::from_millis(10),
                error_mode: None,
            },
        );
    }

    pub fn add_provider_with_priority(&self, name: &str, priority: u8) {
        let mut providers = self.providers.lock().unwrap();
        providers.insert(
            name.to_string(),
            MockProvider {
                name: name.to_string(),
                capabilities: vec!["text-generation".to_string()],
                healthy: true,
                priority,
                response_time: Duration::from_millis(10),
                error_mode: None,
            },
        );
    }

    pub fn get_providers(&self) -> Vec<MockProvider> {
        let providers = self.providers.lock().unwrap();
        providers.values().cloned().collect()
    }
}

/// Mock AI Request for testing
#[derive(Clone)]
pub struct AIRequest {
    pub prompt: String,
    pub capability: String,
    pub timeout: Option<Duration>,
    pub streaming: bool,
    pub max_tokens: i32,
}

impl AIRequest {
    pub fn new(prompt: &str, capability: &str) -> Self {
        Self {
            prompt: prompt.to_string(),
            capability: capability.to_string(),
            timeout: Some(Duration::from_secs(30)),
            streaming: false,
            max_tokens: 1000,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_streaming(mut self, streaming: bool) -> Self {
        self.streaming = streaming;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: i32) -> Self {
        self.max_tokens = max_tokens;
        self
    }
}

/// Mock AI Response for testing
pub struct AIResponse {
    pub content: String,
    pub provider_used: String,
    pub tokens_used: usize,
}

/// Mock orchestration function (to be replaced with real implementation)
pub async fn orchestrate(
    _ecosystem: &MockEcosystemManager,
    _request: AIRequest,
) -> Result<AIResponse, crate::error::Error> {
    // This is a placeholder - actual tests should mock at the boundary
    Ok(AIResponse {
        content: "mock response".to_string(),
        provider_used: "mock".to_string(),
        tokens_used: 100,
    })
}

/// Mock provider selection (to be replaced with real implementation)
pub async fn select_provider(
    providers: &[MockProvider],
    _request: &AIRequest,
) -> Option<MockProvider> {
    // Return highest priority healthy provider
    providers
        .iter()
        .filter(|p| p.healthy)
        .max_by_key(|p| p.priority)
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_ecosystem_creation() {
        let ecosystem = MockEcosystemManager::new();
        assert_eq!(ecosystem.get_providers().len(), 0);
    }

    #[test]
    fn test_add_healthy_provider() {
        let ecosystem = MockEcosystemManager::new();
        ecosystem.add_healthy_provider("test-provider");
        assert_eq!(ecosystem.get_providers().len(), 1);
    }

    #[test]
    fn test_provider_selection() {
        let providers = vec![
            MockProvider {
                name: "low".to_string(),
                capabilities: vec![],
                healthy: true,
                priority: 1,
                response_time: Duration::from_millis(10),
                error_mode: None,
            },
            MockProvider {
                name: "high".to_string(),
                capabilities: vec![],
                healthy: true,
                priority: 10,
                response_time: Duration::from_millis(10),
                error_mode: None,
            },
        ];

        let request = AIRequest::new("test", "test");
        let selected = tokio_test::block_on(select_provider(&providers, &request));
        
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "high");
    }
}

