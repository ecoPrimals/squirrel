// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Service Health Monitor Implementation
//!
//! This module contains the health monitoring functionality for services.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, warn, error};

use crate::error::Result;
use super::types::{HealthCheckConfig, ServiceHealthStatus, ServiceHealth, HealthStatus};

/// Service health monitor
#[derive(Debug)]
pub struct ServiceHealthMonitor {
    /// Health check configurations
    health_checks: Arc<RwLock<HashMap<String, HealthCheckConfig>>>,
    /// Health status cache
    health_status: Arc<RwLock<HashMap<String, ServiceHealthStatus>>>,
    /// Health check interval
    check_interval: Duration,
}

impl ServiceHealthMonitor {
    /// Create a new service health monitor
    pub fn new(check_interval: Duration) -> Self {
        Self {
            health_checks: Arc::new(RwLock::new(HashMap::new())),
            health_status: Arc::new(RwLock::new(HashMap::new())),
            check_interval,
        }
    }
    
    /// Add health check configuration for a service
    pub async fn add_health_check(&self, service_id: String, config: HealthCheckConfig) {
        let mut checks = self.health_checks.write().await;
        checks.insert(service_id, config);
    }
    
    /// Remove health check configuration for a service
    pub async fn remove_health_check(&self, service_id: &str) {
        let mut checks = self.health_checks.write().await;
        checks.remove(service_id);
        
        let mut status = self.health_status.write().await;
        status.remove(service_id);
    }
    
    /// Get health status for a service
    pub async fn get_health_status(&self, service_id: &str) -> Option<ServiceHealthStatus> {
        let status = self.health_status.read().await;
        status.get(service_id).cloned()
    }
    
    /// Get all health statuses
    pub async fn get_all_health_statuses(&self) -> HashMap<String, ServiceHealthStatus> {
        let status = self.health_status.read().await;
        status.clone()
    }
    
    /// Perform health check for a specific service
    pub async fn check_service_health(&self, service_id: &str) -> Result<ServiceHealthStatus> {
        let config = {
            let checks = self.health_checks.read().await;
            checks.get(service_id).cloned()
        };
        
        let config = match config {
            Some(config) => config,
            None => {
                warn!("No health check configuration found for service: {}", service_id);
                return Ok(ServiceHealthStatus {
                    service_id: service_id.to_string(),
                    status: HealthStatus::Unknown,
                    last_check: chrono::Utc::now(),
                    details: HashMap::new(),
                });
            }
        };
        
        debug!("Performing health check for service: {}", service_id);
        
        let start_time = std::time::Instant::now();
        let health_status = self.perform_health_check(&config).await;
        let check_duration = start_time.elapsed();
        
        let status = ServiceHealthStatus {
            service_id: service_id.to_string(),
            status: health_status,
            last_check: chrono::Utc::now(),
            details: HashMap::from([
                ("check_duration_ms".to_string(), serde_json::json!(check_duration.as_millis())),
                ("endpoint".to_string(), serde_json::json!(config.endpoint)),
            ]),
        };
        
        // Update status cache
        {
            let mut status_cache = self.health_status.write().await;
            status_cache.insert(service_id.to_string(), status.clone());
        }
        
        Ok(status)
    }
    
    /// Perform health check for all configured services
    pub async fn check_all_services(&self) -> Result<HashMap<String, ServiceHealthStatus>> {
        let service_ids = {
            let checks = self.health_checks.read().await;
            checks.keys().cloned().collect::<Vec<_>>()
        };
        
        let mut results = HashMap::new();
        
        for service_id in service_ids {
            match self.check_service_health(&service_id).await {
                Ok(status) => {
                    results.insert(service_id, status);
                }
                Err(e) => {
                    error!("Health check failed for service {}: {}", service_id, e);
                    results.insert(service_id.clone(), ServiceHealthStatus {
                        service_id,
                        status: HealthStatus::Unhealthy,
                        last_check: chrono::Utc::now(),
                        details: HashMap::from([
                            ("error".to_string(), serde_json::json!(e.to_string())),
                        ]),
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    /// Start background health monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        let health_checks = Arc::clone(&self.health_checks);
        let health_status = Arc::clone(&self.health_status);
        let interval = self.check_interval;
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                let service_ids = {
                    let checks = health_checks.read().await;
                    checks.keys().cloned().collect::<Vec<_>>()
                };
                
                for service_id in service_ids {
                    // Perform health check (simplified)
                    let status = ServiceHealthStatus {
                        service_id: service_id.clone(),
                        status: HealthStatus::Healthy, // Simplified - would perform actual check
                        last_check: chrono::Utc::now(),
                        details: HashMap::new(),
                    };
                    
                    let mut status_cache = health_status.write().await;
                    status_cache.insert(service_id, status);
                }
            }
        });
        
        debug!("Started background health monitoring with interval: {:?}", interval);
        Ok(())
    }
    
    /// Perform the actual health check
    async fn perform_health_check(&self, config: &HealthCheckConfig) -> HealthStatus {
        // Simulate health check - in a real implementation, this would make HTTP requests
        // to the service's health endpoint
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Simple simulation based on endpoint
        if config.endpoint.contains("unhealthy") {
            HealthStatus::Unhealthy
        } else if config.endpoint.contains("degraded") {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
    
    /// Get health summary
    pub async fn get_health_summary(&self) -> HealthSummary {
        let statuses = self.health_status.read().await;
        
        let mut healthy_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;
        let mut unknown_count = 0;
        
        for status in statuses.values() {
            match status.status {
                HealthStatus::Healthy => healthy_count += 1,
                HealthStatus::Degraded => degraded_count += 1,
                HealthStatus::Unhealthy => unhealthy_count += 1,
                HealthStatus::Unknown => unknown_count += 1,
            }
        }
        
        let total_services = statuses.len();
        let overall_health = if unhealthy_count > 0 {
            HealthStatus::Unhealthy
        } else if degraded_count > 0 {
            HealthStatus::Degraded
        } else if healthy_count > 0 {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        };
        
        HealthSummary {
            overall_health,
            total_services,
            healthy_count,
            degraded_count,
            unhealthy_count,
            unknown_count,
            last_updated: chrono::Utc::now(),
        }
    }
}

/// Health summary
#[derive(Debug, Clone)]
pub struct HealthSummary {
    pub overall_health: HealthStatus,
    pub total_services: usize,
    pub healthy_count: usize,
    pub degraded_count: usize,
    pub unhealthy_count: usize,
    pub unknown_count: usize,
    pub last_updated: chrono::DateTime<chrono::Utc>,
} 