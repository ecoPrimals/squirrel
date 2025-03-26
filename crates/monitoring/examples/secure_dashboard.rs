//! Secure Dashboard Example
//!
//! This example demonstrates how to configure and run a secure dashboard
//! with TLS encryption, authentication, and other security features.

use std::path::PathBuf;
use std::collections::HashMap;
use std::time::Duration;
use std::error::Error;
use serde_json::json;
use squirrel_monitoring::dashboard::config::{
    DashboardConfig, 
    ComponentSettings,
    SecuritySettings
};
use squirrel_monitoring::dashboard::manager::{
    DashboardManager,
    Component
};
use squirrel_monitoring::dashboard::security::{
    AuthConfig,
    RateLimitConfig,
    AuthType,
    MonitoringRole,
    Permission,
    MaskingRule,
    AuditStorage,
    AuditConfig
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Create a secure configuration
    let config = create_secure_config()?;
    
    // Create a dashboard manager with the secure configuration
    let dashboard = Arc::new(DashboardManager::new(config));
    
    // Register some example components
    register_example_components(dashboard.clone()).await?;
    
    // Start the dashboard
    dashboard.start().await?;
    
    println!("Secure dashboard started. Press Ctrl+C to stop.");
    
    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    
    // Stop the dashboard
    dashboard.stop().await?;
    
    println!("Dashboard stopped.");
    
    Ok(())
}

/// Create a secure dashboard configuration
fn create_secure_config() -> Result<DashboardConfig, Box<dyn Error>> {
    // Start with a default configuration
    let mut config = DashboardConfig::default();
    
    // Configure server settings
    if config.server.is_none() {
        config.server = Some(Default::default());
    }
    
    let server_config = config.server.as_mut().unwrap();
    server_config.host = "127.0.0.1".to_string();
    server_config.port = 8765;
    
    // Configure TLS (commented out for example only)
    /*
    if config.security.tls.is_none() {
        config.security.tls = Some(TlsConfig {
            cert_path: PathBuf::from("path/to/cert.pem"),
            key_path: PathBuf::from("path/to/key.pem"),
            min_tls_version: TlsVersion::Tls13,
            cipher_preferences: CipherPreferences::Modern,
        });
    }
    */
    
    // Configure authentication
    let mut users = HashMap::new();
    users.insert("admin".to_string(), MonitoringRole::Administrator);
    users.insert("operator".to_string(), MonitoringRole::Operator);
    users.insert("viewer".to_string(), MonitoringRole::Viewer);
    
    // Custom role with specific permissions
    let custom_permissions = vec![
        Permission::ViewDashboards,
        Permission::ViewMetrics,
        Permission::ViewAlerts,
        Permission::AcknowledgeAlerts,
    ];
    users.insert("custom".to_string(), MonitoringRole::Custom(custom_permissions));
    
    // Initialize security settings
    let mut security = SecuritySettings::default();
    
    // Auth config
    security.auth = Some(AuthConfig {
        auth_type: AuthType::Bearer,
        token_expiration: 8 * 60 * 60, // 8 hours
        require_reauth: true,
        users,
    });
    
    // Configure rate limiting
    security.rate_limit = Some(RateLimitConfig {
        max_connections_per_ip: 20,
        max_messages_per_minute: 300,
        max_subscription_requests_per_minute: 50,
    });
    
    // Configure allowed origins
    security.allowed_origins = vec![
        "http://localhost:3000".to_string(),
        "https://example.com".to_string(),
    ];
    
    // Configure data masking
    security.data_masking = vec![
        MaskingRule::new(r"[0-9]{4}-[0-9]{4}-[0-9]{4}-[0-9]{4}", "****-****-****-****"),
        MaskingRule::new(r#"password\s*=\s*['"].*?['"]"#, r#"password="*****""#),
    ];
    
    // Enable audit logging
    security.audit = Some(AuditConfig {
        enabled: true,
        storage: AuditStorage::File(PathBuf::from("logs/audit.log")),
        include_user_context: true,
        tamper_proof: true,
    });
    
    // Update the security settings
    config.security = security;
    
    Ok(config)
}

/// Register some example components
async fn register_example_components(dashboard: Arc<DashboardManager>) -> Result<(), Box<dyn Error>> {
    // CPU usage component
    let cpu_component = Component {
        id: "system_cpu".to_string(),
        name: "CPU Usage".to_string(),
        component_type: "gauge".to_string(),
        config: ComponentSettings {
            show_metrics: Some(true),
            show_alerts: Some(true),
            show_health: Some(true),
            show_network: Some(true),
            show_analytics: Some(true),
        },
        data: Some(json!({
            "usage": 42.5,
            "cores": 8,
            "processes": 120
        })),
        last_updated: Some(chrono::Utc::now().timestamp_millis() as u64),
    };
    
    // Memory usage component
    let memory_component = Component {
        id: "system_memory".to_string(),
        name: "Memory Usage".to_string(),
        component_type: "gauge".to_string(),
        config: ComponentSettings {
            show_metrics: Some(true),
            show_alerts: Some(true),
            show_health: Some(true),
            show_network: Some(true),
            show_analytics: Some(true),
        },
        data: Some(json!({
            "total": 16_384,
            "used": 8_192,
            "free": 8_192,
            "swap_used": 1_024
        })),
        last_updated: Some(chrono::Utc::now().timestamp_millis() as u64),
    };
    
    // Network traffic component
    let network_component = Component {
        id: "network_traffic".to_string(),
        name: "Network Traffic".to_string(),
        component_type: "line".to_string(),
        config: ComponentSettings {
            show_metrics: Some(true),
            show_alerts: Some(true),
            show_health: Some(true),
            show_network: Some(true),
            show_analytics: Some(true),
        },
        data: Some(json!({
            "rx_bytes": 1_500_000,
            "tx_bytes": 500_000,
            "connections": 42
        })),
        last_updated: Some(chrono::Utc::now().timestamp_millis() as u64),
    };
    
    // Register components
    dashboard.register_component(Arc::new(MockSystemCpuComponent{
        component: cpu_component
    })).await?;
    dashboard.register_component(Arc::new(MockSystemMemoryComponent{
        component: memory_component
    })).await?;
    dashboard.register_component(Arc::new(MockNetworkTrafficComponent{
        component: network_component
    })).await?;
    
    // Start a background task to update components
    let dashboard_clone = dashboard.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            // Update CPU component with random data
            let cpu_usage = rand::random::<f64>() * 100.0;
            let _ = dashboard_clone.update_component("system_cpu", json!({
                "usage": cpu_usage,
                "cores": 8,
                "processes": 120 + (rand::random::<f64>() * 10.0) as u64
            })).await;
            
            // Update memory component
            let memory_used = 4_096 + (rand::random::<f64>() * 8_192.0) as u64;
            let _ = dashboard_clone.update_component("system_memory", json!({
                "total": 16_384,
                "used": memory_used,
                "free": 16_384 - memory_used,
                "swap_used": 1_024 + (rand::random::<f64>() * 512.0) as u64
            })).await;
            
            // Update network component
            let rx_bytes = 1_000_000 + (rand::random::<f64>() * 1_000_000.0) as u64;
            let tx_bytes = 400_000 + (rand::random::<f64>() * 200_000.0) as u64;
            let _ = dashboard_clone.update_component("network_traffic", json!({
                "rx_bytes": rx_bytes,
                "tx_bytes": tx_bytes,
                "connections": 30 + (rand::random::<f64>() * 20.0) as u64
            })).await;
        }
    });
    
    Ok(())
}

// Mock implementations for DashboardComponent trait
use squirrel_core::error::Result as SquirrelResult;
use squirrel_monitoring::dashboard::DashboardComponent;
use squirrel_monitoring::dashboard::Update;
use async_trait::async_trait;

#[derive(Debug)]
struct MockSystemCpuComponent {
    component: Component
}

#[async_trait]
impl DashboardComponent for MockSystemCpuComponent {
    fn id(&self) -> &str {
        &self.component.id
    }
    
    async fn start(&self) -> SquirrelResult<()> {
        Ok(())
    }
    
    async fn get_data(&self) -> SquirrelResult<serde_json::Value> {
        Ok(self.component.data.clone().unwrap_or(json!({})))
    }
    
    async fn last_update(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.component.last_updated.map(|ts| {
            chrono::DateTime::from_timestamp_millis(ts as i64).unwrap_or_default()
        })
    }
    
    async fn get_update(&self) -> SquirrelResult<Update> {
        Ok(Update {
            component_id: self.component.id.clone(),
            data: self.component.data.clone().unwrap_or(json!({})),
            timestamp: chrono::Utc::now()
        })
    }
    
    async fn handle_event(&self, _event: serde_json::Value) -> SquirrelResult<()> {
        Ok(())
    }
    
    async fn stop(&self) -> SquirrelResult<()> {
        Ok(())
    }
}

#[derive(Debug)]
struct MockSystemMemoryComponent {
    component: Component
}

#[async_trait]
impl DashboardComponent for MockSystemMemoryComponent {
    fn id(&self) -> &str {
        &self.component.id
    }
    
    async fn start(&self) -> SquirrelResult<()> {
        Ok(())
    }
    
    async fn get_data(&self) -> SquirrelResult<serde_json::Value> {
        Ok(self.component.data.clone().unwrap_or(json!({})))
    }
    
    async fn last_update(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.component.last_updated.map(|ts| {
            chrono::DateTime::from_timestamp_millis(ts as i64).unwrap_or_default()
        })
    }
    
    async fn get_update(&self) -> SquirrelResult<Update> {
        Ok(Update {
            component_id: self.component.id.clone(),
            data: self.component.data.clone().unwrap_or(json!({})),
            timestamp: chrono::Utc::now()
        })
    }
    
    async fn handle_event(&self, _event: serde_json::Value) -> SquirrelResult<()> {
        Ok(())
    }
    
    async fn stop(&self) -> SquirrelResult<()> {
        Ok(())
    }
}

#[derive(Debug)]
struct MockNetworkTrafficComponent {
    component: Component
}

#[async_trait]
impl DashboardComponent for MockNetworkTrafficComponent {
    fn id(&self) -> &str {
        &self.component.id
    }
    
    async fn start(&self) -> SquirrelResult<()> {
        Ok(())
    }
    
    async fn get_data(&self) -> SquirrelResult<serde_json::Value> {
        Ok(self.component.data.clone().unwrap_or(json!({})))
    }
    
    async fn last_update(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.component.last_updated.map(|ts| {
            chrono::DateTime::from_timestamp_millis(ts as i64).unwrap_or_default()
        })
    }
    
    async fn get_update(&self) -> SquirrelResult<Update> {
        Ok(Update {
            component_id: self.component.id.clone(),
            data: self.component.data.clone().unwrap_or(json!({})),
            timestamp: chrono::Utc::now()
        })
    }
    
    async fn handle_event(&self, _event: serde_json::Value) -> SquirrelResult<()> {
        Ok(())
    }
    
    async fn stop(&self) -> SquirrelResult<()> {
        Ok(())
    }
} 