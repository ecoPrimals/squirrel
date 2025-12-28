//! Workflow Template Engine
//!
//! Manages workflow templates for reusable patterns.
//! Supports template creation, instantiation, versioning, and parameter substitution.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use serde::{Serialize, Deserialize};

use crate::error::{Result, types::MCPError};
use super::types::*;

/// Workflow template engine
///
/// Manages workflow templates for reusable patterns.
/// Supports template creation, instantiation, versioning, and parameter substitution.
#[derive(Debug)]
pub struct WorkflowTemplateEngine {
    /// Template registry
    templates: Arc<RwLock<HashMap<String, WorkflowTemplate>>>,
    
    /// Template configuration
    config: TemplateEngineConfig,
}

/// Template engine configuration
#[derive(Debug, Clone)]
pub struct TemplateEngineConfig {
    /// Enable template versioning
    pub enable_versioning: bool,
    
    /// Maximum templates to store
    pub max_templates: usize,
    
    /// Allow template overwrite
    pub allow_overwrite: bool,
}

impl Default for TemplateEngineConfig {
    fn default() -> Self {
        Self {
            enable_versioning: true,
            max_templates: 1000,
            allow_overwrite: false,
        }
    }
}

/// Workflow template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    /// Template ID
    pub id: String,
    
    /// Template name
    pub name: String,
    
    /// Template description
    pub description: String,
    
    /// Template version
    pub version: String,
    
    /// Template parameters
    pub parameters: Vec<TemplateParameter>,
    
    /// Workflow definition (with placeholders)
    pub workflow: WorkflowDefinition,
    
    /// Template metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Template parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    /// Parameter name
    pub name: String,
    
    /// Parameter description
    pub description: String,
    
    /// Parameter type
    pub param_type: String,
    
    /// Default value
    pub default_value: Option<serde_json::Value>,
    
    /// Required parameter
    pub required: bool,
    
    /// Validation rules
    pub validation: Option<serde_json::Value>,
}

impl WorkflowTemplateEngine {
    /// Create a new template engine
    pub fn new(config: TemplateEngineConfig) -> Self {
        Self {
            templates: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Register a template
    pub async fn register_template(&self, template: WorkflowTemplate) -> Result<()> {
        let mut templates = self.templates.write().await;
        
        // Check if template exists
        if templates.contains_key(&template.id) && !self.config.allow_overwrite {
            return Err(MCPError::InvalidArgument(format!(
                "Template already exists: {}",
                template.id
            ))
            .into());
        }
        
        // Check max templates
        if templates.len() >= self.config.max_templates {
            return Err(
                MCPError::InvalidArgument("Maximum template limit reached".to_string()).into(),
            );
        }
        
        info!("Registered template: {}", template.id);
        templates.insert(template.id.clone(), template);
        
        Ok(())
    }
    
    /// Instantiate a workflow from template
    pub async fn instantiate_template(
        &self,
        template_id: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<WorkflowDefinition> {
        let templates = self.templates.read().await;
        let template = templates.get(template_id).ok_or_else(|| {
            MCPError::InvalidArgument(format!("Template not found: {}", template_id))
        })?;
        
        // Validate parameters
        self.validate_parameters(template, &parameters)?;
        
        // Clone workflow and substitute parameters
        let mut workflow = template.workflow.clone();
        workflow.id = uuid::Uuid::new_v4().to_string();
        
        // Add template metadata
        workflow
            .metadata
            .insert("template_id".to_string(), serde_json::json!(template_id));
        workflow
            .metadata
            .insert("template_version".to_string(), serde_json::json!(template.version));
        workflow
            .metadata
            .insert("parameters".to_string(), serde_json::json!(parameters));
        workflow
            .metadata
            .insert(
                "instantiated_at".to_string(),
                serde_json::json!(chrono::Utc::now().to_rfc3339()),
            );
        
        // TODO: Implement parameter substitution in workflow steps
        // This would involve replacing placeholders like {{param_name}} in step definitions
        
        info!("Instantiated workflow from template: {}", template_id);
        
        Ok(workflow)
    }
    
    /// Validate template parameters
    fn validate_parameters(
        &self,
        template: &WorkflowTemplate,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        for param in &template.parameters {
            if param.required && !parameters.contains_key(&param.name) {
                return Err(MCPError::InvalidArgument(format!(
                    "Required parameter missing: {}",
                    param.name
                ))
                .into());
            }
            
            // TODO: Implement validation rules
            // Check parameter types, ranges, patterns, etc.
        }
        Ok(())
    }
    
    /// Get template by ID
    pub async fn get_template(&self, template_id: &str) -> Result<Option<WorkflowTemplate>> {
        let templates = self.templates.read().await;
        Ok(templates.get(template_id).cloned())
    }
    
    /// List all templates
    pub async fn list_templates(
        &self,
        tags: Option<Vec<String>>,
    ) -> Result<Vec<WorkflowTemplate>> {
        let templates = self.templates.read().await;
        
        let mut result: Vec<WorkflowTemplate> = templates.values().cloned().collect();
        
        // Filter by tags if provided
        if let Some(filter_tags) = tags {
            result.retain(|t| filter_tags.iter().any(|tag| t.tags.contains(tag)));
        }
        
        // Sort by name for consistent ordering
        result.sort_by(|a, b| a.name.cmp(&b.name));
        
        Ok(result)
    }
    
    /// Delete template
    pub async fn delete_template(&self, template_id: &str) -> Result<()> {
        let mut templates = self.templates.write().await;
        templates.remove(template_id).ok_or_else(|| {
            MCPError::InvalidArgument(format!("Template not found: {}", template_id))
        })?;
        
        info!("Deleted template: {}", template_id);
        Ok(())
    }
    
    /// Update template
    pub async fn update_template(&self, template: WorkflowTemplate) -> Result<()> {
        let mut templates = self.templates.write().await;
        
        if !templates.contains_key(&template.id) {
            return Err(MCPError::InvalidArgument(format!(
                "Template not found: {}",
                template.id
            ))
            .into());
        }
        
        info!("Updated template: {}", template.id);
        templates.insert(template.id.clone(), template);
        
        Ok(())
    }
    
    /// Get template count
    pub async fn template_count(&self) -> usize {
        let templates = self.templates.read().await;
        templates.len()
    }
    
    /// Search templates by name or description
    pub async fn search_templates(&self, query: &str) -> Result<Vec<WorkflowTemplate>> {
        let templates = self.templates.read().await;
        let query_lower = query.to_lowercase();
        
        let result: Vec<WorkflowTemplate> = templates
            .values()
            .filter(|t| {
                t.name.to_lowercase().contains(&query_lower)
                    || t.description.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect();
        
        Ok(result)
    }
    
    /// Get templates by tag
    pub async fn get_templates_by_tag(&self, tag: &str) -> Result<Vec<WorkflowTemplate>> {
        let templates = self.templates.read().await;
        
        let result: Vec<WorkflowTemplate> = templates
            .values()
            .filter(|t| t.tags.contains(&tag.to_string()))
            .cloned()
            .collect();
        
        Ok(result)
    }
    
    /// Get configuration
    pub fn config(&self) -> &TemplateEngineConfig {
        &self.config
    }
}

