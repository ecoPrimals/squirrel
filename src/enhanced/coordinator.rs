//! AI Coordinator for enhanced MCP functionality
//! 
//! Coordination and orchestration of AI capabilities within MCP.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// AI Coordinator for managing AI capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AICoordinator {
    /// Coordinator ID
    pub id: String,
    /// Active AI models
    pub models: HashMap<String, AIModel>,
    /// Configuration
    pub config: AICoordinatorConfig,
}

impl AICoordinator {
    /// Create a new AI coordinator
    pub fn new(config: AICoordinatorConfig) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            models: HashMap::new(),
            config,
        }
    }
}

/// AI Coordinator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AICoordinatorConfig {
    /// Maximum concurrent models
    pub max_models: usize,
    /// Default model timeout
    pub default_timeout: u64,
    /// Model selection strategy
    pub selection_strategy: String,
}

impl Default for AICoordinatorConfig {
    fn default() -> Self {
        Self {
            max_models: 10,
            default_timeout: 30,
            selection_strategy: "round_robin".to_string(),
        }
    }
}

/// AI Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModel {
    /// Model ID
    pub id: String,
    /// Model name
    pub name: String,
    /// Model version
    pub version: String,
    /// Model capabilities
    pub capabilities: Vec<String>,
    /// Model status
    pub status: AIModelStatus,
}

/// AI Model status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIModelStatus {
    /// Model is available
    Available,
    /// Model is busy
    Busy,
    /// Model is offline
    Offline,
    /// Model has error
    Error(String),
} 