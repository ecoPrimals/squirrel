//! MCP Port Manager
//! 
//! Manages port allocation and lifecycle for MCP tools.
//! Ensures secure and efficient port usage with proper validation and monitoring.

use std::collections::{HashMap, HashSet};
use std::net::{TcpListener, SocketAddr};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use std::time::{Duration, SystemTime};
use anyhow::{Result, Context as _};
use tracing::{info, warn, debug};
use std::ops::RangeInclusive;

/// Port allocation range for MCP tools
const MIN_PORT: u16 = 1024;
const MAX_PORT: u16 = 65535;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const PORT_RANGE: RangeInclusive<u16> = MIN_PORT..=MAX_PORT;

/// Port status tracking
#[derive(Debug, Clone)]
pub struct PortStatus {
    pub port: u16,
    pub allocated_at: SystemTime,
    pub last_activity: SystemTime,
    pub connection_count: u64,
    pub bytes_transferred: u64,
}

/// Port security configuration
#[derive(Debug, Clone)]
pub struct PortSecurity {
    pub allowed_ips: HashSet<String>,
    pub max_connections: u32,
}

/// Port Manager for MCP tools
pub struct PortManager {
    active_ports: Arc<RwLock<HashMap<u16, PortStatus>>>,
    security_config: Arc<RwLock<HashMap<u16, PortSecurity>>>,
    next_port: Arc<Mutex<u16>>,
    timeout: Duration,
}

impl Default for PortManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PortManager {
    /// Create a new PortManager instance
    pub fn new() -> Self {
        Self {
            active_ports: Arc::new(RwLock::new(HashMap::new())),
            security_config: Arc::new(RwLock::new(HashMap::new())),
            next_port: Arc::new(Mutex::new(MIN_PORT)),
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Allocate a new port for an MCP tool
    pub async fn allocate_port(&self) -> Result<u16> {
        let mut next_port = self.next_port.lock().await;
        let mut active_ports = self.active_ports.write().await;

        // Find next available port
        while active_ports.contains_key(&*next_port) || !self.is_port_available(*next_port).await? {
            *next_port = if *next_port == MAX_PORT {
                MIN_PORT
            } else {
                *next_port + 1
            };
        }

        let port = *next_port;
        
        // Try to bind to the port to ensure it's available
        let addr = format!("127.0.0.1:{}", port);
        TcpListener::bind(&addr)
            .with_context(|| format!("Failed to bind to port {}", port))?;

        // Record port status
        active_ports.insert(port, PortStatus {
            port,
            allocated_at: SystemTime::now(),
            last_activity: SystemTime::now(),
            connection_count: 0,
            bytes_transferred: 0,
        });

        info!(port = port, "Allocated new port");
        Ok(port)
    }

    /// Release an allocated port
    pub async fn release_port(&self, port: u16) -> Result<()> {
        let mut active_ports = self.active_ports.write().await;
        let mut security_config = self.security_config.write().await;

        if active_ports.remove(&port).is_none() {
            warn!(port = port, "Attempted to release unallocated port");
            return Ok(());
        }

        security_config.remove(&port);
        info!(port = port, "Released port");
        Ok(())
    }

    /// Configure security for a port
    pub async fn configure_security(&self, port: u16, config: PortSecurity) -> Result<()> {
        let active_ports = self.active_ports.read().await;
        if !active_ports.contains_key(&port) {
            return Err(anyhow::anyhow!("Port {} not allocated", port));
        }

        let mut security_config = self.security_config.write().await;
        security_config.insert(port, config);
        
        debug!(port = port, "Updated security configuration");
        Ok(())
    }

    /// Check if an IP is allowed to connect to a port
    pub async fn is_ip_allowed(&self, port: u16, ip: &str) -> Result<bool> {
        let security_config = self.security_config.read().await;
        
        match security_config.get(&port) {
            Some(config) => Ok(config.allowed_ips.contains(ip)),
            None => Ok(true), // No security config means all IPs allowed
        }
    }

    /// Update port activity metrics
    pub async fn update_metrics(&self, port: u16, bytes: u64) -> Result<()> {
        let mut active_ports = self.active_ports.write().await;
        
        if let Some(status) = active_ports.get_mut(&port) {
            status.last_activity = SystemTime::now();
            status.connection_count += 1;
            status.bytes_transferred += bytes;
        }

        Ok(())
    }

    /// Get status for a specific port
    pub async fn get_port_status(&self, port: u16) -> Result<Option<PortStatus>> {
        let active_ports = self.active_ports.read().await;
        Ok(active_ports.get(&port).cloned())
    }

    /// Get all active ports and their status
    pub async fn get_active_ports(&self) -> Result<Vec<PortStatus>> {
        let active_ports = self.active_ports.read().await;
        Ok(active_ports.values().cloned().collect())
    }

    /// Set the timeout duration for port operations
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Check if a port is available
    async fn is_port_available(&self, port: u16) -> Result<bool> {
        if !PORT_RANGE.contains(&port) {
            return Ok(false);
        }

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        match TcpListener::bind(addr) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Clean up inactive ports
    pub async fn cleanup_inactive_ports(&self, max_idle_time: Duration) -> Result<()> {
        let mut active_ports = self.active_ports.write().await;
        let mut security_config = self.security_config.write().await;
        let now = SystemTime::now();

        let inactive_ports: Vec<u16> = active_ports
            .iter()
            .filter(|(_, status)| {
                now.duration_since(status.last_activity)
                    .map(|idle| idle > max_idle_time)
                    .unwrap_or(false)
            })
            .map(|(&port, _)| port)
            .collect();

        for port in inactive_ports {
            active_ports.remove(&port);
            security_config.remove(&port);
            info!(port = port, "Cleaned up inactive port");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_port_allocation() -> Result<()> {
        let manager = PortManager::new();
        
        // Allocate a port
        let port = manager.allocate_port().await?;
        assert!(PORT_RANGE.contains(&port));
        
        // Verify port is active
        let status = manager.get_port_status(port).await?;
        assert!(status.is_some());
        
        // Release port
        manager.release_port(port).await?;
        
        // Verify port is released
        let status = manager.get_port_status(port).await?;
        assert!(status.is_none());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_port_security() -> Result<()> {
        let manager = PortManager::new();
        let port = manager.allocate_port().await?;
        
        // Configure security
        let mut allowed_ips = HashSet::new();
        allowed_ips.insert("127.0.0.1".to_string());
        
        let security = PortSecurity {
            allowed_ips,
            max_connections: 10,
        };
        
        manager.configure_security(port, security).await?;
        
        // Test IP validation
        assert!(manager.is_ip_allowed(port, "127.0.0.1").await?);
        assert!(!manager.is_ip_allowed(port, "192.168.1.1").await?);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_cleanup_inactive_ports() -> Result<()> {
        let manager = PortManager::new();
        let port = manager.allocate_port().await?;
        
        // Wait for port to become inactive
        sleep(Duration::from_millis(100)).await;
        
        // Clean up inactive ports
        manager.cleanup_inactive_ports(Duration::from_millis(50)).await?;
        
        // Verify port was cleaned up
        let status = manager.get_port_status(port).await?;
        assert!(status.is_none());
        
        Ok(())
    }
} 