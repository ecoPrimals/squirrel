// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! # Graceful Shutdown Manager
//!
//! This module provides comprehensive shutdown coordination for all system components,
//! ensuring proper resource cleanup, connection closure, and graceful service termination.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Notify, RwLock, mpsc};
use tracing::{debug, error, info, warn};

use crate::error::PrimalError;
use crate::observability::{CorrelationId, OperationContext};

/// Shutdown phase definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShutdownPhase {
    /// Stop accepting new requests/connections
    StopAccepting = 0,
    /// Drain existing requests
    DrainRequests = 1,
    /// Close network connections
    CloseConnections = 2,
    /// Cleanup resources (files, memory, etc.)
    CleanupResources = 3,
    /// Shutdown background tasks
    ShutdownTasks = 4,
    /// Final cleanup and exit
    FinalCleanup = 5,
}

impl ShutdownPhase {
    /// Returns a human-readable description of the shutdown phase.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::StopAccepting => "Stop accepting new requests",
            Self::DrainRequests => "Drain existing requests",
            Self::CloseConnections => "Close network connections",
            Self::CleanupResources => "Cleanup resources",
            Self::ShutdownTasks => "Shutdown background tasks",
            Self::FinalCleanup => "Final cleanup",
        }
    }
}

/// Shutdown handler trait for components
#[async_trait::async_trait]
pub trait ShutdownHandler: Send + Sync {
    /// Component name for logging
    fn component_name(&self) -> &str;

    /// Execute shutdown for this component
    async fn shutdown(&self, phase: ShutdownPhase) -> Result<(), PrimalError>;

    /// Check if component has finished shutdown
    async fn is_shutdown_complete(&self) -> bool;

    /// Get estimated shutdown time for planning
    fn estimated_shutdown_time(&self) -> Duration {
        Duration::from_secs(10) // Default 10 seconds
    }
}

/// Shutdown manager coordinates graceful shutdown across all components
pub struct ShutdownManager {
    /// Registered shutdown handlers by component
    handlers: Arc<RwLock<HashMap<String, Arc<dyn ShutdownHandler>>>>,

    /// Shutdown notification
    shutdown_notify: Arc<Notify>,

    /// Shutdown completion tracking
    shutdown_complete: Arc<RwLock<bool>>,

    /// Shutdown requested tracking
    shutdown_requested: Arc<RwLock<bool>>,

    /// Shutdown timeout configuration
    phase_timeouts: HashMap<ShutdownPhase, Duration>,

    /// Shutdown coordination channels
    shutdown_tx: Option<mpsc::Sender<ShutdownSignal>>,
    shutdown_rx: Arc<RwLock<Option<mpsc::Receiver<ShutdownSignal>>>>,
}

/// Shutdown signal types
#[derive(Debug, Clone)]
pub enum ShutdownSignal {
    /// Graceful shutdown request
    Graceful,
    /// Immediate shutdown request
    Immediate,
    /// Shutdown timeout exceeded
    Timeout(ShutdownPhase),
}

impl ShutdownManager {
    /// Create a new shutdown manager
    #[must_use]
    pub fn new() -> Self {
        let (shutdown_tx, shutdown_rx) = mpsc::channel(10);

        let mut phase_timeouts = HashMap::new();
        phase_timeouts.insert(ShutdownPhase::StopAccepting, Duration::from_secs(5));
        phase_timeouts.insert(ShutdownPhase::DrainRequests, Duration::from_secs(30));
        phase_timeouts.insert(ShutdownPhase::CloseConnections, Duration::from_secs(10));
        phase_timeouts.insert(ShutdownPhase::CleanupResources, Duration::from_secs(15));
        phase_timeouts.insert(ShutdownPhase::ShutdownTasks, Duration::from_secs(10));
        phase_timeouts.insert(ShutdownPhase::FinalCleanup, Duration::from_secs(5));

        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            shutdown_notify: Arc::new(Notify::new()),
            shutdown_complete: Arc::new(RwLock::new(false)),
            shutdown_requested: Arc::new(RwLock::new(false)),
            phase_timeouts,
            shutdown_tx: Some(shutdown_tx),
            shutdown_rx: Arc::new(RwLock::new(Some(shutdown_rx))),
        }
    }

    /// Register a component for shutdown coordination
    pub async fn register_handler(
        &self,
        component_name: String,
        handler: Arc<dyn ShutdownHandler>,
    ) {
        let mut handlers = self.handlers.write().await;
        handlers.insert(component_name.clone(), handler);

        info!(
            component = %component_name,
            operation = "shutdown_handler_registered",
            "Component registered for graceful shutdown"
        );
    }

    /// Unregister a component (if needed during runtime)
    pub async fn unregister_handler(&self, component_name: &str) -> bool {
        let mut handlers = self.handlers.write().await;
        let removed = handlers.remove(component_name).is_some();

        if removed {
            info!(
                component = %component_name,
                operation = "shutdown_handler_unregistered",
                "Component unregistered from shutdown coordination"
            );
        }

        removed
    }

    /// Request graceful shutdown
    pub async fn request_shutdown(&self) -> Result<(), PrimalError> {
        // Set shutdown requested flag
        *self.shutdown_requested.write().await = true;

        if let Some(ref tx) = self.shutdown_tx {
            tx.send(ShutdownSignal::Graceful).await.map_err(|e| {
                PrimalError::Internal(format!("Failed to send shutdown signal: {e}"))
            })?;
        }

        info!(
            operation = "shutdown_requested",
            "Graceful shutdown requested"
        );

        Ok(())
    }

    /// Start the shutdown coordination process
    pub async fn coordinate_shutdown(&self) -> Result<(), PrimalError> {
        let correlation_id = CorrelationId::new();
        let ctx = OperationContext::with_correlation_id("system_shutdown", correlation_id);
        ctx.log_start();

        // Take ownership of the receiver
        let mut shutdown_rx = {
            let mut rx_guard = self.shutdown_rx.write().await;
            rx_guard.take().ok_or_else(|| {
                PrimalError::Internal("Shutdown receiver already taken".to_string())
            })?
        };

        info!(
            correlation_id = %ctx.correlation_id,
            operation = "shutdown_coordination_start",
            "Starting shutdown coordination"
        );

        // Wait for shutdown signal
        match shutdown_rx.recv().await {
            Some(ShutdownSignal::Graceful) => {
                info!(
                    correlation_id = %ctx.correlation_id,
                    signal_type = "graceful",
                    operation = "shutdown_signal_received",
                    "Received graceful shutdown signal"
                );
                self.execute_graceful_shutdown(&ctx).await?;
            }
            Some(ShutdownSignal::Immediate) => {
                warn!(
                    correlation_id = %ctx.correlation_id,
                    signal_type = "immediate",
                    operation = "shutdown_signal_received",
                    "Received immediate shutdown signal"
                );
                self.execute_immediate_shutdown(&ctx).await?;
            }
            Some(ShutdownSignal::Timeout(phase)) => {
                error!(
                    correlation_id = %ctx.correlation_id,
                    phase = ?phase,
                    operation = "shutdown_timeout",
                    "Shutdown timeout exceeded"
                );
                return Err(PrimalError::Internal(format!(
                    "Shutdown timeout in phase: {phase:?}"
                )));
            }
            None => {
                warn!(
                    correlation_id = %ctx.correlation_id,
                    operation = "shutdown_channel_closed",
                    "Shutdown channel closed unexpectedly"
                );
            }
        }

        // Mark shutdown complete
        {
            let mut complete = self.shutdown_complete.write().await;
            *complete = true;
        }

        let _ = ctx.complete_success();
        Ok(())
    }

    /// Execute graceful shutdown through all phases
    async fn execute_graceful_shutdown(&self, ctx: &OperationContext) -> Result<(), PrimalError> {
        let phases = [
            ShutdownPhase::StopAccepting,
            ShutdownPhase::DrainRequests,
            ShutdownPhase::CloseConnections,
            ShutdownPhase::CleanupResources,
            ShutdownPhase::ShutdownTasks,
            ShutdownPhase::FinalCleanup,
        ];

        for phase in &phases {
            let phase_start = Instant::now();
            let timeout = self
                .phase_timeouts
                .get(phase)
                .copied()
                .unwrap_or(Duration::from_secs(10));

            info!(
                correlation_id = %ctx.correlation_id,
                phase = ?phase,
                phase_description = phase.description(),
                timeout_ms = timeout.as_millis(),
                operation = "shutdown_phase_start",
                "Starting shutdown phase"
            );

            // Execute phase with timeout
            let phase_result =
                tokio::time::timeout(timeout, self.execute_shutdown_phase(*phase, ctx)).await;

            let phase_duration = phase_start.elapsed();

            match phase_result {
                Ok(Ok(())) => {
                    info!(
                        correlation_id = %ctx.correlation_id,
                        phase = ?phase,
                        duration_ms = phase_duration.as_millis(),
                        operation = "shutdown_phase_complete",
                        "Shutdown phase completed successfully"
                    );
                }
                Ok(Err(e)) => {
                    error!(
                        correlation_id = %ctx.correlation_id,
                        phase = ?phase,
                        duration_ms = phase_duration.as_millis(),
                        error = %e,
                        operation = "shutdown_phase_error",
                        "Shutdown phase failed"
                    );
                    return Err(e);
                }
                Err(_) => {
                    error!(
                        correlation_id = %ctx.correlation_id,
                        phase = ?phase,
                        timeout_ms = timeout.as_millis(),
                        operation = "shutdown_phase_timeout",
                        "Shutdown phase timed out"
                    );
                    return Err(PrimalError::Internal(format!(
                        "Shutdown phase {phase:?} timed out"
                    )));
                }
            }
        }

        info!(
            correlation_id = %ctx.correlation_id,
            operation = "graceful_shutdown_complete",
            "Graceful shutdown completed successfully"
        );

        Ok(())
    }

    /// Execute immediate shutdown (best effort, no phases)
    async fn execute_immediate_shutdown(&self, ctx: &OperationContext) -> Result<(), PrimalError> {
        let handlers = self.handlers.read().await;
        let mut shutdown_tasks = Vec::new();

        info!(
            correlation_id = %ctx.correlation_id,
            component_count = handlers.len(),
            operation = "immediate_shutdown_start",
            "Starting immediate shutdown of all components"
        );

        // Start shutdown for all components simultaneously
        for (component_name, handler) in handlers.iter() {
            let handler_clone = handler.clone();
            let component_name_clone = component_name.clone();
            let correlation_id = ctx.correlation_id.clone();

            let task = tokio::spawn(async move {
                let timeout = Duration::from_secs(5); // Short timeout for immediate shutdown

                let shutdown_result = tokio::time::timeout(
                    timeout,
                    handler_clone.shutdown(ShutdownPhase::FinalCleanup),
                )
                .await;

                match shutdown_result {
                    Ok(Ok(())) => {
                        info!(
                            correlation_id = %correlation_id,
                            component = %component_name_clone,
                            operation = "immediate_shutdown_component_success",
                            "Component shutdown completed"
                        );
                    }
                    Ok(Err(e)) => {
                        error!(
                            correlation_id = %correlation_id,
                            component = %component_name_clone,
                            error = %e,
                            operation = "immediate_shutdown_component_error",
                            "Component shutdown failed"
                        );
                    }
                    Err(_) => {
                        error!(
                            correlation_id = %correlation_id,
                            component = %component_name_clone,
                            operation = "immediate_shutdown_component_timeout",
                            "Component shutdown timed out"
                        );
                    }
                }
            });

            shutdown_tasks.push(task);
        }

        // Wait for all shutdowns to complete or timeout
        let mut successful = 0;
        let mut failed = 0;

        for task in shutdown_tasks {
            match task.await {
                Ok(()) => successful += 1,
                Err(_) => failed += 1,
            }
        }

        info!(
            correlation_id = %ctx.correlation_id,
            successful_shutdowns = successful,
            failed_shutdowns = failed,
            operation = "immediate_shutdown_complete",
            "Immediate shutdown completed"
        );

        Ok(())
    }

    /// Execute a specific shutdown phase
    async fn execute_shutdown_phase(
        &self,
        phase: ShutdownPhase,
        ctx: &OperationContext,
    ) -> Result<(), PrimalError> {
        let handlers = self.handlers.read().await;
        let mut phase_tasks = Vec::new();

        // Start phase for all components
        for (component_name, handler) in handlers.iter() {
            let handler_clone = handler.clone();
            let component_name_clone = component_name.clone();
            let correlation_id = ctx.correlation_id.clone();

            let task = tokio::spawn(async move {
                debug!(
                    correlation_id = %correlation_id,
                    component = %component_name_clone,
                    phase = ?phase,
                    operation = "component_phase_start",
                    "Starting shutdown phase for component"
                );

                match handler_clone.shutdown(phase).await {
                    Ok(()) => {
                        debug!(
                            correlation_id = %correlation_id,
                            component = %component_name_clone,
                            phase = ?phase,
                            operation = "component_phase_success",
                            "Component shutdown phase completed"
                        );
                        Ok(())
                    }
                    Err(e) => {
                        error!(
                            correlation_id = %correlation_id,
                            component = %component_name_clone,
                            phase = ?phase,
                            error = %e,
                            operation = "component_phase_error",
                            "Component shutdown phase failed"
                        );
                        Err(e)
                    }
                }
            });

            phase_tasks.push((component_name.clone(), task));
        }

        // Wait for all components to complete the phase
        let mut errors = Vec::new();

        for (component_name, task) in phase_tasks {
            match task.await {
                Ok(Ok(())) => {
                    // Success
                }
                Ok(Err(e)) => {
                    errors.push(format!("{component_name}: {e}"));
                }
                Err(e) => {
                    errors.push(format!("{component_name}: Task panicked: {e}"));
                }
            }
        }

        if !errors.is_empty() {
            return Err(PrimalError::Internal(format!(
                "Shutdown phase {:?} failed for components: {}",
                phase,
                errors.join(", ")
            )));
        }

        Ok(())
    }

    /// Phase timeout configuration (used for testing and introspection).
    #[cfg(test)]
    #[must_use]
    pub(crate) fn phase_timeout(&self, phase: ShutdownPhase) -> Option<Duration> {
        self.phase_timeouts.get(&phase).copied()
    }

    /// Check if shutdown is complete
    pub async fn is_shutdown_complete(&self) -> bool {
        *self.shutdown_complete.read().await
    }

    /// Check if shutdown was requested
    #[must_use]
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested
            .try_read()
            .map(|guard| *guard)
            .unwrap_or(false)
    }

    /// Wait for shutdown completion
    pub async fn wait_for_shutdown(&self) {
        self.shutdown_notify.notified().await;
    }
}

impl Default for ShutdownManager {
    fn default() -> Self {
        Self::new()
    }
}
