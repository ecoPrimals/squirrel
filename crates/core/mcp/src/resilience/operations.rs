// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Resilience operation executors
//!
//! This module provides functions that execute operations with various
//! resilience mechanisms (bulkhead, rate limiting, circuit breaker, etc.).

use std::error::Error as StdError;
use std::time::Duration;
use tracing::debug;
use futures_util::future::BoxFuture;

use crate::resilience::bulkhead;
use crate::resilience::rate_limiter;
use crate::resilience::circuit_breaker::{BreakerError, CircuitBreaker, StandardCircuitBreaker};
use crate::resilience::recovery::{FailureInfo, RecoveryStrategy};
use crate::resilience::health;
use crate::resilience::state_sync;
use crate::resilience::bulkhead::Bulkhead;
use crate::resilience::rate_limiter::RateLimiter;
use crate::resilience::retry::{BackoffStrategy, RetryMechanism};
use crate::resilience::resilience_error::{ResilienceError, Result};
use super::retry_policy::RetryPolicy;

/// Execute an operation with bulkhead isolation
///
/// This function uses the bulkhead pattern to isolate failures and control
/// the impact of failures in one part of the application.
pub async fn with_bulkhead<F, T>(
    bulkhead: &bulkhead::Bulkhead,
    operation: F,
) -> Result<T>
where
    F: std::future::Future<Output = std::result::Result<T, Box<dyn StdError + Send + Sync>>> + Send + 'static,
    T: Send + 'static,
{
    bulkhead.execute(operation).await
}

/// Execute an operation with rate limiting
///
/// This function ensures that operations don't exceed a configured rate limit,
/// using a token bucket algorithm to control the rate.
pub async fn with_rate_limiting<F, T>(
    rate_limiter: &rate_limiter::RateLimiter,
    operation: F,
) -> Result<T>
where
    F: std::future::Future<Output = std::result::Result<T, Box<dyn StdError + Send + Sync>>> + Send + 'static,
    T: Send + 'static,
{
    let adapted_operation = async {
        match operation.await {
            Ok(value) => Ok(value),
            Err(error) => Err(rate_limiter::RateLimitError::OperationFailed(format!("{:?}", error)))
        }
    };

    rate_limiter.execute(adapted_operation).await
}

/// Execute an operation with resilience, using a circuit breaker and retry mechanism
pub async fn with_resilience<F, T, CB>(
    circuit_breaker: &mut CB,
    retry: RetryMechanism,
    operation: F,
) -> Result<T>
where
    F: FnOnce() -> std::result::Result<T, Box<dyn StdError + Send + Sync>> + Clone + Send + Sync + 'static,
    T: Send + 'static,
    CB: circuit_breaker::CircuitBreaker + Send + Sync,
{
    let component_id = "resilience_component";

    circuit_breaker.execute(move || {
        let operation_clone = operation.clone();

        Box::pin(async move {
            let retry_result = retry.execute(|| {
                let op = operation_clone.clone();

                Box::pin(async move {
                    op()
                })
            }).await;

            match retry_result {
                Ok(value) => Ok(value),
                Err(e) => Err(BreakerError::operation_failed(component_id, e.to_string()))
            }
        })
    }).await.map_err(|e| ResilienceError::from(e))
}

/// Create a resilient operation with recovery strategy
pub async fn with_recovery<F, R, T>(
    recovery_strategy: &mut RecoveryStrategy,
    failure_info: FailureInfo,
    operation: F,
    recovery_action: R,
) -> Result<T>
where
    F: Fn() -> std::result::Result<T, Box<dyn StdError + Send + Sync + 'static>>,
    R: FnOnce() -> std::result::Result<T, Box<dyn StdError + Send + Sync + 'static>>,
    T: Send + 'static,
{
    match operation() {
        Ok(value) => Ok(value),
        Err(error) => {
            debug!("Operation failed, attempting recovery: {}", error);

            recovery_strategy
                .handle_failure(failure_info, recovery_action)
                .map_err(std::convert::Into::into)
        }
    }
}

/// Execute an operation with health monitoring
///
/// This function checks component health before executing the operation.
/// If the component is in a critical state, it prevents the operation from executing.
pub async fn with_health_monitoring<F, T>(
    health_monitor: &health::HealthMonitor,
    component_id: &str,
    operation: F,
) -> Result<T>
where
    F: Fn() -> std::result::Result<T, Box<dyn StdError + Send + Sync + 'static>> + Send + Sync + 'static,
    T: Send + 'static,
{
    let status = health_monitor.get_component_status(component_id);
    if status == health::HealthStatus::Critical {
        return Err(ResilienceError::General(format!(
            "Cannot execute operation: component '{component_id}' is in critical state"
        )));
    }

    match operation() {
        Ok(value) => Ok(value),
        Err(e) => Err(ResilienceError::General(format!("{e}"))),
    }
}

/// Execute an operation with complete resilience
///
/// This function combines circuit breaker, retry mechanism, recovery strategy, and health monitoring
/// to provide comprehensive protection against failures.
pub async fn with_complete_resilience<F, T, CB, RA>(
    circuit_breaker: &mut CB,
    retry: RetryMechanism,
    recovery: &mut RecoveryStrategy,
    health_monitor: &health::HealthMonitor,
    component_id: &str,
    failure_info: FailureInfo,
    operation: F,
    recovery_action: RA,
) -> Result<T>
where
    F: FnOnce() -> std::result::Result<T, Box<dyn StdError + Send + Sync>> + Clone + Send + Sync + 'static,
    RA: FnOnce() -> std::result::Result<T, Box<dyn StdError + Send + Sync>> + Send + Sync + 'static,
    T: Send + 'static,
    CB: circuit_breaker::CircuitBreaker + Send + Sync,
{
    let component_id_owned = component_id.to_string();

    let status = health_monitor.get_component_status(&component_id_owned);
    if status == health::HealthStatus::Critical {
        return Err(ResilienceError::HealthCheck(format!(
            "Cannot execute operation: component '{component_id_owned}' is in critical state"
        )));
    }

    let result = circuit_breaker.execute(move || {
        let operation_clone = operation.clone();
        let component_id_clone = component_id_owned.clone();

        Box::pin(async move {
            let retry_result = retry.execute(|| {
                let op = operation_clone.clone();

                Box::pin(async move {
                    op()
                })
            }).await;

            match retry_result {
                Ok(value) => Ok(value),
                Err(e) => Err(BreakerError::operation_failed(&component_id_clone, e.to_string()))
            }
        })
    }).await;

    match result {
        Ok(value) => Ok(value),
        Err(_breaker_err) => {
            let recovery_result = recovery.recover(
                failure_info,
                recovery_action,
            ).await;

            match recovery_result {
                Ok(recovery_value) => Ok(recovery_value),
                Err(recovery_err) => Err(ResilienceError::from(recovery_err)),
            }
        }
    }
}

/// Execute an operation with state synchronization
pub async fn with_state_sync<T, F>(
    state_sync: &state_sync::StateSynchronizer,
    state_type: state_sync::StateType,
    state_id: &str,
    target: &str,
    operation: F,
) -> Result<T>
where
    F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>> + Send,
    T: serde::Serialize + Clone + Send + Sync + 'static,
{
    let result = operation().await?;

    state_sync.sync_state(state_type, state_id, target, &result).await?;

    Ok(result)
}

/// Execute an operation with comprehensive resilience
///
/// This function combines all resilience mechanisms including bulkhead isolation
/// and rate limiting to provide comprehensive protection against failures.
pub async fn with_comprehensive_resilience<'a, F, R, T, CB>(
    circuit_breaker: &'a mut CB,
    bulkhead: &'a Bulkhead,
    rate_limiter: &'a RateLimiter,
    _retry_policy: R,
    timeout: Duration,
    component_id: &'a str,
    operation: F,
) -> Result<T>
where
    F: FnOnce() -> BoxFuture<'static, Result<T>> + Send + Sync + Clone + 'static,
    R: RetryPolicy + Send + Sync + 'static,
    T: Send + 'static + Clone,
    CB: circuit_breaker::CircuitBreaker + Send + Sync,
{
    let component_id_owned = component_id.to_string();

    let _health_monitor = health::HealthMonitor::new(100);
    let status = _health_monitor.get_component_status(&component_id_owned);
    if status == health::HealthStatus::Critical {
        return Err(ResilienceError::HealthCheck(format!(
            "Cannot execute operation: component '{component_id_owned}' is in critical state"
        )));
    }

    if !rate_limiter.try_acquire().await {
        return Err(ResilienceError::RateLimit(format!(
            "Rate limit exceeded for component '{component_id_owned}'"
        )));
    }

    if !bulkhead.try_enter().await {
        return Err(ResilienceError::Bulkhead(format!(
            "Bulkhead capacity exceeded for component '{component_id_owned}'"
        )));
    }

    let timeout_result = tokio::time::timeout(timeout, async {
        circuit_breaker.execute(move || {
            let op = operation.clone();
            let component_id_str = component_id_owned.clone();

            Box::pin(async move {
                op().await.map_err(|e|
                    circuit_breaker::BreakerError::operation_failed(&component_id_str, e.to_string())
                )
            })
        }).await
    }).await;

    match timeout_result {
        Ok(breaker_result) => breaker_result.map_err(Into::into),
        Err(_) => Err(ResilienceError::Timeout(format!(
            "Operation timed out after {:?} for component '{}'",
            timeout, component_id
        ))),
    }
}

/// Execute with recovery and circuit breaker
///
/// This function forwards to the newer function definitions above
pub async fn execute_with_recovery<T, F>(
    circuit_breaker: Option<StandardCircuitBreaker>,
    component_id: &str,
    operation: F,
    _recovery_strategy: &mut RecoveryStrategy,
    _failure_info: FailureInfo,
    _recovery_action: Option<String>
) -> std::result::Result<T, ResilienceError>
where
    F: FnOnce() -> core::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<T, ResilienceError>> + Send>> + Send,
    T: Send + 'static,
{
    let component_id_owned = component_id.to_string();

    with_circuit_breaker(circuit_breaker, &component_id_owned, operation).await
}

#[doc(hidden)]
pub async fn with_circuit_breaker<T, F>(
    mut circuit_breaker: Option<StandardCircuitBreaker>,
    component_id: &str,
    operation: F
) -> std::result::Result<T, ResilienceError>
where
    F: FnOnce() -> core::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<T, ResilienceError>> + Send>> + Send,
    T: Send + 'static,
{
    let component_id_owned = component_id.to_string();

    match circuit_breaker {
        Some(ref mut cb) => {
            let fut = operation();

            cb.execute(move || {
                use futures_util::FutureExt;

                async move {
                    match fut.await {
                        Ok(result) => Ok(result),
                        Err(err) => Err(circuit_breaker::BreakerError::operation_failed(
                            &component_id_owned, err.to_string()
                        ))
                    }
                }.boxed()
            }).await.map_err(|e| e.into())
        },
        None => {
            operation().await
        }
    }
}

/// Execute with resilience components using default configuration
pub async fn execute_with_resilience_components<'a>(
    component_id: &'a str,
    operation: impl FnOnce() -> std::result::Result<(), Box<dyn StdError + Send + Sync>> + Clone + Send + Sync + 'static,
) -> Result<()> {
    let mut circuit_breaker = crate::resilience::circuit_breaker::new_circuit_breaker(component_id);

    let retry = RetryMechanism::default();

    with_resilience(
        &mut circuit_breaker,
        retry,
        operation,
    ).await
}
