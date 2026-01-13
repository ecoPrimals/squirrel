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

/// Substitute parameters in a workflow step
///
/// Replaces placeholders like {{param_name}} with actual parameter values
/// Supports nested JSON structures
fn substitute_parameters_in_step(
    step: &mut WorkflowStep,
    parameters: &HashMap<String, serde_json::Value>,
) -> Result<()> {
    // Substitute in step name
    step.name = substitute_string(&step.name, parameters);
    
    // Substitute in step description
    step.description = substitute_string(&step.description, parameters);
    
    // Substitute in step config
    step.config = substitute_json_value(&step.config, parameters)?;
    
    Ok(())
}

/// Substitute parameters in a string
///
/// Replaces {{param_name}} with the parameter value
/// Example: "Hello {{name}}" with {"name": "World"} -> "Hello World"
fn substitute_string(s: &str, parameters: &HashMap<String, serde_json::Value>) -> String {
    let mut result = s.to_string();
    
    // Find all {{param_name}} patterns
    for (key, value) in parameters {
        let placeholder = format!("{{{{{}}}}}", key);
        let replacement = match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "null".to_string(),
            _ => value.to_string(), // For arrays/objects, use JSON representation
        };
        
        result = result.replace(&placeholder, &replacement);
    }
    
    result
}

/// Substitute parameters in a JSON value
///
/// Recursively processes JSON structures, replacing {{param_name}} in strings
fn substitute_json_value(
    value: &serde_json::Value,
    parameters: &HashMap<String, serde_json::Value>,
) -> Result<serde_json::Value> {
    match value {
        serde_json::Value::String(s) => {
            Ok(serde_json::Value::String(substitute_string(s, parameters)))
        }
        serde_json::Value::Array(arr) => {
            let mut result = Vec::new();
            for item in arr {
                result.push(substitute_json_value(item, parameters)?);
            }
            Ok(serde_json::Value::Array(result))
        }
        serde_json::Value::Object(obj) => {
            let mut result = serde_json::Map::new();
            for (key, val) in obj {
                result.insert(key.clone(), substitute_json_value(val, parameters)?);
            }
            Ok(serde_json::Value::Object(result))
        }
        // Numbers, booleans, and null are returned as-is
        other => Ok(other.clone()),
    }
}

/// Validate a parameter value against its definition
///
/// Checks type compatibility, ranges, patterns, and custom validation rules
fn validate_parameter_value(param: &TemplateParameter, value: &serde_json::Value) -> Result<()> {
    // Type validation
    match param.param_type.as_str() {
        "string" => {
            if !value.is_string() {
                return Err(MCPError::InvalidArgument(format!(
                    "Parameter '{}' must be a string, got {:?}",
                    param.name,
                    value
                )));
            }
            
            // Validate string length if specified
            if let Some(validation) = &param.validation {
                if let Some(min_length) = validation.get("min_length").and_then(|v| v.as_u64()) {
                    if value.as_str().unwrap().len() < min_length as usize {
                        return Err(MCPError::InvalidArgument(format!(
                            "Parameter '{}' must be at least {} characters",
                            param.name, min_length
                        )));
                    }
                }
                
                if let Some(max_length) = validation.get("max_length").and_then(|v| v.as_u64()) {
                    if value.as_str().unwrap().len() > max_length as usize {
                        return Err(MCPError::InvalidArgument(format!(
                            "Parameter '{}' must be at most {} characters",
                            param.name, max_length
                        )));
                    }
                }
                
                // Validate pattern (regex) if specified
                if let Some(pattern) = validation.get("pattern").and_then(|v| v.as_str()) {
                    let regex = regex::Regex::new(pattern).map_err(|e| {
                        MCPError::InvalidArgument(format!(
                            "Invalid regex pattern for parameter '{}': {}",
                            param.name, e
                        ))
                    })?;
                    
                    if !regex.is_match(value.as_str().unwrap()) {
                        return Err(MCPError::InvalidArgument(format!(
                            "Parameter '{}' does not match required pattern: {}",
                            param.name, pattern
                        )));
                    }
                }
            }
        }
        "number" => {
            if !value.is_number() {
                return Err(MCPError::InvalidArgument(format!(
                    "Parameter '{}' must be a number, got {:?}",
                    param.name,
                    value
                )));
            }
            
            // Validate number range if specified
            if let Some(validation) = &param.validation {
                let num_value = value.as_f64().unwrap();
                
                if let Some(min) = validation.get("min").and_then(|v| v.as_f64()) {
                    if num_value < min {
                        return Err(MCPError::InvalidArgument(format!(
                            "Parameter '{}' must be at least {}",
                            param.name, min
                        )));
                    }
                }
                
                if let Some(max) = validation.get("max").and_then(|v| v.as_f64()) {
                    if num_value > max {
                        return Err(MCPError::InvalidArgument(format!(
                            "Parameter '{}' must be at most {}",
                            param.name, max
                        )));
                    }
                }
            }
        }
        "boolean" => {
            if !value.is_boolean() {
                return Err(MCPError::InvalidArgument(format!(
                    "Parameter '{}' must be a boolean, got {:?}",
                    param.name,
                    value
                )));
            }
        }
        "array" => {
            if !value.is_array() {
                return Err(MCPError::InvalidArgument(format!(
                    "Parameter '{}' must be an array, got {:?}",
                    param.name,
                    value
                )));
            }
            
            // Validate array length if specified
            if let Some(validation) = &param.validation {
                let arr = value.as_array().unwrap();
                
                if let Some(min_items) = validation.get("min_items").and_then(|v| v.as_u64()) {
                    if arr.len() < min_items as usize {
                        return Err(MCPError::InvalidArgument(format!(
                            "Parameter '{}' must have at least {} items",
                            param.name, min_items
                        )));
                    }
                }
                
                if let Some(max_items) = validation.get("max_items").and_then(|v| v.as_u64()) {
                    if arr.len() > max_items as usize {
                        return Err(MCPError::InvalidArgument(format!(
                            "Parameter '{}' must have at most {} items",
                            param.name, max_items
                        )));
                    }
                }
            }
        }
        "object" => {
            if !value.is_object() {
                return Err(MCPError::InvalidArgument(format!(
                    "Parameter '{}' must be an object, got {:?}",
                    param.name,
                    value
                )));
            }
        }
        unknown_type => {
            return Err(MCPError::InvalidArgument(format!(
                "Unknown parameter type '{}' for parameter '{}'",
                unknown_type, param.name
            )));
        }
    }
    
    Ok(())
}

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
        
        // Perform parameter substitution in workflow steps
        // Replaces placeholders like {{param_name}} with actual parameter values
        for step in &mut workflow.steps {
            substitute_parameters_in_step(step, parameters)?;
        }
        
        info!("Instantiated workflow from template: {} with {} parameters", template_id, parameters.len());
        
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
            
            // Validate parameter based on its definition
            if let Some(value) = parameters.get(&param.name) {
                validate_parameter_value(&param, value)?;
            }
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

