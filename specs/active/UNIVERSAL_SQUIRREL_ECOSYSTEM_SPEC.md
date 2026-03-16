---
description: ENFORCE universal ecosystem integration patterns for Squirrel AI primal following Songbird standards
globs: ["squirrel/crates/**/*.rs", "squirrel/crates/main/src/**/*.rs"]
crossRefs:
  - UNIVERSAL_PATTERNS_SPECIFICATION.md
  - context/README.md
  - plugins/README.md
---

# Universal Squirrel Ecosystem Integration Specification

## Context
- When implementing Squirrel AI primal as part of the ecoPrimals ecosystem
- When integrating with Songbird service mesh for universal orchestration
- When providing AI capabilities through standardized ecosystem APIs
- When supporting multi-instance deployments with context-aware routing
- When implementing configuration-driven, environment-agnostic services

## Requirements

### Universal Ecosystem Integration
- Implement `EcosystemIntegration` trait for Songbird communication
- Implement `UniversalPrimalProvider` trait for primal functionality
- Support standardized request/response formats across ecosystem
- Enable dynamic service discovery and registration
- Provide context-aware routing and load balancing

### Configuration-Driven Architecture
- Eliminate ALL hardcoded values (ports, endpoints, timeouts)
- Use environment-driven configuration following biomeOS patterns
- Support dynamic port allocation through Songbird
- Enable runtime configuration updates without restarts
- Implement feature flags for capability management

### AI Service Abstraction
- Provide universal AI interfaces for any model or provider
- Support dynamic provider registration and discovery
- Enable capability-based routing (text, code, multimodal)
- Implement intelligent load balancing and failover
- Support both streaming and batch inference modes

## Architecture

### Universal Primal Provider Implementation
```rust
use async_trait::async_trait;
use ecosystem_api::*;

pub struct SquirrelEcosystemProvider {
    primal_id: String,
    instance_id: String,
    context: PrimalContext,
    config: UniversalConfig,
    ai_coordinator: Arc<AICoordinator>,
    service_mesh_client: Arc<ServiceMeshClient>,
    capabilities: Arc<RwLock<Vec<PrimalCapability>>>,
    health_monitor: Arc<HealthMonitor>,
}

#[async_trait]
impl UniversalPrimalProvider for SquirrelEcosystemProvider {
    fn primal_id(&self) -> &str { &self.primal_id }
    fn instance_id(&self) -> &str { &self.instance_id }
    fn primal_type(&self) -> PrimalType { PrimalType::Squirrel }
    fn context(&self) -> &PrimalContext { &self.context }
    
    fn capabilities(&self) -> Vec<PrimalCapability> {
        vec![
            PrimalCapability::ModelInference { 
                models: vec!["gpt-4".to_string(), "claude-3".to_string(), "gemini-pro".to_string()] 
            },
            PrimalCapability::AgentFramework { mcp_support: true },
            PrimalCapability::MachineLearning { training_support: false },
            PrimalCapability::NaturalLanguage { 
                languages: vec!["en".to_string(), "es".to_string(), "fr".to_string()] 
            },
        ]
    }
    
    async fn handle_primal_request(&self, request: PrimalRequest) -> UniversalResult<PrimalResponse> {
        match request.operation.as_str() {
            "ai_inference" => self.handle_ai_inference(request).await,
            "agent_coordinate" => self.handle_agent_coordination(request).await,
            "context_analyze" => self.handle_context_analysis(request).await,
            "model_discovery" => self.handle_model_discovery(request).await,
            _ => Err(PrimalError::UnsupportedOperation(request.operation))
        }
    }
    
    async fn register_with_songbird(&mut self, endpoint: &str) -> UniversalResult<String> {
        let registration = EcosystemServiceRegistration {
            service_id: format!("squirrel-{}", self.instance_id),
            primal_type: PrimalType::Squirrel,
            biome_id: self.context.biome_id.clone(),
            capabilities: ServiceCapabilities {
                core: vec!["ai_inference".to_string(), "mcp_protocol".to_string()],
                extended: vec!["agent_coordination".to_string(), "context_analysis".to_string()],
                integrations: vec!["openai".to_string(), "anthropic".to_string()],
            },
            endpoints: ServiceEndpoints {
                health: format!("{}/health", self.get_base_url()),
                metrics: format!("{}/metrics", self.get_base_url()),
                admin: format!("{}/admin", self.get_base_url()),
                websocket: Some(format!("{}/ws", self.get_base_url())),
            },
            resource_requirements: self.get_resource_requirements(),
            security_config: self.get_security_config(),
            health_check: self.get_health_check_config(),
            metadata: self.get_metadata(),
        };
        
        self.service_mesh_client.register_service(endpoint, registration).await
    }
}

#[async_trait]
impl EcosystemIntegration for SquirrelEcosystemProvider {
    async fn handle_ecosystem_request(&self, request: EcosystemRequest) -> Result<EcosystemResponse, EcosystemError> {
        // Validate security context
        self.validate_security_context(&request.security_context)?;
        
        // Route to appropriate handler
        match request.operation.as_str() {
            "ai_chat" => self.handle_ai_chat_request(request).await,
            "model_inference" => self.handle_model_inference_request(request).await,
            "agent_spawn" => self.handle_agent_spawn_request(request).await,
            "context_query" => self.handle_context_query_request(request).await,
            _ => Err(EcosystemError::UnsupportedOperation),
        }
    }
    
    async fn report_health(&self, health: HealthStatus) -> Result<(), EcosystemError> {
        self.health_monitor.report_health(health).await
            .map_err(|e| EcosystemError::HealthReportFailed(e.to_string()))
    }
}
```

### Universal Configuration Management
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquirrelUniversalConfig {
    // Service configuration (NO hardcoded values)
    pub service: ServiceConfig,
    
    // Songbird integration settings
    pub songbird: SongbirdConfig,
    
    // AI provider configurations
    pub ai_providers: HashMap<String, AIProviderConfig>,
    
    // Security configuration
    pub security: SecurityConfig,
    
    // Resource limits and requirements
    pub resources: ResourceConfig,
    
    // Feature flags
    pub features: FeatureFlags,
    
    // MCP-specific configuration
    pub mcp: MCPConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    // Environment-driven, no defaults
    pub name: String,                    // From SQUIRREL_SERVICE_NAME
    pub version: String,                 // From CARGO_PKG_VERSION
    pub description: String,             // From SQUIRREL_SERVICE_DESCRIPTION
    pub bind_address: String,            // From SQUIRREL_BIND_ADDRESS
    pub port: u16,                       // From SQUIRREL_PORT or dynamic from Songbird
    pub log_level: String,               // From SQUIRREL_LOG_LEVEL
    pub instance_id: String,             // From SQUIRREL_INSTANCE_ID or generated
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProviderConfig {
    pub provider_type: String,           // "openai", "anthropic", "ollama", "custom"
    pub api_key: Option<String>,         // From env vars
    pub endpoint: Option<String>,        // From env vars
    pub model_name: String,              // From env vars
    pub max_tokens: Option<u32>,         // From env vars
    pub temperature: Option<f32>,        // From env vars
    pub enabled: bool,                   // From feature flags
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    pub protocol_version: String,        // From SQUIRREL_MCP_VERSION
    pub max_message_size: usize,         // From SQUIRREL_MCP_MAX_MESSAGE_SIZE
    pub timeout_ms: u64,                 // From SQUIRREL_MCP_TIMEOUT_MS
    pub transport_types: Vec<String>,    // From SQUIRREL_MCP_TRANSPORTS
    pub security_level: String,          // From SQUIRREL_MCP_SECURITY_LEVEL
}

impl SquirrelUniversalConfig {
    /// Load configuration from environment with NO defaults
    pub fn from_environment() -> Result<Self, ConfigError> {
        let service = ServiceConfig {
            name: std::env::var("SQUIRREL_SERVICE_NAME")?,
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: std::env::var("SQUIRREL_SERVICE_DESCRIPTION")?,
            bind_address: std::env::var("SQUIRREL_BIND_ADDRESS")?,
            port: std::env::var("SQUIRREL_PORT")?.parse()?,
            log_level: std::env::var("SQUIRREL_LOG_LEVEL")?,
            instance_id: std::env::var("SQUIRREL_INSTANCE_ID")
                .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string()),
        };
        
        let songbird = SongbirdConfig {
            discovery_endpoint: std::env::var("SONGBIRD_DISCOVERY_ENDPOINT")?,
            registration_endpoint: std::env::var("SONGBIRD_REGISTRATION_ENDPOINT")?,
            health_endpoint: std::env::var("SONGBIRD_HEALTH_ENDPOINT")?,
            auth_token: std::env::var("SONGBIRD_AUTH_TOKEN").ok(),
            retry_config: RetryConfig {
                max_retries: std::env::var("SONGBIRD_MAX_RETRIES")?.parse()?,
                initial_delay_ms: std::env::var("SONGBIRD_INITIAL_DELAY_MS")?.parse()?,
                max_delay_ms: std::env::var("SONGBIRD_MAX_DELAY_MS")?.parse()?,
                backoff_multiplier: std::env::var("SONGBIRD_BACKOFF_MULTIPLIER")?.parse()?,
            },
        };
        
        // Load AI providers dynamically from environment
        let ai_providers = Self::load_ai_providers_from_env()?;
        
        // Load other configurations...
        
        Ok(Self {
            service,
            songbird,
            ai_providers,
            security: SecurityConfig::from_environment()?,
            resources: ResourceConfig::from_environment()?,
            features: FeatureFlags::from_environment()?,
            mcp: MCPConfig::from_environment()?,
        })
    }
    
    /// Load AI providers dynamically from environment variables
    fn load_ai_providers_from_env() -> Result<HashMap<String, AIProviderConfig>, ConfigError> {
        let mut providers = HashMap::new();
        
        // Scan for AI provider configurations
        for (key, value) in std::env::vars() {
            if key.starts_with("SQUIRREL_AI_PROVIDER_") {
                let provider_name = key
                    .strip_prefix("SQUIRREL_AI_PROVIDER_")
                    .unwrap()
                    .to_lowercase();
                
                let config = AIProviderConfig {
                    provider_type: value,
                    api_key: std::env::var(format!("SQUIRREL_AI_{}_API_KEY", provider_name.to_uppercase())).ok(),
                    endpoint: std::env::var(format!("SQUIRREL_AI_{}_ENDPOINT", provider_name.to_uppercase())).ok(),
                    model_name: std::env::var(format!("SQUIRREL_AI_{}_MODEL", provider_name.to_uppercase()))?,
                    max_tokens: std::env::var(format!("SQUIRREL_AI_{}_MAX_TOKENS", provider_name.to_uppercase()))
                        .ok().and_then(|v| v.parse().ok()),
                    temperature: std::env::var(format!("SQUIRREL_AI_{}_TEMPERATURE", provider_name.to_uppercase()))
                        .ok().and_then(|v| v.parse().ok()),
                    enabled: std::env::var(format!("SQUIRREL_AI_{}_ENABLED", provider_name.to_uppercase()))
                        .unwrap_or_else(|_| "true".to_string()).parse()?,
                };
                
                providers.insert(provider_name, config);
            }
        }
        
        Ok(providers)
    }
}
```

### Universal AI Coordination
```rust
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub struct UniversalAICoordinator {
    providers: Arc<RwLock<HashMap<String, Arc<dyn AIProvider>>>>,
    routing_strategy: RoutingStrategy,
    load_balancer: Arc<LoadBalancer>,
    health_monitor: Arc<HealthMonitor>,
    config: AICoordinatorConfig,
}

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn get_capabilities(&self) -> Vec<AICapability>;
    async fn health_check(&self) -> ProviderHealth;
    async fn inference(&self, request: InferenceRequest) -> Result<InferenceResponse, AIError>;
    async fn stream_inference(&self, request: InferenceRequest) -> Result<InferenceStream, AIError>;
    fn provider_name(&self) -> &str;
    fn provider_type(&self) -> &str;
}

impl UniversalAICoordinator {
    pub fn new(config: AICoordinatorConfig) -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            routing_strategy: config.routing_strategy.clone(),
            load_balancer: Arc::new(LoadBalancer::new(config.load_balancer_config.clone())),
            health_monitor: Arc::new(HealthMonitor::new(config.health_config.clone())),
            config,
        }
    }
    
    /// Register AI provider dynamically
    pub async fn register_provider(&self, name: String, provider: Arc<dyn AIProvider>) -> Result<(), AIError> {
        let capabilities = provider.get_capabilities().await;
        let health = provider.health_check().await;
        
        if health.is_healthy() {
            let mut providers = self.providers.write().await;
            providers.insert(name.clone(), provider);
            
            // Update routing tables
            self.load_balancer.add_provider(name, capabilities).await;
            
            tracing::info!("Registered AI provider: {}", name);
            Ok(())
        } else {
            Err(AIError::ProviderUnhealthy(name))
        }
    }
    
    /// Route AI request to best provider
    pub async fn route_request(&self, request: AIRequest) -> Result<AIResponse, AIError> {
        let provider_name = self.select_provider(&request).await?;
        let provider = self.get_provider(&provider_name).await?;
        
        // Execute request with retries and circuit breaker
        let response = self.execute_with_resilience(provider, request).await?;
        
        // Update metrics
        self.update_metrics(&provider_name, &response).await;
        
        Ok(response)
    }
    
    /// Select best provider based on request and current load
    async fn select_provider(&self, request: &AIRequest) -> Result<String, AIError> {
        match self.routing_strategy {
            RoutingStrategy::RoundRobin => {
                self.load_balancer.round_robin_select(request).await
            }
            RoutingStrategy::LeastConnections => {
                self.load_balancer.least_connections_select(request).await
            }
            RoutingStrategy::CapabilityBased => {
                self.load_balancer.capability_based_select(request).await
            }
            RoutingStrategy::PerformanceBased => {
                self.load_balancer.performance_based_select(request).await
            }
        }
    }
    
    /// Execute request with circuit breaker and retries
    async fn execute_with_resilience(
        &self,
        provider: Arc<dyn AIProvider>,
        request: AIRequest,
    ) -> Result<AIResponse, AIError> {
        let circuit_breaker = self.get_circuit_breaker(&provider.provider_name()).await;
        
        circuit_breaker.execute_with_retry(|| async {
            provider.inference(request.clone()).await
        }).await
    }
}
```

### Dynamic Provider Discovery
```rust
use std::collections::HashMap;
use std::sync::Arc;

pub struct ProviderDiscovery {
    discovered_providers: Arc<RwLock<HashMap<String, ProviderInfo>>>,
    service_mesh_client: Arc<ServiceMeshClient>,
    config: DiscoveryConfig,
}

impl ProviderDiscovery {
    /// Discover AI providers in the ecosystem
    pub async fn discover_providers(&self) -> Result<Vec<ProviderInfo>, DiscoveryError> {
        let mut providers = Vec::new();
        
        // Discover from Songbird service mesh
        let services = self.service_mesh_client.discover_services(ServiceQuery {
            service_type: Some("ai_provider".to_string()),
            capabilities: vec!["model_inference".to_string()],
            ..Default::default()
        }).await?;
        
        for service in services {
            let provider_info = ProviderInfo {
                name: service.name,
                endpoint: service.endpoint,
                provider_type: service.metadata.get("provider_type").cloned().unwrap_or_default(),
                capabilities: service.capabilities,
                health_status: service.health_status,
                metadata: service.metadata,
            };
            
            providers.push(provider_info);
        }
        
        // Discover from environment variables
        let env_providers = self.discover_from_environment().await?;
        providers.extend(env_providers);
        
        // Update cache
        let mut discovered = self.discovered_providers.write().await;
        for provider in &providers {
            discovered.insert(provider.name.clone(), provider.clone());
        }
        
        Ok(providers)
    }
    
    /// Discover providers from environment variables
    async fn discover_from_environment(&self) -> Result<Vec<ProviderInfo>, DiscoveryError> {
        let mut providers = Vec::new();
        
        // Scan environment for provider configurations
        for (key, _value) in std::env::vars() {
            if key.starts_with("SQUIRREL_AI_PROVIDER_") {
                let provider_name = key
                    .strip_prefix("SQUIRREL_AI_PROVIDER_")
                    .unwrap()
                    .to_lowercase();
                
                let provider_info = ProviderInfo {
                    name: provider_name.clone(),
                    endpoint: std::env::var(format!("SQUIRREL_AI_{}_ENDPOINT", provider_name.to_uppercase()))
                        .unwrap_or_else(|_| "local".to_string()),
                    provider_type: std::env::var(format!("SQUIRREL_AI_{}_TYPE", provider_name.to_uppercase()))
                        .unwrap_or_else(|_| "unknown".to_string()),
                    capabilities: vec!["model_inference".to_string()],
                    health_status: "unknown".to_string(),
                    metadata: HashMap::new(),
                };
                
                providers.push(provider_info);
            }
        }
        
        Ok(providers)
    }
}
```

## Implementation Guidelines

### Environment Configuration
```bash
# Service configuration
export SQUIRREL_SERVICE_NAME="squirrel-ai"
export SQUIRREL_SERVICE_DESCRIPTION="Universal AI coordination primal"
export SQUIRREL_BIND_ADDRESS="0.0.0.0"
export SQUIRREL_PORT="0"  # Dynamic port from Songbird
export SQUIRREL_LOG_LEVEL="info"
export SQUIRREL_INSTANCE_ID="squirrel-$(hostname)-$(date +%s)"

# Songbird integration
export SONGBIRD_DISCOVERY_ENDPOINT="http://songbird:8080/api/v1/discovery"
export SONGBIRD_REGISTRATION_ENDPOINT="http://songbird:8080/api/v1/register"
export SONGBIRD_HEALTH_ENDPOINT="http://songbird:8080/api/v1/health"
export SONGBIRD_AUTH_TOKEN="your-auth-token"

# AI provider configurations
export SQUIRREL_AI_PROVIDER_OPENAI="openai"
export SQUIRREL_AI_OPENAI_API_KEY="your-openai-key"
export SQUIRREL_AI_OPENAI_MODEL="gpt-4"
export SQUIRREL_AI_OPENAI_ENABLED="true"

export SQUIRREL_AI_PROVIDER_ANTHROPIC="anthropic"
export SQUIRREL_AI_ANTHROPIC_API_KEY="your-anthropic-key"
export SQUIRREL_AI_ANTHROPIC_MODEL="claude-3-sonnet-20240229"
export SQUIRREL_AI_ANTHROPIC_ENABLED="true"

export SQUIRREL_AI_PROVIDER_OLLAMA="ollama"
export SQUIRREL_AI_OLLAMA_ENDPOINT="http://ollama:11434"
export SQUIRREL_AI_OLLAMA_MODEL="llama2:7b"
export SQUIRREL_AI_OLLAMA_ENABLED="true"

# Security configuration
export SQUIRREL_SECURITY_LEVEL="internal"
export SQUIRREL_TLS_ENABLED="true"
export SQUIRREL_MTLS_REQUIRED="false"
export SQUIRREL_TRUST_DOMAIN="ecosystem.local"

# Feature flags
export SQUIRREL_FEATURES_DEVELOPMENT_MODE="false"
export SQUIRREL_FEATURES_DEBUG_LOGGING="false"
export SQUIRREL_FEATURES_METRICS_ENABLED="true"
export SQUIRREL_FEATURES_TRACING_ENABLED="true"
```

### Deployment Patterns
```yaml
# biome.yaml - BiomeOS deployment configuration
apiVersion: biomeos.io/v1
kind: PrimalManifest
metadata:
  name: squirrel-ai
  namespace: ecosystem
spec:
  primal:
    type: squirrel
    version: "1.0.0"
    instances: 3
    
  resources:
    cpu: "2"
    memory: "4Gi"
    storage: "10Gi"
    
  networking:
    port: 0  # Dynamic from Songbird
    protocol: "http"
    
  environment:
    - name: SQUIRREL_SERVICE_NAME
      value: "squirrel-ai"
    - name: SONGBIRD_DISCOVERY_ENDPOINT
      valueFrom:
        serviceRef:
          name: "songbird"
          port: 8080
          path: "/api/v1/discovery"
    
  dependencies:
    - name: "songbird"
      type: "required"
      version: ">=1.0.0"
    - name: "beardog"
      type: "optional"
      version: ">=1.0.0"
      
  healthcheck:
    path: "/health"
    interval: "30s"
    timeout: "10s"
    retries: 3
    
  autoscaling:
    enabled: true
    minReplicas: 1
    maxReplicas: 10
    targetCPUUtilization: 70
    targetMemoryUtilization: 80
```

## Examples

### Basic Usage
```rust
use squirrel_ecosystem::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment
    let config = SquirrelUniversalConfig::from_environment()?;
    
    // Create universal provider
    let mut provider = SquirrelEcosystemProvider::new(config).await?;
    
    // Register with Songbird
    let songbird_endpoint = std::env::var("SONGBIRD_DISCOVERY_ENDPOINT")?;
    provider.register_with_songbird(&songbird_endpoint).await?;
    
    // Start serving requests
    provider.start().await?;
    
    Ok(())
}
```

### AI Request Handling
```rust
use squirrel_ecosystem::*;

impl SquirrelEcosystemProvider {
    async fn handle_ai_chat_request(&self, request: EcosystemRequest) -> Result<EcosystemResponse, EcosystemError> {
        // Parse request
        let chat_request: ChatRequest = serde_json::from_value(request.payload)?;
        
        // Route to appropriate AI provider
        let ai_request = AIRequest {
            prompt: chat_request.message,
            capabilities: vec!["text_generation".to_string()],
            context: chat_request.context,
            preferences: chat_request.preferences,
        };
        
        // Execute with coordination
        let response = self.ai_coordinator.route_request(ai_request).await?;
        
        // Return ecosystem response
        Ok(EcosystemResponse {
            request_id: request.request_id,
            status: ResponseStatus::Success,
            payload: serde_json::to_value(response)?,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        })
    }
}
```

## Migration Guide

### From Current Implementation
1. **Remove hardcoded values**: Replace all localhost:8080 with environment variables
2. **Implement universal traits**: Add `EcosystemIntegration` and `UniversalPrimalProvider`
3. **Update configuration**: Use `SquirrelUniversalConfig` instead of hardcoded configs
4. **Add provider discovery**: Implement dynamic AI provider discovery
5. **Update communication**: Use standardized ecosystem request/response formats

### Testing Strategy
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ecosystem_test_utils::*;
    
    #[tokio::test]
    async fn test_ecosystem_integration() {
        // Create test environment
        let test_env = EcosystemTestEnvironment::new().await;
        
        // Test service registration
        let provider = create_test_provider().await;
        let registration_id = provider.register_with_songbird(&test_env.songbird_endpoint).await.unwrap();
        
        // Test request handling
        let request = create_test_request();
        let response = provider.handle_ecosystem_request(request).await.unwrap();
        
        assert_eq!(response.status, ResponseStatus::Success);
    }
    
    #[tokio::test]
    async fn test_ai_provider_discovery() {
        // Test dynamic provider discovery
        let discovery = ProviderDiscovery::new(test_config()).await;
        let providers = discovery.discover_providers().await.unwrap();
        
        assert!(!providers.is_empty());
    }
}
```

<version>1.0.0</version> 