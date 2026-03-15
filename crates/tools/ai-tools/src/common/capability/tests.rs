// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Tests for the AI capability system

use super::*;

#[test]
fn test_capabilities() {
    // Test basic capability creation and modification
    let mut capabilities = AICapabilities::default();
    
    // Initially empty
    assert!(capabilities.supported_model_types.is_empty());
    assert!(capabilities.supported_task_types.is_empty());
    assert!(capabilities.max_context_size.is_none());
    assert!(!capabilities.supports_streaming);
    assert!(!capabilities.supports_function_calling);
    assert!(!capabilities.supports_tool_use);
    
    // Add capabilities
    capabilities.add_model_type(ModelType::LargeLanguageModel);
    capabilities.add_task_type(TaskType::TextGeneration);
    capabilities.with_max_context_size(4096);
    capabilities.with_streaming(true);
    capabilities.with_function_calling(true);
    
    // Check they were added
    assert!(capabilities.supports_model_type(ModelType::LargeLanguageModel));
    assert!(capabilities.supports_task(TaskType::TextGeneration));
    assert_eq!(capabilities.max_context_size, Some(4096));
    assert!(capabilities.supports_streaming);
    assert!(capabilities.supports_function_calling);
    assert!(!capabilities.supports_tool_use);
    
    // Test convenience constructors
    let text_gen = AICapabilities::text_generation();
    assert!(text_gen.supports_model_type(ModelType::LargeLanguageModel));
    assert!(text_gen.supports_task(TaskType::TextGeneration));
    
    let embedding = AICapabilities::embedding();
    assert!(embedding.supports_model_type(ModelType::Embedding));
    assert!(embedding.supports_task(TaskType::Embedding));
}

#[test]
fn test_task_creation() {
    // Test task creation and modification
    let mut task = AITask::default();
    
    // Check defaults
    assert_eq!(task.task_type, TaskType::TextGeneration);
    assert!(task.min_context_size.is_none());
    assert!(!task.requires_streaming);
    assert!(!task.requires_function_calling);
    assert!(!task.requires_tool_use);
    
    // Modify task
    task.with_min_context_size(8192);
    task.with_streaming(true);
    task.with_function_calling(true);
    
    // Check modifications
    assert_eq!(task.min_context_size, Some(8192));
    assert!(task.requires_streaming);
    assert!(task.requires_function_calling);
    assert!(!task.requires_tool_use);
    
    // Test convenience constructors
    let text_task = AITask::text_generation();
    assert_eq!(text_task.task_type, TaskType::TextGeneration);
    assert_eq!(text_task.required_model_type, Some(ModelType::LargeLanguageModel));
    
    let embedding_task = AITask::embedding();
    assert_eq!(embedding_task.task_type, TaskType::Embedding);
    assert_eq!(embedding_task.required_model_type, Some(ModelType::Embedding));
}

#[test]
fn test_security_requirements() {
    // Test security requirements
    let default_security = SecurityRequirements::default();
    assert!(!default_security.contains_sensitive_data);
    assert!(default_security.required_data_residency.is_none());
    assert!(!default_security.requires_e2e_encryption);
    
    // Create custom security requirements
    let security = SecurityRequirements {
        contains_sensitive_data: true,
        required_data_residency: Some(vec!["US".to_string(), "EU".to_string()]),
        requires_e2e_encryption: true,
    };
    
    assert!(security.contains_sensitive_data);
    assert_eq!(security.required_data_residency.as_ref().unwrap().len(), 2);
    assert!(security.requires_e2e_encryption);
}

#[test]
fn test_task_matching() {
    // Create capabilities
    let mut basic_capabilities = AICapabilities::default();
    basic_capabilities.add_model_type(ModelType::LargeLanguageModel);
    basic_capabilities.add_task_type(TaskType::TextGeneration);
    basic_capabilities.with_max_context_size(4096);
    basic_capabilities.with_streaming(true);
    
    let mut advanced_capabilities = basic_capabilities.clone();
    advanced_capabilities.with_function_calling(true);
    advanced_capabilities.with_tool_use(true);
    advanced_capabilities.with_max_context_size(8192);
    
    // Create tasks
    let basic_task = AITask {
        task_type: TaskType::TextGeneration,
        required_model_type: Some(ModelType::LargeLanguageModel),
        min_context_size: Some(2000),
        requires_streaming: true,
        requires_function_calling: false,
        requires_tool_use: false,
        max_acceptable_latency: None,
        security_requirements: SecurityRequirements::default(),
    };
    
    let function_task = AITask {
        task_type: TaskType::TextGeneration,
        required_model_type: Some(ModelType::LargeLanguageModel),
        min_context_size: Some(2000),
        requires_streaming: true,
        requires_function_calling: true,
        requires_tool_use: false,
        max_acceptable_latency: None,
        security_requirements: SecurityRequirements::default(),
    };
    
    let large_context_task = AITask {
        task_type: TaskType::TextGeneration,
        required_model_type: Some(ModelType::LargeLanguageModel),
        min_context_size: Some(6000),
        requires_streaming: true,
        requires_function_calling: false,
        requires_tool_use: false,
        max_acceptable_latency: None,
        security_requirements: SecurityRequirements::default(),
    };
    
    // Test matching
    assert!(basic_task.can_be_handled_by(&basic_capabilities));
    assert!(basic_task.can_be_handled_by(&advanced_capabilities));
    
    assert!(!function_task.can_be_handled_by(&basic_capabilities));
    assert!(function_task.can_be_handled_by(&advanced_capabilities));
    
    assert!(!large_context_task.can_be_handled_by(&basic_capabilities));
    assert!(large_context_task.can_be_handled_by(&advanced_capabilities));
} 