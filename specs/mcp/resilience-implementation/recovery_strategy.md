---
version: 1.0.0
last_updated: 2024-07-19
status: implementation
---

# MCP Resilience Framework: Recovery Strategy Implementation

## Overview

This document provides the implementation details for the Recovery Strategy component of the MCP Resilience Framework. The recovery strategy provides a systematic approach to recovering from failures by mapping different error types to appropriate recovery actions.

## Implementation Structure

### 1. Recovery Actions

Recovery actions define the operations that can be taken to recover from failures:

```rust
/// Defines different recovery actions for handling failures
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    /// Retry the operation with the specified retry mechanism
    Retry(Arc<RetryMechanism>),
    
    /// Execute a fallback operation that produces a result
    Fallback {
        /// The function to call as a fallback
        fallback: Arc<dyn Fn() -> BoxFuture<'static, Result<Box<dyn Any + Send + Sync>, Error>> + Send + Sync>,
        
        /// Optional description for logging and metrics
        description: Option<String>,
    },
    
    /// Reset a component or subsystem to recover
    Reset {
        /// Function to reset the system
        reset_fn: Arc<dyn Fn() -> BoxFuture<'static, Result<(), Error>> + Send + Sync>,
        
        /// Component or subsystem being reset
        target: String,
    },
    
    /// Restart a component or the entire system
    Restart {
        /// Function to restart the system
        restart_fn: Arc<dyn Fn() -> BoxFuture<'static, Result<(), Error>> + Send + Sync>,
        
        /// Component or subsystem being restarted
        target: String,
        
        /// Whether this is a graceful restart
        graceful: bool,
    },
    
    /// Custom recovery action
    Custom {
        /// The action to execute
        action: Arc<dyn Fn() -> BoxFuture<'static, Result<(), Error>> + Send + Sync>,
        
        /// Description of the action
        description: String,
    },
}
```

### 2. Error Classification

To intelligently map errors to recovery actions, we need a system to classify errors:

```rust
/// Classifies errors to determine appropriate recovery actions
#[derive(Debug)]
pub struct ErrorClassifier {
    /// Maps error types to categories
    type_categories: HashMap<TypeId, ErrorCategory>,
    
    /// Function to classify errors that aren't in the type map
    dynamic_classifier: Option<Arc<dyn Fn(&dyn Error) -> ErrorCategory + Send + Sync>>,
}

/// Categories of errors for different recovery approaches
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// Temporary/transient error that might succeed on retry
    Transient,
    
    /// Error due to resource constraints (memory, connections, etc.)
    ResourceConstraint,
    
    /// Error due to an invalid state
    InvalidState,
    
    /// Error due to bad input or request
    InvalidInput,
    
    /// Error in dependent service or component
    DependencyFailure,
    
    /// System-level failure
    SystemFailure,
    
    /// Error due to security or access control
    Security,
    
    /// Uncategorized error
    Uncategorized,
}

impl ErrorClassifier {
    /// Creates a new error classifier
    pub fn new() -> Self {
        Self {
            type_categories: HashMap::new(),
            dynamic_classifier: None,
        }
    }
    
    /// Registers an error type with a category
    pub fn register<E: Error + 'static>(&mut self, category: ErrorCategory) {
        self.type_categories.insert(TypeId::of::<E>(), category);
    }
    
    /// Sets a dynamic classifier function
    pub fn set_dynamic_classifier<F>(&mut self, classifier: F)
    where
        F: Fn(&dyn Error) -> ErrorCategory + Send + Sync + 'static,
    {
        self.dynamic_classifier = Some(Arc::new(classifier));
    }
    
    /// Classifies an error into a category
    pub fn classify(&self, error: &dyn Error) -> ErrorCategory {
        let type_id = error.type_id();
        
        if let Some(category) = self.type_categories.get(&type_id) {
            return *category;
        }
        
        if let Some(classifier) = &self.dynamic_classifier {
            return classifier(error);
        }
        
        ErrorCategory::Uncategorized
    }
}
```

### 3. Recovery Strategy Implementation

The core implementation enables mapping error types and categories to appropriate recovery actions:

```rust
/// Recovery strategy with multiple recovery options for different error types
pub struct RecoveryStrategy {
    /// Maps error types to recovery actions
    type_actions: HashMap<TypeId, Vec<RecoveryAction>>,
    
    /// Maps error categories to recovery actions
    category_actions: HashMap<ErrorCategory, Vec<RecoveryAction>>,
    
    /// Default action if no specific handler is found
    default_action: Option<RecoveryAction>,
    
    /// Error classifier for categorizing errors
    classifier: ErrorClassifier,
    
    /// Metrics collection for recovery operations
    #[cfg(feature = "metrics")]
    metrics: RecoveryMetrics,
}

impl RecoveryStrategy {
    /// Creates a new recovery strategy
    pub fn new(classifier: ErrorClassifier) -> Self {
        #[cfg(feature = "metrics")]
        let metrics = RecoveryMetrics::new();
        
        Self {
            type_actions: HashMap::new(),
            category_actions: HashMap::new(),
            default_action: None,
            classifier,
            #[cfg(feature = "metrics")]
            metrics,
        }
    }
    
    /// Registers a recovery action for a specific error type
    pub fn register_for_type<E: Error + 'static>(&mut self, action: RecoveryAction) {
        let type_id = TypeId::of::<E>();
        let actions = self.type_actions.entry(type_id).or_insert_with(Vec::new);
        actions.push(action);
    }
    
    /// Registers a recovery action for an error category
    pub fn register_for_category(&mut self, category: ErrorCategory, action: RecoveryAction) {
        let actions = self.category_actions.entry(category).or_insert_with(Vec::new);
        actions.push(action);
    }
    
    /// Sets the default recovery action
    pub fn set_default_action(&mut self, action: RecoveryAction) {
        self.default_action = Some(action);
    }
    
    /// Recovers from an error by executing appropriate recovery actions
    pub async fn recover<T>(&self, error: Box<dyn Error + Send + Sync>) -> Result<Option<T>, RecoveryError>
    where
        T: Any + Send + Sync + 'static,
    {
        let type_id = error.type_id();
        let error_category = self.classifier.classify(error.as_ref());
        
        #[cfg(feature = "metrics")]
        self.metrics.record_recovery_attempt(error_category);
        
        // First try type-specific actions
        if let Some(actions) = self.type_actions.get(&type_id) {
            if let Ok(result) = self.execute_actions::<T>(actions, error.as_ref()).await {
                return Ok(result);
            }
        }
        
        // Next try category-specific actions
        if let Some(actions) = self.category_actions.get(&error_category) {
            if let Ok(result) = self.execute_actions::<T>(actions, error.as_ref()).await {
                return Ok(result);
            }
        }
        
        // Finally try default action
        if let Some(action) = &self.default_action {
            if let Ok(result) = self.execute_action::<T>(action, error.as_ref()).await {
                return Ok(result);
            }
        }
        
        // No successful recovery
        #[cfg(feature = "metrics")]
        self.metrics.record_recovery_failure();
        
        Err(RecoveryError::AllStrategiesFailed(format!("All recovery strategies failed for error: {}", error)))
    }
    
    /// Executes a list of recovery actions in order until one succeeds
    async fn execute_actions<T>(&self, actions: &[RecoveryAction], error: &dyn Error) -> Result<Option<T>, RecoveryError>
    where
        T: Any + Send + Sync + 'static,
    {
        for action in actions {
            match self.execute_action::<T>(action, error).await {
                Ok(result) => return Ok(result),
                Err(_) => continue, // Try next action
            }
        }
        
        Err(RecoveryError::AllActionsFailed("All recovery actions failed".into()))
    }
    
    /// Executes a single recovery action
    async fn execute_action<T>(&self, action: &RecoveryAction, error: &dyn Error) -> Result<Option<T>, RecoveryError>
    where
        T: Any + Send + Sync + 'static,
    {
        match action {
            RecoveryAction::Retry(retry_mechanism) => {
                #[cfg(feature = "metrics")]
                self.metrics.record_action_attempt("retry");
                
                // Retry isn't directly implemented here since it depends on the operation
                // This would typically be handled by the caller
                Err(RecoveryError::UnsupportedAction("Retry action must be handled by caller".into()))
            }
            
            RecoveryAction::Fallback { fallback, description } => {
                #[cfg(feature = "metrics")]
                self.metrics.record_action_attempt("fallback");
                
                match fallback().await {
                    Ok(result) => {
                        if let Some(typed_result) = result.downcast_ref::<T>() {
                            let cloned_result = typed_result.clone();
                            
                            #[cfg(feature = "metrics")]
                            self.metrics.record_action_success("fallback");
                            
                            return Ok(Some(cloned_result));
                        }
                        
                        Err(RecoveryError::TypeMismatch("Fallback result type mismatch".into()))
                    }
                    Err(err) => {
                        #[cfg(feature = "metrics")]
                        self.metrics.record_action_failure("fallback");
                        
                        Err(RecoveryError::ActionFailed(format!("Fallback failed: {}", err)))
                    }
                }
            }
            
            RecoveryAction::Reset { reset_fn, target } => {
                #[cfg(feature = "metrics")]
                self.metrics.record_action_attempt("reset");
                
                match reset_fn().await {
                    Ok(()) => {
                        #[cfg(feature = "metrics")]
                        self.metrics.record_action_success("reset");
                        
                        // Reset doesn't produce a result, just fixes the system state
                        Ok(None)
                    }
                    Err(err) => {
                        #[cfg(feature = "metrics")]
                        self.metrics.record_action_failure("reset");
                        
                        Err(RecoveryError::ActionFailed(format!("Reset of {} failed: {}", target, err)))
                    }
                }
            }
            
            RecoveryAction::Restart { restart_fn, target, graceful } => {
                #[cfg(feature = "metrics")]
                self.metrics.record_action_attempt("restart");
                
                match restart_fn().await {
                    Ok(()) => {
                        #[cfg(feature = "metrics")]
                        self.metrics.record_action_success("restart");
                        
                        // Restart doesn't produce a result, just restarts the system
                        Ok(None)
                    }
                    Err(err) => {
                        #[cfg(feature = "metrics")]
                        self.metrics.record_action_failure("restart");
                        
                        Err(RecoveryError::ActionFailed(format!("Restart of {} failed: {}", target, err)))
                    }
                }
            }
            
            RecoveryAction::Custom { action, description } => {
                #[cfg(feature = "metrics")]
                self.metrics.record_action_attempt("custom");
                
                match action().await {
                    Ok(()) => {
                        #[cfg(feature = "metrics")]
                        self.metrics.record_action_success("custom");
                        
                        // Custom action doesn't produce a result
                        Ok(None)
                    }
                    Err(err) => {
                        #[cfg(feature = "metrics")]
                        self.metrics.record_action_failure("custom");
                        
                        Err(RecoveryError::ActionFailed(format!("Custom action '{}' failed: {}", description, err)))
                    }
                }
            }
        }
    }
}
```

### 4. Error Types for Recovery Strategy

The recovery system provides specific error types for recovery operations:

```rust
#[derive(Debug, Error)]
pub enum RecoveryError {
    #[error("All recovery strategies failed: {0}")]
    AllStrategiesFailed(String),
    
    #[error("All recovery actions failed: {0}")]
    AllActionsFailed(String),
    
    #[error("Recovery action failed: {0}")]
    ActionFailed(String),
    
    #[error("Type mismatch in recovery: {0}")]
    TypeMismatch(String),
    
    #[error("Unsupported recovery action: {0}")]
    UnsupportedAction(String),
    
    #[error("Recovery error: {0}")]
    Other(String),
}
```

### 5. Optional Metrics Collection

When the "metrics" feature is enabled, the recovery strategy collects operational metrics:

```rust
#[cfg(feature = "metrics")]
#[derive(Debug)]
pub struct RecoveryMetrics {
    /// Total number of recovery attempts
    attempts: AtomicU64,
    
    /// Number of successful recoveries
    successes: AtomicU64,
    
    /// Number of failed recoveries
    failures: AtomicU64,
    
    /// Per-category attempts
    category_attempts: RwLock<HashMap<ErrorCategory, u64>>,
    
    /// Per-action attempts
    action_attempts: RwLock<HashMap<String, u64>>,
    
    /// Per-action successes
    action_successes: RwLock<HashMap<String, u64>>,
    
    /// Per-action failures
    action_failures: RwLock<HashMap<String, u64>>,
}

#[cfg(feature = "metrics")]
impl RecoveryMetrics {
    pub fn new() -> Self {
        Self {
            attempts: AtomicU64::new(0),
            successes: AtomicU64::new(0),
            failures: AtomicU64::new(0),
            category_attempts: RwLock::new(HashMap::new()),
            action_attempts: RwLock::new(HashMap::new()),
            action_successes: RwLock::new(HashMap::new()),
            action_failures: RwLock::new(HashMap::new()),
        }
    }
    
    pub fn record_recovery_attempt(&self, category: ErrorCategory) {
        self.attempts.fetch_add(1, Ordering::SeqCst);
        
        let mut categories = self.category_attempts.write().unwrap();
        let count = categories.entry(category).or_insert(0);
        *count += 1;
    }
    
    pub fn record_recovery_success(&self) {
        self.successes.fetch_add(1, Ordering::SeqCst);
    }
    
    pub fn record_recovery_failure(&self) {
        self.failures.fetch_add(1, Ordering::SeqCst);
    }
    
    pub fn record_action_attempt(&self, action_type: &str) {
        let mut actions = self.action_attempts.write().unwrap();
        let count = actions.entry(action_type.to_string()).or_insert(0);
        *count += 1;
    }
    
    pub fn record_action_success(&self, action_type: &str) {
        let mut successes = self.action_successes.write().unwrap();
        let count = successes.entry(action_type.to_string()).or_insert(0);
        *count += 1;
        
        self.record_recovery_success();
    }
    
    pub fn record_action_failure(&self, action_type: &str) {
        let mut failures = self.action_failures.write().unwrap();
        let count = failures.entry(action_type.to_string()).or_insert(0);
        *count += 1;
    }
    
    pub fn get_metrics(&self) -> RecoveryMetricsSnapshot {
        RecoveryMetricsSnapshot {
            attempts: self.attempts.load(Ordering::SeqCst),
            successes: self.successes.load(Ordering::SeqCst),
            failures: self.failures.load(Ordering::SeqCst),
            category_attempts: self.category_attempts.read().unwrap().clone(),
            action_attempts: self.action_attempts.read().unwrap().clone(),
            action_successes: self.action_successes.read().unwrap().clone(),
            action_failures: self.action_failures.read().unwrap().clone(),
        }
    }
}

#[cfg(feature = "metrics")]
#[derive(Debug, Clone, Serialize)]
pub struct RecoveryMetricsSnapshot {
    pub attempts: u64,
    pub successes: u64,
    pub failures: u64,
    pub category_attempts: HashMap<ErrorCategory, u64>,
    pub action_attempts: HashMap<String, u64>,
    pub action_successes: HashMap<String, u64>,
    pub action_failures: HashMap<String, u64>,
}
```

## Usage Examples

### Basic Recovery Strategy Usage

```rust
// Create an error classifier
let mut classifier = ErrorClassifier::new();
classifier.register::<ApiError>(ErrorCategory::DependencyFailure);
classifier.register::<TimeoutError>(ErrorCategory::Transient);
classifier.register::<ValidationError>(ErrorCategory::InvalidInput);

// Create a recovery strategy
let mut recovery_strategy = RecoveryStrategy::new(classifier);

// Register fallback for API errors
recovery_strategy.register_for_type::<ApiError>(RecoveryAction::Fallback {
    fallback: Arc::new(|| Box::pin(async {
        // Return cached data as fallback
        let cached_data = get_cached_data().await?;
        Ok(Box::new(cached_data) as Box<dyn Any + Send + Sync>)
    })),
    description: Some("Use cached data".to_string()),
});

// Register retry for transient errors
let retry = Arc::new(RetryMechanism::new(RetryConfig {
    max_attempts: 3,
    backoff_strategy: BackoffStrategy::Exponential {
        initial_delay_ms: 100,
        multiplier: 2.0,
        max_delay_ms: 5000,
    },
    should_retry: None,
    name: None,
}));

recovery_strategy.register_for_category(
    ErrorCategory::Transient,
    RecoveryAction::Retry(retry),
);

// Register system reset for dependency failures
recovery_strategy.register_for_category(
    ErrorCategory::DependencyFailure,
    RecoveryAction::Reset {
        reset_fn: Arc::new(|| Box::pin(async {
            // Reset connection pool
            connection_pool.reset().await?;
            Ok(())
        })),
        target: "connection_pool".to_string(),
    },
);

// Set default action
recovery_strategy.set_default_action(RecoveryAction::Custom {
    action: Arc::new(|| Box::pin(async {
        // Log error and notify monitoring system
        log::error!("Unhandled error occurred");
        monitoring::send_alert("Unhandled error").await?;
        Ok(())
    })),
    description: "Log and alert".to_string(),
});

// Use recovery strategy to handle errors
async fn process_with_recovery(recovery_strategy: &RecoveryStrategy) -> Result<Data, Error> {
    match api_client.get_data().await {
        Ok(data) => Ok(data),
        Err(err) => {
            // Attempt recovery
            if let Some(recovered_data) = recovery_strategy.recover::<Data>(Box::new(err)).await? {
                Ok(recovered_data)
            } else {
                // Recovery didn't produce data
                Err(anyhow::anyhow!("Could not recover data"))
            }
        }
    }
}
```

### Integration with Circuit Breaker and Retry

```rust
// Create complete resilience system
let circuit_breaker = Arc::new(CircuitBreaker::default());
let retry_mechanism = Arc::new(RetryMechanism::default());

let mut classifier = ErrorClassifier::new();
classifier.register::<McpError>(ErrorCategory::DependencyFailure);
classifier.register::<TimeoutError>(ErrorCategory::Transient);

let mut recovery_strategy = RecoveryStrategy::new(classifier);
recovery_strategy.register_for_category(
    ErrorCategory::Transient,
    RecoveryAction::Retry(retry_mechanism.clone()),
);

// Implement resilient operation
async fn perform_resilient_operation(
    circuit_breaker: &CircuitBreaker,
    recovery_strategy: &RecoveryStrategy,
) -> Result<Data, Error> {
    // Try with circuit breaker
    match circuit_breaker
        .execute(async { api_client.get_data().await })
        .await
    {
        Ok(data) => Ok(data),
        Err(ResilienceError::CircuitOpen) => {
            // Circuit is open, use fallback
            let fallback_data = get_fallback_data().await?;
            Ok(fallback_data)
        }
        Err(ResilienceError::Operation(err)) => {
            // Attempt recovery
            if let Some(recovered_data) = recovery_strategy.recover::<Data>(err).await? {
                Ok(recovered_data)
            } else {
                // Recovery didn't produce data
                Err(anyhow::anyhow!("Could not recover data"))
            }
        }
        Err(err) => Err(anyhow::anyhow!("Resilience error: {}", err)),
    }
}
```

### MCP Protocol Integration

```rust
// Create a complete resilient MCP protocol
struct ResilientMcpProtocol {
    inner: Arc<dyn McpProtocol>,
    circuit_breaker: Arc<CircuitBreaker>,
    recovery_strategy: Arc<RecoveryStrategy>,
}

impl ResilientMcpProtocol {
    pub fn new(
        protocol: Arc<dyn McpProtocol>,
        circuit_breaker: Arc<CircuitBreaker>,
        recovery_strategy: Arc<RecoveryStrategy>,
    ) -> Self {
        Self {
            inner: protocol,
            circuit_breaker,
            recovery_strategy,
        }
    }

    pub async fn send_message(&self, message: McpMessage) -> Result<McpResponse, McpError> {
        // Execute with circuit breaker
        match self
            .circuit_breaker
            .execute(async { self.inner.send_message(message.clone()).await })
            .await
        {
            Ok(response) => Ok(response),
            Err(ResilienceError::CircuitOpen) => {
                Err(McpError::ServiceUnavailable(
                    "Circuit is open, service unavailable".into(),
                ))
            }
            Err(ResilienceError::Operation(err)) => {
                // Attempt recovery
                match self.recovery_strategy.recover::<McpResponse>(err).await {
                    Ok(Some(response)) => Ok(response),
                    Ok(None) => Err(McpError::RecoveryIncomplete(
                        "Recovery did not produce a response".into(),
                    )),
                    Err(recovery_err) => Err(McpError::RecoveryFailed(format!(
                        "Recovery failed: {}",
                        recovery_err
                    ))),
                }
            }
            Err(err) => Err(McpError::Internal(format!("Resilience error: {}", err))),
        }
    }
}
```

## Testing

### Unit Testing 

```rust
#[tokio::test]
async fn test_recovery_fallback() {
    // Create a recovery strategy with a fallback
    let mut classifier = ErrorClassifier::new();
    classifier.register::<TestError>(ErrorCategory::DependencyFailure);
    
    let mut recovery_strategy = RecoveryStrategy::new(classifier);
    recovery_strategy.register_for_type::<TestError>(RecoveryAction::Fallback {
        fallback: Arc::new(|| Box::pin(async {
            Ok(Box::new("fallback_value") as Box<dyn Any + Send + Sync>)
        })),
        description: Some("Test fallback".to_string()),
    });
    
    // Test recovery
    let error = TestError::new("test error");
    let result = recovery_strategy.recover::<&str>(Box::new(error)).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some("fallback_value"));
}

#[tokio::test]
async fn test_recovery_reset() {
    // Create a recovery strategy with a reset action
    let mut classifier = ErrorClassifier::new();
    classifier.register::<TestError>(ErrorCategory::InvalidState);
    
    let mut was_reset = Arc::new(AtomicBool::new(false));
    let was_reset_clone = was_reset.clone();
    
    let mut recovery_strategy = RecoveryStrategy::new(classifier);
    recovery_strategy.register_for_category(
        ErrorCategory::InvalidState,
        RecoveryAction::Reset {
            reset_fn: Arc::new(move || {
                let was_reset = was_reset_clone.clone();
                Box::pin(async move {
                    was_reset.store(true, Ordering::SeqCst);
                    Ok(())
                })
            }),
            target: "test_system".to_string(),
        },
    );
    
    // Test recovery
    let error = TestError::new("test error");
    let result = recovery_strategy.recover::<()>(Box::new(error)).await;
    
    assert!(result.is_ok());
    assert!(was_reset.load(Ordering::SeqCst));
}

#[tokio::test]
async fn test_error_classifier() {
    let mut classifier = ErrorClassifier::new();
    classifier.register::<io::Error>(ErrorCategory::DependencyFailure);
    classifier.register::<ParseIntError>(ErrorCategory::InvalidInput);
    
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let parse_error = "abc".parse::<i32>().unwrap_err();
    let other_error = TestError::new("test error");
    
    assert_eq!(classifier.classify(&io_error), ErrorCategory::DependencyFailure);
    assert_eq!(classifier.classify(&parse_error), ErrorCategory::InvalidInput);
    assert_eq!(classifier.classify(&other_error), ErrorCategory::Uncategorized);
    
    // Test dynamic classifier
    let mut classifier_with_dynamic = ErrorClassifier::new();
    classifier_with_dynamic.set_dynamic_classifier(|err| {
        if err.to_string().contains("test") {
            ErrorCategory::SystemFailure
        } else {
            ErrorCategory::Uncategorized
        }
    });
    
    assert_eq!(
        classifier_with_dynamic.classify(&other_error),
        ErrorCategory::SystemFailure
    );
}
```

## Conclusion

The Recovery Strategy implementation provides a flexible mechanism for handling different types of failures in the MCP system. It supports:

1. Multiple recovery actions (retry, fallback, reset, restart, custom)
2. Error classification by type and category
3. Prioritized recovery strategies
4. Detailed metrics collection
5. Integration with other resilience components

This implementation satisfies the requirements outlined in the resilience framework specification and provides a powerful system for recovering from failures in MCP applications. 