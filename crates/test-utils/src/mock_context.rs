use std::sync::Arc;
use tokio::sync::RwLock;
use squirrel_core::error::Result;
use uuid::Uuid;

/// Mock implementation of a context adapter for testing
#[derive(Debug, Default)]
pub struct MockContext {
    /// Whether the mock is initialized
    pub initialized: bool,
    /// Mock context data
    pub data: Vec<(String, String)>,
}

impl MockContext {
    /// Create a new mock context
    pub fn new() -> Self {
        Self {
            initialized: false,
            data: Vec::new(),
        }
    }
    
    /// Initialize the mock context
    pub fn initialize(&mut self) -> Result<()> {
        self.initialized = true;
        Ok(())
    }
    
    /// Check if the mock context is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Add data to the mock context
    pub fn add_data(&mut self, key: &str, value: &str) {
        self.data.push((key.to_string(), value.to_string()));
    }
    
    /// Get data from the mock context
    pub fn get_data(&self, key: &str) -> Option<String> {
        self.data.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }
}

/// Create a new mock context with shared ownership
pub fn create_mock_context() -> Arc<RwLock<MockContext>> {
    Arc::new(RwLock::new(MockContext::new()))
}

/// Create an initialized mock context with shared ownership
pub fn create_initialized_mock_context() -> Result<Arc<RwLock<MockContext>>> {
    let context = Arc::new(RwLock::new(MockContext::new()));
    {
        let mut ctx = context.try_write().unwrap();
        ctx.initialize()?;
    }
    Ok(context)
} 