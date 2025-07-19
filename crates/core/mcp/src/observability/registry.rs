//! Component registry for observability framework
//!
//! This module provides component registration and tracking functionality
//! for the observability system.

use std::collections::HashMap;
use std::time::SystemTime;

use crate::observability::health::HealthStatus;

/// Registry for tracking monitored components
pub struct ComponentRegistry {
    components: HashMap<String, ComponentInfo>,
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Register a new component
    pub fn register_component(&mut self, component: ComponentInfo) {
        self.components.insert(component.id.clone(), component);
    }

    /// Update component health status
    pub fn update_component_health(&mut self, component_id: &str, status: HealthStatus) {
        if let Some(component) = self.components.get_mut(component_id) {
            component.health_status = status;
            component.last_health_check = Some(SystemTime::now());
        }
    }

    /// Get total number of registered components
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Get all registered components
    pub fn get_components(&self) -> Vec<&ComponentInfo> {
        self.components.values().collect()
    }

    /// Get component by ID
    pub fn get_component(&self, id: &str) -> Option<&ComponentInfo> {
        self.components.get(id)
    }

    /// Get component by ID (mutable)
    pub fn get_component_mut(&mut self, id: &str) -> Option<&mut ComponentInfo> {
        self.components.get_mut(id)
    }

    /// Remove a component
    pub fn remove_component(&mut self, id: &str) -> Option<ComponentInfo> {
        self.components.remove(id)
    }

    /// Get components by type
    pub fn get_components_by_type(&self, component_type: ComponentType) -> Vec<&ComponentInfo> {
        self.components
            .values()
            .filter(|c| c.component_type == component_type)
            .collect()
    }

    /// Get unhealthy components
    pub fn get_unhealthy_components(&self) -> Vec<&ComponentInfo> {
        self.components
            .values()
            .filter(|c| matches!(c.health_status, HealthStatus::Unhealthy | HealthStatus::Unknown))
            .collect()
    }
}

/// Information about a registered component
pub struct ComponentInfo {
    pub id: String,
    pub name: String,
    pub component_type: ComponentType,
    pub health_status: HealthStatus,
    pub last_health_check: Option<SystemTime>,
    pub metadata: HashMap<String, String>,
}

impl ComponentInfo {
    /// Create a new component info
    pub fn new(id: String, name: String, component_type: ComponentType) -> Self {
        Self {
            id,
            name,
            component_type,
            health_status: HealthStatus::Unknown,
            last_health_check: None,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Update health status
    pub fn update_health(&mut self, status: HealthStatus) {
        self.health_status = status;
        self.last_health_check = Some(SystemTime::now());
    }
}

/// Types of components that can be registered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentType {
    McpCore,
    McpProtocol,
    McpTransport,
    McpSession,
    McpSecurity,
    McpPersistence,
    Plugin,
    External,
}

impl ComponentType {
    /// Get display name for component type
    pub fn display_name(&self) -> &'static str {
        match self {
            ComponentType::McpCore => "MCP Core",
            ComponentType::McpProtocol => "MCP Protocol",
            ComponentType::McpTransport => "MCP Transport",
            ComponentType::McpSession => "MCP Session",
            ComponentType::McpSecurity => "MCP Security",
            ComponentType::McpPersistence => "MCP Persistence",
            ComponentType::Plugin => "Plugin",
            ComponentType::External => "External",
        }
    }
} 