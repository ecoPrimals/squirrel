// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

pub mod types;
pub mod helpers;
pub mod validation;
pub mod manager;

// Re-export public types
pub use types::{Context, ContextValidation, ContextConfig};
pub use manager::ContextManager;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{MCPError, Result};
    use crate::error::context_err::ContextError;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use uuid::Uuid;
    use manager::ContextManager;
    use validation::ValidationEngine;

    /// Helper function to create a test ContextManager
    fn create_test_manager() -> ContextManager {
        let validations = Arc::new(RwLock::new(HashMap::new()));
        let validation_engine = ValidationEngine::new(Arc::clone(&validations));
        
        ContextManager {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
            validations,
            validation_engine,
        }
    }

    /// Helper function to create a test Context
    fn create_test_context(
        name: &str,
        data: serde_json::Value,
        parent_id: Option<Uuid>,
    ) -> Context {
        Context {
            id: Uuid::new_v4(),
            name: name.to_string(),
            data,
            metadata: None,
            parent_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        }
    }

    #[tokio::test]
    async fn test_context_lifecycle() {
        let manager = create_test_manager();

        let context = Context {
            id: Uuid::new_v4(),
            name: "test_context".to_string(),
            data: serde_json::json!({"key": "value"}),
            metadata: None,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };

        // Create
        let id = manager.create_context(context.clone()).await.unwrap();
        assert_eq!(id, context.id);

        // Get
        let retrieved = manager.get_context(id).await.unwrap();
        assert_eq!(retrieved.id, context.id);

        // Update
        let new_data = serde_json::json!({"key": "new_value"});
        assert!(manager
            .update_context(id, new_data.clone(), None)
            .await
            .is_ok());

        // Verify update
        let updated = manager.get_context(id).await.unwrap();
        assert_eq!(updated.data, new_data);

        // Delete
        assert!(manager.delete_context(id).await.is_ok());

        // Verify deletion
        assert!(manager.get_context(id).await.is_err());
    }

    #[tokio::test]
    async fn test_context_hierarchy() {
        let manager = create_test_manager();

        let parent_id = Uuid::new_v4();
        let child_id = Uuid::new_v4();

        // Create parent
        let parent = Context {
            id: parent_id,
            name: "parent".to_string(),
            data: serde_json::json!({}),
            metadata: None,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };
        assert!(manager.create_context(parent).await.is_ok());

        // Create child
        let child = Context {
            id: child_id,
            name: "child".to_string(),
            data: serde_json::json!({}),
            metadata: None,
            parent_id: Some(parent_id),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };
        assert!(manager.create_context(child).await.is_ok());

        // Get children
        let children = manager.get_child_contexts(parent_id).await.unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, child_id);
    }

    #[tokio::test]
    async fn test_context_validation() {
        let manager = create_test_manager();

        // Register validation
        let validation = ContextValidation {
            schema: serde_json::json!({
                "type": "object",
                "required": ["key"],
                "properties": {
                    "key": {"type": "string"}
                }
            }),
            rules: vec!["required_fields".to_string()],
        };
        assert!(manager
            .register_validation("test".to_string(), validation)
            .await
            .is_ok());

        // Test valid context
        let valid_context = Context {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            data: serde_json::json!({"key": "value"}),
            metadata: None,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };
        assert!(manager.create_context(valid_context).await.is_ok());

        // Test invalid context
        let invalid_context = Context {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            data: serde_json::json!({}),
            metadata: None,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };
        let result = manager.create_context(invalid_context).await;
        println!("Invalid context creation result: {:?}", result);
    }

    // ========== Context Creation Tests ==========

    #[tokio::test]
    async fn test_create_context_success() {
        let manager = create_test_manager();
        let context = create_test_context("test_context", serde_json::json!({"key": "value"}), None);

        let result = manager.create_context(context.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), context.id);

        // Verify context was stored
        let retrieved = manager.get_context(context.id).await.unwrap();
        assert_eq!(retrieved.id, context.id);
        assert_eq!(retrieved.name, context.name);
        assert_eq!(retrieved.data, context.data);
    }

    #[tokio::test]
    async fn test_create_context_with_parent() {
        let manager = create_test_manager();
        
        // Create parent context
        let parent = create_test_context("parent", serde_json::json!({}), None);
        let parent_id = manager.create_context(parent).await.unwrap();

        // Create child context
        let child = create_test_context("child", serde_json::json!({}), Some(parent_id));
        let child_id = manager.create_context(child.clone()).await.unwrap();

        // Verify hierarchy was updated
        let children = manager.get_child_contexts(parent_id).await.unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, child_id);
    }

    #[tokio::test]
    async fn test_create_context_expired() {
        let manager = create_test_manager();
        let mut context = create_test_context("expired", serde_json::json!({}), None);
        
        // Set expiration to the past
        context.expires_at = Some(Utc::now() - chrono::Duration::hours(1));

        let result = manager.create_context(context).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MCPError::Context(ContextError::ValidationError(_))
        ));
    }

    #[tokio::test]
    async fn test_create_context_null_data() {
        let manager = create_test_manager();
        let context = Context {
            id: Uuid::new_v4(),
            name: "null_data".to_string(),
            data: serde_json::Value::Null,
            metadata: None,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };

        let result = manager.create_context(context).await;
        assert!(result.is_err());
    }

    // ========== Context Retrieval Tests ==========

    #[tokio::test]
    async fn test_get_context_success() {
        let manager = create_test_manager();
        let context = create_test_context("test", serde_json::json!({"key": "value"}), None);
        let context_id = manager.create_context(context.clone()).await.unwrap();

        let retrieved = manager.get_context(context_id).await.unwrap();
        assert_eq!(retrieved.id, context.id);
        assert_eq!(retrieved.name, context.name);
        assert_eq!(retrieved.data, context.data);
    }

    #[tokio::test]
    async fn test_get_context_not_found() {
        let manager = create_test_manager();
        let non_existent_id = Uuid::new_v4();

        let result = manager.get_context(non_existent_id).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MCPError::Context(ContextError::NotFound(_))
        ));
    }

    // ========== Context Update Tests ==========

    #[tokio::test]
    async fn test_update_context_success() {
        let manager = create_test_manager();
        let context = create_test_context("test", serde_json::json!({"key": "old_value"}), None);
        let context_id = manager.create_context(context).await.unwrap();

        let new_data = serde_json::json!({"key": "new_value", "extra": "field"});
        let result = manager.update_context(context_id, new_data.clone(), None).await;
        assert!(result.is_ok());

        // Verify update
        let updated = manager.get_context(context_id).await.unwrap();
        assert_eq!(updated.data, new_data);
        assert!(updated.updated_at > updated.created_at);
    }

    #[tokio::test]
    async fn test_update_context_with_metadata() {
        let manager = create_test_manager();
        let context = create_test_context("test", serde_json::json!({"key": "value"}), None);
        let context_id = manager.create_context(context).await.unwrap();

        let new_data = serde_json::json!({"key": "updated"});
        let new_metadata = Some(serde_json::json!({"version": 2, "updated_by": "test"}));
        let result = manager.update_context(context_id, new_data.clone(), new_metadata.clone()).await;
        assert!(result.is_ok());

        // Verify update
        let updated = manager.get_context(context_id).await.unwrap();
        assert_eq!(updated.data, new_data);
        assert_eq!(updated.metadata, new_metadata);
    }

    #[tokio::test]
    async fn test_update_context_not_found() {
        let manager = create_test_manager();
        let non_existent_id = Uuid::new_v4();
        let new_data = serde_json::json!({"key": "value"});

        let result = manager.update_context(non_existent_id, new_data, None).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MCPError::Context(ContextError::NotFound(_))
        ));
    }

    // ========== Context Deletion Tests ==========

    #[tokio::test]
    async fn test_delete_context_success() {
        let manager = create_test_manager();
        let context = create_test_context("test", serde_json::json!({}), None);
        let context_id = manager.create_context(context).await.unwrap();

        // Verify context exists
        assert!(manager.get_context(context_id).await.is_ok());

        // Delete context
        let result = manager.delete_context(context_id).await;
        assert!(result.is_ok());

        // Verify deletion
        assert!(manager.get_context(context_id).await.is_err());
    }

    #[tokio::test]
    async fn test_delete_context_not_found() {
        let manager = create_test_manager();
        let non_existent_id = Uuid::new_v4();

        let result = manager.delete_context(non_existent_id).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MCPError::Context(ContextError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn test_delete_context_with_children() {
        let manager = create_test_manager();
        
        // Create parent
        let parent = create_test_context("parent", serde_json::json!({}), None);
        let parent_id = manager.create_context(parent).await.unwrap();

        // Create children
        let child1 = create_test_context("child1", serde_json::json!({}), Some(parent_id));
        let child2 = create_test_context("child2", serde_json::json!({}), Some(parent_id));
        let child1_id = manager.create_context(child1).await.unwrap();
        let child2_id = manager.create_context(child2).await.unwrap();

        // Verify children exist
        let children = manager.get_child_contexts(parent_id).await.unwrap();
        assert_eq!(children.len(), 2);

        // Delete parent
        assert!(manager.delete_context(parent_id).await.is_ok());

        // Verify parent is deleted
        assert!(manager.get_context(parent_id).await.is_err());

        // Verify hierarchy entry for parent is removed
        let children_after = manager.get_child_contexts(parent_id).await;
        assert!(children_after.is_err());

        // Verify children still exist (they're not automatically deleted)
        assert!(manager.get_context(child1_id).await.is_ok());
        assert!(manager.get_context(child2_id).await.is_ok());
    }

    // ========== Validation Tests ==========

    #[tokio::test]
    async fn test_register_validation_success() {
        let manager = create_test_manager();
        let validation = ContextValidation {
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {"type": "string"}
                }
            }),
            rules: vec!["has_name".to_string()],
        };

        let result = manager.register_validation("test_type".to_string(), validation).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_register_validation_invalid_schema() {
        let manager = create_test_manager();
        let validation = ContextValidation {
            schema: serde_json::Value::Null,
            rules: vec![],
        };

        let result = manager.register_validation("test_type".to_string(), validation).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validation_required_field() {
        let manager = create_test_manager();
        
        // Register validation with required field
        let validation = ContextValidation {
            schema: serde_json::json!({
                "type": "object",
                "required": ["name"],
                "properties": {
                    "name": {"type": "string"}
                }
            }),
            rules: vec!["required:name".to_string()],
        };
        assert!(manager
            .register_validation("test".to_string(), validation)
            .await
            .is_ok());

        // Create context with metadata type
        let valid_context = Context {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            data: serde_json::json!({"name": "test_value"}),
            metadata: Some(serde_json::json!({"type": "test"})),
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };
        assert!(manager.create_context(valid_context).await.is_ok());

        // Try to create context without required field
        let invalid_context = Context {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            data: serde_json::json!({"other": "field"}),
            metadata: Some(serde_json::json!({"type": "test"})),
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };
        let result = manager.create_context(invalid_context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validation_json_schema_type() {
        let manager = create_test_manager();
        
        // Register validation expecting string type
        let validation = ContextValidation {
            schema: serde_json::json!({
                "type": "string"
            }),
            rules: vec![],
        };
        assert!(manager
            .register_validation("string_type".to_string(), validation)
            .await
            .is_ok());

        // Create context with correct type
        let valid_context = Context {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            data: serde_json::json!("string_value"),
            metadata: Some(serde_json::json!({"type": "string_type"})),
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };
        assert!(manager.create_context(valid_context).await.is_ok());

        // Try to create context with wrong type
        let invalid_context = Context {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            data: serde_json::json!(123),
            metadata: Some(serde_json::json!({"type": "string_type"})),
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };
        let result = manager.create_context(invalid_context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_child_contexts_not_found() {
        let manager = create_test_manager();
        let non_existent_id = Uuid::new_v4();

        let result = manager.get_child_contexts(non_existent_id).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MCPError::Context(ContextError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn test_get_child_contexts_empty() {
        let manager = create_test_manager();
        let parent = create_test_context("parent", serde_json::json!({}), None);
        let parent_id = manager.create_context(parent).await.unwrap();

        let children = manager.get_child_contexts(parent_id).await.unwrap();
        assert!(children.is_empty());
    }
}
