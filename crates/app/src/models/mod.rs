//! Application domain models module
//!
//! This module provides the domain models for the Squirrel application.

/// Basic model for application entities
#[derive(Debug, Clone)]
pub struct BaseModel {
    /// Unique identifier for the model
    pub id: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Last updated timestamp
    pub updated_at: u64,
}

impl BaseModel {
    /// Creates a new base model with the given ID
    #[must_use] pub fn new(id: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            id,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_model_creation() {
        let model = BaseModel::new("test-id".to_string());
        assert_eq!(model.id, "test-id");
        assert_eq!(model.created_at, model.updated_at);
    }
} 