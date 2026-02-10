// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Native AI Provider Models and Configuration
//!
//! This module contains all configuration types, data structures, and model-related
//! code for the native AI provider implementation.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

use crate::types::{
    UniversalAIRequest, UniversalAIResponse,
    ModelCapabilities, ProviderHealth, RequestMetadata,
    AIRequestType,
};
use crate::error::ProviderResult;

/// Native AI Provider Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeAIConfig {
    /// Model configuration
    pub model_config: ModelConfig,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Performance settings
    pub performance: PerformanceConfig,
    /// Capabilities
    pub capabilities: ModelCapabilities,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

/// Model configuration for native AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model type (e.g., "llama", "gpt-style", "custom")
    pub model_type: String,
    /// Model path or identifier
    pub model_path: String,
    /// Model parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Maximum context length
    pub max_context_length: usize,
    /// Temperature settings
    pub temperature: f32,
    /// Top-p sampling
    pub top_p: f32,
    /// Maximum tokens to generate
    pub max_tokens: usize,
}

/// Resource limits for native AI processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage (MB)
    pub max_memory_mb: u64,
    /// Maximum CPU usage (percentage)
    pub max_cpu_percent: u32,
    /// Maximum request timeout
    pub max_timeout: Duration,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable GPU acceleration
    pub gpu_enabled: bool,
    /// GPU device ID
    pub gpu_device_id: Option<u32>,
    /// Batch size for processing
    pub batch_size: usize,
    /// Thread count for CPU processing
    pub cpu_threads: usize,
    /// Enable optimizations
    pub optimizations_enabled: bool,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check interval
    pub check_interval: Duration,
    /// Health check timeout
    pub check_timeout: Duration,
    /// Maximum failed checks before unhealthy
    pub max_failed_checks: u32,
    /// Enable detailed metrics
    pub detailed_metrics: bool,
}

/// Performance metrics for native AI processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total requests processed
    pub total_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// Current memory usage (MB)
    pub current_memory_mb: u64,
    /// Current CPU usage (%)
    pub current_cpu_percent: f32,
    /// Requests per second
    pub requests_per_second: f64,
    /// Model load time
    pub model_load_time_ms: u64,
    /// Last updated timestamp
    pub last_updated: std::time::SystemTime,
}

/// Model instance for native AI
#[derive(Debug)]
pub struct ModelInstance {
    /// Model type
    pub model_type: String,
    /// Model path
    pub model_path: String,
    /// Load time
    pub load_time: Instant,
    /// Status
    pub status: ModelStatus,
    /// Capabilities
    pub capabilities: ModelCapabilities,
    /// Process handle (if external)
    pub process_handle: Option<tokio::process::Child>,
}

/// Model status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelStatus {
    /// Model is currently loading
    Loading,
    /// Model is ready for requests
    Ready,
    /// Model encountered an error
    Error(String),
    /// Model is not loaded
    Unloaded,
}

/// Request information for tracking active requests
#[derive(Debug, Clone)]
pub struct RequestInfo {
    /// Unique request identifier
    pub request_id: String,
    /// Type of AI request
    pub request_type: AIRequestType,
    /// When the request started
    pub start_time: Instant,
    /// Estimated completion time
    pub estimated_completion: Option<Instant>,
    /// Additional request metadata
    pub metadata: RequestMetadata,
}

/// Queued request waiting for processing
#[derive(Debug)]
pub struct QueuedRequest {
    /// The AI request to process
    pub request: UniversalAIRequest,
    /// Channel to send the response back
    pub response_sender: tokio::sync::oneshot::Sender<ProviderResult<UniversalAIResponse>>,
    /// When the request was queued
    pub queued_at: Instant,
}

/// Provider state containing all runtime data
#[derive(Debug)]
pub struct ProviderState {
    /// Current health status
    pub health: Arc<RwLock<ProviderHealth>>,
    /// Active requests being processed
    pub active_requests: Arc<Mutex<HashMap<String, RequestInfo>>>,
    /// Performance metrics and statistics
    pub metrics: Arc<RwLock<PerformanceMetrics>>,
    /// Loaded model instance
    pub model: Arc<Mutex<Option<ModelInstance>>>,
    /// Queue of requests waiting to be processed
    pub request_queue: Arc<Mutex<Vec<QueuedRequest>>>,
}

impl Default for NativeAIConfig {
    fn default() -> Self {
        Self {
            model_config: ModelConfig::default(),
            resource_limits: ResourceLimits::default(),
            performance: PerformanceConfig::default(),
            capabilities: ModelCapabilities::default(),
            health_check: HealthCheckConfig::default(),
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_type: "llama".to_string(),
            model_path: "./models/default".to_string(),
            parameters: HashMap::new(),
            max_context_length: 4096,
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: 1024,
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 4096,
            max_cpu_percent: 80,
            max_timeout: Duration::from_secs(120),
            max_concurrent_requests: 4,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            gpu_enabled: false,
            gpu_device_id: None,
            batch_size: 1,
            cpu_threads: num_cpus::get(),
            optimizations_enabled: true,
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            check_timeout: Duration::from_secs(10),
            max_failed_checks: 3,
            detailed_metrics: true,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            failed_requests: 0,
            avg_response_time_ms: 0.0,
            current_memory_mb: 0,
            current_cpu_percent: 0.0,
            requests_per_second: 0.0,
            model_load_time_ms: 0,
            last_updated: std::time::SystemTime::now(),
        }
    }
}

impl ModelInstance {
    /// Create a new model instance
    pub fn new(
        model_type: String,
        model_path: String,
        capabilities: ModelCapabilities,
    ) -> Self {
        Self {
            model_type,
            model_path,
            load_time: Instant::now(),
            status: ModelStatus::Loading,
            capabilities,
            process_handle: None,
        }
    }

    /// Check if the model is ready for requests
    pub fn is_ready(&self) -> bool {
        self.status == ModelStatus::Ready
    }

    /// Get the model's current status
    pub fn status(&self) -> &ModelStatus {
        &self.status
    }

    /// Set the model status
    pub fn set_status(&mut self, status: ModelStatus) {
        self.status = status;
    }

    /// Get the time since the model was loaded
    pub fn uptime(&self) -> Duration {
        self.load_time.elapsed()
    }

    /// Check if the model supports a specific request type
    pub fn supports_request_type(&self, request_type: &AIRequestType) -> bool {
        match request_type {
            AIRequestType::TextGeneration => self.capabilities.text_generation,
            AIRequestType::TextCompletion => self.capabilities.text_completion,
            AIRequestType::Embedding => self.capabilities.embedding,
            AIRequestType::Classification => self.capabilities.classification,
            AIRequestType::Summarization => self.capabilities.summarization,
            AIRequestType::Translation => self.capabilities.translation,
            AIRequestType::QuestionAnswering => self.capabilities.question_answering,
            AIRequestType::CodeGeneration => self.capabilities.code_generation,
        }
    }
}

impl RequestInfo {
    /// Create a new request info
    pub fn new(
        request_id: String,
        request_type: AIRequestType,
        metadata: RequestMetadata,
    ) -> Self {
        Self {
            request_id,
            request_type,
            start_time: Instant::now(),
            estimated_completion: None,
            metadata,
        }
    }

    /// Get the elapsed time since the request started
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Set the estimated completion time
    pub fn set_estimated_completion(&mut self, duration: Duration) {
        self.estimated_completion = Some(self.start_time + duration);
    }

    /// Check if the request is past its estimated completion time
    pub fn is_overdue(&self) -> bool {
        if let Some(estimated) = self.estimated_completion {
            Instant::now() > estimated
        } else {
            false
        }
    }
}

impl QueuedRequest {
    /// Create a new queued request
    pub fn new(
        request: UniversalAIRequest,
        response_sender: tokio::sync::oneshot::Sender<ProviderResult<UniversalAIResponse>>,
    ) -> Self {
        Self {
            request,
            response_sender,
            queued_at: Instant::now(),
        }
    }

    /// Get the time the request has been waiting in queue
    pub fn queue_time(&self) -> Duration {
        self.queued_at.elapsed()
    }
}

impl ProviderState {
    /// Create a new provider state
    pub fn new() -> Self {
        Self {
            health: Arc::new(RwLock::new(ProviderHealth::Unknown)),
            active_requests: Arc::new(Mutex::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            model: Arc::new(Mutex::new(None)),
            request_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get the current health status
    pub async fn get_health(&self) -> ProviderHealth {
        *self.health.read().await
    }

    /// Set the health status
    pub async fn set_health(&self, health: ProviderHealth) {
        *self.health.write().await = health;
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }

    /// Update performance metrics
    pub async fn update_metrics<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut PerformanceMetrics),
    {
        let mut metrics = self.metrics.write().await;
        update_fn(&mut *metrics);
        metrics.last_updated = std::time::SystemTime::now();
    }

    /// Add an active request
    pub async fn add_active_request(&self, request_id: String, info: RequestInfo) {
        self.active_requests.lock().await.insert(request_id, info);
    }

    /// Remove an active request
    pub async fn remove_active_request(&self, request_id: &str) -> Option<RequestInfo> {
        self.active_requests.lock().await.remove(request_id)
    }

    /// Get the number of active requests
    pub async fn active_request_count(&self) -> usize {
        self.active_requests.lock().await.len()
    }

    /// Queue a request for processing
    pub async fn queue_request(&self, request: QueuedRequest) {
        self.request_queue.lock().await.push(request);
    }

    /// Dequeue the next request for processing
    pub async fn dequeue_request(&self) -> Option<QueuedRequest> {
        let mut queue = self.request_queue.lock().await;
        if !queue.is_empty() {
            Some(queue.remove(0))
        } else {
            None
        }
    }

    /// Get the number of queued requests
    pub async fn queue_length(&self) -> usize {
        self.request_queue.lock().await.len()
    }

    /// Check if the model is loaded and ready
    pub async fn is_model_ready(&self) -> bool {
        if let Some(model) = self.model.lock().await.as_ref() {
            model.is_ready()
        } else {
            false
        }
    }

    /// Set the loaded model instance
    pub async fn set_model(&self, model: ModelInstance) {
        *self.model.lock().await = Some(model);
    }

    /// Remove the loaded model
    pub async fn unload_model(&self) {
        *self.model.lock().await = None;
    }
}

impl Default for ProviderState {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration validation helpers
impl NativeAIConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate model config
        if self.model_config.model_path.is_empty() {
            return Err("Model path cannot be empty".to_string());
        }

        if self.model_config.max_context_length == 0 {
            return Err("Max context length must be greater than 0".to_string());
        }

        if self.model_config.max_tokens == 0 {
            return Err("Max tokens must be greater than 0".to_string());
        }

        // Validate resource limits
        if self.resource_limits.max_memory_mb == 0 {
            return Err("Max memory must be greater than 0".to_string());
        }

        if self.resource_limits.max_cpu_percent == 0 || self.resource_limits.max_cpu_percent > 100 {
            return Err("Max CPU percent must be between 1 and 100".to_string());
        }

        if self.resource_limits.max_concurrent_requests == 0 {
            return Err("Max concurrent requests must be greater than 0".to_string());
        }

        // Validate performance config
        if self.performance.batch_size == 0 {
            return Err("Batch size must be greater than 0".to_string());
        }

        if self.performance.cpu_threads == 0 {
            return Err("CPU threads must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Get the model identifier for caching purposes
    pub fn model_id(&self) -> String {
        format!("{}:{}", self.model_config.model_type, self.model_config.model_path)
    }

    /// Check if GPU acceleration is available and enabled
    pub fn is_gpu_enabled(&self) -> bool {
        self.performance.gpu_enabled && self.performance.gpu_device_id.is_some()
    }

    /// Get the effective thread count for processing
    pub fn effective_thread_count(&self) -> usize {
        if self.performance.gpu_enabled {
            1 // GPU processing typically uses fewer CPU threads
        } else {
            self.performance.cpu_threads
        }
    }

    /// Calculate memory requirement based on model and configuration
    pub fn estimated_memory_usage(&self) -> u64 {
        // Base model memory + context buffer + processing overhead
        let base_memory = self.resource_limits.max_memory_mb / 2; // Conservative estimate
        let context_memory = (self.model_config.max_context_length * 4) as u64 / 1024 / 1024; // Rough estimate
        let processing_memory = (self.performance.batch_size as u64 * 128) / 1024; // Processing overhead
        
        base_memory + context_memory + processing_memory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_ai_config_default() {
        let config = NativeAIConfig::default();
        assert_eq!(config.model_config.model_type, "llama");
        assert_eq!(config.resource_limits.max_memory_mb, 4096);
        assert!(!config.performance.gpu_enabled);
    }

    #[test]
    fn test_config_validation() {
        let mut config = NativeAIConfig::default();
        assert!(config.validate().is_ok());

        // Test invalid model path
        config.model_config.model_path = String::new();
        assert!(config.validate().is_err());

        // Reset and test invalid memory
        config = NativeAIConfig::default();
        config.resource_limits.max_memory_mb = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_model_instance() {
        let capabilities = ModelCapabilities::default();
        let mut model = ModelInstance::new(
            "test".to_string(),
            "/path/to/model".to_string(),
            capabilities,
        );

        assert!(!model.is_ready());
        assert_eq!(model.status(), &ModelStatus::Loading);

        model.set_status(ModelStatus::Ready);
        assert!(model.is_ready());
    }

    #[test]
    fn test_request_info() {
        let mut info = RequestInfo::new(
            "test-123".to_string(),
            AIRequestType::TextGeneration,
            RequestMetadata::default(),
        );

        assert_eq!(info.request_id, "test-123");
        assert!(!info.is_overdue());

        info.set_estimated_completion(Duration::ZERO);
        // With a zero-duration estimate, it's already overdue
        assert!(info.is_overdue());
    }

    #[tokio::test]
    async fn test_provider_state() {
        let state = ProviderState::new();
        
        // Test health
        assert_eq!(state.get_health().await, ProviderHealth::Unknown);
        state.set_health(ProviderHealth::Healthy).await;
        assert_eq!(state.get_health().await, ProviderHealth::Healthy);

        // Test active requests
        let info = RequestInfo::new(
            "test".to_string(),
            AIRequestType::TextGeneration,
            RequestMetadata::default(),
        );
        state.add_active_request("test".to_string(), info).await;
        assert_eq!(state.active_request_count().await, 1);

        let removed = state.remove_active_request("test").await;
        assert!(removed.is_some());
        assert_eq!(state.active_request_count().await, 0);
    }
} 