// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Parser for MDC/YAML rule format
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

use super::error::{Result, RuleError};
use super::models::{Rule, RuleAction, RuleCondition, RuleMetadata};

/// Rule parser for parsing MDC/YAML rules
#[derive(Debug)]
pub struct RuleParser;

impl RuleParser {
    /// Parse a rule from a file
    pub async fn parse_file(path: impl AsRef<Path>) -> Result<Rule> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).await.map_err(RuleError::IoError)?;

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
            }
            None => Err(RuleError::ParseError(
                "No frontmatter found in rule".to_string(),
            )),
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
                    sections.insert(current_section.to_string(), current_content.join("\n"));
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
                sections.insert(current_section.to_string(), current_content.join("\n"));
            }
        }

        Ok(sections)
    }

    /// Create a rule from frontmatter and sections
    fn create_rule(frontmatter: Value, sections: HashMap<String, String>) -> Result<Rule> {
        // Extract basic rule properties from frontmatter
        let id = frontmatter
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or(RuleError::ParseError(
                "Missing or invalid 'id' in rule frontmatter".to_string(),
            ))?
            .to_string();

        let name = frontmatter
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or(RuleError::ParseError(
                "Missing or invalid 'name' in rule frontmatter".to_string(),
            ))?
            .to_string();

        let description = frontmatter
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let version = frontmatter
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0")
            .to_string();

        let category = frontmatter
            .get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();

        let priority = frontmatter
            .get("priority")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;

        // Parse patterns from frontmatter
        let patterns = frontmatter
            .get("patterns")
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
            frontmatter
                .get("pattern")
                .and_then(|v| v.as_str())
                .map(|s| vec![s.to_string()])
                .unwrap_or_default()
        } else {
            patterns
        };

        // If still no patterns, return error
        if patterns.is_empty() {
            return Err(RuleError::ParseError(
                "Rule must have at least one pattern".to_string(),
            ));
        }

        // Parse conditions
        let conditions = sections
            .get("Conditions")
            .map(|s| Self::parse_conditions(s))
            .transpose()?
            .unwrap_or_else(Vec::new);

        // Parse actions
        let actions = sections
            .get("Actions")
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
                    Err(e) => {
                        return Err(RuleError::ParseError(format!(
                            "Failed to parse condition: {e}"
                        )));
                    }
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
                    Err(e) => {
                        return Err(RuleError::ParseError(format!(
                            "Failed to parse action: {e}"
                        )));
                    }
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
            return Err(RuleError::ValidationError(
                "Rule ID is required".to_string(),
            ));
        }

        if rule.name.is_empty() {
            return Err(RuleError::ValidationError(
                "Rule name is required".to_string(),
            ));
        }

        if rule.patterns.is_empty() {
            return Err(RuleError::ValidationError(
                "Rule must have at least one pattern".to_string(),
            ));
        }

        // Validate ID
        if rule.id.is_empty() {
            return Err(RuleError::ValidationError(
                "Rule ID cannot be empty".to_string(),
            ));
        }

        // Validate name
        if rule.name.is_empty() {
            return Err(RuleError::ValidationError(
                "Rule name cannot be empty".to_string(),
            ));
        }

        // Validate description
        if rule.description.is_empty() {
            return Err(RuleError::ValidationError(
                "Rule description cannot be empty".to_string(),
            ));
        }

        // Validate version format (basic semver check)
        if rule.version.is_empty() {
            return Err(RuleError::ValidationError(
                "Rule version cannot be empty".to_string(),
            ));
        }

        // Basic semver validation
        if !rule.version.chars().any(|c| c.is_ascii_digit()) {
            return Err(RuleError::ValidationError(
                "Rule version must contain at least one digit".to_string(),
            ));
        }

        // Validate category
        if rule.category.is_empty() {
            return Err(RuleError::ValidationError(
                "Rule category cannot be empty".to_string(),
            ));
        }

        // Validate priority range
        if rule.priority < 0 {
            return Err(RuleError::ValidationError(
                "Rule priority must be non-negative".to_string(),
            ));
        }

        if rule.priority > 10000 {
            return Err(RuleError::ValidationError(
                "Rule priority must be less than 10000".to_string(),
            ));
        }

        // Validate patterns content
        for pattern in &rule.patterns {
            if pattern.is_empty() {
                return Err(RuleError::ValidationError(
                    "Rule patterns cannot be empty".to_string(),
                ));
            }

            // Basic pattern validation - check for common issues
            if pattern.len() < 2 {
                return Err(RuleError::ValidationError(
                    "Rule patterns must be at least 2 characters long".to_string(),
                ));
            }
        }

        // Validate conditions
        if rule.conditions.is_empty() {
            return Err(RuleError::ValidationError(
                "Rule must have at least one condition".to_string(),
            ));
        }

        // Validate actions
        if rule.actions.is_empty() {
            return Err(RuleError::ValidationError(
                "Rule must have at least one action".to_string(),
            ));
        }

        // Validate ID format (alphanumeric, hyphens, underscores only)
        if !rule
            .id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(RuleError::ValidationError(
                "Rule ID must contain only alphanumeric characters, hyphens, and underscores"
                    .to_string(),
            ));
        }

        // Validate name length
        if rule.name.len() > 100 {
            return Err(RuleError::ValidationError(
                "Rule name must be 100 characters or less".to_string(),
            ));
        }

        // Validate description length
        if rule.description.len() > 1000 {
            return Err(RuleError::ValidationError(
                "Rule description must be 1000 characters or less".to_string(),
            ));
        }

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
            output.push_str(&format!("  - \"{pattern}\"\n"));
        }
    }

    // Add metadata
    let metadata_json = serde_json::to_string_pretty(&rule.metadata)?;
    output.push_str("metadata: |\n");
    for line in metadata_json.lines() {
        output.push_str(&format!("  {line}\n"));
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
        if (content.starts_with("---\n") || content.starts_with("---\r\n"))
            && let Some(end_index) = content[4..].find("---")
        {
            let frontmatter = &content[4..end_index + 4];
            let remaining = &content[(end_index + 4 + 4)..];
            return Ok((Some(frontmatter.to_string()), remaining.to_string()));
        }

        Ok((None, content.to_string()))
    }

    /// Parse YAML frontmatter to a Value
    pub fn parse_yaml_frontmatter(frontmatter: &str) -> Result<Value> {
        serde_yaml_ng::from_str(frontmatter)
            .map_err(|e| RuleError::ParseError(format!("Failed to parse YAML frontmatter: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use std::path::Path;
    use tempfile::NamedTempFile;

    const VALID_RULE_CONTENT: &str = r#"---
id: parse-file-test
name: Parse File Test
description: Testing parse_file from disk
patterns:
  - "test"
---

## Conditions

```json
{
  "type": "match",
  "config": {
    "path": "test",
    "pattern": ".*"
  }
}
```

## Actions

```json
{
  "type": "log",
  "config": {
    "level": "info",
    "message": "test"
  }
}
```
"#;

    #[tokio::test]
    async fn test_parse_file_valid() {
        let mut temp_file = NamedTempFile::new().expect("create temp file");
        temp_file
            .write_all(VALID_RULE_CONTENT.as_bytes())
            .expect("write");
        temp_file.flush().expect("flush");

        let path = temp_file.path();
        let result = RuleParser::parse_file(path).await;

        assert!(
            result.is_ok(),
            "Should parse valid rule file: {:?}",
            result.err()
        );
        let rule = result.expect("parse succeeds");
        assert_eq!(rule.id, "parse-file-test");
        assert_eq!(rule.name, "Parse File Test");
    }

    #[tokio::test]
    async fn test_parse_file_not_found() {
        let path = Path::new("/nonexistent/path/that/does/not/exist.mdc");
        let result = RuleParser::parse_file(path).await;

        assert!(result.is_err(), "Should fail for missing file");
        assert!(matches!(result.unwrap_err(), RuleError::IoError(_)));
    }

    #[tokio::test]
    async fn test_parse_file_path_ref() {
        let mut temp_file = NamedTempFile::new().expect("create temp file");
        temp_file
            .write_all(VALID_RULE_CONTENT.as_bytes())
            .expect("write");
        temp_file.flush().expect("flush");

        let path: &Path = temp_file.path();
        let result = RuleParser::parse_file(path).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_yaml_frontmatter_invalid() {
        let invalid_yaml = "id: unclosed: quote \"test";
        let result = FrontmatterParser::parse_yaml_frontmatter(invalid_yaml);

        assert!(result.is_err(), "Should fail on invalid YAML");
        match result.unwrap_err() {
            RuleError::ParseError(msg) => {
                assert!(
                    msg.contains("frontmatter"),
                    "Error should mention frontmatter"
                );
            }
            _ => unreachable!("Expected ParseError"),
        }
    }

    #[test]
    fn test_parse_yaml_frontmatter_valid() {
        let valid_yaml = "id: test\nname: Test\npatterns:\n  - \"x\"";
        let result = FrontmatterParser::parse_yaml_frontmatter(valid_yaml);

        assert!(result.is_ok());
        let value = result.expect("yaml frontmatter parses");
        assert_eq!(value.get("id").and_then(|v| v.as_str()), Some("test"));
        assert_eq!(value.get("name").and_then(|v| v.as_str()), Some("Test"));
    }

    #[test]
    fn test_rule_to_mdc_roundtrip_semantic() {
        let content = r#"---
id: roundtrip-semantic
name: Roundtrip Semantic
description: Verify rule_to_mdc preserves data
version: 3.0.0
category: custom
priority: 42
patterns:
  - "pat1"
  - "pat2"
metadata:
  key1: "value1"
  key2: 123
---

## Conditions

```json
{"type": "exists", "config": {"path": "x"}}
```

## Actions

```json
{"type": "log", "config": {"level": "info", "message": "msg"}}
```
"#;

        let rule = RuleParser::parse_string(content).expect("parse");
        let mdc = rule_to_mdc(&rule).expect("serialize");

        assert!(mdc.starts_with("---\n"));
        assert!(mdc.contains("id: roundtrip-semantic"));
        assert!(mdc.contains("name: Roundtrip Semantic"));
        assert!(mdc.contains("version: 3.0.0"));
        assert!(mdc.contains("category: custom"));
        assert!(mdc.contains("priority: 42"));
        assert!(mdc.contains("pat1"));
        assert!(mdc.contains("pat2"));
        assert!(mdc.contains("metadata: |"));
        assert!(mdc.contains("```json"));
    }

    #[test]
    fn test_rule_to_mdc_with_metadata() {
        let mut metadata = RuleMetadata::new();
        metadata.set("author", json!("tester"));
        metadata.set("count", json!(42));

        let rule = Rule {
            id: "meta-rule".to_string(),
            name: "Meta Rule".to_string(),
            description: "Has metadata".to_string(),
            version: "1.0.0".to_string(),
            category: "test".to_string(),
            priority: 50,
            patterns: vec!["ab".to_string()],
            conditions: vec![RuleCondition::Exists {
                path: "x".to_string(),
            }],
            actions: vec![RuleAction::LogMessage {
                level: "info".to_string(),
                message: "test".to_string(),
            }],
            metadata,
        };

        let mdc = rule_to_mdc(&rule).expect("serialize");
        assert!(mdc.contains("author"));
        assert!(mdc.contains("tester"));
    }

    #[test]
    fn test_parse_empty_string() {
        let result = RuleParser::parse_string("");
        assert!(result.is_err());
        if let RuleError::ParseError(msg) = result.unwrap_err() {
            assert!(msg.contains("frontmatter"));
        }
    }

    #[test]
    fn test_parse_invalid_yaml_frontmatter() {
        let content = r#"---
id: test
name: Test
invalid: [unclosed
---

## Conditions

```json
{"type": "match", "config": {"path": "x", "pattern": ".*"}}
```

## Actions

```json
{"type": "log", "config": {"level": "info", "message": "x"}}
```
"#;

        let result = RuleParser::parse_string(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_all_condition_types() {
        let content = r#"---
id: all-conditions
name: All Conditions
description: Test
patterns:
  - "ab"
---

## Conditions

```json
{"type": "match", "config": {"path": "a", "pattern": "x"}}
```

```json
{"type": "exists", "config": {"path": "b"}}
```

```json
{"type": "compare", "config": {"path1": "a", "path2": "b", "operator": "eq"}}
```

```json
{"type": "js", "config": {"expression": "true"}}
```

```json
{"type": "custom", "config": {"id": "c1", "config": {}}}
```

## Actions

```json
{"type": "log", "config": {"level": "info", "message": "ok"}}
```
"#;

        let result = RuleParser::parse_string(content);
        assert!(result.is_ok(), "{:?}", result.err());
        let rule = result.expect("parse succeeds");
        assert_eq!(rule.conditions.len(), 5);
    }

    #[test]
    fn test_parse_all_action_types() {
        let content = r#"---
id: all-actions
name: All Actions
description: Test
patterns:
  - "ab"
---

## Conditions

```json
{"type": "exists", "config": {"path": "x"}}
```

## Actions

```json
{"type": "modify", "config": {"path": "x", "value": "y"}}
```

```json
{"type": "recovery", "config": {"name": "r1", "description": "d1"}}
```

```json
{"type": "transform", "config": {"id": "t1", "input_path": "i", "output_path": "o"}}
```

```json
{"type": "command", "config": {"command": "echo", "args": ["hi"]}}
```

```json
{"type": "api", "config": {"url": "http://x", "method": "GET"}}
```

```json
{"type": "log", "config": {"level": "info", "message": "m"}}
```

```json
{"type": "notify", "config": {"title": "t", "message": "m", "level": "info"}}
```

```json
{"type": "custom", "config": {"id": "c1", "config": {}}}
```
"#;

        let result = RuleParser::parse_string(content);
        assert!(result.is_ok(), "{:?}", result.err());
        let rule = result.expect("parse succeeds");
        assert_eq!(rule.actions.len(), 8);
    }

    #[test]
    fn test_parse_section_trailing_newline() {
        let content = r#"---
id: trailing-nl
name: Trailing Newline
description: x
patterns:
  - "ab"
---

## Conditions

```json
{"type": "exists", "config": {"path": "x"}}
```

## Actions

```json
{"type": "log", "config": {"level": "info", "message": "x"}}
```

"#;

        let result = RuleParser::parse_string(content);
        assert!(result.is_ok());
    }
}
