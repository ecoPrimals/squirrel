---
title: Dashboard Backward Compatibility Adapter
version: 1.0.0
date: 2024-06-22
status: Proposed
---

# Dashboard Backward Compatibility Adapter

## Overview

This document specifies a backward compatibility adapter to help users transition from the original `squirrel-monitoring` dashboard functionality to the new `dashboard-core` and UI implementation crates. The adapter provides the same interfaces as the original dashboard module but redirects calls to the new architecture.

## Requirements

1. Provide drop-in replacements for key types from the original dashboard module
2. Forward calls to the equivalent functionality in the new architecture
3. Maintain the same API surface as much as possible
4. Include clear deprecation notices to encourage migration
5. Document migration path for each adapter component

## Implementation Details

### Location

The compatibility adapter should be placed in a dedicated crate:

```
crates/dashboard-compat/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    ├── manager.rs
    ├── component.rs
    ├── types.rs
    └── config.rs
```

### Adapter Components

#### 1. DashboardManager Adapter

```rust
/// Adapter for the original DashboardManager
///
/// This is a compatibility wrapper that redirects to the new dashboard-core crate.
/// It is recommended to migrate to the new architecture directly.
///
/// # Deprecation Notice
///
/// This adapter is provided for backward compatibility and will be removed in a future version.
/// Please migrate to the dashboard-core crate directly.
#[deprecated(
    since = "0.1.0",
    note = "Use dashboard_core::DashboardService directly instead"
)]
pub struct DashboardManager {
    /// Internal reference to the new dashboard service
    dashboard_service: Arc<dyn DashboardService>,
    
    /// Configuration used by the manager
    config: DashboardConfig,
}

impl DashboardManager {
    /// Creates a new dashboard manager
    ///
    /// This creates a backward compatibility wrapper around the new dashboard service.
    pub fn new(config: DashboardConfig) -> Self {
        let new_config = convert_config(config.clone());
        let dashboard_service = Arc::new(DefaultDashboardService::new(new_config));
        
        Self {
            dashboard_service,
            config,
        }
    }
    
    /// Start the dashboard manager
    ///
    /// This method starts the underlying dashboard service.
    pub async fn start(&self, addr: SocketAddr) -> Result<()> {
        // Forward to new implementation
        // Note: The address parameter is ignored as server functionality
        // is now handled differently in the new architecture
        Ok(())
    }
    
    /// Register a component with the dashboard
    pub async fn register_component(&self, component: Arc<dyn LegacyDashboardComponent>) -> Result<()> {
        let component_adapter = ComponentAdapter::new(component);
        // No direct equivalent in new architecture, but we can approximate
        Ok(())
    }
    
    // Other methods following the same pattern...
}

// Helper function to convert old config to new config
fn convert_config(old_config: OldDashboardConfig) -> dashboard_core::config::DashboardConfig {
    dashboard_core::config::DashboardConfig {
        refresh_interval: old_config.refresh_interval,
        data_retention_period: old_config.data_retention_period.unwrap_or(Duration::from_secs(3600)),
        enable_compression: old_config.enable_compression.unwrap_or(true),
        compression_level: old_config.compression_level.unwrap_or(6),
        max_connections: old_config.max_connections.unwrap_or(100),
        debug: old_config.debug.unwrap_or(false),
    }
}
```

#### 2. DashboardComponent Adapter

```rust
/// Type alias for backward compatibility
pub type DashboardComponent = LegacyDashboardComponent;

/// Original dashboard component trait
#[async_trait]
pub trait LegacyDashboardComponent: Send + Sync {
    /// Get the component ID
    fn id(&self) -> &str;
    
    /// Get the component name
    fn name(&self) -> &str;
    
    /// Get the component type
    fn component_type(&self) -> &str;
    
    /// Get component data
    async fn get_data(&self) -> Result<serde_json::Value>;
}

/// Adapter to bridge legacy components to new architecture
struct ComponentAdapter {
    component: Arc<dyn LegacyDashboardComponent>,
}

impl ComponentAdapter {
    fn new(component: Arc<dyn LegacyDashboardComponent>) -> Self {
        Self { component }
    }
    
    // Methods to adapt to new architecture...
}
```

#### 3. Update Types

```rust
/// Type alias for backward compatibility
pub type Update = LegacyUpdate;

/// Original update type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyUpdate {
    /// Component ID
    pub component_id: String,
    
    /// Update data
    pub data: serde_json::Value,
    
    /// Update timestamp
    pub timestamp: DateTime<Utc>,
}

// Conversion functions
fn convert_to_new_update(update: LegacyUpdate) -> dashboard_core::DashboardUpdate {
    // Convert legacy update to new format
    dashboard_core::DashboardUpdate {
        update_type: determine_update_type(&update.component_id),
        data: update.data,
        timestamp: update.timestamp,
    }
}

fn determine_update_type(component_id: &str) -> dashboard_core::UpdateType {
    // Map legacy component IDs to new update types
    match component_id {
        "system_metrics" => dashboard_core::UpdateType::Metrics,
        "alerts" => dashboard_core::UpdateType::Alerts,
        "health" => dashboard_core::UpdateType::Health,
        "network" => dashboard_core::UpdateType::Network,
        _ => dashboard_core::UpdateType::Custom(component_id.to_string()),
    }
}
```

## Usage Examples

### Original Code

```rust
use squirrel_monitoring::dashboard::{DashboardManager, DashboardComponent};

async fn setup_dashboard() {
    let config = Default::default();
    let manager = DashboardManager::new(config);
    
    manager.start("127.0.0.1:8080".parse().unwrap()).await.unwrap();
    
    let component = Arc::new(MyCustomComponent::new());
    manager.register_component(component).await.unwrap();
}
```

### Using Compatibility Adapter

```rust
use dashboard_compat::{DashboardManager, DashboardComponent};

async fn setup_dashboard() {
    let config = Default::default();
    let manager = DashboardManager::new(config);
    
    manager.start("127.0.0.1:8080".parse().unwrap()).await.unwrap();
    
    let component = Arc::new(MyCustomComponent::new());
    manager.register_component(component).await.unwrap();
}
```

### Migration to New Architecture

```rust
use dashboard_core::{
    config::DashboardConfig,
    service::{DashboardService, DefaultDashboardService},
};
use std::sync::Arc;

async fn setup_dashboard() {
    let config = DashboardConfig::default();
    let service = Arc::new(DefaultDashboardService::new(config));
    
    // Create UI implementation
    let mut terminal_ui = ui_terminal::TuiDashboard::new(service.clone());
    
    // Register metrics update function
    let service_clone = service.clone();
    tokio::spawn(async move {
        loop {
            // Collect metrics
            let metrics = collect_metrics().await;
            
            // Update dashboard
            service_clone.update_system_metrics(metrics).await.unwrap();
            
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
    
    // Run UI
    terminal_ui.run().await.unwrap();
}
```

## Deprecation Timeline

1. **Initial Release**: Include compatibility adapter with deprecation notices
2. **6 Months**: Issue warnings when adapter is used
3. **12 Months**: Remove compatibility adapter

## Migration Documentation

The compatibility adapter should include comprehensive documentation on how to migrate from the old API to the new one, including:

- Mapping of old types to new types
- Examples for common use cases
- Step-by-step migration guide

## Conclusion

The backward compatibility adapter provides a transitional path for users of the original dashboard functionality, allowing them to adopt the new architecture gradually. By providing clear deprecation notices and migration guidance, users will be encouraged to update their code to use the new API directly. 