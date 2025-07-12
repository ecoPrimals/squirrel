# Universal ecoPrimals Patterns Framework

## 🌟 Vision
**Every primal follows the same patterns for configuration, communication, security, and orchestration.**

## 🏗️ Core Architecture Principles

### 1. **Primal Autonomy**
- Each primal is self-contained and independently deployable
- No primal should be dependent on another for core functionality
- Graceful degradation when other primals are unavailable

### 2. **Universal Interfaces**
- Common configuration patterns across all primals
- Standardized service discovery mechanisms
- Consistent error handling and logging
- Unified monitoring and metrics

### 3. **Security-First Design**
- All security operations delegated to Beardog
- Zero hardcoded credentials
- Encrypted primal-to-primal communication
- Audit logging for all inter-primal operations

## 🔧 Universal Pattern Specifications

### 1. Configuration Pattern

**Standard Structure:**
```rust
// Universal configuration for all primals
pub struct PrimalConfig {
    pub identity: PrimalIdentity,
    pub ecosystem: EcosystemConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
    pub discovery: DiscoveryConfig,
}

pub struct PrimalIdentity {
    pub primal_type: PrimalType,
    pub instance_id: String,
    pub version: String,
    pub capabilities: Vec<String>,
}

pub struct EcosystemConfig {
    pub beardog_endpoint: Option<String>,     // Security
    pub songbird_endpoint: Option<String>,    // Orchestration
    pub nestgate_endpoint: Option<String>,    // Storage
    pub toadstool_endpoint: Option<String>,   // Compute
    pub squirrel_endpoint: Option<String>,    // MCP/AI
}
```

**Environment Variables:**
```bash
# Universal primal configuration
PRIMAL_TYPE=squirrel
PRIMAL_INSTANCE_ID=squirrel-001
PRIMAL_VERSION=1.0.0

# Ecosystem endpoints
BEARDOG_ENDPOINT=https://beardog.local:8443
SONGBIRD_ENDPOINT=https://songbird.local:8080
NESTGATE_ENDPOINT=https://nestgate.local:8080
TOADSTOOL_ENDPOINT=https://toadstool.local:8080
SQUIRREL_ENDPOINT=https://squirrel.local:8080

# Security (managed by Beardog)
BEARDOG_API_KEY=${BEARDOG_API_KEY}
BEARDOG_JWT_SECRET_KEY_ID=${BEARDOG_JWT_SECRET_KEY_ID}
```

### 2. Service Discovery Pattern

**Universal Discovery Interface:**
```rust
#[async_trait]
pub trait PrimalDiscovery {
    async fn register_with_ecosystem(&self) -> Result<()>;
    async fn discover_primals(&self) -> Result<Vec<PrimalEndpoint>>;
    async fn health_check(&self) -> Result<PrimalHealth>;
    async fn unregister_from_ecosystem(&self) -> Result<()>;
}

pub struct PrimalEndpoint {
    pub primal_type: PrimalType,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub health_status: HealthStatus,
    pub last_seen: DateTime<Utc>,
}
```

### 3. Security Pattern

**Universal Security Delegation:**
```rust
// All primals delegate security to Beardog
#[async_trait]
pub trait PrimalSecurity {
    async fn authenticate_request(&self, request: &PrimalRequest) -> Result<AuthContext>;
    async fn authorize_operation(&self, auth: &AuthContext, operation: &str) -> Result<bool>;
    async fn audit_log(&self, operation: &str, context: &AuthContext) -> Result<()>;
}

// Implementation delegates to Beardog
impl PrimalSecurity for UniversalSecurityClient {
    async fn authenticate_request(&self, request: &PrimalRequest) -> Result<AuthContext> {
        self.beardog_client.authenticate(request).await
    }
    
    async fn authorize_operation(&self, auth: &AuthContext, operation: &str) -> Result<bool> {
        self.beardog_client.authorize(auth, operation).await
    }
    
    async fn audit_log(&self, operation: &str, context: &AuthContext) -> Result<()> {
        self.beardog_client.audit(operation, context).await
    }
}
```

### 4. Inter-Primal Communication Pattern

**Universal Transport:**
```rust
#[async_trait]
pub trait PrimalTransport {
    async fn send_to_primal(&self, target: PrimalType, message: PrimalMessage) -> Result<PrimalResponse>;
    async fn broadcast_to_ecosystem(&self, message: PrimalMessage) -> Result<Vec<PrimalResponse>>;
    async fn subscribe_to_primal(&self, source: PrimalType, topic: &str) -> Result<PrimalStream>;
}

pub struct PrimalMessage {
    pub id: String,
    pub from: PrimalType,
    pub to: PrimalType,
    pub operation: String,
    pub payload: serde_json::Value,
    pub security_context: Option<AuthContext>,
    pub timestamp: DateTime<Utc>,
}
```

### 5. Monitoring Pattern

**Universal Metrics:**
```rust
pub struct PrimalMetrics {
    pub primal_type: PrimalType,
    pub instance_id: String,
    pub uptime: Duration,
    pub request_count: u64,
    pub error_count: u64,
    pub response_time_avg: Duration,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub ecosystem_connections: Vec<PrimalConnection>,
}

// All primals report to Songbird for orchestration
#[async_trait]
pub trait PrimalMonitoring {
    async fn report_metrics(&self, metrics: &PrimalMetrics) -> Result<()>;
    async fn report_health(&self, health: &PrimalHealth) -> Result<()>;
    async fn report_error(&self, error: &PrimalError) -> Result<()>;
}
```

## 🔄 Implementation Strategy

### Phase 1: Universal Config Framework
1. Create `primal-config` crate with universal patterns
2. Migrate all existing config to universal structure
3. Implement environment variable standardization
4. Add configuration validation

### Phase 2: Service Discovery
1. Implement universal discovery interface
2. Create Songbird service discovery service
3. Migrate existing discovery to universal patterns
4. Add health check standardization

### Phase 3: Security Standardization
1. Audit all security delegation to Beardog
2. Remove hardcoded credentials
3. Implement universal security client
4. Add audit logging to all operations

### Phase 4: Inter-Primal Communication
1. Create universal transport layer
2. Implement secure primal-to-primal communication
3. Add message routing and queuing
4. Implement broadcast and subscription patterns

### Phase 5: Monitoring Integration
1. Standardize metrics collection
2. Implement universal health checks
3. Create monitoring dashboards
4. Add alerting and anomaly detection

## 🎯 Success Metrics

### Configuration Standardization
- [ ] 100% of primals use universal config pattern
- [ ] 0 hardcoded credentials in production
- [ ] Single configuration management system

### Service Discovery
- [ ] All primals auto-register with Songbird
- [ ] Real-time primal availability tracking
- [ ] Automated failover capabilities

### Security
- [ ] 100% security operations through Beardog
- [ ] Full audit trail for all operations
- [ ] Zero security vulnerabilities

### Communication
- [ ] Standardized primal-to-primal protocols
- [ ] Encrypted inter-primal communication
- [ ] Message queuing and reliability

### Monitoring
- [ ] Universal metrics collection
- [ ] Real-time health monitoring
- [ ] Automated alerting system

## 📋 Implementation Checklist

### Immediate Actions (Week 1)
- [ ] Create universal config crate
- [ ] Audit current configuration sprawl
- [ ] Design Songbird orchestration service
- [ ] Standardize environment variables

### Short-term (Month 1)
- [ ] Implement universal discovery interface
- [ ] Create Songbird service discovery
- [ ] Migrate security to universal patterns
- [ ] Add universal transport layer

### Medium-term (Quarter 1)
- [ ] Full inter-primal communication
- [ ] Universal monitoring integration
- [ ] Automated deployment patterns
- [ ] Comprehensive testing framework

---

**🌟 Goal: Every primal in the ecosystem follows the same patterns, making the entire system more maintainable, secure, and scalable.** 