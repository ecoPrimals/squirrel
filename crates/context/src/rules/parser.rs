//! Parser for MDC/YAML rule format
use std::path::Path;
use std::collections::HashMap;
use serde_json::Value;
use tokio::fs;

use super::error::{Result, RuleError};
use super::models::{Rule, RuleCondition, RuleAction, RuleMetadata};

/// Rule parser for parsing MDC/YAML rules
#[derive(Debug)]
pub struct RuleParser;

impl RuleParser {
    /// Parse a rule from a file
    pub async fn parse_file(path: impl AsRef<Path>) -> Result<Rule> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).await
            .map_err(RuleError::IoError)?;
        
        Self::parse_string(&content)
    }
    
    /// Parse a rule from a string
    pub fn parse_string(content: &str) -> Result<Rule> {
        // Extract frontmatter
        let (frontmatter, remaining) = Self::parse_frontmatter(content)?;
        
        // Parse sections
        let sections = Self::parse_sections(&remaining)?;
        
        // Create rule
        let rule = Self::create_rule(frontmatter, sections)?;
        
        // Validate rule
        Self::validate_rule(&rule)?;
        
        Ok(rule)
    }
    
    /// Parse frontmatter from a string
    fn parse_frontmatter(content: &str) -> Result<(Value, String)> {
        let (frontmatter_opt, remaining) = FrontmatterParser::extract_frontmatter(content)?;
        
        match frontmatter_opt {
            Some(frontmatter) => {
                let frontmatter_value = FrontmatterParser::parse_yaml_frontmatter(&frontmatter)?;
                Ok((frontmatter_value, remaining))
            },
            None => Err(RuleError::ParseError("No frontmatter found in rule".to_string())),
        }
    }
    
    /// Parse sections from a string
    fn parse_sections(content: &str) -> Result<HashMap<String, String>> {
        let mut sections = HashMap::new();
        let mut current_section = "";
        let mut current_content = Vec::new();
        
        // Split content into lines
        let lines: Vec<&str> = content.lines().collect();
        
        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("## ") {
                // If we've been collecting a section, add it to the map
                if !current_section.is_empty() && !current_content.is_empty() {
                    sections.insert(
                        current_section.to_string(),
                        current_content.join("\n"),
                    );
                    current_content.clear();
                }
                
                // Start a new section
                if let Some(section_name) = line.strip_prefix("## ") {
                    current_section = section_name;
                }
            } else if !current_section.is_empty() {
                // Add line to current section
                current_content.push(*line);
            }
            
            // Handle the last section
            if i == lines.len() - 1 && !current_section.is_empty() && !current_content.is_empty() {
                sections.insert(
                    current_section.to_string(),
                    current_content.join("\n"),
                );
            }
        }
        
        Ok(sections)
    }
    
    /// Create a rule from frontmatter and sections
    fn create_rule(frontmatter: Value, sections: HashMap<String, String>) -> Result<Rule> {
        // Extract basic rule properties from frontmatter
        let id = frontmatter.get("id")
            .and_then(|v| v.as_str())
            .ok_or(RuleError::ParseError("Missing or invalid 'id' in rule frontmatter".to_string()))?
            .to_string();
        
        let name = frontmatter.get("name")
            .and_then(|v| v.as_str())
            .ok_or(RuleError::ParseError("Missing or invalid 'name' in rule frontmatter".to_string()))?
            .to_string();
        
        let description = frontmatter.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let version = frontmatter.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0")
            .to_string();
        
        let category = frontmatter.get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();
        
        let priority = frontmatter.get("priority")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        
        // Parse patterns from frontmatter
        let patterns = frontmatter.get("patterns")
            .and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>()
                })
            })
            .unwrap_or_default();
        
        // If no patterns, try a single pattern
        let patterns = if patterns.is_empty() {
            frontmatter.get("pattern")
                .and_then(|v| v.as_str())
                .map(|s| vec![s.to_string()])
                .unwrap_or_default()
        } else {
            patterns
        };
        
        // If still no patterns, return error
        if patterns.is_empty() {
            return Err(RuleError::ParseError("Rule must have at least one pattern".to_string()));
        }
        
        // Parse conditions
        let conditions = sections.get("Conditions")
            .map(|s| Self::parse_conditions(s))
            .transpose()?
            .unwrap_or_else(Vec::new);
        
        // Parse actions
        let actions = sections.get("Actions")
            .map(|s| Self::parse_actions(s))
            .transpose()?
            .unwrap_or_else(Vec::new);
        
        // Extract metadata
        let mut metadata = RuleMetadata::new();
        if let Some(meta_obj) = frontmatter.get("metadata").and_then(|v| v.as_object()) {
            for (key, value) in meta_obj {
                metadata.set(key, value.clone());
            }
        }
        
        // Create rule
        let rule = Rule {
            id,
            name,
            description,
            version,
            category,
            priority,
            patterns,
            conditions,
            actions,
            metadata,
        };
        
        Ok(rule)
    }
    
    /// Parse conditions from a section
    fn parse_conditions(section: &str) -> Result<Vec<RuleCondition>> {
        let mut conditions = Vec::new();
        
        // Simple parsing - we'll improve this with a proper parser in the future
        // For now, we'll just look for JSON objects surrounded by ```json and ```
        let lines: Vec<&str> = section.lines().collect();
        let mut in_json = false;
        let mut json_content = String::new();
        
        for line in lines {
            if line.trim() == "```json" {
                in_json = true;
                json_content.clear();
            } else if line.trim() == "```" && in_json {
                in_json = false;
                
                // Parse JSON to condition
                match serde_json::from_str::<RuleCondition>(&json_content) {
                    Ok(condition) => conditions.push(condition),
                    Err(e) => return Err(RuleError::ParseError(format!("Failed to parse condition: {}", e))),
                }
            } else if in_json {
                json_content.push_str(line);
                json_content.push('\n');
            }
        }
        
        Ok(conditions)
    }
    
    /// Parse actions from a section
    fn parse_actions(section: &str) -> Result<Vec<RuleAction>> {
        let mut actions = Vec::new();
        
        // Simple parsing - we'll improve this with a proper parser in the future
        // For now, we'll just look for JSON objects surrounded by ```json and ```
        let lines: Vec<&str> = section.lines().collect();
        let mut in_json = false;
        let mut json_content = String::new();
        
        for line in lines {
            if line.trim() == "```json" {
                in_json = true;
                json_content.clear();
            } else if line.trim() == "```" && in_json {
                in_json = false;
                
                // Parse JSON to action
                match serde_json::from_str::<RuleAction>(&json_content) {
                    Ok(action) => actions.push(action),
                    Err(e) => return Err(RuleError::ParseError(format!("Failed to parse action: {}", e))),
                }
            } else if in_json {
                json_content.push_str(line);
                json_content.push('\n');
            }
        }
        
        Ok(actions)
    }
    
    /// Validate a rule
    fn validate_rule(rule: &Rule) -> Result<()> {
        // Check required fields
        if rule.id.is_empty() {
            return Err(RuleError::ValidationError("Rule ID is required".to_string()));
        }
        
        if rule.name.is_empty() {
            return Err(RuleError::ValidationError("Rule name is required".to_string()));
        }
        
        if rule.patterns.is_empty() {
            return Err(RuleError::ValidationError("Rule must have at least one pattern".to_string()));
        }
        
        // TODO: Add more validation as needed
        
        Ok(())
    }
}

/// Convert a rule to MDC format for saving
pub fn rule_to_mdc(rule: &Rule) -> Result<String> {
    let mut output = String::new();
    
    // Generate frontmatter
    output.push_str("---\n");
    
    // Add metadata fields
    output.push_str(&format!("id: {}\n", rule.id));
    output.push_str(&format!("name: {}\n", rule.name));
    output.push_str(&format!("description: {}\n", rule.description));
    output.push_str(&format!("category: {}\n", rule.category));
    output.push_str(&format!("priority: {}\n", rule.priority));
    
    // Add version
    output.push_str(&format!("version: {}\n", rule.version));
    
    // Add patterns
    if !rule.patterns.is_empty() {
        output.push_str("patterns:\n");
        for pattern in &rule.patterns {
            output.push_str(&format!("  - \"{}\"\n", pattern));
        }
    }
    
    // Add metadata
    let metadata_json = serde_json::to_string_pretty(&rule.metadata)?;
    output.push_str("metadata: |\n");
    for line in metadata_json.lines() {
        output.push_str(&format!("  {}\n", line));
    }
    
    // End frontmatter
    output.push_str("---\n\n");
    
    // Add conditions section if conditions exist
    if !rule.conditions.is_empty() {
        output.push_str("# Conditions\n\n");
        let conditions_json = serde_json::to_string_pretty(&rule.conditions)?;
        output.push_str("```json\n");
        output.push_str(&conditions_json);
        output.push_str("\n```\n\n");
    }
    
    // Add actions section if actions exist
    if !rule.actions.is_empty() {
        output.push_str("# Actions\n\n");
        let actions_json = serde_json::to_string_pretty(&rule.actions)?;
        output.push_str("```json\n");
        output.push_str(&actions_json);
        output.push_str("\n```\n\n");
    }
    
    Ok(output)
}

/// Simple frontmatter parser
pub struct FrontmatterParser;

impl FrontmatterParser {
    /// Extract frontmatter from content
    pub fn extract_frontmatter(content: &str) -> Result<(Option<String>, String)> {
        if content.starts_with("---\n") || content.starts_with("---\r\n") {
            if let Some(end_index) = content[4..].find("---") {
                let frontmatter = &content[4..end_index + 4];
                let remaining = &content[(end_index + 4 + 4)..];
                return Ok((Some(frontmatter.to_string()), remaining.to_string()));
            }
        }
        
        Ok((None, content.to_string()))
    }
    
    /// Parse YAML frontmatter to a Value
    pub fn parse_yaml_frontmatter(frontmatter: &str) -> Result<Value> {
        serde_yaml::from_str(frontmatter)
            .map_err(|e| RuleError::ParseError(format!("Failed to parse YAML frontmatter: {}", e)))
    }
} 