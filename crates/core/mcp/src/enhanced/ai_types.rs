//! AI types and data structures for the enhanced coordinator
//!
//! This module provides core data types for AI requests, responses,
//! and related structures used throughout the coordinator system.

use std::collections::HashMap;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use futures::Stream;

use crate::error::types::Result;
use crate::enhanced::providers::ProviderType;

/// Universal AI request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalAIRequest {
    /// Request ID
    pub id: String,
    
    /// Target model or system
    pub model: String,
    
    /// Request type
    pub request_type: AIRequestType,
    
    /// Request payload (flexible JSON)
    pub payload: serde_json::Value,
    
    /// Messages for chat-style requests
    pub messages: Vec<MessageContent>,
    
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Context and metadata
    pub context: RequestContext,
    
    /// Routing hints
    pub hints: RoutingHints,
    
    /// Quality requirements
    pub requirements: QualityRequirements,
}

/// Universal AI response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalAIResponse {
    /// Response ID
    pub id: String,
    
    /// Provider name
    pub provider: String,
    
    /// Model used
    pub model: String,
    
    /// Response type
    pub response_type: AIRequestType,
    
    /// Response content
    pub content: String,
    
    /// Cost
    pub cost: f64,
    
    /// Duration
    pub duration: Duration,
    
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Message content for chat-style requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContent {
    /// Message role (user, assistant, system)
    pub role: String,
    
    /// Message content
    pub content: String,
}

/// AI request types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIRequestType {
    /// Text generation/chat
    TextGeneration,
    
    /// Image generation
    ImageGeneration,
    
    /// Image understanding
    ImageAnalysis,
    
    /// Audio generation
    AudioGeneration,
    
    /// Audio transcription
    AudioTranscription,
    
    /// Video generation
    VideoGeneration,
    
    /// Video analysis
    VideoAnalysis,
    
    /// Embeddings
    Embeddings,
    
    /// Fine-tuning
    FineTuning,
    
    /// Model evaluation
    Evaluation,
    
    /// Custom function
    Custom(String),
    
    /// Future AI capabilities
    Future(String),
}

/// Request context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Routing hints for provider selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingHints {
    pub prefer_local: bool,
    pub max_cost: Option<f64>,
    pub max_latency: Option<Duration>,
    pub quality_requirements: Vec<String>,
}

/// Quality requirements for requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    pub min_quality_score: Option<f64>,
    pub require_streaming: bool,
    pub require_tools: bool,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub provider: String,
    pub model_type: ProviderType,
    pub capabilities: Vec<String>,
    pub performance: Option<PerformanceMetrics>,
}

/// Model capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub max_tokens: Option<usize>,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub cost_per_token: Option<f64>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_latency: Duration,
    pub success_rate: f64,
    pub cost_per_request: Option<f64>,
    pub quality_score: Option<f64>,
}

/// Cost estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    /// Estimated cost
    pub estimated_cost: f64,
    
    /// Currency
    pub currency: String,
    
    /// Cost breakdown
    pub breakdown: HashMap<String, f64>,
}

/// AI requirements for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequirements {
    pub min_model_size: Option<String>,
    pub required_capabilities: Vec<String>,
    pub max_cost: Option<f64>,
    pub max_latency: Option<Duration>,
}

/// AI metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_cost: f64,
    #[serde(with = "duration_serde")]
    pub avg_latency: Duration,
}

/// Provider health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub healthy: bool,
    pub latency: Option<Duration>,
    pub error_rate: Option<f64>,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Universal AI stream type
pub type UniversalAIStream = Box<dyn Stream<Item = Result<UniversalAIResponse>> + Send + Unpin>;

/// Serialization helper for Duration
pub mod duration_serde {
    use std::time::Duration;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(duration: &Duration, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

impl AIMetrics {
    /// Create new AI metrics
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_cost: 0.0,
            avg_latency: Duration::from_millis(0),
        }
    }

    /// Update metrics with a new request
    pub fn update(&mut self, success: bool, cost: f64, latency: Duration) {
        self.total_requests += 1;
        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }
        self.total_cost += cost;
        
        // Update average latency (simple moving average)
        let total_latency = self.avg_latency.as_millis() as u64 * (self.total_requests - 1) + latency.as_millis() as u64;
        self.avg_latency = Duration::from_millis(total_latency / self.total_requests);
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }

    /// Get average cost per request
    pub fn avg_cost_per_request(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.total_cost / self.total_requests as f64
        }
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self {
            user_id: None,
            session_id: None,
            metadata: HashMap::new(),
        }
    }
}

impl Default for RoutingHints {
    fn default() -> Self {
        Self {
            prefer_local: false,
            max_cost: None,
            max_latency: None,
            quality_requirements: Vec::new(),
        }
    }
}

impl Default for QualityRequirements {
    fn default() -> Self {
        Self {
            min_quality_score: None,
            require_streaming: false,
            require_tools: false,
        }
    }
}

impl AIRequestType {
    /// Check if this request type supports streaming
    pub fn supports_streaming(&self) -> bool {
        matches!(self, 
            AIRequestType::TextGeneration | 
            AIRequestType::ImageGeneration | 
            AIRequestType::AudioGeneration
        )
    }

    /// Check if this request type supports tools
    pub fn supports_tools(&self) -> bool {
        matches!(self, 
            AIRequestType::TextGeneration | 
            AIRequestType::Custom(_)
        )
    }

    /// Get the display name for this request type
    pub fn display_name(&self) -> &str {
        match self {
            AIRequestType::TextGeneration => "Text Generation",
            AIRequestType::ImageGeneration => "Image Generation",
            AIRequestType::ImageAnalysis => "Image Analysis",
            AIRequestType::AudioGeneration => "Audio Generation",
            AIRequestType::AudioTranscription => "Audio Transcription",
            AIRequestType::VideoGeneration => "Video Generation",
            AIRequestType::VideoAnalysis => "Video Analysis",
            AIRequestType::Embeddings => "Embeddings",
            AIRequestType::FineTuning => "Fine-tuning",
            AIRequestType::Evaluation => "Model Evaluation",
            AIRequestType::Custom(name) => name,
            AIRequestType::Future(name) => name,
        }
    }
} 