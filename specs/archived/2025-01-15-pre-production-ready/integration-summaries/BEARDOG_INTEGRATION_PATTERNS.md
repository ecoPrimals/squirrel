# Beardog Security Integration Patterns

## 🎯 **Executive Summary**

This document establishes universal patterns for integrating Beardog security across all ecoPrimals. It consolidates the various security approaches currently in use and provides a standardized framework for consistent security implementation.

**Status**: Production-Ready Security Integration Patterns  
**Target**: All ecoPrimals (Squirrel, Songbird, Nestgate, Toadstool, Beardog)  
**Priority**: HIGH - Security Foundation

---

## 🏗️ **Universal Security Architecture**

### **Security Flow**
```
Primal Request → Universal Security Client → Beardog Integration → Auth/Encryption/Audit
                                    ↓
                         Security Context → Authorization → Secure Operations
```

### **Core Principles**
1. **Security-First**: All security operations delegated to Beardog
2. **Zero Hardcoding**: No hardcoded credentials or secrets
3. **Consistent Patterns**: Same security interface across all primals
4. **Graceful Degradation**: Fallback to local security when Beardog unavailable
5. **Audit Trail**: Comprehensive logging for all security operations

---

## 🔧 **Universal Security Configuration**

### **1. Standardized Security Config**

```rust
// File: universal-patterns/src/config/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Beardog service endpoint
    pub beardog_endpoint: Option<Url>,
    
    /// Authentication method
    pub auth_method: AuthMethod,
    
    /// Token/credential storage
    pub credential_storage: CredentialStorage,
    
    /// Encryption settings
    pub encryption: EncryptionConfig,
    
    /// Enable audit logging
    pub audit_logging: bool,
    
    /// Security fallback settings
    pub fallback: SecurityFallback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFallback {
    /// Enable local fallback when Beardog unavailable
    pub enable_local_fallback: bool,
    
    /// Local auth method for fallback
    pub local_auth_method: AuthMethod,
    
    /// Fallback timeout (seconds)
    pub fallback_timeout: u64,
}
```

### **2. Environment Variable Pattern**

```bash
# Universal Beardog Configuration
BEARDOG_ENDPOINT=https://beardog.domain.com:8443
BEARDOG_API_KEY=${BEARDOG_API_KEY}
BEARDOG_SERVICE_ID=${PRIMAL_NAME}-${ENVIRONMENT}
BEARDOG_JWT_SECRET_KEY_ID=${BEARDOG_JWT_SECRET_KEY_ID}

# Fallback Configuration
BEARDOG_FALLBACK_ENABLED=true
BEARDOG_FALLBACK_TIMEOUT=30

# Encryption Configuration
BEARDOG_ENCRYPTION_ALGORITHM=AES256-GCM
BEARDOG_HSM_PROVIDER=default
BEARDOG_KEY_ROTATION_ENABLED=true

# Audit Configuration
BEARDOG_AUDIT_ENABLED=true
BEARDOG_COMPLIANCE_MODE=enterprise
```

### **3. Configuration Builder Pattern**

```rust
// Universal configuration builder for all primals
impl ConfigBuilder {
    /// Configure Beardog security (recommended)
    pub fn beardog_security() -> Self {
        Self::new()
            .beardog_endpoint(env::var("BEARDOG_ENDPOINT").unwrap_or_default())
            .beardog_auth(env::var("BEARDOG_SERVICE_ID").unwrap_or_default())
            .enable_audit_logging()
            .enable_inter_primal_encryption()
            .enable_at_rest_encryption()
            .enable_fallback()
    }
    
    /// Configure development security (local fallback)
    pub fn development_security() -> Self {
        Self::new()
            .beardog_endpoint_optional(env::var("BEARDOG_ENDPOINT").ok())
            .fallback_auth(AuthMethod::Token { token_file: PathBuf::from("dev-token.txt") })
            .enable_local_fallback()
    }
    
    /// Configure production security (strict)
    pub fn production_security() -> Self {
        Self::new()
            .beardog_endpoint(env::var("BEARDOG_ENDPOINT").expect("BEARDOG_ENDPOINT required"))
            .beardog_auth(env::var("BEARDOG_SERVICE_ID").expect("BEARDOG_SERVICE_ID required"))
            .enable_audit_logging()
            .enable_inter_primal_encryption()
            .enable_at_rest_encryption()
            .disable_fallback() // Fail-safe in production
    }
}
```

---

## 🔐 **Universal Security Client**

### **1. Standardized Security Interface**

```rust
// File: universal-patterns/src/security/mod.rs
#[async_trait]
pub trait UniversalSecurityProvider: Send + Sync {
    /// Authenticate credentials
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthResult, SecurityError>;
    
    /// Authorize an action
    async fn authorize(&self, principal: &Principal, action: &str, resource: &str) -> Result<bool, SecurityError>;
    
    /// Encrypt data
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError>;
    
    /// Decrypt data
    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError>;
    
    /// Sign data
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError>;
    
    /// Verify signature
    async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, SecurityError>;
    
    /// Audit log operation
    async fn audit_log(&self, operation: &str, context: &SecurityContext) -> Result<(), SecurityError>;
    
    /// Health check
    async fn health_check(&self) -> Result<SecurityHealth, SecurityError>;
}

/// Universal security client for all primals
pub struct UniversalSecurityClient {
    primary: Arc<dyn UniversalSecurityProvider>,
    fallback: Option<Arc<dyn UniversalSecurityProvider>>,
    config: SecurityConfig,
}

impl UniversalSecurityClient {
    /// Create a new universal security client
    pub async fn new(config: SecurityConfig) -> Result<Self, SecurityError> {
        // Create primary Beardog provider
        let primary = Arc::new(BeardogSecurityProvider::new(config.clone()).await?);
        
        // Create fallback provider if enabled
        let fallback = if config.fallback.enable_local_fallback {
            Some(Arc::new(LocalSecurityProvider::new(config.clone()).await?))
        } else {
            None
        };
        
        Ok(Self {
            primary,
            fallback,
            config,
        })
    }
    
    /// Get security provider with fallback
    async fn get_provider(&self) -> Arc<dyn UniversalSecurityProvider> {
        // Check if primary is healthy
        if let Ok(_) = self.primary.health_check().await {
            return self.primary.clone();
        }
        
        // Fall back to local provider if available
        if let Some(fallback) = &self.fallback {
            if let Ok(_) = fallback.health_check().await {
                tracing::warn!("Falling back to local security provider");
                return fallback.clone();
            }
        }
        
        // Return primary even if unhealthy (will fail gracefully)
        self.primary.clone()
    }
}

#[async_trait]
impl UniversalSecurityProvider for UniversalSecurityClient {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthResult, SecurityError> {
        let provider = self.get_provider().await;
        provider.authenticate(credentials).await
    }
    
    async fn authorize(&self, principal: &Principal, action: &str, resource: &str) -> Result<bool, SecurityError> {
        let provider = self.get_provider().await;
        provider.authorize(principal, action, resource).await
    }
    
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let provider = self.get_provider().await;
        provider.encrypt(data).await
    }
    
    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let provider = self.get_provider().await;
        provider.decrypt(encrypted_data).await
    }
    
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let provider = self.get_provider().await;
        provider.sign(data).await
    }
    
    async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, SecurityError> {
        let provider = self.get_provider().await;
        provider.verify(data, signature).await
    }
    
    async fn audit_log(&self, operation: &str, context: &SecurityContext) -> Result<(), SecurityError> {
        let provider = self.get_provider().await;
        provider.audit_log(operation, context).await
    }
    
    async fn health_check(&self) -> Result<SecurityHealth, SecurityError> {
        let provider = self.get_provider().await;
        provider.health_check().await
    }
}
```

### **2. Beardog HTTP Client Integration**

```rust
// File: universal-patterns/src/security/beardog.rs
pub struct BeardogSecurityProvider {
    client: reqwest::Client,
    config: BeardogConfig,
    endpoint: Url,
    service_id: String,
}

impl BeardogSecurityProvider {
    pub async fn new(config: SecurityConfig) -> Result<Self, SecurityError> {
        let endpoint = config.beardog_endpoint
            .ok_or_else(|| SecurityError::Configuration("Beardog endpoint not configured".to_string()))?;
        
        let service_id = match config.auth_method {
            AuthMethod::Beardog { service_id } => service_id,
            _ => return Err(SecurityError::Configuration("Invalid auth method for Beardog".to_string())),
        };
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| SecurityError::Network(e.to_string()))?;
        
        Ok(Self {
            client,
            config: BeardogConfig::from_security_config(&config)?,
            endpoint,
            service_id,
        })
    }
}

#[async_trait]
impl UniversalSecurityProvider for BeardogSecurityProvider {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthResult, SecurityError> {
        let url = self.endpoint.join("/api/v1/auth/authenticate")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;
        
        let request = AuthRequest {
            service_id: self.service_id.clone(),
            credentials: credentials.clone(),
            timestamp: Utc::now(),
        };
        
        let response = self.client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;
        
        if response.status().is_success() {
            let auth_result: AuthResult = response.json().await
                .map_err(|e| SecurityError::Network(e.to_string()))?;
            
            // Audit log authentication
            self.audit_log("authenticate", &SecurityContext::from_auth_result(&auth_result)).await?;
            
            Ok(auth_result)
        } else {
            Err(SecurityError::Authentication(format!("HTTP {}: {}", response.status(), response.text().await.unwrap_or_default())))
        }
    }
    
    async fn authorize(&self, principal: &Principal, action: &str, resource: &str) -> Result<bool, SecurityError> {
        let url = self.endpoint.join("/api/v1/auth/authorize")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;
        
        let request = AuthorizationRequest {
            service_id: self.service_id.clone(),
            principal: principal.clone(),
            action: action.to_string(),
            resource: resource.to_string(),
            timestamp: Utc::now(),
        };
        
        let response = self.client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;
        
        if response.status().is_success() {
            let result: AuthorizationResult = response.json().await
                .map_err(|e| SecurityError::Network(e.to_string()))?;
            
            // Audit log authorization
            self.audit_log(&format!("authorize:{}", action), &SecurityContext::from_principal(principal)).await?;
            
            Ok(result.authorized)
        } else {
            Err(SecurityError::Authorization(format!("HTTP {}: {}", response.status(), response.text().await.unwrap_or_default())))
        }
    }
    
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let url = self.endpoint.join("/api/v1/crypto/encrypt")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;
        
        let request = EncryptionRequest {
            service_id: self.service_id.clone(),
            data: BASE64_STANDARD.encode(data),
            algorithm: self.config.encryption_algorithm.clone(),
            key_id: self.config.encryption_key_id.clone(),
        };
        
        let response = self.client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;
        
        if response.status().is_success() {
            let result: EncryptionResult = response.json().await
                .map_err(|e| SecurityError::Network(e.to_string()))?;
            
            BASE64_STANDARD.decode(result.encrypted_data)
                .map_err(|e| SecurityError::Encryption(e.to_string()))
        } else {
            Err(SecurityError::Encryption(format!("HTTP {}: {}", response.status(), response.text().await.unwrap_or_default())))
        }
    }
    
    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let url = self.endpoint.join("/api/v1/crypto/decrypt")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;
        
        let request = DecryptionRequest {
            service_id: self.service_id.clone(),
            encrypted_data: BASE64_STANDARD.encode(encrypted_data),
            key_id: self.config.encryption_key_id.clone(),
        };
        
        let response = self.client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;
        
        if response.status().is_success() {
            let result: DecryptionResult = response.json().await
                .map_err(|e| SecurityError::Network(e.to_string()))?;
            
            BASE64_STANDARD.decode(result.decrypted_data)
                .map_err(|e| SecurityError::Encryption(e.to_string()))
        } else {
            Err(SecurityError::Encryption(format!("HTTP {}: {}", response.status(), response.text().await.unwrap_or_default())))
        }
    }
    
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let url = self.endpoint.join("/api/v1/crypto/sign")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;
        
        let request = SigningRequest {
            service_id: self.service_id.clone(),
            data: BASE64_STANDARD.encode(data),
            algorithm: self.config.signing_algorithm.clone(),
            key_id: self.config.signing_key_id.clone(),
        };
        
        let response = self.client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;
        
        if response.status().is_success() {
            let result: SigningResult = response.json().await
                .map_err(|e| SecurityError::Network(e.to_string()))?;
            
            BASE64_STANDARD.decode(result.signature)
                .map_err(|e| SecurityError::Encryption(e.to_string()))
        } else {
            Err(SecurityError::Encryption(format!("HTTP {}: {}", response.status(), response.text().await.unwrap_or_default())))
        }
    }
    
    async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, SecurityError> {
        let url = self.endpoint.join("/api/v1/crypto/verify")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;
        
        let request = VerificationRequest {
            service_id: self.service_id.clone(),
            data: BASE64_STANDARD.encode(data),
            signature: BASE64_STANDARD.encode(signature),
            key_id: self.config.signing_key_id.clone(),
        };
        
        let response = self.client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;
        
        if response.status().is_success() {
            let result: VerificationResult = response.json().await
                .map_err(|e| SecurityError::Network(e.to_string()))?;
            
            Ok(result.valid)
        } else {
            Err(SecurityError::Encryption(format!("HTTP {}: {}", response.status(), response.text().await.unwrap_or_default())))
        }
    }
    
    async fn audit_log(&self, operation: &str, context: &SecurityContext) -> Result<(), SecurityError> {
        let url = self.endpoint.join("/api/v1/audit/log")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;
        
        let request = AuditLogRequest {
            service_id: self.service_id.clone(),
            operation: operation.to_string(),
            context: context.clone(),
            timestamp: Utc::now(),
        };
        
        let response = self.client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            tracing::warn!("Audit log failed: HTTP {}", response.status());
        }
        
        Ok(())
    }
    
    async fn health_check(&self) -> Result<SecurityHealth, SecurityError> {
        let url = self.endpoint.join("/api/v1/health")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;
        
        let response = self.client
            .get(url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;
        
        if response.status().is_success() {
            let health: SecurityHealth = response.json().await
                .map_err(|e| SecurityError::Network(e.to_string()))?;
            Ok(health)
        } else {
            Err(SecurityError::Network(format!("HTTP {}", response.status())))
        }
    }
}
```

---

## 🔄 **Primal Integration Patterns**

### **1. Primal Initialization Pattern**

```rust
// Universal pattern for all primals
pub struct PrimalSecurityManager {
    security_client: Arc<UniversalSecurityClient>,
    config: SecurityConfig,
}

impl PrimalSecurityManager {
    pub async fn new() -> Result<Self, SecurityError> {
        // Load configuration from environment
        let config = SecurityConfig::from_env()?;
        
        // Create universal security client
        let security_client = Arc::new(UniversalSecurityClient::new(config.clone()).await?);
        
        Ok(Self {
            security_client,
            config,
        })
    }
    
    /// Get security client for operations
    pub fn security_client(&self) -> Arc<UniversalSecurityClient> {
        self.security_client.clone()
    }
    
    /// Initialize primal with security
    pub async fn initialize_primal<P: Primal>(&self, primal: &mut P) -> Result<(), SecurityError> {
        // Set security client
        primal.set_security_client(self.security_client.clone());
        
        // Perform initial authentication
        let credentials = self.load_service_credentials().await?;
        let auth_result = self.security_client.authenticate(&credentials).await?;
        
        // Store authentication context
        primal.set_auth_context(auth_result.context);
        
        // Register with Beardog
        self.register_primal_with_beardog(primal).await?;
        
        Ok(())
    }
    
    async fn load_service_credentials(&self) -> Result<Credentials, SecurityError> {
        // Implementation depends on credential storage method
        match &self.config.credential_storage {
            CredentialStorage::Environment => {
                Ok(Credentials::ApiKey {
                    key: env::var("BEARDOG_API_KEY").map_err(|_| SecurityError::Configuration("BEARDOG_API_KEY not set".to_string()))?,
                    service_id: env::var("BEARDOG_SERVICE_ID").map_err(|_| SecurityError::Configuration("BEARDOG_SERVICE_ID not set".to_string()))?,
                })
            }
            CredentialStorage::File { path } => {
                let content = tokio::fs::read_to_string(path).await
                    .map_err(|e| SecurityError::Configuration(e.to_string()))?;
                let credentials: Credentials = serde_json::from_str(&content)
                    .map_err(|e| SecurityError::Configuration(e.to_string()))?;
                Ok(credentials)
            }
            CredentialStorage::Beardog => {
                // Beardog-managed credentials (bootstrap scenario)
                Ok(Credentials::Bootstrap {
                    service_id: self.config.service_id.clone(),
                })
            }
        }
    }
    
    async fn register_primal_with_beardog<P: Primal>(&self, primal: &P) -> Result<(), SecurityError> {
        let registration = PrimalRegistration {
            primal_id: primal.id().to_string(),
            primal_type: primal.primal_type(),
            capabilities: primal.capabilities(),
            endpoints: primal.endpoints(),
            security_requirements: primal.security_requirements(),
        };
        
        // Register with Beardog
        let url = self.config.beardog_endpoint.as_ref().unwrap().join("/api/v1/primals/register")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;
        
        let response = reqwest::Client::new()
            .post(url)
            .json(&registration)
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(SecurityError::Network(format!("Registration failed: HTTP {}", response.status())));
        }
        
        Ok(())
    }
}
```

### **2. Request Authentication Pattern**

```rust
// Middleware pattern for all primals
pub struct SecurityMiddleware {
    security_client: Arc<UniversalSecurityClient>,
}

impl SecurityMiddleware {
    pub fn new(security_client: Arc<UniversalSecurityClient>) -> Self {
        Self { security_client }
    }
    
    /// Authenticate incoming request
    pub async fn authenticate_request(&self, request: &PrimalRequest) -> Result<SecurityContext, SecurityError> {
        // Extract credentials from request
        let credentials = self.extract_credentials(request)?;
        
        // Authenticate with Beardog
        let auth_result = self.security_client.authenticate(&credentials).await?;
        
        // Create security context
        let context = SecurityContext {
            principal: auth_result.principal,
            token: auth_result.token,
            expires_at: auth_result.expires_at,
            permissions: auth_result.permissions,
            metadata: auth_result.metadata,
        };
        
        Ok(context)
    }
    
    /// Authorize specific operation
    pub async fn authorize_operation(&self, context: &SecurityContext, operation: &str, resource: &str) -> Result<bool, SecurityError> {
        self.security_client.authorize(&context.principal, operation, resource).await
    }
    
    /// Security wrapper for operations
    pub async fn secure_operation<F, R>(&self, request: &PrimalRequest, operation: &str, resource: &str, func: F) -> Result<R, SecurityError>
    where
        F: FnOnce(SecurityContext) -> Result<R, SecurityError>,
    {
        // Authenticate
        let context = self.authenticate_request(request).await?;
        
        // Authorize
        let authorized = self.authorize_operation(&context, operation, resource).await?;
        if !authorized {
            return Err(SecurityError::Authorization(format!("Access denied for operation: {}", operation)));
        }
        
        // Audit log
        self.security_client.audit_log(&format!("operation:{}", operation), &context).await?;
        
        // Execute operation
        func(context)
    }
    
    fn extract_credentials(&self, request: &PrimalRequest) -> Result<Credentials, SecurityError> {
        // Extract from headers, tokens, certificates, etc.
        if let Some(token) = request.headers.get("Authorization") {
            if token.starts_with("Bearer ") {
                return Ok(Credentials::Bearer {
                    token: token.strip_prefix("Bearer ").unwrap().to_string(),
                });
            }
        }
        
        if let Some(api_key) = request.headers.get("X-API-Key") {
            return Ok(Credentials::ApiKey {
                key: api_key.to_string(),
                service_id: request.headers.get("X-Service-ID").unwrap_or("unknown").to_string(),
            });
        }
        
        Err(SecurityError::InvalidCredentials("No valid credentials found".to_string()))
    }
}
```

### **3. Inter-Primal Communication Pattern**

```rust
// Secure communication between primals
pub struct SecurePrimalClient {
    security_client: Arc<UniversalSecurityClient>,
    http_client: reqwest::Client,
    service_credentials: Credentials,
}

impl SecurePrimalClient {
    pub async fn new(security_client: Arc<UniversalSecurityClient>) -> Result<Self, SecurityError> {
        let service_credentials = Credentials::ServiceAccount {
            service_id: env::var("PRIMAL_SERVICE_ID").map_err(|_| SecurityError::Configuration("PRIMAL_SERVICE_ID not set".to_string()))?,
            api_key: env::var("PRIMAL_API_KEY").map_err(|_| SecurityError::Configuration("PRIMAL_API_KEY not set".to_string()))?,
        };
        
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| SecurityError::Network(e.to_string()))?;
        
        Ok(Self {
            security_client,
            http_client,
            service_credentials,
        })
    }
    
    /// Make secure request to another primal
    pub async fn secure_request<T: Serialize, R: DeserializeOwned>(&self, endpoint: &str, request: &T) -> Result<R, SecurityError> {
        // Authenticate service
        let auth_result = self.security_client.authenticate(&self.service_credentials).await?;
        
        // Encrypt request payload
        let request_data = serde_json::to_vec(request)
            .map_err(|e| SecurityError::Other(e.to_string()))?;
        let encrypted_data = self.security_client.encrypt(&request_data).await?;
        
        // Make HTTP request with authentication
        let response = self.http_client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", auth_result.token))
            .header("Content-Type", "application/octet-stream")
            .header("X-Encrypted", "true")
            .body(encrypted_data)
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(SecurityError::Network(format!("HTTP {}", response.status())));
        }
        
        // Decrypt response
        let encrypted_response = response.bytes().await
            .map_err(|e| SecurityError::Network(e.to_string()))?;
        let decrypted_data = self.security_client.decrypt(&encrypted_response).await?;
        
        // Deserialize response
        let result: R = serde_json::from_slice(&decrypted_data)
            .map_err(|e| SecurityError::Other(e.to_string()))?;
        
        Ok(result)
    }
}
```

---

## 📊 **Security Monitoring & Audit**

### **1. Security Metrics**

```rust
// Security metrics collection
pub struct SecurityMetrics {
    auth_attempts: Counter,
    auth_failures: Counter,
    auth_latency: Histogram,
    crypto_operations: Counter,
    audit_events: Counter,
}

impl SecurityMetrics {
    pub fn new() -> Self {
        Self {
            auth_attempts: Counter::new("beardog_auth_attempts_total", "Total authentication attempts"),
            auth_failures: Counter::new("beardog_auth_failures_total", "Total authentication failures"),
            auth_latency: Histogram::new("beardog_auth_latency_seconds", "Authentication latency"),
            crypto_operations: Counter::new("beardog_crypto_operations_total", "Total cryptographic operations"),
            audit_events: Counter::new("beardog_audit_events_total", "Total audit events"),
        }
    }
    
    pub fn record_auth_attempt(&self, success: bool, latency: Duration) {
        self.auth_attempts.increment();
        if !success {
            self.auth_failures.increment();
        }
        self.auth_latency.observe(latency.as_secs_f64());
    }
    
    pub fn record_crypto_operation(&self, operation_type: &str) {
        self.crypto_operations.increment_by(1, &[("operation", operation_type)]);
    }
    
    pub fn record_audit_event(&self, event_type: &str) {
        self.audit_events.increment_by(1, &[("event_type", event_type)]);
    }
}
```

### **2. Security Health Dashboard**

```rust
// Security health monitoring
pub struct SecurityHealthMonitor {
    security_client: Arc<UniversalSecurityClient>,
    metrics: SecurityMetrics,
}

impl SecurityHealthMonitor {
    pub async fn health_check(&self) -> SecurityHealthReport {
        let mut report = SecurityHealthReport::new();
        
        // Check Beardog connectivity
        match self.security_client.health_check().await {
            Ok(health) => {
                report.beardog_status = health.status;
                report.beardog_latency = health.latency;
            }
            Err(e) => {
                report.beardog_status = HealthStatus::Unhealthy;
                report.beardog_error = Some(e.to_string());
            }
        }
        
        // Check authentication
        let test_auth = self.test_authentication().await;
        report.auth_status = if test_auth.is_ok() { HealthStatus::Healthy } else { HealthStatus::Unhealthy };
        
        // Check encryption
        let test_crypto = self.test_encryption().await;
        report.crypto_status = if test_crypto.is_ok() { HealthStatus::Healthy } else { HealthStatus::Unhealthy };
        
        report
    }
    
    async fn test_authentication(&self) -> Result<(), SecurityError> {
        let test_credentials = Credentials::Test {
            service_id: "health-check".to_string(),
        };
        
        self.security_client.authenticate(&test_credentials).await?;
        Ok(())
    }
    
    async fn test_encryption(&self) -> Result<(), SecurityError> {
        let test_data = b"health-check-data";
        let encrypted = self.security_client.encrypt(test_data).await?;
        let decrypted = self.security_client.decrypt(&encrypted).await?;
        
        if decrypted == test_data {
            Ok(())
        } else {
            Err(SecurityError::Encryption("Encryption round-trip failed".to_string()))
        }
    }
}
```

---

## 🚀 **Implementation Guide**

### **Phase 1: Core Integration (Week 1)**

1. **Update Universal Patterns**
   - [ ] Implement `UniversalSecurityClient` in `universal-patterns/src/security/mod.rs`
   - [ ] Add Beardog HTTP client integration
   - [ ] Update configuration patterns

2. **Update Existing Integrations**
   - [ ] Refactor `core/auth/src/lib.rs` to use universal patterns
   - [ ] Update MCP security configurations
   - [ ] Align configuration builders

3. **Environment Configuration**
   - [ ] Create standardized environment variable templates
   - [ ] Update deployment scripts
   - [ ] Add configuration validation

### **Phase 2: Testing & Validation (Week 2)**

1. **Integration Testing**
   - [ ] Create comprehensive security integration tests
   - [ ] Test fallback mechanisms
   - [ ] Validate all security operations

2. **Performance Testing**
   - [ ] Benchmark authentication operations
   - [ ] Test encryption performance
   - [ ] Validate scalability

3. **Security Testing**
   - [ ] Penetration testing
   - [ ] Security audit
   - [ ] Compliance validation

### **Phase 3: Documentation & Training (Week 3)**

1. **Developer Documentation**
   - [ ] API documentation
   - [ ] Integration examples
   - [ ] Best practices guide

2. **Deployment Documentation**
   - [ ] Environment setup guides
   - [ ] Configuration examples
   - [ ] Troubleshooting guide

3. **Team Training**
   - [ ] Security patterns training
   - [ ] Integration workshops
   - [ ] Best practices review

---

## 🎯 **Success Criteria**

### **Technical Success**
- [ ] All primals use unified security patterns
- [ ] Zero hardcoded credentials across codebase
- [ ] Comprehensive security test coverage (>95%)
- [ ] Sub-100ms authentication latency
- [ ] 99.9% security operation success rate

### **Operational Success**
- [ ] Automated security monitoring
- [ ] Comprehensive audit trail
- [ ] Real-time security alerts
- [ ] Fallback mechanisms tested
- [ ] Production deployment validated

### **Compliance Success**
- [ ] Enterprise security standards met
- [ ] Audit compliance verified
- [ ] Security policy enforcement
- [ ] Incident response procedures
- [ ] Documentation complete

---

## 🔍 **Monitoring & Alerting**

### **Key Metrics**
- Authentication success rate
- Authorization latency
- Encryption operation throughput
- Audit log completeness
- Security incident count

### **Alert Thresholds**
- Authentication failure rate > 5%
- Average latency > 500ms
- Beardog connectivity issues
- Encryption failures
- Audit log gaps

### **Dashboard Components**
- Security health overview
- Authentication metrics
- Encryption performance
- Audit trail summary
- Incident tracking

---

This comprehensive Beardog integration pattern provides a solid foundation for secure, scalable, and maintainable security across all ecoPrimals. The standardized approach ensures consistency while providing flexibility for primal-specific requirements. 