// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! AI types and data structures for the enhanced coordinator
//!
//! This module provides core data types for AI requests, responses,
//! and related structures used throughout the coordinator system.

use std::collections::HashMap;
use std::time::Duration;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use futures::Stream;
use std::sync::LazyLock;

use crate::error::types::Result;
use crate::enhanced::providers::ProviderType;

/// String interning for common AI values
static AI_STRINGS: LazyLock<HashMap<&'static str, Arc<str>>> = LazyLock::new(|| {
        let mut map = HashMap::new();
        // Common AI models
        map.insert("gpt-4", Arc::from("gpt-4"));
        map.insert("gpt-3.5-turbo", Arc::from("gpt-3.5-turbo"));
        map.insert("claude-3-opus", Arc::from("claude-3-opus"));
        map.insert("claude-3-sonnet", Arc::from("claude-3-sonnet"));
        map.insert("claude-3-haiku", Arc::from("claude-3-haiku"));
        map.insert("gemini-pro", Arc::from("gemini-pro"));
        map.insert("llama-2-70b", Arc::from("llama-2-70b"));
        map.insert("llama-2-13b", Arc::from("llama-2-13b"));
        map.insert("llama-2-7b", Arc::from("llama-2-7b"));
        
        // Common providers
        map.insert("openai", Arc::from("openai"));
        map.insert("anthropic", Arc::from("anthropic"));
        map.insert("google", Arc::from("google"));
        map.insert("local", Arc::from("local"));
        map.insert("local-server", Arc::from("local-server"));
        map.insert("model-hub", Arc::from("model-hub"));
        
        // Common roles
        map.insert("user", Arc::from("user"));
        map.insert("assistant", Arc::from("assistant"));
        map.insert("system", Arc::from("system"));
        map.insert("function", Arc::from("function"));
        
        // Common metadata keys
        map.insert("temperature", Arc::from("temperature"));
        map.insert("max_tokens", Arc::from("max_tokens"));
        map.insert("top_p", Arc::from("top_p"));
        map.insert("frequency_penalty", Arc::from("frequency_penalty"));
        map.insert("presence_penalty", Arc::from("presence_penalty"));
        map.insert("stop", Arc::from("stop"));
        map.insert("stream", Arc::from("stream"));
        
        // Request type strings
        map.insert("text_generation", Arc::from("text_generation"));
        map.insert("chat_completion", Arc::from("chat_completion"));
        map.insert("image_generation", Arc::from("image_generation"));
        map.insert("embeddings", Arc::from("embeddings"));
        map.insert("fine_tuning", Arc::from("fine_tuning"));
        
        map
});

/// Get Arc<str> for AI string with zero allocation for common values
pub fn intern_ai_string(s: &str) -> Arc<str> {
    AI_STRINGS.get(s)
        .cloned()
        .unwrap_or_else(|| Arc::from(s))
}

/// Universal AI request structure with Arc<str> optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalAIRequest {
    /// Request ID as Arc<str> for efficient sharing across async boundaries
    #[serde(serialize_with = "serialize_arc_str", deserialize_with = "deserialize_arc_str")]
    pub id: Arc<str>,
    
    /// Target model or system with string interning
    #[serde(serialize_with = "serialize_arc_str", deserialize_with = "deserialize_arc_str")]
    pub model: Arc<str>,
    
    /// Request type
    pub request_type: AIRequestType,
    
    /// Request payload (flexible JSON)
    pub payload: serde_json::Value,
    
    /// Messages for chat-style requests with Arc<str> optimization
    pub messages: Vec<MessageContent>,
    
    /// Additional metadata with Arc<str> keys and values
    #[serde(serialize_with = "serialize_metadata", deserialize_with = "deserialize_metadata")]
    pub metadata: HashMap<Arc<str>, Arc<serde_json::Value>>,
    
    /// Context and metadata
    pub context: RequestContext,
    
    /// Routing hints
    pub hints: RoutingHints,
    
    /// Quality requirements
    pub requirements: QualityRequirements,
}

impl UniversalAIRequest {
    /// Create new UniversalAIRequest with string interning optimization
    pub fn new(
        id: &str,
        model: &str,
        request_type: AIRequestType,
        payload: serde_json::Value,
        messages: Vec<MessageContent>,
    ) -> Self {
        Self {
            id: Arc::from(id),
            model: intern_ai_string(model),
            request_type,
            payload,
            messages,
            metadata: HashMap::new(),
            context: RequestContext::default(),
            hints: RoutingHints::default(),
            requirements: QualityRequirements::default(),
        }
    }

    /// Add metadata efficiently using string interning
    pub fn add_metadata(&mut self, key: &str, value: serde_json::Value) {
        let key_arc = intern_ai_string(key);
        self.metadata.insert(key_arc, Arc::new(value));
    }

    /// Get metadata efficiently without allocation
    pub fn get_metadata(&self, key: &str) -> Option<&Arc<serde_json::Value>> {
        self.metadata.iter()
            .find(|(k, _)| k.as_ref() == key)
            .map(|(_, v)| v)
    }
}

/// Universal AI response structure with Arc<str> optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalAIResponse {
    /// Response ID as Arc<str>
    #[serde(serialize_with = "serialize_arc_str", deserialize_with = "deserialize_arc_str")]
    pub id: Arc<str>,
    
    /// Provider name with string interning
    #[serde(serialize_with = "serialize_arc_str", deserialize_with = "deserialize_arc_str")]
    pub provider: Arc<str>,
    
    /// Model used with string interning
    #[serde(serialize_with = "serialize_arc_str", deserialize_with = "deserialize_arc_str")]
    pub model: Arc<str>,
    
    /// Response type
    pub response_type: AIRequestType,
    
    /// Response content as Arc<str> for efficient sharing
    #[serde(serialize_with = "serialize_arc_str", deserialize_with = "deserialize_arc_str")]
    pub content: Arc<str>,
    
    /// Cost
    pub cost: f64,
    
    /// Duration
    pub duration: Duration,
    
    /// Metadata with Arc<str> keys and values
    #[serde(serialize_with = "serialize_metadata", deserialize_with = "deserialize_metadata")]
    pub metadata: HashMap<Arc<str>, Arc<serde_json::Value>>,
}

impl UniversalAIResponse {
    /// Create new UniversalAIResponse with string interning optimization
    pub fn new(
        id: &str,
        provider: &str,
        model: &str,
        response_type: AIRequestType,
        content: &str,
        cost: f64,
        duration: Duration,
    ) -> Self {
        Self {
            id: Arc::from(id),
            provider: intern_ai_string(provider),
            model: intern_ai_string(model),
            response_type,
            content: Arc::from(content),
            cost,
            duration,
            metadata: HashMap::new(),
        }
    }
}

/// Message content for chat-style requests with Arc<str> optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContent {
    /// Message role with string interning (user, assistant, system)
    #[serde(serialize_with = "serialize_arc_str", deserialize_with = "deserialize_arc_str")]
    pub role: Arc<str>,
    
    /// Message content as Arc<str> for efficient sharing
    #[serde(serialize_with = "serialize_arc_str", deserialize_with = "deserialize_arc_str")]
    pub content: Arc<str>,
}

impl MessageContent {
    /// Create new MessageContent with string interning
    pub fn new(role: &str, content: &str) -> Self {
        Self {
            role: intern_ai_string(role),
            content: Arc::from(content),
        }
    }

    /// Create user message
    pub fn user(content: &str) -> Self {
        Self::new("user", content)
    }

    /// Create assistant message
    pub fn assistant(content: &str) -> Self {
        Self::new("assistant", content)
    }

    /// Create system message
    pub fn system(content: &str) -> Self {
        Self::new("system", content)
    }
}

// Serde helper functions for Arc<str> serialization
fn serialize_arc_str<S>(arc_str: &Arc<str>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(arc_str)
}

fn deserialize_arc_str<'de, D>(deserializer: D) -> std::result::Result<Arc<str>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Arc::from(s))
}

fn serialize_metadata<S>(map: &HashMap<Arc<str>, Arc<serde_json::Value>>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let json_map: HashMap<&str, &serde_json::Value> = map.iter()
        .map(|(k, v)| (k.as_ref(), v.as_ref()))
        .collect();
    json_map.serialize(serializer)
}

fn deserialize_metadata<'de, D>(deserializer: D) -> std::result::Result<HashMap<Arc<str>, Arc<serde_json::Value>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let json_map = HashMap::<String, serde_json::Value>::deserialize(deserializer)?;
    Ok(json_map.into_iter()
        .map(|(k, v)| (intern_ai_string(&k), Arc::new(v)))
        .collect())
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

/// Request context with Arc<str> optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// Session ID as Arc<str>
    #[serde(serialize_with = "serialize_arc_str", deserialize_with = "deserialize_arc_str")]
    pub session_id: Arc<str>,
    
    /// User ID as Arc<str>
    #[serde(serialize_with = "serialize_arc_str", deserialize_with = "deserialize_arc_str")]
    pub user_id: Arc<str>,
    
    /// Additional context with Arc<str> keys and values
    #[serde(serialize_with = "serialize_metadata", deserialize_with = "deserialize_metadata")]
    pub additional_context: HashMap<Arc<str>, Arc<serde_json::Value>>,
}

impl Default for RequestContext {
    fn default() -> Self {
        Self {
            session_id: Arc::from("default_session"),
            user_id: Arc::from("anonymous"),
            additional_context: HashMap::new(),
        }
    }
}

/// Routing hints with Arc<str> optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingHints {
    /// Preferred provider as Arc<str>
    #[serde(serialize_with = "serialize_optional_arc_str", deserialize_with = "deserialize_optional_arc_str")]
    pub preferred_provider: Option<Arc<str>>,
    
    /// Required capabilities with Arc<str>
    pub required_capabilities: Vec<Arc<str>>,
    
    /// Cost constraints
    pub max_cost: Option<f64>,
    
    /// Latency requirements
    pub max_latency: Option<Duration>,
    
    /// Priority level
    pub priority: RequestPriority,
}

impl Default for RoutingHints {
    fn default() -> Self {
        Self {
            preferred_provider: None,
            required_capabilities: Vec::new(),
            max_cost: None,
            max_latency: None,
            priority: RequestPriority::Normal,
        }
    }
}

/// Quality requirements with Arc<str> optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    /// Minimum quality score
    pub min_quality_score: Option<f64>,
    
    /// Required output format as Arc<str>
    #[serde(serialize_with = "serialize_optional_arc_str", deserialize_with = "deserialize_optional_arc_str")]
    pub required_format: Option<Arc<str>>,
    
    /// Language requirements with Arc<str>
    pub language_requirements: Vec<Arc<str>>,
    
    /// Safety requirements
    pub safety_level: SafetyLevel,
}

impl Default for QualityRequirements {
    fn default() -> Self {
        Self {
            min_quality_score: None,
            required_format: None,
            language_requirements: Vec::new(),
            safety_level: SafetyLevel::Standard,
        }
    }
}

/// Request priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Safety levels for AI requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyLevel {
    Minimal,
    Standard,
    High,
    Maximum,
}

/// Cost estimate with Arc<str> optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    /// Estimated cost
    pub estimated_cost: f64,
    
    /// Currency as Arc<str>
    #[serde(serialize_with = "serialize_arc_str", deserialize_with = "deserialize_arc_str")]
    pub currency: Arc<str>,
    
    /// Cost breakdown with Arc<str> keys
    #[serde(serialize_with = "serialize_cost_breakdown", deserialize_with = "deserialize_cost_breakdown")]
    pub breakdown: HashMap<Arc<str>, f64>,
}

impl CostEstimate {
    /// Create new cost estimate with string interning
    pub fn new(estimated_cost: f64, currency: &str, breakdown: HashMap<&str, f64>) -> Self {
        Self {
            estimated_cost,
            currency: Arc::from(currency),
            breakdown: breakdown.into_iter()
                .map(|(k, v)| (intern_ai_string(k), v))
                .collect(),
        }
    }
}

/// AI requirements for tool execution with Arc<str> optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequirements {
    /// Minimum model size as Arc<str>
    #[serde(serialize_with = "serialize_optional_arc_str", deserialize_with = "deserialize_optional_arc_str")]
    pub min_model_size: Option<Arc<str>>,
    
    /// Required capabilities with Arc<str>
    pub required_capabilities: Vec<Arc<str>>,
    
    /// Maximum cost
    pub max_cost: Option<f64>,
    
    /// Maximum latency
    pub max_latency: Option<Duration>,
}

impl Default for AIRequirements {
    fn default() -> Self {
        Self {
            min_model_size: None,
            required_capabilities: Vec::new(),
            max_cost: None,
            max_latency: None,
        }
    }
}

/// AI metrics with Arc<str> keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_cost: f64,
    #[serde(with = "duration_serde")]
    pub avg_latency: Duration,
}

/// Provider health information with Arc<str> optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub healthy: bool,
    pub latency: Option<Duration>,
    pub error_rate: Option<f64>,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Universal AI stream type with Arc<str> optimization
pub type UniversalAIStream = Box<dyn Stream<Item = Result<UniversalAIResponse>> + Send + Unpin>;

// Additional serde helper functions
fn serialize_optional_arc_str<S>(opt: &Option<Arc<str>>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match opt {
        Some(arc_str) => serializer.serialize_some(arc_str.as_ref()),
        None => serializer.serialize_none(),
    }
}

fn deserialize_optional_arc_str<'de, D>(deserializer: D) -> std::result::Result<Option<Arc<str>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt_string = Option::<String>::deserialize(deserializer)?;
    Ok(opt_string.map(|s| Arc::from(s)))
}

fn serialize_cost_breakdown<S>(map: &HashMap<Arc<str>, f64>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let string_map: HashMap<&str, f64> = map.iter()
        .map(|(k, v)| (k.as_ref(), *v))
        .collect();
    string_map.serialize(serializer)
}

fn deserialize_cost_breakdown<'de, D>(deserializer: D) -> std::result::Result<HashMap<Arc<str>, f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let string_map = HashMap::<String, f64>::deserialize(deserializer)?;
    Ok(string_map.into_iter()
        .map(|(k, v)| (intern_ai_string(&k), v))
        .collect())
}

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