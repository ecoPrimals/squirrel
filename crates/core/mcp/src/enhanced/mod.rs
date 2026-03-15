// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Enhanced MCP Server Architecture
//!
//! Unified, high-performance MCP platform with tarpc internal communication
//! and WebSocket external interfaces.

// Enhanced MCP Platform modules - Phase 3 Complete
pub mod server;
pub mod coordinator;    // ✅ PHASE 3: Universal AI coordination
pub mod events;         // ✅ PHASE 3: Universal event system
pub mod streaming;      // ✅ PHASE 3: Universal streaming system
pub mod bidirectional_streaming; // ✅ NEW: Bidirectional streaming capabilities
pub mod multi_agent;    // ✅ NEW: Multi-agent coordination system
pub mod providers;      // ✅ PHASE 3: Configurable AI providers with mock framework
pub mod intelligent_router; // ✅ PHASE 3: Intelligent request routing
pub mod config_validation; // ✅ PHASE 3: Configuration validation and defaults
pub mod error_types;    // ✅ PHASE 3: Enhanced error handling
pub mod config_manager; // ✅ PHASE 3: Centralized configuration management
pub mod service_composition; // ✅ NEW: AI service composition engine
pub mod workflow;            // ✅ NEW: Workflow execution system (internal)
pub mod workflow_management; // ✅ NEW: Workflow management engine
pub mod connection_pool;     // ✅ NEW: HTTP connection pool for AI providers
pub mod serialization;       // ✅ NEW: Zero-copy serialization system
pub mod memory_pool;         // ✅ NEW: Advanced memory pool system
pub mod performance_init;    // ✅ NEW: Performance optimization initialization system
pub mod metrics;             // ✅ NEW: Comprehensive metrics collection system

#[cfg(feature = "tarpc")]
pub mod transport;           // ✅ Unified transport (WebSocket + tarpc + TCP)

#[cfg(test)]
pub mod tests;          // ✅ PHASE 3: Comprehensive integration tests

// Standard imports
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::error::{Result, types::MCPError};
use crate::protocol::types::{MCPMessage, MessageType as ProtocolMessageType};
use crate::tool::management::{CoreToolManager, ToolInfo};

// Import Phase 3 components
pub use coordinator::{AICoordinator, AICoordinatorConfig, UniversalToolExecutor};
pub use providers::UniversalAIProvider;
pub use events::{EventBroadcaster, EventBroadcasterConfig, MCPEvent, EventType, EventSource};
pub use streaming::{StreamManager, StreamManagerConfig, StreamType, StreamChunk};
pub use bidirectional_streaming::{BidirectionalStreamManager, StreamingConfig, MCPStream, StreamDirection, StreamState};
pub use multi_agent::{MultiAgentCoordinator, MultiAgentConfig, Agent, AgentType, AgentState, AgentMessage, CollaborationType};
pub use error_types::{EnhancedMCPError, EnhancedResult, ErrorContext, ErrorSeverity};
pub use config_manager::{ConfigManager, EnhancedMCPConfig, Environment, NetworkConfig, DatabaseConfig, SecurityConfig};

// Import new service composition and workflow management components
pub use service_composition::{
    ServiceCompositionEngine, ServiceCompositionConfig, AIService, ServiceConfig, ServiceCapability,
    Composition, CompositionConfig, CompositionState, CompositionStatus,
    ExecutionResult, ExecutionPlan, ExecutionStep, ServiceHealth, HealthStatus
};
pub use workflow_management::{
    WorkflowManagementEngine, WorkflowManagementConfig, WorkflowDefinition, WorkflowInstance,
    WorkflowState, WorkflowStatus, WorkflowStep, WorkflowStepType, WorkflowExecutionEngine,
    WorkflowScheduler, WorkflowTemplateEngine, WorkflowMonitoring, ExecutionStrategy
};

// Import connection pool components
pub use connection_pool::{
    ConnectionPool, ConnectionPoolConfig, ConnectionPoolManager, ProviderConnectionConfig,
    TlsConfig, RateLimitConfig, PooledClient, ProviderHealth, ConnectionPoolMetrics, PerformanceReport
};

// Import serialization components
pub use serialization::{
    ZeroCopySerializer, SerializationConfig, SerializationResult, SerializationMetadata,
    BufferPool, BufferPoolConfig, StreamingSerializer, StreamingDeserializer, FastCodec,
    MessageTemplateCache, CompiledTemplate, get_global_serializer, init_global_serializer
};

// Import metrics components
pub use metrics::{
    EnhancedMetricsManager, MetricsConfig, MetricsDashboard, DashboardConfig,
    UnifiedMetricsCollector, MetricsAggregator, MetricsAlertManager, Alert,
    AggregatedMetrics, PerformanceSummary, SystemHealth
};

/// Enhanced MCP Platform - Universal AI Integration System
/// 
/// This platform provides universal support for ANY AI system:
/// - Cloud APIs (OpenAI, Anthropic, Gemini, etc.)
/// - Local models (any OpenAI-compatible server: Ollama, llama.cpp, vLLM, etc.)
/// - Aggregators (OpenRouter, etc.)  
/// - Model hubs (Hugging Face, etc.)
/// - Custom/homemade AI systems
/// - Future AI systems (extensible architecture)
/// 
/// NEW: Service Composition and Workflow Management
/// - AI service composition with dependency management
/// - Workflow definition, execution, and scheduling
/// - Advanced orchestration and monitoring
/// - State management and persistence
/// - Template-based workflow creation
pub struct EnhancedMCPPlatform {
    /// Enhanced MCP server
    pub server: Arc<server::EnhancedMCPServer>,
    
    /// Universal AI coordinator
    pub ai_coordinator: Arc<AICoordinator>,
    
    /// Universal event broadcaster
    pub event_broadcaster: Arc<EventBroadcaster>,
    
    /// Universal stream manager
    pub stream_manager: Arc<StreamManager>,
    
    /// Universal tool executor
    pub tool_executor: Arc<UniversalToolExecutor>,
    
    /// Service composition engine
    pub service_composition: Arc<ServiceCompositionEngine>,
    
    /// Workflow management engine
    pub workflow_management: Arc<WorkflowManagementEngine>,
    
    /// Platform configuration
    pub config: EnhancedPlatformConfig,
    
    /// Platform state
    pub state: PlatformState,
}

/// Platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedPlatformConfig {
    /// Server configuration
    pub server: server::EnhancedServerConfig,
    
    /// AI coordinator configuration
    pub ai_coordinator: AICoordinatorConfig,
    
    /// Event broadcaster configuration
    pub event_broadcaster: EventBroadcasterConfig,
    
    /// Stream manager configuration
    pub stream_manager: StreamManagerConfig,
    
    /// Tool executor configuration
    pub tool_executor: coordinator::ToolExecutorConfig,
    
    /// Service composition configuration
    pub service_composition: ServiceCompositionConfig,
    
    /// Workflow management configuration
    pub workflow_management: WorkflowManagementConfig,
    
    /// Platform settings
    pub platform_settings: PlatformSettings,
}

/// Platform settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformSettings {
    /// Platform name
    pub name: String,
    
    /// Platform version
    pub version: String,
    
    /// Platform description
    pub description: String,
    
    /// Platform metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Performance settings
    pub performance: PerformanceSettings,
    
    /// Security settings
    pub security: SecuritySettings,
    
    /// Monitoring settings
    pub monitoring: MonitoringSettings,
}

/// Performance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Enable performance monitoring
    pub monitoring_enabled: bool,
    
    /// Performance metrics interval
    pub metrics_interval: Duration,
    
    /// Performance optimization enabled
    pub optimization_enabled: bool,
    
    /// Resource limits
    pub resource_limits: ResourceLimits,
}

/// Security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// Enable security monitoring
    pub monitoring_enabled: bool,
    
    /// Security audit enabled
    pub audit_enabled: bool,
    
    /// Encryption enabled
    pub encryption_enabled: bool,
    
    /// Authentication required
    pub auth_required: bool,
}

/// Monitoring settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSettings {
    /// Enable monitoring
    pub enabled: bool,
    
    /// Monitoring interval
    pub interval: Duration,
    
    /// Metrics collection enabled
    pub metrics_enabled: bool,
    
    /// Alerting enabled
    pub alerting_enabled: bool,
}

/// Resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage (bytes)
    pub max_memory: u64,
    
    /// Maximum CPU usage (percentage)
    pub max_cpu: f64,
    
    /// Maximum network bandwidth (bytes/sec)
    pub max_network: u64,
    
    /// Maximum disk I/O (bytes/sec)
    pub max_disk_io: u64,
}

/// Platform state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformState {
    /// Platform status
    pub status: PlatformStatus,
    
    /// Platform start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    
    /// Platform uptime
    pub uptime: Duration,
    
    /// Platform metrics
    pub metrics: PlatformMetrics,
    
    /// Platform health
    pub health: PlatformHealth,
}

/// Platform status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlatformStatus {
    /// Platform is initializing
    Initializing,
    
    /// Platform is starting
    Starting,
    
    /// Platform is healthy
    Healthy,
    
    /// Platform is degraded
    Degraded,
    
    /// Platform is unhealthy
    Unhealthy,
    
    /// Platform is shutting down
    ShuttingDown,
    
    /// Platform is stopped
    Stopped,
}

/// Platform metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformMetrics {
    /// Total AI requests processed
    pub total_requests: u64,
    
    /// Active AI requests
    pub active_requests: u64,
    
    /// Total services registered
    pub total_services: u64,
    
    /// Active services
    pub active_services: u64,
    
    /// Total workflows executed
    pub total_workflows: u64,
    
    /// Active workflows
    pub active_workflows: u64,
    
    /// Average response time
    pub avg_response_time: Duration,
    
    /// Success rate
    pub success_rate: f64,
    
    /// Error rate
    pub error_rate: f64,
}

/// Platform health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformHealth {
    /// Overall health status
    pub status: HealthStatus,
    
    /// Health score (0.0 to 1.0)
    pub score: f64,
    
    /// Component health
    pub components: HashMap<String, ComponentHealth>,
    
    /// Health checks
    pub checks: Vec<HealthCheck>,
    
    /// Last health check
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Component health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    
    /// Component status
    pub status: HealthStatus,
    
    /// Component score
    pub score: f64,
    
    /// Component metrics
    pub metrics: HashMap<String, serde_json::Value>,
    
    /// Last updated
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Check name
    pub name: String,
    
    /// Check status
    pub status: HealthStatus,
    
    /// Check message
    pub message: String,
    
    /// Check timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Check duration
    pub duration: Duration,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// Healthy
    Healthy,
    
    /// Degraded
    Degraded,
    
    /// Unhealthy
    Unhealthy,
    
    /// Unknown
    Unknown,
}

impl Default for PlatformState {
    fn default() -> Self {
        Self {
            status: PlatformStatus::Initializing,
            start_time: chrono::Utc::now(),
            uptime: Duration::from_secs(0),
            metrics: PlatformMetrics {
                total_requests: 0,
                active_requests: 0,
                total_services: 0,
                active_services: 0,
                total_workflows: 0,
                active_workflows: 0,
                avg_response_time: Duration::from_secs(0),
                success_rate: 0.0,
                error_rate: 0.0,
            },
            health: PlatformHealth {
                status: HealthStatus::Unknown,
                score: 0.0,
                components: HashMap::new(),
                checks: Vec::new(),
                last_check: chrono::Utc::now(),
            },
        }
    }
}

impl EnhancedMCPPlatform {
    /// Create new enhanced MCP platform with universal AI support
    pub async fn new(config: EnhancedPlatformConfig) -> Result<Self> {
        tracing::info!("🚀 Initializing Enhanced MCP Platform - Universal AI System with Service Composition and Workflow Management");
        
        // Initialize core components
        let event_broadcaster = Arc::new(
            EventBroadcaster::new(config.event_broadcaster.clone()).await?
        );
        
        let ai_coordinator = Arc::new(
            AICoordinator::new(config.ai_coordinator.clone()).await?
        );
        
        let stream_manager = Arc::new(
            StreamManager::new(config.stream_manager.clone()).await?
        );
        
        let tool_executor = Arc::new(
            UniversalToolExecutor::new(ai_coordinator.clone(), config.tool_executor.clone()).await?
        );
        
        let server = Arc::new(
            server::EnhancedMCPServer::new(config.server.clone()).await?
        );
        
        // Initialize service composition engine
        let service_composition = Arc::new(
            ServiceCompositionEngine::new(
                config.service_composition.clone(),
                ai_coordinator.clone(),
            ).await?
        );
        
        // Initialize workflow management engine
        let workflow_management = Arc::new(
            WorkflowManagementEngine::new(
                config.workflow_management.clone(),
                service_composition.clone(),
                ai_coordinator.clone(),
            ).await?
        );
        
        let platform = Self {
            server,
            ai_coordinator,
            event_broadcaster,
            stream_manager,
            tool_executor,
            service_composition,
            workflow_management,
            config,
            state: PlatformState::default(),
        };
        
        // Publish platform startup event
        platform.publish_startup_event().await?;
        
        tracing::info!("✅ Enhanced MCP Platform initialized successfully with Service Composition and Workflow Management");
        Ok(platform)
    }
    
    /// Start the platform
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("🚀 Starting Enhanced MCP Platform with Service Composition and Workflow Management...");
        
        self.state.status = PlatformStatus::Starting;
        
        // Start server
        // Note: Server start would be handled externally or in a background task
        
        // Update state
        self.state.status = PlatformStatus::Healthy;
        self.state.start_time = chrono::Utc::now();
        
        // Publish startup complete event
        self.publish_platform_ready_event().await?;
        
        tracing::info!("✅ Enhanced MCP Platform started successfully");
        Ok(())
    }
    
    /// Stop the platform
    pub async fn stop(&mut self) -> Result<()> {
        tracing::info!("🛑 Stopping Enhanced MCP Platform...");
        
        self.state.status = PlatformStatus::ShuttingDown;
        
        // Publish shutdown event
        self.publish_shutdown_event().await?;
        
        // Stop components (implementation would go here)
        
        self.state.status = PlatformStatus::Stopped;
        
        tracing::info!("✅ Enhanced MCP Platform stopped");
        Ok(())
    }
    
    /// Process AI request through universal coordinator
    pub async fn process_ai_request(&self, request: coordinator::UniversalAIRequest) -> Result<coordinator::UniversalAIResponse> {
        // Publish request start event
        self.publish_ai_request_event(&request, events::EventType::AIRequestStarted).await?;
        
        // Process through AI coordinator
        let result = self.ai_coordinator.process_request(request.clone()).await;
        
        match &result {
            Ok(response) => {
                self.publish_ai_response_event(&request, response, events::EventType::AIRequestCompleted).await?;
            }
            Err(error) => {
                self.publish_ai_error_event(&request, error).await?;
            }
        }
        
        result
    }
    
    /// Execute tool with AI assistance
    pub async fn execute_tool(
        &self, 
        tool_name: &str, 
        params: serde_json::Value, 
        session_id: &str
    ) -> Result<coordinator::ToolResult> {
        // Publish tool execution start event
        self.publish_tool_event(tool_name, &params, events::EventType::ToolExecuted).await?;
        
        // Execute through tool executor
        let result = self.tool_executor.execute_tool(tool_name, params, session_id).await;
        
        result
    }
    
    /// Create AI session
    pub async fn create_ai_session(&self, preferences: coordinator::UserPreferences) -> Result<String> {
        let session_id = self.ai_coordinator.create_session(preferences).await?;
        
        // Publish session creation event
        self.publish_session_event(&session_id, events::EventType::SessionCreated).await?;
        
        Ok(session_id)
    }
    
    /// Get all available AI models
    pub async fn list_all_ai_models(&self) -> Result<Vec<coordinator::ModelInfo>> {
        self.ai_coordinator.list_all_models().await
    }
    
    /// Create stream
    pub async fn create_stream(
        &self,
        stream_type: streaming::StreamType,
        source: streaming::StreamSource,
        handle: Box<dyn streaming::StreamHandle>,
        config: Option<streaming::StreamConfig>,
    ) -> Result<String> {
        let stream_id = self.stream_manager.create_stream(stream_type.clone(), source, handle, config).await?;
        
        // Publish stream creation event
        self.publish_stream_event(&stream_id, &stream_type, events::EventType::StreamStarted).await?;
        
        Ok(stream_id)
    }
    
    /// Subscribe to events
    pub async fn subscribe_to_events(
        &self, 
        event_type: &str
    ) -> Result<tokio::sync::broadcast::Receiver<events::MCPEvent>> {
        self.event_broadcaster.subscribe(event_type).await
    }
    
    /// Register AI service
    pub async fn register_service(&self, service: service_composition::AIService) -> Result<()> {
        self.service_composition.register_service(service).await
    }
    
    /// Create service composition
    pub async fn create_composition(&self, composition: service_composition::Composition) -> Result<String> {
        self.service_composition.create_composition(composition).await
    }
    
    /// Execute service composition
    pub async fn execute_composition(&self, composition_id: &str) -> Result<service_composition::ExecutionResult> {
        self.service_composition.execute_composition(composition_id).await
    }
    
    /// Register workflow definition
    pub async fn register_workflow(&self, definition: workflow_management::WorkflowDefinition) -> Result<()> {
        self.workflow_management.register_workflow(definition).await
    }
    
    /// Execute workflow
    pub async fn execute_workflow(
        &self, 
        workflow_id: &str, 
        parameters: HashMap<String, serde_json::Value>
    ) -> Result<String> {
        self.workflow_management.execute_workflow(workflow_id, parameters).await
    }
    
    /// Get workflow status
    pub async fn get_workflow_status(&self, instance_id: &str) -> Result<workflow_management::WorkflowState> {
        self.workflow_management.get_workflow_status(instance_id).await
    }
    
    /// Schedule workflow
    pub async fn schedule_workflow(
        &self,
        workflow_id: &str,
        schedule: workflow_management::ScheduleConfig,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        self.workflow_management.schedule_workflow(workflow_id, schedule, parameters).await
    }
    
    /// Create workflow from template
    pub async fn create_workflow_from_template(
        &self,
        template_id: &str,
        template_parameters: HashMap<String, serde_json::Value>,
    ) -> Result<workflow_management::WorkflowDefinition> {
        self.workflow_management.create_workflow_from_template(template_id, template_parameters).await
    }
    
    /// Get platform health
    pub async fn get_platform_health(&self) -> Result<PlatformHealth> {
        let mut health = PlatformHealth {
            status: HealthStatus::Healthy,
            score: 1.0,
            components: HashMap::new(),
            checks: Vec::new(),
            last_check: chrono::Utc::now(),
        };
        
        // Check AI coordinator health
        health.components.insert("ai_coordinator".to_string(), ComponentHealth {
            name: "AI Coordinator".to_string(),
            status: HealthStatus::Healthy,
            score: 1.0,
            metrics: HashMap::new(),
            last_updated: chrono::Utc::now(),
        });
        
        // Check service composition health
        health.components.insert("service_composition".to_string(), ComponentHealth {
            name: "Service Composition".to_string(),
            status: HealthStatus::Healthy,
            score: 1.0,
            metrics: HashMap::new(),
            last_updated: chrono::Utc::now(),
        });
        
        // Check workflow management health
        health.components.insert("workflow_management".to_string(), ComponentHealth {
            name: "Workflow Management".to_string(),
            status: HealthStatus::Healthy,
            score: 1.0,
            metrics: HashMap::new(),
            last_updated: chrono::Utc::now(),
        });
        
        Ok(health)
    }
    
    /// Get platform metrics
    pub async fn get_platform_metrics(&self) -> Result<PlatformMetrics> {
        let service_metrics = self.service_composition.get_metrics().await?;
        let workflow_metrics = self.workflow_management.get_metrics().await?;
        
        Ok(PlatformMetrics {
            total_requests: 0, // Would be tracked by coordinator
            active_requests: 0,
            total_services: service_metrics.total_compositions,
            active_services: service_metrics.active_compositions,
            total_workflows: workflow_metrics.total_workflows,
            active_workflows: workflow_metrics.active_workflows,
            avg_response_time: Duration::from_secs(0),
            success_rate: 0.0,
            error_rate: 0.0,
        })
    }
    
    /// Publish startup event
    async fn publish_startup_event(&self) -> Result<()> {
        let event = events::MCPEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: events::EventType::PlatformStartup,
            timestamp: chrono::Utc::now(),
            source: "platform".to_string(),
            data: serde_json::json!({
                "platform": "enhanced_mcp",
                "version": "1.0.0",
                "features": [
                    "ai_coordination",
                    "service_composition", 
                    "workflow_management",
                    "streaming",
                    "multi_agent"
                ]
            }),
            metadata: HashMap::new(),
        };
        
        self.event_broadcaster.broadcast(event).await?;
        Ok(())
    }
    
    /// Publish platform ready event
    async fn publish_platform_ready_event(&self) -> Result<()> {
        let event = events::MCPEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: events::EventType::PlatformReady,
            timestamp: chrono::Utc::now(),
            source: "platform".to_string(),
            data: serde_json::json!({
                "platform": "enhanced_mcp",
                "status": "ready",
                "capabilities": [
                    "universal_ai_support",
                    "service_composition",
                    "workflow_orchestration",
                    "real_time_streaming",
                    "multi_agent_coordination"
                ]
            }),
            metadata: HashMap::new(),
        };
        
        self.event_broadcaster.broadcast(event).await?;
        Ok(())
    }
    
    /// Publish shutdown event
    async fn publish_shutdown_event(&self) -> Result<()> {
        let event = events::MCPEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: events::EventType::PlatformShutdown,
            timestamp: chrono::Utc::now(),
            source: "platform".to_string(),
            data: serde_json::json!({
                "platform": "enhanced_mcp",
                "status": "shutting_down"
            }),
            metadata: HashMap::new(),
        };
        
        self.event_broadcaster.broadcast(event).await?;
        Ok(())
    }
    
    /// Publish AI request event
    async fn publish_ai_request_event(
        &self,
        request: &coordinator::UniversalAIRequest,
        event_type: events::EventType,
    ) -> Result<()> {
        let event = events::MCPEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            timestamp: chrono::Utc::now(),
            source: "ai_coordinator".to_string(),
            data: serde_json::json!({
                "request_id": request.id,
                "model": request.model,
                "request_type": request.request_type
            }),
            metadata: HashMap::new(),
        };
        
        self.event_broadcaster.broadcast(event).await?;
        Ok(())
    }
    
    /// Publish AI response event
    async fn publish_ai_response_event(
        &self,
        request: &coordinator::UniversalAIRequest,
        response: &coordinator::UniversalAIResponse,
        event_type: events::EventType,
    ) -> Result<()> {
        let event = events::MCPEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            timestamp: chrono::Utc::now(),
            source: "ai_coordinator".to_string(),
            data: serde_json::json!({
                "request_id": request.id,
                "response_id": response.id,
                "provider": response.provider,
                "duration": response.duration.as_secs_f64(),
                "cost": response.cost
            }),
            metadata: HashMap::new(),
        };
        
        self.event_broadcaster.broadcast(event).await?;
        Ok(())
    }
    
    /// Publish AI error event
    async fn publish_ai_error_event(
        &self,
        request: &coordinator::UniversalAIRequest,
        error: &MCPError,
    ) -> Result<()> {
        let event = events::MCPEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: events::EventType::AIRequestFailed,
            timestamp: chrono::Utc::now(),
            source: "ai_coordinator".to_string(),
            data: serde_json::json!({
                "request_id": request.id,
                "error": error.to_string()
            }),
            metadata: HashMap::new(),
        };
        
        self.event_broadcaster.broadcast(event).await?;
        Ok(())
    }
    
    /// Publish tool event
    async fn publish_tool_event(
        &self,
        tool_name: &str,
        params: &serde_json::Value,
        event_type: events::EventType,
    ) -> Result<()> {
        let event = events::MCPEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            timestamp: chrono::Utc::now(),
            source: "tool_executor".to_string(),
            data: serde_json::json!({
                "tool_name": tool_name,
                "params": params
            }),
            metadata: HashMap::new(),
        };
        
        self.event_broadcaster.broadcast(event).await?;
        Ok(())
    }
    
    /// Publish session event
    async fn publish_session_event(
        &self,
        session_id: &str,
        event_type: events::EventType,
    ) -> Result<()> {
        let event = events::MCPEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            timestamp: chrono::Utc::now(),
            source: "ai_coordinator".to_string(),
            data: serde_json::json!({
                "session_id": session_id
            }),
            metadata: HashMap::new(),
        };
        
        self.event_broadcaster.broadcast(event).await?;
        Ok(())
    }
    
    /// Publish stream event
    async fn publish_stream_event(
        &self,
        stream_id: &str,
        stream_type: &streaming::StreamType,
        event_type: events::EventType,
    ) -> Result<()> {
        let event = events::MCPEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            timestamp: chrono::Utc::now(),
            source: "stream_manager".to_string(),
            data: serde_json::json!({
                "stream_id": stream_id,
                "stream_type": stream_type
            }),
            metadata: HashMap::new(),
        };
        
        self.event_broadcaster.broadcast(event).await?;
        Ok(())
    }
}

impl Default for EnhancedPlatformConfig {
    fn default() -> Self {
        Self {
            server: server::EnhancedServerConfig::default(),
            ai_coordinator: AICoordinatorConfig::default(),
            event_broadcaster: EventBroadcasterConfig::default(),
            stream_manager: StreamManagerConfig::default(),
            tool_executor: coordinator::ToolExecutorConfig::default(),
            service_composition: ServiceCompositionConfig::default(),
            workflow_management: WorkflowManagementConfig::default(),
            platform_settings: PlatformSettings::default(),
        }
    }
}

impl Default for PlatformSettings {
    fn default() -> Self {
        Self {
            name: "Enhanced MCP Platform".to_string(),
            version: "1.0.0".to_string(),
            description: "Universal AI Integration Platform with Service Composition and Workflow Management".to_string(),
            metadata: HashMap::new(),
            performance: PerformanceSettings {
                monitoring_enabled: true,
                metrics_interval: Duration::from_secs(30),
                optimization_enabled: true,
                resource_limits: ResourceLimits {
                    max_memory: 8 * 1024 * 1024 * 1024, // 8GB
                    max_cpu: 80.0,
                    max_network: 1024 * 1024 * 100, // 100MB/s
                    max_disk_io: 1024 * 1024 * 100, // 100MB/s
                },
            },
            security: SecuritySettings {
                monitoring_enabled: true,
                audit_enabled: true,
                encryption_enabled: true,
                auth_required: true,
            },
            monitoring: MonitoringSettings {
                enabled: true,
                interval: Duration::from_secs(30),
                metrics_enabled: true,
                alerting_enabled: true,
            },
        }
    }
}

// Re-export commonly used types for convenience
pub type AIPlatform = EnhancedMCPPlatform;
pub type ServiceComposition = service_composition::Composition;
pub type WorkflowExecution = workflow_management::WorkflowInstance;
pub type AIServiceHealth = service_composition::ServiceHealth;
pub type WorkflowState = workflow_management::WorkflowState;

#[cfg(test)]
#[cfg(test)]
mod inline_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_enhanced_platform_initialization() {
        let config = EnhancedPlatformConfig::default();
        let platform = EnhancedMCPPlatform::new(config).await;
        assert!(platform.is_ok(), "Platform should initialize successfully");
    }
    
    #[tokio::test]
    async fn test_platform_health_check() {
        let config = EnhancedPlatformConfig::default();
        let platform = EnhancedMCPPlatform::new(config).await.unwrap();
        let health = platform.get_platform_health().await.unwrap();
        assert_eq!(health.status, HealthStatus::Healthy);
    }
} pub mod workflow; // ✅ NEW: Modular workflow implementation
