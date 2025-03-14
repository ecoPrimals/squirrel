use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use crate::ai::mcp_tools::{
    types::MCPError,
    context::MachineContext,
};
use futures::Stream;

/// Represents a system prompt template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub template: String,
    pub variables: Vec<String>,
    pub description: String,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub version: u32,
}

/// Configuration for LLM interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub stop_sequences: Vec<String>,
    pub timeout: u64,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            stop_sequences: vec![],
            timeout: 30,
        }
    }
}

/// Response from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub text: String,
    pub usage: LLMUsage,
    pub finish_reason: String,
}

/// Usage statistics for LLM calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Service for managing LLM interactions
pub struct LLMService {
    #[allow(dead_code)]
    context: Arc<RwLock<MachineContext>>,
    templates: Arc<RwLock<HashMap<String, PromptTemplate>>>,
    config: LLMConfig,
}

impl LLMService {
    /// Create a new LLM service
    pub fn new(context: Arc<RwLock<MachineContext>>, config: LLMConfig) -> Self {
        Self {
            context,
            templates: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Register a new prompt template
    pub fn register_template(&self, template: PromptTemplate) -> Result<(), MCPError> {
        let mut templates = self.templates.write()
            .map_err(|_| MCPError::ProtocolError("Failed to acquire lock".to_string()))?;
        
        templates.insert(template.id.clone(), template);
        Ok(())
    }

    /// Get a prompt template by ID
    pub fn get_template(&self, id: &str) -> Result<Option<PromptTemplate>, MCPError> {
        let templates = self.templates.read()
            .map_err(|_| MCPError::ProtocolError("Failed to acquire lock".to_string()))?;
        
        Ok(templates.get(id).cloned())
    }

    /// List all available prompt templates
    pub fn list_templates(&self) -> Result<Vec<PromptTemplate>, MCPError> {
        let templates = self.templates.read()
            .map_err(|_| MCPError::ProtocolError("Failed to acquire lock".to_string()))?;
        
        Ok(templates.values().cloned().collect())
    }

    /// Render a prompt template with variables
    pub fn render_template(&self, id: &str, variables: HashMap<String, String>) -> Result<String, MCPError> {
        let template = self.get_template(id)?
            .ok_or_else(|| MCPError::CommandNotFound(format!("Template {} not found", id)))?;

        let mut rendered = template.template.clone();
        for var in &template.variables {
            let value = variables.get(var)
                .ok_or_else(|| MCPError::InvalidArguments(format!("Missing variable: {}", var)))?;
            rendered = rendered.replace(&format!("{{{}}}", var), value);
        }

        Ok(rendered)
    }

    /// Send a prompt to the LLM and get a response
    pub async fn complete(&self, _prompt: String) -> Result<LLMResponse, MCPError> {
        // TODO: Implement actual LLM API call
        // This is a placeholder implementation
        Ok(LLMResponse {
            text: "Placeholder response".to_string(),
            usage: LLMUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
            finish_reason: "stop".to_string(),
        })
    }

    /// Stream responses from the LLM
    pub async fn stream_complete(&self, _prompt: String) -> Result<impl Stream<Item = Result<String, MCPError>>, MCPError> {
        // TODO: Implement actual LLM API streaming
        // This is a placeholder implementation
        Ok(futures::stream::iter(vec![Ok("Placeholder response".to_string())]))
    }

    /// Update LLM configuration
    pub fn update_config(&mut self, config: LLMConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_llm_service() {
        let context = Arc::new(RwLock::new(MachineContext::new().unwrap()));
        let config = LLMConfig::default();
        let service = LLMService::new(context, config);

        // Test template registration
        let template = PromptTemplate {
            id: "test".to_string(),
            template: "Hello, {name}!".to_string(),
            variables: vec!["name".to_string()],
            description: "Test template".to_string(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            version: 1,
        };
        service.register_template(template.clone()).unwrap();

        // Test template retrieval
        let retrieved = service.get_template("test").unwrap().unwrap();
        assert_eq!(retrieved.id, "test");

        // Test template rendering
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), "world".to_string());
        let rendered = service.render_template("test", variables).unwrap();
        assert_eq!(rendered, "Hello, world!");

        // Test LLM completion
        let response = service.complete("Test prompt".to_string()).await.unwrap();
        assert_eq!(response.finish_reason, "stop");
    }
} 