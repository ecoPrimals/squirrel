use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;
use tokio::sync::mpsc;
use tracing::{error, info, instrument, warn};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use anyhow::{Result, Context};
use std::time::Duration;

// MERGE NOTE: Using MCPError from main for better error context and severity levels
use crate::mcp::error::{MCPError, PortErrorKind, ErrorContext, ErrorSeverity};

// MERGE NOTE: Keeping PortConfig from main for better configuration management
#[derive(Debug, Clone)]
pub struct PortConfig {
    pub min_port: u16,
    pub max_port: u16,
    pub port: u16,
    pub protocol: String,
    pub max_connections: u32,
    pub timeout: Duration,
    pub keep_alive: bool,
    pub tls_enabled: bool,
    pub access_control: PortAccessControl,
    pub metrics_enabled: bool,
    pub reserved_ports: HashSet<u16>,
}

impl Default for PortConfig {
    fn default() -> Self {
        Self {
            min_port: 1024,
            max_port: 65535,
            port: 8080,
            protocol: "tcp".to_string(),
            max_connections: 10,
            timeout: Duration::from_secs(30),
            keep_alive: true,
            tls_enabled: false,
            access_control: PortAccessControl::AllowAll,
            metrics_enabled: true,
            reserved_ports: HashSet::new(),
        }
    }
}

// MERGE NOTE: Keeping PortAccessControl from main for security features
#[derive(Debug, Clone)]
pub enum PortAccessControl {
    AllowAll,
    AllowList(HashSet<u16>),
    DenyList(HashSet<u16>),
}

impl Default for PortAccessControl {
    fn default() -> Self {
        Self::AllowAll
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Critical,
}

// MERGE NOTE: Using enhanced PortState from main for better state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortState {
    pub port: u16,
    pub status: PortStatus,
    pub allocated_to: Option<String>,
    pub allocation_time: Option<chrono::DateTime<chrono::Utc>>,
    pub metrics: Option<PortMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortStatus {
    Available,
    Reserved,
    InUse,
    Blocked,
}

// MERGE NOTE: Keeping PortMetrics from main for better monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMetrics {
    pub connections: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub error_count: u64,
}

// MERGE NOTE: Using enhanced PortManager from main with better state management
#[derive(Debug)]
pub struct PortManager {
    config: PortConfig,
    port_states: Arc<TokioRwLock<HashMap<u16, PortState>>>,
    metrics_enabled: bool,
    reserved_ports: Arc<TokioRwLock<HashSet<u16>>>,
}

impl PortManager {
    // MERGE NOTE: Using new() from main with better config validation
    #[instrument(skip(config))]
    pub fn new(config: PortConfig) -> Result<Self, MCPError> {
        if config.min_port >= config.max_port {
            return Err(MCPError::Port {
                kind: PortErrorKind::InvalidRange,
                context: ErrorContext::new("initialize_port_manager", "port_manager")
                    .with_severity(ErrorSeverity::High)
                    .not_recoverable(),
                port: config.min_port,
            });
        }

        Ok(Self {
            config,
            port_states: Arc::new(TokioRwLock::new(HashMap::new())),
            metrics_enabled: true,
            reserved_ports: Arc::new(TokioRwLock::new(HashSet::new())),
        })
    }

    // MERGE NOTE: Using enhanced allocate_port from main with requester tracking
    #[instrument]
    pub async fn allocate_port(&self, requester: &str) -> Result<u16, MCPError> {
        let mut states = self.port_states.write().await;
        
        for port in self.config.min_port..=self.config.max_port {
            if self.config.reserved_ports.contains(&port) {
                continue;
            }

            if let Some(state) = states.get(&port) {
                if matches!(state.status, PortStatus::InUse | PortStatus::Reserved) {
                    continue;
                }
            }

            if let Err(e) = self.validate_port(port).await {
                warn!(port = port, error = %e, "Port validation failed");
                continue;
            }

            let state = PortState {
                port,
                status: PortStatus::InUse,
                allocated_to: Some(requester.to_string()),
                allocation_time: Some(chrono::Utc::now()),
                metrics: if self.metrics_enabled {
                    Some(PortMetrics {
                        connections: 0,
                        bytes_sent: 0,
                        bytes_received: 0,
                        last_activity: chrono::Utc::now(),
                        error_count: 0,
                    })
                } else {
                    None
                },
            };

            states.insert(port, state);
            info!(port = port, requester = requester, "Port allocated successfully");
            return Ok(port);
        }

        Err(MCPError::Port {
            kind: PortErrorKind::NotAvailable(0),
            context: ErrorContext::new("allocate_port", "port_manager")
                .with_severity(ErrorSeverity::Error),
            port: 0,
        })
    }

    // MERGE NOTE: Using enhanced validate_port from main with better error context
    #[instrument]
    pub async fn validate_port(&self, port: u16) -> Result<(), MCPError> {
        if port < self.config.min_port || port > self.config.max_port {
            return Err(MCPError::Port {
                kind: PortErrorKind::InvalidRange(port),
                severity: ErrorSeverity::Error,
                message: format!("Port {} is outside valid range", port),
            });
        }

        match &self.config.access_control {
            PortAccessControl::AllowAll => {}
            PortAccessControl::AllowList(allowed) => {
                if !allowed.contains(&port) {
                    return Err(MCPError::Port {
                        kind: PortErrorKind::AccessDenied(port),
                        severity: ErrorSeverity::Error,
                        message: format!("Port {} is not in allowed list", port),
                    });
                }
            }
            PortAccessControl::DenyList(denied) => {
                if denied.contains(&port) {
                    return Err(MCPError::Port {
                        kind: PortErrorKind::AccessDenied(port),
                        severity: ErrorSeverity::Error,
                        message: format!("Port {} is in denied list", port),
                    });
                }
            }
        }

        Ok(())
    }

    // MERGE NOTE: Using enhanced release_port from main with ownership verification
    #[instrument]
    pub async fn release_port(&self, port: u16, requester: &str) -> Result<(), MCPError> {
        let mut states = self.port_states.write().await;
        
        match states.get_mut(&port) {
            Some(state) => {
                if let Some(owner) = &state.allocated_to {
                    if owner != requester {
                        return Err(MCPError::Port {
                            kind: PortErrorKind::AccessDenied(port),
                            context: ErrorContext::new("release_port", "port_manager")
                                .with_severity(ErrorSeverity::Error),
                            port,
                        });
                    }
                }

                state.status = PortStatus::Available;
                state.allocated_to = None;
                state.allocation_time = None;
                if let Some(metrics) = &mut state.metrics {
                    metrics.last_activity = chrono::Utc::now();
                }

                info!(port = port, requester = requester, "Port released successfully");
                Ok(())
            }
            None => Err(MCPError::Port {
                kind: PortErrorKind::NotAvailable(port),
                context: ErrorContext::new("release_port", "port_manager")
                    .with_severity(ErrorSeverity::Error),
                port,
            }),
        }
    }

    // MERGE NOTE: Adding new methods from main for better port management
    #[instrument]
    pub async fn get_port_state(&self, port: u16) -> Result<PortState, MCPError> {
        let states = self.port_states.read().await;
        states.get(&port)
            .cloned()
            .ok_or_else(|| MCPError::Port {
                kind: PortErrorKind::NotAvailable(port),
                context: ErrorContext::new("get_port_state", "port_manager")
                    .with_severity(ErrorSeverity::Error),
                port,
            })
    }

    // MERGE NOTE: Adding metrics methods from main
    #[instrument]
    pub async fn update_metrics(&self, port: u16, bytes_sent: u64, bytes_received: u64) -> Result<(), MCPError> {
        if !self.metrics_enabled {
            return Ok(());
        }

        let mut states = self.port_states.write().await;
        if let Some(state) = states.get_mut(&port) {
            if let Some(metrics) = &mut state.metrics {
                metrics.bytes_sent += bytes_sent;
                metrics.bytes_received += bytes_received;
                metrics.last_activity = chrono::Utc::now();
            }
        }

        Ok(())
    }

    #[instrument]
    pub async fn record_error(&self, port: u16) -> Result<(), MCPError> {
        if !self.metrics_enabled {
            return Ok(());
        }

        let mut states = self.port_states.write().await;
        if let Some(state) = states.get_mut(&port) {
            if let Some(metrics) = &mut state.metrics {
                metrics.error_count += 1;
                metrics.last_activity = chrono::Utc::now();
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn reserve_port(&self, port: u16) -> Result<()> {
        // Validate port range
        if port < self.config.min_port || port > self.config.max_port {
            return Err(anyhow::anyhow!("Port {} is out of valid range", port));
        }

        // Check access control
        match &self.config.access_control {
            PortAccessControl::AllowAll => {}
            PortAccessControl::AllowList(allowed) => {
                if !allowed.contains(&port) {
                    return Err(anyhow::anyhow!("Port {} is not in allowed list", port));
                }
            }
            PortAccessControl::DenyList(denied) => {
                if denied.contains(&port) {
                    return Err(anyhow::anyhow!("Port {} is in denied list", port));
                }
            }
        }

        // Reserve port
        let mut reserved = self.reserved_ports.write().await;
        if reserved.contains(&port) {
            return Err(anyhow::anyhow!("Port {} is already reserved", port));
        }
        reserved.insert(port);

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn unreserve_port(&self, port: u16) -> Result<()> {
        let mut reserved = self.reserved_ports.write().await;
        if !reserved.remove(&port) {
            return Err(anyhow::anyhow!("Port {} is not reserved", port));
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn is_port_reserved(&self, port: u16) -> Result<bool> {
        let reserved = self.reserved_ports.read().await;
        Ok(reserved.contains(&port))
    }
}

// MERGE NOTE: Keeping tests from main branch
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_port_reservation() {
        let config = PortConfig::default();
        let manager = PortManager::new(config).unwrap();

        // Test valid port reservation
        manager.reserve_port(8080).await.unwrap();
        assert!(manager.is_port_reserved(8080).await.unwrap());

        // Test duplicate reservation
        assert!(manager.reserve_port(8080).await.is_err());

        // Test port release
        manager.release_port(8080, "test_requester").await.unwrap();
        assert!(!manager.is_port_reserved(8080).await.unwrap());

        // Test releasing non-reserved port
        assert!(manager.release_port(8080, "test_requester").await.is_err());
    }
}

#[derive(Debug, Clone)]
pub struct Port {
    // ... existing code ...
}