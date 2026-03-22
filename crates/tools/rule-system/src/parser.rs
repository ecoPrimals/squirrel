// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Parser for rule files
//!
//! Methods use `&self` for future extensibility and consistent API

#![allow(clippy::unused_self)]

use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

use crate::error::{RuleParserError, RuleSystemError, RuleSystemResult};
use crate::models::{Rule, RuleAction, RuleCondition};

/// Parser configuration
#[derive(Debug, Clone)]
#[expect(clippy::struct_excessive_bools, reason = "Parser configuration flags")]
pub struct ParserConfig {
    /// Whether to validate rules during parsing
    pub validate: bool,
    /// Whether to extract metadata from frontmatter
    pub extract_metadata: bool,
    /// Whether to parse conditions from the body
    pub parse_conditions: bool,
    /// Whether to parse actions from the body
    pub parse_actions: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            validate: true,
            extract_metadata: true,
            parse_conditions: true,
            parse_actions: true,
        }
    }
}

/// Front matter format type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontMatterFormat {
    /// YAML front matter (---...---)
    Yaml,
    /// TOML front matter (+++...+++)
    Toml,
    /// JSON front matter (;;;...;;;)
    Json,
}

/// Section of a rule file
#[derive(Debug, Clone)]
pub struct RuleSection {
    /// Name of the section
    pub name: String,
    /// Content of the section
    pub content: String,
}

/// Rule parser
#[derive(Debug)]
pub struct RuleParser {
    /// Parser configuration
    config: ParserConfig,
}

impl RuleParser {
    /// Creates a new rule parser with the given configuration
    #[must_use]
    pub fn new(config: ParserConfig) -> Self {
        Self { config }
    }

    /// Parse a rule from a file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub async fn parse_rule_file(&self, path: impl AsRef<Path>) -> RuleSystemResult<Rule> {
        let path = path.as_ref();

        // Read the file content
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| RuleSystemError::from(RuleParserError::IoError(e)))?;

        // Parse the rule
        let mut rule = self
            .parse_rule(&content)
            .map_err(RuleSystemError::ParseError)?;

        // Set the file path
        rule.file_path = Some(path.to_path_buf());

        Ok(rule)
    }

    /// Parse a rule from a string
    ///
    /// # Errors
    ///
    /// Returns an error if the rule cannot be parsed.
    pub fn parse_rule(&self, content: &str) -> Result<Rule, RuleParserError> {
        // Parse frontmatter and body
        let (frontmatter, body) = self.parse_frontmatter(content)?;

        // Parse sections
        let sections = self.parse_sections(body)?;

        // Create rule from frontmatter and sections
        let rule = self.create_rule(&frontmatter, sections)?;

        // Validate the rule
        if self.config.validate {
            self.validate_rule(&rule)?;
        }

        Ok(rule)
    }

    /// Parse frontmatter from content
    ///
    /// Returns a tuple of (frontmatter, body) if successful.
    pub fn parse_frontmatter<'a>(
        &self,
        content: &'a str,
    ) -> Result<(HashMap<String, Value>, &'a str), RuleParserError> {
        let (format, start, end) = self.detect_frontmatter(content)?;

        // Extract frontmatter content
        let frontmatter_content = &content[start..end];

        // Parse frontmatter based on format
        let frontmatter = match format {
            FrontMatterFormat::Yaml => self.parse_yaml_frontmatter(frontmatter_content)?,
            FrontMatterFormat::Toml => self.parse_toml_frontmatter(frontmatter_content)?,
            FrontMatterFormat::Json => self.parse_json_frontmatter(frontmatter_content)?,
        };

        // Extract body
        let body = &content[(end + 1)..];

        Ok((frontmatter, body))
    }

    /// Detect frontmatter in content
    ///
    /// # Errors
    ///
    /// Returns an error if the frontmatter format cannot be detected.
    fn detect_frontmatter(
        &self,
        content: &str,
    ) -> Result<(FrontMatterFormat, usize, usize), RuleParserError> {
        // YAML frontmatter (---...---)
        if content.starts_with("---\n") || content.starts_with("---\r\n") {
            let end_pattern = "\n---";
            if let Some(end_pos) = content[3..].find(end_pattern) {
                return Ok((FrontMatterFormat::Yaml, 4, 3 + end_pos));
            }
        }

        // TOML frontmatter (+++...+++)
        if content.starts_with("+++\n") || content.starts_with("+++\r\n") {
            let end_pattern = "\n+++";
            if let Some(end_pos) = content[3..].find(end_pattern) {
                return Ok((FrontMatterFormat::Toml, 4, 3 + end_pos));
            }
        }

        // JSON frontmatter (;;;...;;;)
        if content.starts_with(";;;\n") || content.starts_with(";;;\r\n") {
            let end_pattern = "\n;;;";
            if let Some(end_pos) = content[3..].find(end_pattern) {
                return Ok((FrontMatterFormat::Json, 4, 3 + end_pos));
            }
        }

        Err(RuleParserError::InvalidFrontmatter(
            "No valid frontmatter found".to_string(),
        ))
    }

    /// Parse YAML frontmatter
    ///
    /// # Errors
    ///
    /// Returns an error if the YAML cannot be parsed.
    fn parse_yaml_frontmatter(
        &self,
        content: &str,
    ) -> Result<HashMap<String, Value>, RuleParserError> {
        serde_yaml_ng::from_str(content).map_err(|e| RuleParserError::YamlError(e.to_string()))
    }

    /// Parse TOML frontmatter
    ///
    /// # Errors
    ///
    /// Returns an error if the TOML cannot be parsed.
    fn parse_toml_frontmatter(
        &self,
        content: &str,
    ) -> Result<HashMap<String, Value>, RuleParserError> {
        let value: toml::Value =
            toml::from_str(content).map_err(|e| RuleParserError::TomlError(e.to_string()))?;

        // Convert TOML value to serde_json::Value
        let json_value =
            serde_json::to_value(value).map_err(|e| RuleParserError::JsonError(e.to_string()))?;

        // Extract as HashMap
        if let Value::Object(map) = json_value {
            let mut result = HashMap::new();
            for (key, value) in map {
                result.insert(key, value);
            }
            Ok(result)
        } else {
            Err(RuleParserError::TomlError(
                "Root is not a table".to_string(),
            ))
        }
    }

    /// Parse JSON frontmatter
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON cannot be parsed.
    fn parse_json_frontmatter(
        &self,
        content: &str,
    ) -> Result<HashMap<String, Value>, RuleParserError> {
        let value: Value =
            serde_json::from_str(content).map_err(|e| RuleParserError::JsonError(e.to_string()))?;

        // Extract as HashMap
        if let Value::Object(map) = value {
            let mut result = HashMap::new();
            for (key, value) in map {
                result.insert(key, value);
            }
            Ok(result)
        } else {
            Err(RuleParserError::JsonError(
                "Root is not an object".to_string(),
            ))
        }
    }

    /// Parse sections from the body of a rule file.
    ///
    /// Returns an error if the sections cannot be parsed.
    fn parse_sections(&self, body: &str) -> Result<Vec<RuleSection>, RuleParserError> {
        let mut sections = Vec::new();
        let mut current_section: Option<(String, Vec<String>)> = None;

        // Define regex for section headers
        let section_regex = Regex::new(r"^## (.+)$")
            .map_err(|e| RuleParserError::Other(format!("Failed to compile section regex: {e}")))?;

        // Process lines
        for line in body.lines() {
            // Check if line is a section header
            if let Some(captures) = section_regex.captures(line) {
                // If we have a current section, add it to sections
                if let Some((name, content)) = current_section.take() {
                    sections.push(RuleSection {
                        name,
                        content: content.join("\n"),
                    });
                }

                // Start new section - safely access the capture group
                let section_name = captures
                    .get(1)
                    .map_or_else(|| "Unknown".to_string(), |m| m.as_str().trim().to_string());
                current_section = Some((section_name, Vec::new()));
            } else if let Some((_, content)) = &mut current_section {
                // Add line to current section
                content.push(line.to_string());
            }
        }

        // Add the last section
        if let Some((name, content)) = current_section {
            sections.push(RuleSection {
                name,
                content: content.join("\n"),
            });
        }

        Ok(sections)
    }

    /// Create a rule from frontmatter and sections
    ///
    /// # Errors
    ///
    /// Returns an error if the rule cannot be created.
    fn create_rule(
        &self,
        frontmatter: &HashMap<String, Value>,
        sections: Vec<RuleSection>,
    ) -> Result<Rule, RuleParserError> {
        // Extract required fields from frontmatter
        let id = self
            .extract_string_field(frontmatter, "id")
            .map_err(|_| RuleParserError::MissingField("id".to_string()))?;

        // Start building the rule
        let mut rule = Rule::new(id);

        // Extract other fields from frontmatter
        if let Ok(name) = self.extract_string_field(frontmatter, "name") {
            rule.name = name;
        }

        if let Ok(description) = self.extract_string_field(frontmatter, "description") {
            rule.description = description;
        }

        if let Ok(version) = self.extract_string_field(frontmatter, "version") {
            rule.version = version;
        }

        if let Ok(category) = self.extract_string_field(frontmatter, "category") {
            rule.category = category;
        }

        if let Ok(priority) = self.extract_integer_field(frontmatter, "priority") {
            rule.priority = priority;
        }

        // Extract patterns
        if let Ok(patterns) = self.extract_string_array_field(frontmatter, "patterns") {
            rule.patterns = patterns;
        } else if let Ok(pattern) = self.extract_string_field(frontmatter, "pattern") {
            rule.patterns = vec![pattern];
        }

        // Extract dependencies
        if let Ok(dependencies) = self.extract_string_array_field(frontmatter, "dependencies") {
            rule.dependencies = dependencies;
        }

        // Process sections
        for section in sections {
            match section.name.to_lowercase().as_str() {
                "conditions" => {
                    if self.config.parse_conditions {
                        let conditions = self.parse_conditions(&section.content)?;
                        rule.conditions.extend(conditions);
                    }
                }
                "actions" => {
                    if self.config.parse_actions {
                        let actions = self.parse_actions(&section.content)?;
                        rule.actions.extend(actions);
                    }
                }
                _ => {
                    // For custom sections, add them to metadata
                    if self.config.extract_metadata {
                        rule.metadata
                            .insert(section.name.clone(), Value::String(section.content.clone()));
                    }
                }
            }
        }

        Ok(rule)
    }

    /// Parse conditions from section content
    ///
    /// # Errors
    ///
    /// Returns an error if the conditions cannot be parsed.
    fn parse_conditions(&self, content: &str) -> Result<Vec<RuleCondition>, RuleParserError> {
        // For now, we'll parse conditions as YAML
        serde_yaml_ng::from_str(content)
            .map_err(|e| RuleParserError::YamlError(format!("Failed to parse conditions: {e}")))
    }

    /// Parse actions from section content
    ///
    /// # Errors
    ///
    /// Returns an error if the actions cannot be parsed.
    fn parse_actions(&self, content: &str) -> Result<Vec<RuleAction>, RuleParserError> {
        // For now, we'll parse actions as YAML
        serde_yaml_ng::from_str(content)
            .map_err(|e| RuleParserError::YamlError(format!("Failed to parse actions: {e}")))
    }

    /// Validate a rule
    ///
    /// # Errors
    ///
    /// Returns an error if the rule is invalid.
    fn validate_rule(&self, rule: &Rule) -> Result<(), RuleParserError> {
        // Check required fields
        if rule.id.is_empty() {
            return Err(RuleParserError::MissingField("id".to_string()));
        }

        if rule.name.is_empty() {
            return Err(RuleParserError::MissingField("name".to_string()));
        }

        if rule.patterns.is_empty() {
            return Err(RuleParserError::MissingField("patterns".to_string()));
        }

        // Validate actions and conditions (basic validation)
        if rule.conditions.is_empty() {
            return Err(RuleParserError::InvalidFieldValue {
                field: "conditions".to_string(),
                reason: "At least one condition is required".to_string(),
            });
        }

        if rule.actions.is_empty() {
            return Err(RuleParserError::InvalidFieldValue {
                field: "actions".to_string(),
                reason: "At least one action is required".to_string(),
            });
        }

        Ok(())
    }

    /// Extract a string field from a `HashMap`
    ///
    /// # Errors
    ///
    /// Returns an error if the field is not found or is not a string.
    fn extract_string_field(
        &self,
        map: &HashMap<String, Value>,
        key: &str,
    ) -> Result<String, RuleParserError> {
        map.get(key)
            .ok_or_else(|| RuleParserError::MissingField(key.to_string()))?
            .as_str()
            .map(std::string::ToString::to_string)
            .ok_or_else(|| RuleParserError::InvalidFieldValue {
                field: key.to_string(),
                reason: "Not a string".to_string(),
            })
    }

    /// Extract an integer field from a `HashMap`
    ///
    /// # Errors
    ///
    /// Returns an error if the field is not found or is not an integer.
    fn extract_integer_field(
        &self,
        map: &HashMap<String, Value>,
        key: &str,
    ) -> Result<i32, RuleParserError> {
        map.get(key)
            .ok_or_else(|| RuleParserError::MissingField(key.to_string()))?
            .as_i64()
            .and_then(|i| i32::try_from(i).ok())
            .ok_or_else(|| RuleParserError::InvalidFieldValue {
                field: key.to_string(),
                reason: "Not an integer".to_string(),
            })
    }

    /// Extract a string array field from a `HashMap`
    ///
    /// # Errors
    ///
    /// Returns an error if the field is not found or is not a string array.
    fn extract_string_array_field(
        &self,
        map: &HashMap<String, Value>,
        key: &str,
    ) -> Result<Vec<String>, RuleParserError> {
        let array = map
            .get(key)
            .ok_or_else(|| RuleParserError::MissingField(key.to_string()))?
            .as_array()
            .ok_or_else(|| RuleParserError::InvalidFieldValue {
                field: key.to_string(),
                reason: "Not an array".to_string(),
            })?;

        let mut result = Vec::new();

        for value in array {
            let string = value
                .as_str()
                .ok_or_else(|| RuleParserError::InvalidFieldValue {
                    field: key.to_string(),
                    reason: "Array contains non-string elements".to_string(),
                })?
                .to_string();

            result.push(string);
        }

        Ok(result)
    }
}

impl Default for RuleParser {
    fn default() -> Self {
        Self::new(ParserConfig::default())
    }
}

/// Create a rule from a file path
///
/// # Errors
///
/// Returns an error if the rule cannot be parsed.
pub async fn parse_rule_file(path: impl AsRef<Path>) -> RuleSystemResult<Rule> {
    let parser = RuleParser::default();
    parser.parse_rule_file(path).await
}

/// Create a rule from content
///
/// # Errors
///
/// Returns an error if the rule cannot be parsed.
pub fn parse_rule_content(content: &str) -> Result<Rule, RuleParserError> {
    let parser = RuleParser::default();
    parser.parse_rule(content)
}
