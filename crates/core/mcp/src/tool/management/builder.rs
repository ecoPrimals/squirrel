// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tool Builder Implementation
//!
//! This module provides the builder pattern for creating Tool instances
//! with proper validation and configuration.

use std::sync::Arc;
use super::types::*;
use crate::tool::cleanup::RecoveryHook;
use crate::tool::cleanup::ResourceManager;

/// Error that can occur during tool building
#[derive(Debug, thiserror::Error)]
pub enum ToolBuildError {
    #[error("Tool ID is required")]
    MissingId,
    #[error("Tool name is required")]
    MissingName,
    #[error("Tool version is required")]
    MissingVersion,
    #[error("Tool description is required")]
    MissingDescription,
    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl From<ToolBuildError> for ToolError {
    fn from(error: ToolBuildError) -> Self {
        match error {
            ToolBuildError::MissingId => ToolError::ValidationError("Tool missing ID".to_string()),
            ToolBuildError::MissingName => ToolError::ValidationError("Tool missing name".to_string()),
            ToolBuildError::MissingVersion => ToolError::ValidationError("Tool missing version".to_string()),
            ToolBuildError::MissingDescription => ToolError::ValidationError("Tool missing description".to_string()),
            ToolBuildError::ValidationError(msg) => ToolError::ValidationError(msg),
        }
    }
}

/// Builder for creating `Tool` instances
#[derive(Default)]
pub struct ToolBuilder {
    id: Option<String>,
    name: Option<String>,
    version: String,
    description: String,
    capabilities: Vec<Capability>,
    security_level: u8,
    // Lifecycle hook for the tool
    lifecycle_hook: Option<Arc<dyn ToolLifecycleHook + Send + Sync>>,
    // Resource manager for the tool
    resource_manager: Option<Arc<dyn ResourceManager + Send + Sync>>,
    // Recovery hook for the tool
    recovery_hook: Option<Arc<RecoveryHook>>,
}

impl ToolBuilder {
    /// Creates a new `ToolBuilder` with default values
    #[must_use] 
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            version: "0.1.0".to_string(),
            description: String::new(),
            capabilities: Vec::new(),
            security_level: 0,
            lifecycle_hook: None,
            resource_manager: None,
            recovery_hook: None,
        }
    }

    /// Sets the tool ID
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the tool name
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the tool version
    #[must_use]
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Sets the tool description
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Adds a capability to the tool
    #[must_use] 
    pub fn capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Sets the security level for the tool
    #[must_use] 
    pub const fn security_level(mut self, level: u8) -> Self {
        self.security_level = level;
        self
    }

    /// Sets the lifecycle hook for the tool
    #[must_use]
    pub fn lifecycle_hook(mut self, hook: Arc<dyn ToolLifecycleHook + Send + Sync>) -> Self {
        self.lifecycle_hook = Some(hook);
        self
    }

    /// Sets the resource manager for the tool
    #[must_use]
    pub fn resource_manager(mut self, manager: Arc<dyn ResourceManager + Send + Sync>) -> Self {
        self.resource_manager = Some(manager);
        self
    }

    /// Sets the recovery hook for the tool
    #[must_use]
    pub fn recovery_hook(mut self, hook: Arc<RecoveryHook>) -> Self {
        self.recovery_hook = Some(hook);
        self
    }

    /// Builds the `Tool` instance
    #[must_use] 
    pub fn build(self) -> Result<Tool, ToolBuildError> {
        let id = self.id.ok_or(ToolBuildError::MissingId)?;
        let name = self.name.ok_or(ToolBuildError::MissingName)?;

        // Validate required fields
        if self.description.is_empty() {
            return Err(ToolBuildError::ValidationError("Description cannot be empty".to_string()));
        }

        Ok(Tool {
            id,
            name,
            version: self.version,
            description: self.description,
            capabilities: self.capabilities,
            security_level: self.security_level,
        })
    }
}

impl Tool {
    /// Creates a new builder for Tool
    #[must_use] 
    pub fn builder() -> ToolBuilder {
        ToolBuilder::new()
    }
} 