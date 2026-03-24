// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Message Templates for Fast Serialization
//!
//! This module provides a template-based serialization system that pre-compiles
//! common message structures for maximum performance in hot paths.

use std::collections::HashMap;
use std::sync::Arc;
use bytes::{Bytes, BytesMut, BufMut};
use serde::{Serialize, Deserialize};
use serde_json;
use tracing::{debug, warn};

use crate::error::{Result, types::MCPError};
use crate::protocol::types::{MCPMessage, MessageType};
use crate::enhanced::coordinator::{UniversalAIRequest, UniversalAIResponse, Message};

/// Template cache for storing compiled message templates
#[derive(Debug)]
pub struct MessageTemplateCache {
    /// Template storage
    templates: HashMap<String, CompiledTemplate>,
    
    /// Cache configuration
    config: TemplateCacheConfig,
    
    /// Cache statistics
    stats: TemplateCacheStats,
}

/// Configuration for template cache
#[derive(Debug, Clone)]
pub struct TemplateCacheConfig {
    /// Maximum number of templates to cache
    pub max_templates: usize,
    
    /// Enable template compilation optimization
    pub enable_compilation: bool,
    
    /// Template validation level
    pub validation_level: ValidationLevel,
}

/// Template validation levels
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationLevel {
    /// No validation (fastest)
    None,
    /// Basic structure validation
    Basic,
    /// Full validation (slowest)
    Full,
}

/// Cache statistics
#[derive(Debug, Default, Clone)]
pub struct TemplateCacheStats {
    /// Templates compiled
    pub templates_compiled: u64,
    
    /// Template hits
    pub cache_hits: u64,
    
    /// Template misses
    pub cache_misses: u64,
    
    /// Compilation time saved (nanoseconds)
    pub time_saved_ns: u64,
    
    /// Total template usage
    pub total_usage: u64,
}

/// A compiled message template for fast serialization
#[derive(Debug, Clone)]
pub struct CompiledTemplate {
    /// Template name/identifier
    pub name: String,
    
    /// Template structure
    pub structure: TemplateStructure,
    
    /// Pre-compiled parts
    pub compiled_parts: Vec<CompiledPart>,
    
    /// Field mappings
    pub field_map: HashMap<String, FieldMapping>,
    
    /// Template metadata
    pub metadata: TemplateMetadata,
}

/// Template structure definition
#[derive(Debug, Clone)]
pub struct TemplateStructure {
    /// Static prefix
    pub prefix: String,
    
    /// Dynamic fields
    pub fields: Vec<TemplateField>,
    
    /// Static suffix
    pub suffix: String,
    
    /// Estimated size
    pub estimated_size: usize,
}

/// A field in a template
#[derive(Debug, Clone)]
pub struct TemplateField {
    /// Field name
    pub name: String,
    
    /// Field type
    pub field_type: FieldType,
    
    /// Position in template
    pub position: usize,
    
    /// Is field optional?
    pub optional: bool,
    
    /// Default value
    pub default_value: Option<String>,
}

/// Field types in templates
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    /// String field
    String,
    /// Number field
    Number,
    /// Boolean field
    Boolean,
    /// Array field
    Array,
    /// Object field
    Object,
    /// Custom serialized field
    Custom(String),
}

/// Compiled part of a template
#[derive(Debug, Clone)]
pub enum CompiledPart {
    /// Static text
    Static(String),
    /// Dynamic field reference
    Field(String),
    /// Conditional section
    Conditional {
        condition: String,
        true_part: Box<CompiledPart>,
        false_part: Option<Box<CompiledPart>>,
    },
    /// Repeated section
    Loop {
        iterator: String,
        template: Box<CompiledPart>,
    },
}

/// Field mapping for template compilation
#[derive(Debug, Clone)]
pub struct FieldMapping {
    /// Source field path
    pub source_path: String,
    
    /// Target position in template
    pub target_position: usize,
    
    /// Transformation function
    pub transform: Option<TransformFunction>,
}

/// Template metadata
#[derive(Debug, Clone)]
pub struct TemplateMetadata {
    /// Template creation time
    pub created_at: std::time::Instant,
    
    /// Last used time
    pub last_used: std::time::Instant,
    
    /// Usage count
    pub usage_count: u64,
    
    /// Performance metrics
    pub performance: TemplatePerformance,
}

/// Template performance metrics
#[derive(Debug, Clone, Default)]
pub struct TemplatePerformance {
    /// Average serialization time (nanoseconds)
    pub avg_serialize_time_ns: u64,
    
    /// Min serialization time (nanoseconds)
    pub min_serialize_time_ns: u64,
    
    /// Max serialization time (nanoseconds)
    pub max_serialize_time_ns: u64,
    
    /// Total time saved vs standard serialization
    pub time_saved_ns: u64,
}

/// Transform function for field values
#[derive(Debug, Clone)]
pub enum TransformFunction {
    /// No transformation
    Identity,
    /// Escape JSON string
    JsonEscape,
    /// Format timestamp
    FormatTimestamp(String),
    /// Custom transformation
    Custom(String),
}

impl MessageTemplateCache {
    /// Create a new template cache
    pub fn new(max_size: usize) -> Self {
        Self {
            templates: HashMap::new(),
            config: TemplateCacheConfig {
                max_templates: max_size,
                enable_compilation: true,
                validation_level: ValidationLevel::Basic,
            },
            stats: TemplateCacheStats::default(),
        }
    }
    
    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&CompiledTemplate> {
        self.templates.get(name)
    }
    
    /// Compile and store a template
    pub fn compile_template(&mut self, name: String, definition: TemplateDefinition) -> Result<()> {
        debug!("Compiling template: {}", name);
        
        let template = self.compile_template_internal(name.clone(), definition)?;
        
        // Check cache size limit
        if self.templates.len() >= self.config.max_templates {
            self.evict_oldest_template();
        }
        
        self.templates.insert(name, template);
        self.stats.templates_compiled += 1;
        
        Ok(())
    }
    
    /// Compile template from definition
    fn compile_template_internal(&self, name: String, definition: TemplateDefinition) -> Result<CompiledTemplate> {
        let start_time = std::time::Instant::now();
        
        // Parse template structure
        let structure = self.parse_template_structure(&definition)?;
        
        // Compile template parts
        let compiled_parts = self.compile_template_parts(&structure)?;
        
        // Build field mappings
        let field_map = self.build_field_mappings(&structure)?;
        
        let metadata = TemplateMetadata {
            created_at: start_time,
            last_used: start_time,
            usage_count: 0,
            performance: TemplatePerformance::default(),
        };
        
        Ok(CompiledTemplate {
            name,
            structure,
            compiled_parts,
            field_map,
            metadata,
        })
    }
    
    /// Parse template structure from definition
    fn parse_template_structure(&self, definition: &TemplateDefinition) -> Result<TemplateStructure> {
        let mut fields = Vec::new();
        let mut estimated_size = definition.template.len();
        
        // Parse template string for field placeholders
        let template_str = &definition.template;
        let mut current_pos = 0;
        
        while let Some(start) = template_str[current_pos..].find("{{") {
            let abs_start = current_pos + start;
            if let Some(end) = template_str[abs_start..].find("}}") {
                let abs_end = abs_start + end;
                let field_expr = &template_str[abs_start + 2..abs_end];
                
                // Parse field expression
                let field = self.parse_field_expression(field_expr, abs_start)?;
                estimated_size += field.name.len(); // Rough estimate
                fields.push(field);
                
                current_pos = abs_end + 2;
            } else {
                return Err(MCPError::Internal("Unclosed template field".to_string()));
            }
        }
        
        Ok(TemplateStructure {
            prefix: definition.template.clone(), // Simplified - would extract actual prefix
            fields,
            suffix: String::new(), // Simplified - would extract actual suffix
            estimated_size,
        })
    }
    
    /// Parse field expression like "field_name:string:optional"
    fn parse_field_expression(&self, expr: &str, position: usize) -> Result<TemplateField> {
        let parts: Vec<&str> = expr.split(':').collect();
        if parts.is_empty() {
            return Err(MCPError::Internal("Empty field expression".to_string()));
        }
        
        let name = parts[0].trim().to_string();
        let field_type = if parts.len() > 1 {
            match parts[1].trim() {
                "string" => FieldType::String,
                "number" => FieldType::Number,
                "boolean" => FieldType::Boolean,
                "array" => FieldType::Array,
                "object" => FieldType::Object,
                custom => FieldType::Custom(custom.to_string()),
            }
        } else {
            FieldType::String
        };
        
        let optional = parts.len() > 2 && parts[2].trim() == "optional";
        
        Ok(TemplateField {
            name,
            field_type,
            position,
            optional,
            default_value: None,
        })
    }
    
    /// Compile template parts for efficient rendering
    fn compile_template_parts(&self, structure: &TemplateStructure) -> Result<Vec<CompiledPart>> {
        let mut parts = Vec::new();
        
        // Add prefix if not empty
        if !structure.prefix.is_empty() {
            parts.push(CompiledPart::Static(structure.prefix.clone()));
        }
        
        // Add field references
        for field in &structure.fields {
            parts.push(CompiledPart::Field(field.name.clone()));
        }
        
        // Add suffix if not empty  
        if !structure.suffix.is_empty() {
            parts.push(CompiledPart::Static(structure.suffix.clone()));
        }
        
        Ok(parts)
    }
    
    /// Build field mappings
    fn build_field_mappings(&self, structure: &TemplateStructure) -> Result<HashMap<String, FieldMapping>> {
        let mut mappings = HashMap::new();
        
        for (index, field) in structure.fields.iter().enumerate() {
            let mapping = FieldMapping {
                source_path: field.name.clone(),
                target_position: index,
                transform: match field.field_type {
                    FieldType::String => Some(TransformFunction::JsonEscape),
                    _ => Some(TransformFunction::Identity),
                },
            };
            mappings.insert(field.name.clone(), mapping);
        }
        
        Ok(mappings)
    }
    
    /// Evict oldest template from cache
    fn evict_oldest_template(&mut self) {
        if let Some((oldest_name, _)) = self.templates
            .iter()
            .min_by_key(|(_, template)| template.metadata.last_used)
            .map(|(name, template)| (name.clone(), template.clone()))
        {
            self.templates.remove(&oldest_name);
            debug!("Evicted oldest template: {}", oldest_name);
        }
    }
    
    /// Get cache statistics
    pub fn get_stats(&self) -> TemplateCacheStats {
        self.stats.clone()
    }
    
    /// Clear all templates
    pub fn clear(&mut self) {
        self.templates.clear();
        self.stats = TemplateCacheStats::default();
    }
}

/// Template definition for compilation
#[derive(Debug, Clone)]
pub struct TemplateDefinition {
    /// Template string with placeholders
    pub template: String,
    
    /// Field definitions
    pub fields: Vec<FieldDefinition>,
    
    /// Template options
    pub options: TemplateOptions,
}

/// Field definition in template
#[derive(Debug, Clone)]
pub struct FieldDefinition {
    /// Field name
    pub name: String,
    
    /// Field type
    pub field_type: FieldType,
    
    /// Is field required?
    pub required: bool,
    
    /// Default value
    pub default: Option<String>,
}

/// Template options
#[derive(Debug, Clone, Default)]
pub struct TemplateOptions {
    /// Enable caching
    pub enable_cache: bool,
    
    /// Validation level
    pub validation: ValidationLevel,
    
    /// Performance optimization level
    pub optimization: OptimizationLevel,
}

/// Optimization levels
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationLevel {
    /// No optimization
    None,
    /// Basic optimization
    Basic,
    /// Aggressive optimization
    Aggressive,
}

impl Default for OptimizationLevel {
    fn default() -> Self {
        Self::Basic
    }
}

impl Default for ValidationLevel {
    fn default() -> Self {
        Self::Basic
    }
}

/// Template factory for creating common templates
pub struct TemplateFactory;

impl TemplateFactory {
    /// Create MCP message template
    pub fn create_mcp_message_template() -> TemplateDefinition {
        TemplateDefinition {
            template: r#"{"id":"{{id:string}}","type_":"{{type:string}}","payload":{{payload:object}},"timestamp":{{timestamp:number}},"version":{"major":1,"minor":0},"security":{{security:object:optional}}}"#.to_string(),
            fields: vec![
                FieldDefinition {
                    name: "id".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    default: None,
                },
                FieldDefinition {
                    name: "type".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    default: None,
                },
                FieldDefinition {
                    name: "payload".to_string(),
                    field_type: FieldType::Object,
                    required: true,
                    default: Some("{}".to_string()),
                },
                FieldDefinition {
                    name: "timestamp".to_string(),
                    field_type: FieldType::Number,
                    required: true,
                    default: None,
                },
                FieldDefinition {
                    name: "security".to_string(),
                    field_type: FieldType::Object,
                    required: false,
                    default: Some("{}".to_string()),
                },
            ],
            options: TemplateOptions {
                enable_cache: true,
                validation: ValidationLevel::Basic,
                optimization: OptimizationLevel::Aggressive,
            },
        }
    }
    
    /// Create AI request template
    pub fn create_ai_request_template() -> TemplateDefinition {
        TemplateDefinition {
            template: r#"{"id":"{{id:string}}","model":"{{model:string}}","messages":{{messages:array}},"parameters":{{parameters:object:optional}}}"#.to_string(),
            fields: vec![
                FieldDefinition {
                    name: "id".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    default: None,
                },
                FieldDefinition {
                    name: "model".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    default: None,
                },
                FieldDefinition {
                    name: "messages".to_string(),
                    field_type: FieldType::Array,
                    required: true,
                    default: Some("[]".to_string()),
                },
                FieldDefinition {
                    name: "parameters".to_string(),
                    field_type: FieldType::Object,
                    required: false,
                    default: Some("{}".to_string()),
                },
            ],
            options: TemplateOptions {
                enable_cache: true,
                validation: ValidationLevel::Basic,
                optimization: OptimizationLevel::Basic,
            },
        }
    }
    
    /// Create AI response template
    pub fn create_ai_response_template() -> TemplateDefinition {
        TemplateDefinition {
            template: r#"{"id":"{{id:string}}","provider":"{{provider:string}}","content":"{{content:string}}","cost":{{cost:number}},"duration_ms":{{duration:number}}}"#.to_string(),
            fields: vec![
                FieldDefinition {
                    name: "id".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    default: None,
                },
                FieldDefinition {
                    name: "provider".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    default: None,
                },
                FieldDefinition {
                    name: "content".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    default: None,
                },
                FieldDefinition {
                    name: "cost".to_string(),
                    field_type: FieldType::Number,
                    required: true,
                    default: Some("0.0".to_string()),
                },
                FieldDefinition {
                    name: "duration".to_string(),
                    field_type: FieldType::Number,
                    required: true,
                    default: Some("0".to_string()),
                },
            ],
            options: TemplateOptions {
                enable_cache: true,
                validation: ValidationLevel::Basic,
                optimization: OptimizationLevel::Basic,
            },
        }
    }
    
    /// Create error message template
    pub fn create_error_template() -> TemplateDefinition {
        TemplateDefinition {
            template: r#"{"id":"{{id:string}}","type_":"Error","error":"{{error:string}}","code":{{code:number:optional}},"details":"{{details:string:optional}}"}"#.to_string(),
            fields: vec![
                FieldDefinition {
                    name: "id".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    default: None,
                },
                FieldDefinition {
                    name: "error".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    default: None,
                },
                FieldDefinition {
                    name: "code".to_string(),
                    field_type: FieldType::Number,
                    required: false,
                    default: Some("500".to_string()),
                },
                FieldDefinition {
                    name: "details".to_string(),
                    field_type: FieldType::String,
                    required: false,
                    default: None,
                },
            ],
            options: TemplateOptions {
                enable_cache: true,
                validation: ValidationLevel::Basic,
                optimization: OptimizationLevel::Aggressive,
            },
        }
    }
}

/// Template renderer for generating output from templates
pub struct TemplateRenderer;

impl TemplateRenderer {
    /// Render a template with given data
    pub fn render(template: &CompiledTemplate, data: &HashMap<String, serde_json::Value>) -> Result<Bytes> {
        let start_time = std::time::Instant::now();
        let mut buffer = BytesMut::with_capacity(template.structure.estimated_size);
        
        // Render each compiled part
        for part in &template.compiled_parts {
            Self::render_part(part, data, &mut buffer)?;
        }
        
        let result = buffer.freeze();
        debug!("Template rendered in {:?}: {} bytes", start_time.elapsed(), result.len());
        
        Ok(result)
    }
    
    /// Render a single template part
    fn render_part(
        part: &CompiledPart,
        data: &HashMap<String, serde_json::Value>,
        buffer: &mut BytesMut,
    ) -> Result<()> {
        match part {
            CompiledPart::Static(text) => {
                buffer.put_slice(text.as_bytes());
            }
            CompiledPart::Field(field_name) => {
                if let Some(value) = data.get(field_name) {
                    let value_str = match value {
                        serde_json::Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        other => serde_json::to_string(other).map_err(|e| {
                            MCPError::Internal(format!("Field serialization failed: {}", e))
                        })?,
                    };
                    buffer.put_slice(value_str.as_bytes());
                } else {
                    return Err(MCPError::Internal(format!("Missing required field: {}", field_name)));
                }
            }
            CompiledPart::Conditional { condition, true_part, false_part } => {
                let should_render = data.get(condition)
                    .map(|v| !v.is_null() && v != &serde_json::Value::Bool(false))
                    .unwrap_or(false);
                
                if should_render {
                    Self::render_part(true_part, data, buffer)?;
                } else if let Some(false_part) = false_part {
                    Self::render_part(false_part, data, buffer)?;
                }
            }
            CompiledPart::Loop { iterator, template } => {
                if let Some(array) = data.get(iterator) {
                    if let serde_json::Value::Array(items) = array {
                        for (i, item) in items.iter().enumerate() {
                            if i > 0 {
                                buffer.put_u8(b',');
                            }
                            
                            // Create context with array item
                            let mut item_data = data.clone();
                            if let serde_json::Value::Object(obj) = item {
                                for (key, value) in obj {
                                    item_data.insert(key.clone(), value.clone());
                                }
                            }
                            
                            Self::render_part(template, &item_data, buffer)?;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Utilities for template management
pub struct TemplateUtils;

impl TemplateUtils {
    /// Validate template definition
    pub fn validate_template(definition: &TemplateDefinition) -> Result<()> {
        // Check for balanced braces
        let brace_count = definition.template.matches("{{").count();
        if brace_count != definition.template.matches("}}").count() {
            return Err(MCPError::Internal("Unbalanced template braces".to_string()));
        }
        
        // Validate field references
        for field in &definition.fields {
            if field.name.is_empty() {
                return Err(MCPError::Internal("Empty field name".to_string()));
            }
            
            if !definition.template.contains(&format!("{{{{{}}}", field.name)) {
                return Err(MCPError::Internal(format!("Field '{}' not found in template", field.name)));
            }
        }
        
        Ok(())
    }
    
    /// Optimize template for performance
    pub fn optimize_template(definition: TemplateDefinition) -> TemplateDefinition {
        // Template optimization logic would go here
        // For now, just return the original
        definition
    }
    
    /// Estimate template performance
    pub fn estimate_performance(definition: &TemplateDefinition) -> TemplatePerformanceEstimate {
        let field_count = definition.fields.len();
        let template_size = definition.template.len();
        
        // Simple heuristic-based estimation
        let estimated_render_time_ns = (field_count * 1000) + (template_size * 10);
        
        TemplatePerformanceEstimate {
            estimated_render_time_ns: estimated_render_time_ns as u64,
            complexity_score: field_count as f64 + (template_size as f64 / 100.0),
            memory_usage_bytes: template_size * 2, // Rough estimate
        }
    }
}

/// Template performance estimate
#[derive(Debug, Clone)]
pub struct TemplatePerformanceEstimate {
    /// Estimated render time in nanoseconds
    pub estimated_render_time_ns: u64,
    
    /// Template complexity score
    pub complexity_score: f64,
    
    /// Estimated memory usage in bytes
    pub memory_usage_bytes: usize,
} 