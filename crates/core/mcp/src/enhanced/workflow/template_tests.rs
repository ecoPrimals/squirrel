// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Comprehensive tests for Workflow Templates
//!
//! Tests cover template creation, parameter validation, parameter substitution,
//! workflow instantiation, and template management.

use super::templates::*;
use super::types::*;
use std::collections::HashMap;

#[tokio::test]
async fn test_template_engine_creation() {
    let config = TemplateEngineConfig::default();
    let engine = WorkflowTemplateEngine::new(config);
    
    let templates = engine.list_templates(None).await.unwrap();
    assert_eq!(templates.len(), 0);
}

#[tokio::test]
async fn test_register_template() {
    let engine = WorkflowTemplateEngine::new(TemplateEngineConfig::default());
    
    let workflow = Workflow {
        id: "template-workflow-1".to_string(),
        name: "Test Workflow".to_string(),
        description: "A test workflow".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![],
        metadata: HashMap::new(),
    };
    
    let template = WorkflowTemplate {
        id: "template-1".to_string(),
        name: "Test Template".to_string(),
        description: "A test template".to_string(),
        version: "1.0.0".to_string(),
        workflow,
        parameters: vec![],
        tags: vec!["test".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    engine.register_template(template).await.unwrap();
    
    let templates = engine.list_templates(None).await.unwrap();
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].id, "template-1");
}

#[tokio::test]
async fn test_get_template() {
    let engine = WorkflowTemplateEngine::new(TemplateEngineConfig::default());
    
    let workflow = Workflow {
        id: "template-workflow-2".to_string(),
        name: "Test Workflow 2".to_string(),
        description: "Another test workflow".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![],
        metadata: HashMap::new(),
    };
    
    let template = WorkflowTemplate {
        id: "template-2".to_string(),
        name: "Test Template 2".to_string(),
        description: "Another test template".to_string(),
        version: "1.0.0".to_string(),
        workflow,
        parameters: vec![],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    engine.register_template(template).await.unwrap();
    
    let retrieved = engine.get_template("template-2").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "Test Template 2");
}

#[tokio::test]
async fn test_delete_template() {
    let engine = WorkflowTemplateEngine::new(TemplateEngineConfig::default());
    
    let workflow = Workflow {
        id: "template-workflow-3".to_string(),
        name: "Test Workflow 3".to_string(),
        description: "Yet another test workflow".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![],
        metadata: HashMap::new(),
    };
    
    let template = WorkflowTemplate {
        id: "template-3".to_string(),
        name: "Test Template 3".to_string(),
        description: "Yet another test template".to_string(),
        version: "1.0.0".to_string(),
        workflow,
        parameters: vec![],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    engine.register_template(template).await.unwrap();
    assert_eq!(engine.list_templates(None).await.unwrap().len(), 1);
    
    engine.delete_template("template-3").await.unwrap();
    assert_eq!(engine.list_templates(None).await.unwrap().len(), 0);
}

#[tokio::test]
async fn test_instantiate_template_basic() {
    let engine = WorkflowTemplateEngine::new(TemplateEngineConfig::default());
    
    let workflow = Workflow {
        id: "template-workflow-4".to_string(),
        name: "Basic Workflow".to_string(),
        description: "A basic workflow".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![],
        metadata: HashMap::new(),
    };
    
    let template = WorkflowTemplate {
        id: "template-4".to_string(),
        name: "Basic Template".to_string(),
        description: "A basic template".to_string(),
        version: "1.0.0".to_string(),
        workflow,
        parameters: vec![],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    engine.register_template(template).await.unwrap();
    
    let parameters = HashMap::new();
    let workflow = engine.instantiate_template("template-4", parameters).await.unwrap();
    
    assert_ne!(workflow.id, "template-workflow-4"); // Should have new ID
    assert_eq!(workflow.name, "Basic Workflow");
}

#[tokio::test]
async fn test_parameter_validation_required() {
    let engine = WorkflowTemplateEngine::new(TemplateEngineConfig::default());
    
    let workflow = Workflow {
        id: "template-workflow-5".to_string(),
        name: "Parameterized Workflow".to_string(),
        description: "A workflow with required parameters".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![],
        metadata: HashMap::new(),
    };
    
    let template = WorkflowTemplate {
        id: "template-5".to_string(),
        name: "Parameterized Template".to_string(),
        description: "A template with required parameters".to_string(),
        version: "1.0.0".to_string(),
        workflow,
        parameters: vec![
            TemplateParameter {
                name: "user_id".to_string(),
                param_type: "string".to_string(),
                description: "User ID".to_string(),
                required: true,
                default_value: None,
                validation: None,
            },
        ],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    engine.register_template(template).await.unwrap();
    
    // Missing required parameter should fail
    let parameters = HashMap::new();
    let result = engine.instantiate_template("template-5", parameters).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_parameter_validation_type_string() {
    let engine = WorkflowTemplateEngine::new(TemplateEngineConfig::default());
    
    let workflow = Workflow {
        id: "template-workflow-6".to_string(),
        name: "String Param Workflow".to_string(),
        description: "A workflow with string parameter".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![],
        metadata: HashMap::new(),
    };
    
    let template = WorkflowTemplate {
        id: "template-6".to_string(),
        name: "String Param Template".to_string(),
        description: "A template with string parameter".to_string(),
        version: "1.0.0".to_string(),
        workflow,
        parameters: vec![
            TemplateParameter {
                name: "message".to_string(),
                param_type: "string".to_string(),
                description: "Message text".to_string(),
                required: true,
                default_value: None,
                validation: None,
            },
        ],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    engine.register_template(template).await.unwrap();
    
    // Valid string parameter
    let mut parameters = HashMap::new();
    parameters.insert("message".to_string(), serde_json::json!("Hello, World!"));
    let result = engine.instantiate_template("template-6", parameters).await;
    assert!(result.is_ok());
    
    // Invalid type (number instead of string)
    let mut parameters = HashMap::new();
    parameters.insert("message".to_string(), serde_json::json!(123));
    let result = engine.instantiate_template("template-6", parameters).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_parameter_validation_type_number() {
    let engine = WorkflowTemplateEngine::new(TemplateEngineConfig::default());
    
    let workflow = Workflow {
        id: "template-workflow-7".to_string(),
        name: "Number Param Workflow".to_string(),
        description: "A workflow with number parameter".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![],
        metadata: HashMap::new(),
    };
    
    let mut validation = HashMap::new();
    validation.insert("min".to_string(), serde_json::json!(0));
    validation.insert("max".to_string(), serde_json::json!(100));
    
    let template = WorkflowTemplate {
        id: "template-7".to_string(),
        name: "Number Param Template".to_string(),
        description: "A template with number parameter".to_string(),
        version: "1.0.0".to_string(),
        workflow,
        parameters: vec![
            TemplateParameter {
                name: "count".to_string(),
                param_type: "number".to_string(),
                description: "Count value".to_string(),
                required: true,
                default_value: None,
                validation: Some(validation),
            },
        ],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    engine.register_template(template).await.unwrap();
    
    // Valid number within range
    let mut parameters = HashMap::new();
    parameters.insert("count".to_string(), serde_json::json!(50));
    let result = engine.instantiate_template("template-7", parameters).await;
    assert!(result.is_ok());
    
    // Number below minimum
    let mut parameters = HashMap::new();
    parameters.insert("count".to_string(), serde_json::json!(-10));
    let result = engine.instantiate_template("template-7", parameters).await;
    assert!(result.is_err());
    
    // Number above maximum
    let mut parameters = HashMap::new();
    parameters.insert("count".to_string(), serde_json::json!(150));
    let result = engine.instantiate_template("template-7", parameters).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_parameter_substitution_simple() {
    let engine = WorkflowTemplateEngine::new(TemplateEngineConfig::default());
    
    let step = WorkflowStep {
        id: "step-1".to_string(),
        name: "Greet {{name}}".to_string(),
        description: "Say hello to {{name}}".to_string(),
        step_type: WorkflowStepType::AIService,
        config: serde_json::json!({
            "message": "Hello, {{name}}!"
        }),
        depends_on: vec![],
        timeout: None,
        retry_policy: None,
    };
    
    let workflow = Workflow {
        id: "template-workflow-8".to_string(),
        name: "Greeting Workflow".to_string(),
        description: "A workflow that greets someone".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![step],
        metadata: HashMap::new(),
    };
    
    let template = WorkflowTemplate {
        id: "template-8".to_string(),
        name: "Greeting Template".to_string(),
        description: "A template for greetings".to_string(),
        version: "1.0.0".to_string(),
        workflow,
        parameters: vec![
            TemplateParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description: "Name to greet".to_string(),
                required: true,
                default_value: None,
                validation: None,
            },
        ],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    engine.register_template(template).await.unwrap();
    
    let mut parameters = HashMap::new();
    parameters.insert("name".to_string(), serde_json::json!("Alice"));
    let workflow = engine.instantiate_template("template-8", parameters).await.unwrap();
    
    assert_eq!(workflow.steps[0].name, "Greet Alice");
    assert_eq!(workflow.steps[0].description, "Say hello to Alice");
    assert_eq!(workflow.steps[0].config["message"], "Hello, Alice!");
}

#[tokio::test]
async fn test_parameter_substitution_multiple() {
    let engine = WorkflowTemplateEngine::new(TemplateEngineConfig::default());
    
    let step = WorkflowStep {
        id: "step-1".to_string(),
        name: "Process {{resource}} for {{user}}".to_string(),
        description: "Processing workflow".to_string(),
        step_type: WorkflowStepType::AIService,
        config: serde_json::json!({
            "resource": "{{resource}}",
            "user": "{{user}}",
            "count": "{{count}}"
        }),
        depends_on: vec![],
        timeout: None,
        retry_policy: None,
    };
    
    let workflow = Workflow {
        id: "template-workflow-9".to_string(),
        name: "Processing Workflow".to_string(),
        description: "A workflow that processes resources".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![step],
        metadata: HashMap::new(),
    };
    
    let template = WorkflowTemplate {
        id: "template-9".to_string(),
        name: "Processing Template".to_string(),
        description: "A template for processing".to_string(),
        version: "1.0.0".to_string(),
        workflow,
        parameters: vec![
            TemplateParameter {
                name: "resource".to_string(),
                param_type: "string".to_string(),
                description: "Resource to process".to_string(),
                required: true,
                default_value: None,
                validation: None,
            },
            TemplateParameter {
                name: "user".to_string(),
                param_type: "string".to_string(),
                description: "User ID".to_string(),
                required: true,
                default_value: None,
                validation: None,
            },
            TemplateParameter {
                name: "count".to_string(),
                param_type: "number".to_string(),
                description: "Count".to_string(),
                required: true,
                default_value: None,
                validation: None,
            },
        ],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    engine.register_template(template).await.unwrap();
    
    let mut parameters = HashMap::new();
    parameters.insert("resource".to_string(), serde_json::json!("document.pdf"));
    parameters.insert("user".to_string(), serde_json::json!("user123"));
    parameters.insert("count".to_string(), serde_json::json!(42));
    let workflow = engine.instantiate_template("template-9", parameters).await.unwrap();
    
    assert_eq!(workflow.steps[0].name, "Process document.pdf for user123");
    assert_eq!(workflow.steps[0].config["resource"], "document.pdf");
    assert_eq!(workflow.steps[0].config["user"], "user123");
    assert_eq!(workflow.steps[0].config["count"], "42");
}

#[tokio::test]
async fn test_parameter_validation_string_length() {
    let engine = WorkflowTemplateEngine::new(TemplateEngineConfig::default());
    
    let workflow = Workflow {
        id: "template-workflow-10".to_string(),
        name: "String Length Workflow".to_string(),
        description: "A workflow with string length validation".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![],
        metadata: HashMap::new(),
    };
    
    let mut validation = HashMap::new();
    validation.insert("min_length".to_string(), serde_json::json!(3));
    validation.insert("max_length".to_string(), serde_json::json!(10));
    
    let template = WorkflowTemplate {
        id: "template-10".to_string(),
        name: "String Length Template".to_string(),
        description: "A template with string length validation".to_string(),
        version: "1.0.0".to_string(),
        workflow,
        parameters: vec![
            TemplateParameter {
                name: "code".to_string(),
                param_type: "string".to_string(),
                description: "Code value".to_string(),
                required: true,
                default_value: None,
                validation: Some(validation),
            },
        ],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    engine.register_template(template).await.unwrap();
    
    // Valid string within length range
    let mut parameters = HashMap::new();
    parameters.insert("code".to_string(), serde_json::json!("ABC123"));
    let result = engine.instantiate_template("template-10", parameters).await;
    assert!(result.is_ok());
    
    // String too short
    let mut parameters = HashMap::new();
    parameters.insert("code".to_string(), serde_json::json!("AB"));
    let result = engine.instantiate_template("template-10", parameters).await;
    assert!(result.is_err());
    
    // String too long
    let mut parameters = HashMap::new();
    parameters.insert("code".to_string(), serde_json::json!("ABCDEFGHIJK"));
    let result = engine.instantiate_template("template-10", parameters).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_parameter_validation_pattern() {
    let engine = WorkflowTemplateEngine::new(TemplateEngineConfig::default());
    
    let workflow = Workflow {
        id: "template-workflow-11".to_string(),
        name: "Pattern Workflow".to_string(),
        description: "A workflow with pattern validation".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![],
        metadata: HashMap::new(),
    };
    
    let mut validation = HashMap::new();
    validation.insert("pattern".to_string(), serde_json::json!(r"^[A-Z]{3}-\d{3}$"));
    
    let template = WorkflowTemplate {
        id: "template-11".to_string(),
        name: "Pattern Template".to_string(),
        description: "A template with pattern validation".to_string(),
        version: "1.0.0".to_string(),
        workflow,
        parameters: vec![
            TemplateParameter {
                name: "ticket_id".to_string(),
                param_type: "string".to_string(),
                description: "Ticket ID (format: ABC-123)".to_string(),
                required: true,
                default_value: None,
                validation: Some(validation),
            },
        ],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    engine.register_template(template).await.unwrap();
    
    // Valid pattern
    let mut parameters = HashMap::new();
    parameters.insert("ticket_id".to_string(), serde_json::json!("ABC-123"));
    let result = engine.instantiate_template("template-11", parameters).await;
    assert!(result.is_ok());
    
    // Invalid pattern (lowercase)
    let mut parameters = HashMap::new();
    parameters.insert("ticket_id".to_string(), serde_json::json!("abc-123"));
    let result = engine.instantiate_template("template-11", parameters).await;
    assert!(result.is_err());
    
    // Invalid pattern (wrong format)
    let mut parameters = HashMap::new();
    parameters.insert("ticket_id".to_string(), serde_json::json!("ABC123"));
    let result = engine.instantiate_template("template-11", parameters).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_templates_with_tags() {
    let engine = WorkflowTemplateEngine::new(TemplateEngineConfig::default());
    
    // Register templates with different tags
    for i in 0..3 {
        let workflow = Workflow {
            id: format!("template-workflow-tag-{}", i),
            name: format!("Tagged Workflow {}", i),
            description: "A tagged workflow".to_string(),
            version: "1.0.0".to_string(),
            steps: vec![],
            metadata: HashMap::new(),
        };
        
        let tags = if i == 0 {
            vec!["production".to_string(), "critical".to_string()]
        } else if i == 1 {
            vec!["development".to_string(), "test".to_string()]
        } else {
            vec!["production".to_string(), "test".to_string()]
        };
        
        let template = WorkflowTemplate {
            id: format!("template-tag-{}", i),
            name: format!("Tagged Template {}", i),
            description: "A tagged template".to_string(),
            version: "1.0.0".to_string(),
            workflow,
            parameters: vec![],
            tags,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        engine.register_template(template).await.unwrap();
    }
    
    // List all templates
    let all_templates = engine.list_templates(None).await.unwrap();
    assert_eq!(all_templates.len(), 3);
    
    // List templates with "production" tag
    let prod_templates = engine.list_templates(Some(vec!["production".to_string()])).await.unwrap();
    assert_eq!(prod_templates.len(), 2);
    
    // List templates with "development" tag
    let dev_templates = engine.list_templates(Some(vec!["development".to_string()])).await.unwrap();
    assert_eq!(dev_templates.len(), 1);
}

#[tokio::test]
async fn test_parameter_validation_array() {
    let engine = WorkflowTemplateEngine::new(TemplateEngineConfig::default());
    
    let workflow = Workflow {
        id: "template-workflow-12".to_string(),
        name: "Array Param Workflow".to_string(),
        description: "A workflow with array parameter".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![],
        metadata: HashMap::new(),
    };
    
    let mut validation = HashMap::new();
    validation.insert("min_items".to_string(), serde_json::json!(1));
    validation.insert("max_items".to_string(), serde_json::json!(5));
    
    let template = WorkflowTemplate {
        id: "template-12".to_string(),
        name: "Array Param Template".to_string(),
        description: "A template with array parameter".to_string(),
        version: "1.0.0".to_string(),
        workflow,
        parameters: vec![
            TemplateParameter {
                name: "items".to_string(),
                param_type: "array".to_string(),
                description: "List of items".to_string(),
                required: true,
                default_value: None,
                validation: Some(validation),
            },
        ],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    engine.register_template(template).await.unwrap();
    
    // Valid array within size range
    let mut parameters = HashMap::new();
    parameters.insert("items".to_string(), serde_json::json!(["a", "b", "c"]));
    let result = engine.instantiate_template("template-12", parameters).await;
    assert!(result.is_ok());
    
    // Array too small
    let mut parameters = HashMap::new();
    parameters.insert("items".to_string(), serde_json::json!([]));
    let result = engine.instantiate_template("template-12", parameters).await;
    assert!(result.is_err());
    
    // Array too large
    let mut parameters = HashMap::new();
    parameters.insert("items".to_string(), serde_json::json!(["a", "b", "c", "d", "e", "f"]));
    let result = engine.instantiate_template("template-12", parameters).await;
    assert!(result.is_err());
}

