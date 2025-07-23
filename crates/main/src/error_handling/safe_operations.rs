//! Safe Operations Module
//!
//! This module provides safe alternatives to unwrap() and expect() calls
//! throughout the codebase, preventing potential panics in production.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, warn};

/// Safe operation result wrapper
#[derive(Debug, Clone)]
pub struct SafeResult<T> {
    result: Result<T, SafeError>,
    context: String,
    recovery_strategy: RecoveryStrategy,
}

/// Safe error type for operation failures
#[derive(Debug, Clone, thiserror::Error)]
pub enum SafeError {
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        field: Option<String>,
    },

    #[error("Network error: {message}")]
    Network {
        message: String,
        endpoint: Option<String>,
    },

    #[error("Lock acquisition failed: {message}")]
    LockAcquisition { message: String, lock_type: String },

    #[error("Serialization error: {message}")]
    Serialization { message: String, data_type: String },

    #[error("Channel error: {message}")]
    Channel {
        message: String,
        channel_type: String,
    },

    #[error("Timeout error: {message}")]
    Timeout { message: String, duration: Duration },

    #[error("Resource unavailable: {message}")]
    ResourceUnavailable { message: String, resource: String },

    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: Option<String>,
    },

    #[error("Service unavailable: {message}")]
    ServiceUnavailable { message: String, service: String },

    #[error("Internal error: {message}")]
    Internal { message: String },
}

/// Recovery strategy for failed operations
pub enum RecoveryStrategy {
    /// Return a default value
    UseDefault,
    /// Retry the operation with backoff
    Retry {
        max_attempts: u32,
        backoff: Duration,
    },
    /// Use a fallback operation
    Fallback(Box<dyn Fn() -> Result<String, SafeError> + Send + Sync>),
    /// Log and continue
    LogAndContinue,
    /// Propagate the error
    Propagate,
}

impl std::fmt::Debug for RecoveryStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UseDefault => write!(f, "UseDefault"),
            Self::Retry {
                max_attempts,
                backoff,
            } => f
                .debug_struct("Retry")
                .field("max_attempts", max_attempts)
                .field("backoff", backoff)
                .finish(),
            Self::Fallback(_) => write!(f, "Fallback(<function>)"),
            Self::LogAndContinue => write!(f, "LogAndContinue"),
            Self::Propagate => write!(f, "Propagate"),
        }
    }
}

impl Clone for RecoveryStrategy {
    fn clone(&self) -> Self {
        match self {
            Self::UseDefault => Self::UseDefault,
            Self::Retry {
                max_attempts,
                backoff,
            } => Self::Retry {
                max_attempts: *max_attempts,
                backoff: *backoff,
            },
            Self::Fallback(_) => Self::UseDefault, // Can't clone function, fallback to default
            Self::LogAndContinue => Self::LogAndContinue,
            Self::Propagate => Self::Propagate,
        }
    }
}

impl<T> SafeResult<T> {
    /// Create a new safe result
    pub fn new(result: Result<T, SafeError>, context: String) -> Self {
        Self {
            result,
            context,
            recovery_strategy: RecoveryStrategy::Propagate,
        }
    }

    /// Create a successful result
    pub fn success(value: T, context: String) -> Self {
        Self {
            result: Ok(value),
            context,
            recovery_strategy: RecoveryStrategy::UseDefault,
        }
    }

    /// Create a failed result
    pub fn failure(error: SafeError, context: String) -> Self {
        Self {
            result: Err(error),
            context,
            recovery_strategy: RecoveryStrategy::Propagate,
        }
    }

    /// Set recovery strategy
    pub fn with_recovery_strategy(mut self, strategy: RecoveryStrategy) -> Self {
        self.recovery_strategy = strategy;
        self
    }

    /// Execute the operation with recovery
    pub fn execute(self) -> Result<T, SafeError>
    where
        T: Default + Clone,
    {
        match self.result {
            Ok(value) => Ok(value),
            Err(error) => match self.recovery_strategy {
                RecoveryStrategy::UseDefault => {
                    warn!(
                        "Operation failed in {}: {}. Using default value.",
                        self.context, error
                    );
                    Ok(T::default())
                }
                RecoveryStrategy::LogAndContinue => {
                    error!(
                        "Operation failed in {}: {}. Continuing with default.",
                        self.context, error
                    );
                    Ok(T::default())
                }
                RecoveryStrategy::Propagate => {
                    error!("Operation failed in {}: {}", self.context, error);
                    Err(error)
                }
                _ => {
                    error!(
                        "Operation failed in {}: {}. Complex recovery not implemented.",
                        self.context, error
                    );
                    Err(error)
                }
            },
        }
    }

    /// Execute the operation for Result types from external libraries (like reqwest)
    /// This method doesn't require Default + Clone bounds
    pub fn execute_without_default(self) -> Result<T, SafeError> {
        match self.result {
            Ok(value) => Ok(value),
            Err(error) => {
                error!("Operation failed in {}: {}", self.context, error);
                match self.recovery_strategy {
                    RecoveryStrategy::Propagate => Err(error),
                    _ => Err(error), // For now, always propagate when no default is available
                }
            }
        }
    }

    /// Get the result or use a default value
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self.result {
            Ok(value) => value,
            Err(error) => {
                warn!(
                    "Operation failed in {}: {}. Using default value.",
                    self.context, error
                );
                T::default()
            }
        }
    }

    /// Get the result or use a provided value
    pub fn unwrap_or(self, default: T) -> T {
        match self.result {
            Ok(value) => value,
            Err(error) => {
                warn!(
                    "Operation failed in {}: {}. Using provided default.",
                    self.context, error
                );
                default
            }
        }
    }

    /// Get the result or compute a default value
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce(SafeError) -> T,
    {
        match self.result {
            Ok(value) => value,
            Err(error) => {
                warn!(
                    "Operation failed in {}: {}. Computing default.",
                    self.context, error
                );
                f(error)
            }
        }
    }

    /// Log the error and return a default value
    pub fn log_and_default(self) -> T
    where
        T: Default,
    {
        match self.result {
            Ok(value) => value,
            Err(error) => {
                error!("Operation failed in {}: {}", self.context, error);
                T::default()
            }
        }
    }
}

/// Safe operations utility functions
pub struct SafeOps;

impl SafeOps {
    /// Safely acquire a read lock
    pub async fn safe_read_lock<'a, T>(
        lock: &'a Arc<RwLock<T>>,
        context: &'a str,
    ) -> SafeResult<tokio::sync::RwLockReadGuard<'a, T>> {
        let result = match tokio::time::timeout(Duration::from_secs(5), lock.read()).await {
            Ok(guard) => Ok(guard),
            Err(_) => Err(SafeError::Timeout {
                message: format!("Failed to acquire read lock in {}", context),
                duration: Duration::from_secs(5),
            }),
        };

        SafeResult::new(result, context.to_string())
    }

    /// Safely acquire a write lock
    pub async fn safe_write_lock<'a, T>(
        lock: &'a Arc<RwLock<T>>,
        context: &'a str,
    ) -> SafeResult<tokio::sync::RwLockWriteGuard<'a, T>> {
        let result = match tokio::time::timeout(Duration::from_secs(5), lock.write()).await {
            Ok(guard) => Ok(guard),
            Err(_) => Err(SafeError::Timeout {
                message: format!("Failed to acquire write lock in {}", context),
                duration: Duration::from_secs(5),
            }),
        };

        SafeResult::new(result, context.to_string())
    }

    /// Safely acquire a mutex lock
    pub async fn safe_mutex_lock<'a, T>(
        mutex: &'a Arc<Mutex<T>>,
        context: &'a str,
    ) -> SafeResult<tokio::sync::MutexGuard<'a, T>> {
        let result = match tokio::time::timeout(Duration::from_secs(5), mutex.lock()).await {
            Ok(guard) => Ok(guard),
            Err(_) => Err(SafeError::Timeout {
                message: format!("Failed to acquire mutex lock in {}", context),
                duration: Duration::from_secs(5),
            }),
        };

        SafeResult::new(result, context.to_string())
    }

    /// Safely parse a string to a type
    pub fn safe_parse<T>(value: &str, context: &str) -> SafeResult<T>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        let result = value.parse::<T>().map_err(|e| SafeError::Validation {
            message: format!("Failed to parse '{}' in {}: {}", value, context, e),
            field: None,
        });

        SafeResult::new(result, context.to_string())
    }

    /// Safely serialize to JSON
    pub fn safe_serialize<T>(value: &T, context: &str) -> SafeResult<String>
    where
        T: Serialize,
    {
        let result = serde_json::to_string(value).map_err(|e| SafeError::Serialization {
            message: format!("Failed to serialize in {}: {}", context, e),
            data_type: std::any::type_name::<T>().to_string(),
        });

        SafeResult::new(result, context.to_string())
    }

    /// Safely deserialize from JSON
    pub fn safe_deserialize<T>(json: &str, context: &str) -> SafeResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let result = serde_json::from_str::<T>(json).map_err(|e| SafeError::Serialization {
            message: format!("Failed to deserialize in {}: {}", context, e),
            data_type: std::any::type_name::<T>().to_string(),
        });

        SafeResult::new(result, context.to_string())
    }

    /// Safely get a value from a HashMap
    pub fn safe_get<'a, K, V>(
        map: &'a HashMap<K, V>,
        key: &'a K,
        context: &'a str,
    ) -> SafeResult<&'a V>
    where
        K: std::hash::Hash + Eq + std::fmt::Debug,
    {
        let result = map.get(key).ok_or_else(|| SafeError::ResourceUnavailable {
            message: format!("Key '{:?}' not found in {} map", key, context),
            resource: context.to_string(),
        });

        SafeResult::new(result, context.to_string())
    }

    /// Safely get a value from a HashMap with cloning
    pub fn safe_get_cloned<K, V>(map: &HashMap<K, V>, key: &K, context: &str) -> SafeResult<V>
    where
        K: std::hash::Hash + Eq + std::fmt::Debug,
        V: Clone,
    {
        let result = map
            .get(key)
            .cloned()
            .ok_or_else(|| SafeError::ResourceUnavailable {
                message: format!("Key '{:?}' not found in {} map", key, context),
                resource: context.to_string(),
            });

        SafeResult::new(result, context.to_string())
    }

    /// Safely get the first element from a Vec
    pub fn safe_first<'a, T>(vec: &'a [T], context: &'a str) -> SafeResult<&'a T> {
        let result = vec.first().ok_or_else(|| SafeError::ResourceUnavailable {
            message: format!("No first element in {} vector", context),
            resource: context.to_string(),
        });

        SafeResult::new(result, context.to_string())
    }

    /// Safely get the last element from a Vec
    pub fn safe_last<'a, T>(vec: &'a [T], context: &'a str) -> SafeResult<&'a T> {
        let result = vec.last().ok_or_else(|| SafeError::ResourceUnavailable {
            message: format!("No last element in {} vector", context),
            resource: context.to_string(),
        });

        SafeResult::new(result, context.to_string())
    }

    /// Safely get an element at index from a Vec
    pub fn safe_get_index<'a, T>(vec: &'a [T], index: usize, context: &str) -> SafeResult<&'a T> {
        let result = vec
            .get(index)
            .ok_or_else(|| SafeError::ResourceUnavailable {
                message: format!(
                    "Index {} out of bounds in {} vector (len: {})",
                    index,
                    context,
                    vec.len()
                ),
                resource: context.to_string(),
            });

        SafeResult::new(result, context.to_string())
    }

    /// Safely join a tokio task
    pub async fn safe_join<T>(handle: tokio::task::JoinHandle<T>, context: &str) -> SafeResult<T> {
        let result = handle.await.map_err(|e| SafeError::Internal {
            message: format!("Task join failed in {}: {}", context, e),
        });

        SafeResult::new(result, context.to_string())
    }

    /// Safely send a value through a channel
    pub async fn safe_send<T>(
        sender: &tokio::sync::mpsc::Sender<T>,
        value: T,
        context: &str,
    ) -> SafeResult<()> {
        let result = sender.send(value).await.map_err(|e| SafeError::Channel {
            message: format!("Channel send failed in {}: {}", context, e),
            channel_type: "mpsc".to_string(),
        });

        SafeResult::new(result, context.to_string())
    }

    /// Safely receive a value from a channel
    pub async fn safe_receive<T>(
        receiver: &mut tokio::sync::mpsc::Receiver<T>,
        context: &str,
    ) -> SafeResult<Option<T>> {
        let result = Ok(receiver.recv().await);
        SafeResult::new(result, context.to_string())
    }

    /// Safely execute an HTTP request
    pub async fn safe_http_request(
        client: &reqwest::Client,
        url: &str,
        context: &str,
    ) -> SafeResult<reqwest::Response> {
        let result = client
            .get(url)
            .send()
            .await
            .map_err(|e| SafeError::Network {
                message: format!("HTTP request failed in {}: {}", context, e),
                endpoint: Some(url.to_string()),
            });

        SafeResult::new(result, context.to_string())
    }

    /// Safely read a file
    pub async fn safe_read_file(path: &str, context: &str) -> SafeResult<String> {
        let result = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| SafeError::Internal {
                message: format!("File read failed in {}: {}", context, e),
            });

        SafeResult::new(result, context.to_string())
    }

    /// Safely write a file
    pub async fn safe_write_file(path: &str, content: &str, context: &str) -> SafeResult<()> {
        let result = tokio::fs::write(path, content)
            .await
            .map_err(|e| SafeError::Internal {
                message: format!("File write failed in {}: {}", context, e),
            });

        SafeResult::new(result, context.to_string())
    }

    /// Safely parse a URL
    pub fn safe_parse_url(url: &str, context: &str) -> SafeResult<url::Url> {
        let result = url::Url::parse(url).map_err(|e| SafeError::Validation {
            message: format!("URL parsing failed in {}: {}", context, e),
            field: Some("url".to_string()),
        });

        SafeResult::new(result, context.to_string())
    }

    /// Safely execute a closure with timeout
    pub async fn safe_with_timeout<T, F, Fut>(
        timeout: Duration,
        operation: F,
        context: &str,
    ) -> SafeResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let result = tokio::time::timeout(timeout, operation())
            .await
            .map_err(|_| SafeError::Timeout {
                message: format!("Operation timed out in {}", context),
                duration: timeout,
            });

        SafeResult::new(result, context.to_string())
    }
}

/// Macros for safe operations
#[macro_export]
macro_rules! safe_unwrap {
    ($result:expr, $context:expr) => {
        $result.unwrap_or_else(|e| {
            tracing::error!("Safe unwrap failed in {}: {}", $context, e);
            Default::default()
        })
    };
}

#[macro_export]
macro_rules! safe_expect {
    ($result:expr, $context:expr, $default:expr) => {
        $result.unwrap_or_else(|e| {
            tracing::error!("Safe expect failed in {}: {}", $context, e);
            $default
        })
    };
}

#[macro_export]
macro_rules! safe_option {
    ($option:expr, $context:expr) => {
        $option.unwrap_or_else(|| {
            tracing::warn!("Safe option returned None in {}", $context);
            Default::default()
        })
    };
}

#[macro_export]
macro_rules! safe_option_with_default {
    ($option:expr, $context:expr, $default:expr) => {
        $option.unwrap_or_else(|| {
            tracing::warn!("Safe option returned None in {}", $context);
            $default
        })
    };
}

/// Safe configuration helpers
pub struct SafeConfig;

impl SafeConfig {
    /// Safely get an environment variable
    pub fn safe_env_var(key: &str, default: &str, context: &str) -> String {
        std::env::var(key).unwrap_or_else(|_| {
            debug!(
                "Environment variable '{}' not found in {}, using default: {}",
                key, context, default
            );
            default.to_string()
        })
    }

    /// Safely parse an environment variable
    pub fn safe_env_parse<T>(key: &str, default: T, context: &str) -> T
    where
        T: std::str::FromStr + Clone,
        T::Err: std::fmt::Display,
    {
        std::env::var(key)
            .and_then(|val| {
                val.parse::<T>()
                    .map_err(|_e| std::env::VarError::NotPresent)
            })
            .unwrap_or_else(|_| {
                debug!(
                    "Environment variable '{}' not found or invalid in {}, using default",
                    key, context
                );
                default
            })
    }

    /// Safely get a configuration value from a HashMap
    pub fn safe_config_get<V>(
        config: &HashMap<String, V>,
        key: &str,
        default: V,
        context: &str,
    ) -> V
    where
        V: Clone,
    {
        config.get(key).cloned().unwrap_or_else(|| {
            debug!(
                "Configuration key '{}' not found in {}, using default",
                key, context
            );
            default
        })
    }
}

/// Safe session management
pub struct SafeSession;

impl SafeSession {
    /// Safely create a session ID
    pub fn safe_session_id(_context: &str) -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Safely get current timestamp
    pub fn safe_timestamp() -> DateTime<Utc> {
        Utc::now()
    }

    /// Safely validate session data
    pub fn safe_validate_session(
        session_data: &HashMap<String, String>,
        required_fields: &[&str],
        context: &str,
    ) -> SafeResult<()> {
        for field in required_fields {
            if !session_data.contains_key(*field) {
                return SafeResult::failure(
                    SafeError::Validation {
                        message: format!("Required field '{}' missing in session data", field),
                        field: Some(field.to_string()),
                    },
                    context.to_string(),
                );
            }
        }

        SafeResult::success((), context.to_string())
    }
}

/// Safe service operations
pub struct SafeService;

impl SafeService {
    /// Safely check service health
    pub async fn safe_health_check(
        client: &reqwest::Client,
        endpoint: &str,
        context: &str,
    ) -> SafeResult<bool> {
        let result = match client.get(&format!("{}/health", endpoint)).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => Err(SafeError::Network {
                message: format!("Health check failed: {}", e),
                endpoint: Some(endpoint.to_string()),
            }),
        };

        SafeResult::new(result, context.to_string())
    }

    /// Safely register a service
    pub async fn safe_service_register(
        client: &reqwest::Client,
        endpoint: &str,
        registration_data: &serde_json::Value,
        context: &str,
    ) -> SafeResult<()> {
        let result = client
            .post(&format!("{}/register", endpoint))
            .json(registration_data)
            .send()
            .await
            .and_then(|response| {
                if response.status().is_success() {
                    Ok(())
                } else {
                    Err(reqwest::Error::from(
                        response.error_for_status().unwrap_err(),
                    ))
                }
            })
            .map_err(|e| SafeError::Network {
                message: format!("Service registration failed: {}", e),
                endpoint: Some(endpoint.to_string()),
            });

        SafeResult::new(result, context.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::{Mutex, RwLock};

    #[test]
    fn test_safe_result_success() {
        let result = SafeResult::success(42, "test context".to_string());
        assert_eq!(result.unwrap_or_default(), 42);
    }

    #[test]
    fn test_safe_result_failure() {
        let result: SafeResult<i32> = SafeResult::failure(
            SafeError::Configuration {
                message: "Test error".to_string(),
                field: None,
            },
            "test context".to_string(),
        );
        assert_eq!(result.unwrap_or_default(), 0);
    }

    #[test]
    fn test_safe_parse() {
        let result = SafeOps::safe_parse::<i32>("42", "test parse");
        assert_eq!(result.unwrap_or_default(), 42);

        let result = SafeOps::safe_parse::<i32>("invalid", "test parse");
        assert_eq!(result.unwrap_or_default(), 0);
    }

    #[test]
    fn test_safe_get() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), "value".to_string());

        let result = SafeOps::safe_get_cloned(&map, &"key".to_string(), "test map");
        assert_eq!(result.execute().unwrap(), "value");

        let result = SafeOps::safe_get_cloned(&map, &"missing".to_string(), "test map");
        assert!(result.execute().is_err());
    }

    #[test]
    fn test_safe_first() {
        let vec = vec![1, 2, 3];
        let result = SafeOps::safe_first(&vec, "test vec");
        match result.result {
            Ok(value) => assert_eq!(value, &1),
            Err(_) => panic!("Expected success"),
        }

        let empty_vec: Vec<i32> = vec![];
        let result = SafeOps::safe_first(&empty_vec, "test vec");
        match result.result {
            Ok(_) => panic!("Expected error"),
            Err(_) => {} // Expected
        }
    }

    #[test]
    fn test_safe_config() {
        let result = SafeConfig::safe_env_var("NONEXISTENT_VAR", "default", "test");
        assert_eq!(result, "default");

        let result = SafeConfig::safe_env_parse::<i32>("NONEXISTENT_VAR", 42, "test");
        assert_eq!(result, 42);
    }

    #[test]
    fn test_safe_session() {
        let session_id = SafeSession::safe_session_id("test");
        assert!(!session_id.is_empty());

        let timestamp = SafeSession::safe_timestamp();
        assert!(timestamp <= Utc::now());

        let mut session_data = HashMap::new();
        session_data.insert("user_id".to_string(), "123".to_string());

        let result = SafeSession::safe_validate_session(&session_data, &["user_id"], "test");
        assert!(result.execute().is_ok());

        let result =
            SafeSession::safe_validate_session(&session_data, &["user_id", "missing"], "test");
        assert!(result.execute().is_err());
    }

    #[tokio::test]
    async fn test_safe_locks() {
        let data = Arc::new(RwLock::new(42));
        let result = SafeOps::safe_read_lock(&data, "test read lock").await;
        match result.result {
            Ok(guard) => {
                assert_eq!(*guard, 42);
            }
            Err(_) => panic!("Read lock should succeed"),
        }

        let result = SafeOps::safe_write_lock(&data, "test write lock").await;
        match result.result {
            Ok(mut guard) => {
                *guard = 84;
                assert_eq!(*guard, 84);
            }
            Err(_) => panic!("Write lock should succeed"),
        }

        let mutex_data = Arc::new(Mutex::new(42));
        let result = SafeOps::safe_mutex_lock(&mutex_data, "test mutex lock").await;
        match result.result {
            Ok(guard) => {
                assert_eq!(*guard, 42);
            }
            Err(_) => panic!("Mutex lock should succeed"),
        }
    }

    #[tokio::test]
    async fn test_safe_channels() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);

        let result = SafeOps::safe_send(&tx, 42, "test send").await;
        assert!(result.execute().is_ok());

        let result = SafeOps::safe_receive(&mut rx, "test receive").await;
        let received = result.execute().unwrap();
        assert_eq!(received, Some(42));
    }

    #[test]
    fn test_safe_serialize() {
        let data = HashMap::from([("key".to_string(), "value".to_string())]);
        let result = SafeOps::safe_serialize(&data, "test serialize");
        let json = match result.execute() {
            Ok(json) => json,
            Err(_) => {
                panic!("Serialization should succeed");
            }
        };

        let result =
            SafeOps::safe_deserialize::<HashMap<String, String>>(&json, "test deserialize");
        assert!(result.execute().is_ok());
    }

    #[test]
    fn test_safe_url_parse() {
        let result = SafeOps::safe_parse_url("https://example.com", "test url");
        match result.result {
            Ok(url) => assert_eq!(url.scheme(), "https"),
            Err(_) => panic!("Expected valid URL"),
        }

        let result = SafeOps::safe_parse_url("invalid-url", "test url");
        match result.result {
            Ok(_) => panic!("Expected error for invalid URL"),
            Err(_) => {} // Expected
        }
    }
}
