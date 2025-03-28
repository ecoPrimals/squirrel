//! Security policies module for the MCP system
//!
//! This module provides functionality for defining, managing, and enforcing
//! security policies for the MCP system.

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug, error, instrument};

use crate::error::{Result, MCPError};
use crate::error::types::SecurityError;
use crate::security::types::{Action, Permission, PermissionContext};
use crate::types::SecurityLevel;

/// Security policy types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PolicyType {
    /// Authentication policy
    Authentication,
    /// Authorization policy
    Authorization,
    /// Rate limiting policy
    RateLimit,
    /// Password policy
    Password,
    /// Session policy
    Session,
    /// Encryption policy
    Encryption,
    /// General policy
    General,
}

/// Security policy enforcement level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EnforcementLevel {
    /// Advisory only, violations are logged but not enforced
    Advisory,
    /// Warning, violations are logged with warnings but allowed
    Warning,
    /// Enforced, violations are denied and logged
    Enforced,
    /// Critical, violations are denied, logged, and trigger alerts
    Critical,
}

/// Security policy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Policy ID
    pub id: String,
    
    /// Policy name
    pub name: String,
    
    /// Policy description
    pub description: Option<String>,
    
    /// Policy type
    pub policy_type: PolicyType,
    
    /// Enforcement level
    pub enforcement_level: EnforcementLevel,
    
    /// Creation time
    pub created_at: DateTime<Utc>,
    
    /// Last update time
    pub updated_at: DateTime<Utc>,
    
    /// Policy settings (key-value pairs)
    pub settings: HashMap<String, String>,
    
    /// Required permissions to modify this policy
    pub required_permissions: HashSet<Permission>,
    
    /// Security level required to modify this policy
    pub security_level: SecurityLevel,
    
    /// Whether the policy is enabled
    pub enabled: bool,
}

/// Result of a policy evaluation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyEvaluationResult {
    /// Policy passed
    Passed,
    /// Policy warning
    Warning(String),
    /// Policy violation
    Violation(String),
}

/// Policy evaluation context
#[derive(Debug, Clone)]
pub struct PolicyContext {
    /// User ID
    pub user_id: Option<String>,
    
    /// IP address
    pub ip_address: Option<String>,
    
    /// Session token
    pub session_token: Option<String>,
    
    /// Request info
    pub request_info: HashMap<String, String>,
    
    /// Security level
    pub security_level: SecurityLevel,
    
    /// Current time
    pub current_time: DateTime<Utc>,
}

impl Default for PolicyContext {
    fn default() -> Self {
        Self {
            user_id: None,
            ip_address: None,
            session_token: None,
            request_info: HashMap::new(),
            security_level: SecurityLevel::Standard,
            current_time: Utc::now(),
        }
    }
}

/// Security policy manager
#[derive(Debug)]
pub struct PolicyManager {
    /// Policies by ID
    policies: RwLock<HashMap<String, SecurityPolicy>>,
    
    /// Policies by type
    policies_by_type: RwLock<HashMap<PolicyType, HashSet<String>>>,
    
    /// Policy evaluation handlers
    handlers: RwLock<HashMap<String, Arc<dyn PolicyEvaluator + Send + Sync>>>,
    
    /// Whether to enable policy enforcement
    enforcement_enabled: bool,
}

/// Security policy evaluator trait
#[async_trait::async_trait]
pub trait PolicyEvaluator: std::fmt::Debug {
    /// Evaluate a policy against a context
    async fn evaluate(&self, policy: &SecurityPolicy, context: &PolicyContext) -> Result<PolicyEvaluationResult>;
    
    /// Get policy type supported by this evaluator
    fn policy_type(&self) -> PolicyType;
    
    /// Get policy evaluator ID
    fn id(&self) -> String;
}

impl PolicyManager {
    /// Create a new policy manager
    #[instrument]
    pub fn new(enforcement_enabled: bool) -> Self {
        info!("Creating new policy manager with enforcement_enabled={}", enforcement_enabled);
        Self {
            policies: RwLock::new(HashMap::new()),
            policies_by_type: RwLock::new(HashMap::new()),
            handlers: RwLock::new(HashMap::new()),
            enforcement_enabled,
        }
    }
    
    /// Add a policy evaluator
    #[instrument(skip(evaluator))]
    pub async fn add_evaluator(&self, evaluator: Arc<dyn PolicyEvaluator + Send + Sync>) -> Result<()> {
        let evaluator_id = evaluator.id();
        let policy_type = evaluator.policy_type();
        
        debug!("Adding policy evaluator: id={}, type={:?}", evaluator_id, policy_type);
        
        let mut handlers = self.handlers.write().await;
        if handlers.contains_key(&evaluator_id) {
            return Err(MCPError::Security(SecurityError::DuplicateIDError("Policy evaluator with that ID already exists".into())));
        }
        
        handlers.insert(evaluator_id, evaluator);
        Ok(())
    }
    
    /// Add a policy
    #[instrument(skip(policy))]
    pub async fn add_policy(&self, policy: SecurityPolicy) -> Result<()> {
        if policy.id.is_empty() {
            return Err(MCPError::Security(SecurityError::ValidationError("Policy ID cannot be empty".into())));
        }
        
        let mut policies = self.policies.write().await;
        let mut policies_by_type = self.policies_by_type.write().await;
        
        if policies.contains_key(&policy.id) {
            return Err(MCPError::Security(SecurityError::DuplicateIDError("Policy with that ID already exists".into())));
        }
        
        // Update policies by type
        let policy_type_entry = policies_by_type.entry(policy.policy_type.clone()).or_insert_with(HashSet::new);
        policy_type_entry.insert(policy.id.clone());
        
        // Store policy ID for logging
        let policy_id = policy.id.clone();
        
        // Insert policy
        policies.insert(policy_id.clone(), policy);
        
        info!("Added policy: {}", policy_id);
        Ok(())
    }
    
    /// Get a policy by ID
    #[instrument]
    pub async fn get_policy(&self, policy_id: &str) -> Result<SecurityPolicy> {
        let policies = self.policies.read().await;
        policies.get(policy_id)
            .cloned()
            .ok_or_else(|| MCPError::Security(SecurityError::NotFound("Policy not found".into())))
    }
    
    /// Get policies by type
    #[instrument]
    pub async fn get_policies_by_type(&self, policy_type: &PolicyType) -> Result<Vec<SecurityPolicy>> {
        let policies_by_type = self.policies_by_type.read().await;
        let policies = self.policies.read().await;
        
        let policy_ids = policies_by_type.get(policy_type)
            .map(|ids| ids.clone())
            .unwrap_or_else(HashSet::new);
            
        let result = policy_ids.iter()
            .filter_map(|id| policies.get(id).cloned())
            .collect();
            
        Ok(result)
    }
    
    /// Delete a policy
    #[instrument]
    pub async fn delete_policy(&self, policy_id: &str) -> Result<()> {
        let mut policies = self.policies.write().await;
        let mut policies_by_type = self.policies_by_type.write().await;
        
        if let Some(policy) = policies.remove(policy_id) {
            if let Some(type_set) = policies_by_type.get_mut(&policy.policy_type) {
                type_set.remove(policy_id);
            }
            info!("Deleted policy: {}", policy_id);
            Ok(())
        } else {
            Err(MCPError::Security(SecurityError::NotFound("Policy not found".into())))
        }
    }
    
    /// Update a policy
    #[instrument(skip(policy))]
    pub async fn update_policy(&self, policy: SecurityPolicy) -> Result<()> {
        let mut policies = self.policies.write().await;
        
        if !policies.contains_key(&policy.id) {
            return Err(MCPError::Security(SecurityError::NotFound("Policy not found".into())));
        }
        
        // Store policy ID for logging
        let policy_id = policy.id.clone();
        
        policies.insert(policy_id.clone(), policy);
        info!("Updated policy: {}", policy_id);
        Ok(())
    }
    
    /// Evaluate a policy
    #[instrument(skip(context))]
    pub async fn evaluate_policy(&self, policy_id: &str, context: &PolicyContext) -> Result<PolicyEvaluationResult> {
        let policy = self.get_policy(policy_id).await?;
        
        if !policy.enabled {
            debug!("Policy {} is disabled, skipping evaluation", policy_id);
            return Ok(PolicyEvaluationResult::Passed);
        }
        
        let handlers = self.handlers.read().await;
        let handler = handlers.values().find(|h| h.policy_type() == policy.policy_type);
        
        match handler {
            Some(evaluator) => {
                debug!("Evaluating policy {} with evaluator {}", policy_id, evaluator.id());
                let result = evaluator.evaluate(&policy, context).await?;
                
                // Handle enforcement based on policy level and result
                if self.enforcement_enabled && matches!(result, PolicyEvaluationResult::Violation(_)) {
                    match policy.enforcement_level {
                        EnforcementLevel::Advisory => {
                            info!("Advisory policy violation: {} - {:?}", policy_id, result);
                            Ok(PolicyEvaluationResult::Warning(format!("Advisory policy violation: {}", policy_id)))
                        },
                        EnforcementLevel::Warning => {
                            warn!("Warning policy violation: {} - {:?}", policy_id, result);
                            Ok(PolicyEvaluationResult::Warning(format!("Warning policy violation: {}", policy_id)))
                        },
                        EnforcementLevel::Enforced | EnforcementLevel::Critical => {
                            error!("Enforced policy violation: {} - {:?}", policy_id, result);
                            Err(MCPError::Security(SecurityError::PolicyViolation(format!("Policy violation: {}", policy_id))))
                        }
                    }
                } else {
                    Ok(result)
                }
            },
            None => {
                warn!("No evaluator found for policy type: {:?}", policy.policy_type);
                Ok(PolicyEvaluationResult::Warning(format!("No evaluator found for policy type: {:?}", policy.policy_type)))
            }
        }
    }
    
    /// Evaluate all policies of a specific type
    #[instrument(skip(context))]
    pub async fn evaluate_policies_by_type(&self, policy_type: &PolicyType, context: &PolicyContext) -> Result<Vec<(String, PolicyEvaluationResult)>> {
        let policies = self.get_policies_by_type(policy_type).await?;
        let mut results = Vec::new();
        
        for policy in policies {
            if policy.enabled {
                match self.evaluate_policy(&policy.id, context).await {
                    Ok(result) => {
                        results.push((policy.id.clone(), result));
                    },
                    Err(e) => {
                        error!("Error evaluating policy {}: {:?}", policy.id, e);
                        return Err(e);
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    /// Set enforcement enabled
    pub fn set_enforcement_enabled(&mut self, enabled: bool) {
        info!("Setting policy enforcement to: {}", enabled);
        self.enforcement_enabled = enabled;
    }
    
    /// Get all policies
    pub async fn get_all_policies(&self) -> Result<Vec<SecurityPolicy>> {
        let policies = self.policies.read().await;
        Ok(policies.values().cloned().collect())
    }
    
    /// Get enforcement status
    pub fn enforcement_enabled(&self) -> bool {
        self.enforcement_enabled
    }
}

/// Standard policy evaluators

/// Password policy evaluator
#[derive(Debug)]
pub struct PasswordPolicyEvaluator {
    id: String,
}

impl PasswordPolicyEvaluator {
    pub fn new() -> Self {
        Self {
            id: "password_policy_evaluator".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl PolicyEvaluator for PasswordPolicyEvaluator {
    async fn evaluate(&self, policy: &SecurityPolicy, context: &PolicyContext) -> Result<PolicyEvaluationResult> {
        let password = context.request_info.get("password")
            .ok_or_else(|| MCPError::Security(SecurityError::ValidationError("Password not provided".into())))?;
        
        // Get password requirements from policy
        let min_length = policy.settings.get("min_length")
            .map(|s| s.parse::<usize>().unwrap_or(8))
            .unwrap_or(8);
            
        let require_uppercase = policy.settings.get("require_uppercase")
            .map(|s| s.parse::<bool>().unwrap_or(true))
            .unwrap_or(true);
            
        let require_lowercase = policy.settings.get("require_lowercase")
            .map(|s| s.parse::<bool>().unwrap_or(true))
            .unwrap_or(true);
            
        let require_digits = policy.settings.get("require_digits")
            .map(|s| s.parse::<bool>().unwrap_or(true))
            .unwrap_or(true);
            
        let require_special = policy.settings.get("require_special")
            .map(|s| s.parse::<bool>().unwrap_or(true))
            .unwrap_or(true);
        
        // Check password requirements
        let mut violations = Vec::new();
        
        if password.len() < min_length {
            violations.push(format!("Password must be at least {} characters", min_length));
        }
        
        if require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            violations.push("Password must contain at least one uppercase letter".to_string());
        }
        
        if require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            violations.push("Password must contain at least one lowercase letter".to_string());
        }
        
        if require_digits && !password.chars().any(|c| c.is_digit(10)) {
            violations.push("Password must contain at least one digit".to_string());
        }
        
        if require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
            violations.push("Password must contain at least one special character".to_string());
        }
        
        if violations.is_empty() {
            Ok(PolicyEvaluationResult::Passed)
        } else {
            Ok(PolicyEvaluationResult::Violation(violations.join("; ")))
        }
    }
    
    fn policy_type(&self) -> PolicyType {
        PolicyType::Password
    }
    
    fn id(&self) -> String {
        self.id.clone()
    }
}

/// Rate limiting policy evaluator
#[derive(Debug)]
pub struct RateLimitPolicyEvaluator {
    id: String,
    // Store rate limit counters, keyed by IP or user ID
    rate_limits: RwLock<HashMap<String, Vec<DateTime<Utc>>>>,
}

impl RateLimitPolicyEvaluator {
    pub fn new() -> Self {
        Self {
            id: "rate_limit_policy_evaluator".to_string(),
            rate_limits: RwLock::new(HashMap::new()),
        }
    }
    
    async fn cleanup_old_entries(&self) {
        let mut rate_limits = self.rate_limits.write().await;
        let now = Utc::now();
        
        for (_, timestamps) in rate_limits.iter_mut() {
            // Remove entries older than 1 hour
            timestamps.retain(|timestamp| {
                (now - *timestamp).num_seconds() < 3600
            });
        }
    }
}

#[async_trait::async_trait]
impl PolicyEvaluator for RateLimitPolicyEvaluator {
    async fn evaluate(&self, policy: &SecurityPolicy, context: &PolicyContext) -> Result<PolicyEvaluationResult> {
        // Get rate limit settings from policy
        let max_requests = policy.settings.get("max_requests")
            .map(|s| s.parse::<usize>().unwrap_or(100))
            .unwrap_or(100);
            
        let time_window = policy.settings.get("time_window_seconds")
            .map(|s| s.parse::<i64>().unwrap_or(60))
            .unwrap_or(60);
        
        // Get key to use for rate limiting (user_id or ip_address)
        let key = if let Some(user_id) = &context.user_id {
            format!("user:{}", user_id)
        } else if let Some(ip) = &context.ip_address {
            format!("ip:{}", ip)
        } else {
            return Ok(PolicyEvaluationResult::Warning("No user ID or IP address provided for rate limiting".to_string()));
        };
        
        // Cleanup old entries periodically
        self.cleanup_old_entries().await;
        
        // Check rate limit
        let mut rate_limits = self.rate_limits.write().await;
        let now = Utc::now();
        
        let timestamps = rate_limits.entry(key.clone()).or_insert_with(Vec::new);
        
        // Remove old timestamps outside the window
        timestamps.retain(|timestamp| {
            (now - *timestamp).num_seconds() < time_window
        });
        
        // Check if rate limit is exceeded
        if timestamps.len() >= max_requests {
            return Ok(PolicyEvaluationResult::Violation(
                format!("Rate limit exceeded: {} requests in {} seconds (max {})", 
                    timestamps.len(), time_window, max_requests)
            ));
        }
        
        // Add current timestamp
        timestamps.push(now);
        
        Ok(PolicyEvaluationResult::Passed)
    }
    
    fn policy_type(&self) -> PolicyType {
        PolicyType::RateLimit
    }
    
    fn id(&self) -> String {
        self.id.clone()
    }
}

/// Session policy evaluator
#[derive(Debug)]
pub struct SessionPolicyEvaluator {
    id: String,
}

impl SessionPolicyEvaluator {
    pub fn new() -> Self {
        Self {
            id: "session_policy_evaluator".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl PolicyEvaluator for SessionPolicyEvaluator {
    async fn evaluate(&self, policy: &SecurityPolicy, context: &PolicyContext) -> Result<PolicyEvaluationResult> {
        // Get session token from context
        let session_token = match &context.session_token {
            Some(token) => token,
            None => return Ok(PolicyEvaluationResult::Violation("No session token provided".to_string())),
        };
        
        // Get session policy settings
        let max_session_age = policy.settings.get("max_session_age_minutes")
            .map(|s| s.parse::<i64>().unwrap_or(60))
            .unwrap_or(60);
            
        let require_secure = policy.settings.get("require_secure")
            .map(|s| s.parse::<bool>().unwrap_or(true))
            .unwrap_or(true);
            
        let session_created_at = context.request_info.get("session_created_at")
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or_else(|| (Utc::now() - chrono::Duration::minutes(max_session_age + 1)).timestamp());
            
        let secure_connection = context.request_info.get("secure_connection")
            .map(|s| s.parse::<bool>().ok())
            .flatten()
            .unwrap_or(false);
            
        // Check session age
        let now = Utc::now();
        let session_age_minutes = (now.timestamp() - session_created_at) / 60;
        
        if session_age_minutes > max_session_age {
            return Ok(PolicyEvaluationResult::Violation(
                format!("Session expired: {} minutes old (max {})", session_age_minutes, max_session_age)
            ));
        }
        
        // Check secure connection if required
        if require_secure && !secure_connection {
            return Ok(PolicyEvaluationResult::Violation(
                "Secure connection required for this session".to_string()
            ));
        }
        
        Ok(PolicyEvaluationResult::Passed)
    }
    
    fn policy_type(&self) -> PolicyType {
        PolicyType::Session
    }
    
    fn id(&self) -> String {
        self.id.clone()
    }
}

// Add unit tests for the policies module
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_add_policy() {
        let manager = PolicyManager::new(true);
        
        let policy = SecurityPolicy {
            id: "test-policy".to_string(),
            name: "Test Policy".to_string(),
            description: Some("A test policy".to_string()),
            policy_type: PolicyType::Authentication,
            enforcement_level: EnforcementLevel::Enforced,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            settings: HashMap::new(),
            required_permissions: HashSet::new(),
            security_level: SecurityLevel::Standard,
            enabled: true,
        };
        
        let result = manager.add_policy(policy.clone()).await;
        assert!(result.is_ok());
        
        let retrieved = manager.get_policy("test-policy").await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().id, "test-policy");
    }
    
    #[tokio::test]
    async fn test_password_policy_evaluator() {
        let manager = PolicyManager::new(true);
        let evaluator = Arc::new(PasswordPolicyEvaluator::new());
        
        manager.add_evaluator(evaluator).await.unwrap();
        
        let mut settings = HashMap::new();
        settings.insert("min_length".to_string(), "8".to_string());
        settings.insert("require_uppercase".to_string(), "true".to_string());
        settings.insert("require_lowercase".to_string(), "true".to_string());
        settings.insert("require_digits".to_string(), "true".to_string());
        settings.insert("require_special".to_string(), "true".to_string());
        
        let policy = SecurityPolicy {
            id: "password-policy".to_string(),
            name: "Password Policy".to_string(),
            description: Some("Password requirements".to_string()),
            policy_type: PolicyType::Password,
            enforcement_level: EnforcementLevel::Enforced,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            settings,
            required_permissions: HashSet::new(),
            security_level: SecurityLevel::Standard,
            enabled: true,
        };
        
        manager.add_policy(policy).await.unwrap();
        
        // Test valid password
        let mut context = PolicyContext::default();
        let mut request_info = HashMap::new();
        request_info.insert("password".to_string(), "Password1!".to_string());
        context.request_info = request_info;
        
        let result = manager.evaluate_policy("password-policy", &context).await;
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), PolicyEvaluationResult::Passed));
        
        // Test invalid password
        let mut context = PolicyContext::default();
        let mut request_info = HashMap::new();
        request_info.insert("password".to_string(), "password".to_string());
        context.request_info = request_info;
        
        let result = manager.evaluate_policy("password-policy", &context).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(matches!(result, PolicyEvaluationResult::Violation(_)));
    }
    
    #[tokio::test]
    async fn test_rate_limit_policy_evaluator() {
        let manager = PolicyManager::new(true);
        let evaluator = Arc::new(RateLimitPolicyEvaluator::new());
        
        manager.add_evaluator(evaluator).await.unwrap();
        
        let mut settings = HashMap::new();
        settings.insert("max_requests".to_string(), "3".to_string());
        settings.insert("time_window_seconds".to_string(), "60".to_string());
        
        let policy = SecurityPolicy {
            id: "rate-limit-policy".to_string(),
            name: "Rate Limit Policy".to_string(),
            description: Some("Rate limiting requirements".to_string()),
            policy_type: PolicyType::RateLimit,
            enforcement_level: EnforcementLevel::Enforced,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            settings,
            required_permissions: HashSet::new(),
            security_level: SecurityLevel::Standard,
            enabled: true,
        };
        
        manager.add_policy(policy).await.unwrap();
        
        // Test rate limiting
        let mut context = PolicyContext::default();
        context.user_id = Some("test-user".to_string());
        
        // First 3 requests should pass
        for _ in 0..3 {
            let result = manager.evaluate_policy("rate-limit-policy", &context).await;
            assert!(result.is_ok());
            assert!(matches!(result.unwrap(), PolicyEvaluationResult::Passed));
        }
        
        // Fourth request should fail
        let result = manager.evaluate_policy("rate-limit-policy", &context).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MCPError::Security(SecurityError::PolicyViolation(_))));
    }
} 