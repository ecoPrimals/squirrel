use std::collections::HashMap;
use tokio::sync::RwLock;

pub mod executor;

pub use executor::*;

/// Tool management system
#[derive(Debug)]
pub struct ToolManager {
    pub tools: RwLock<HashMap<String, String>>, // Tool name -> Tool status
}

impl ToolManager {
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for ToolManager {
    fn default() -> Self {
        Self::new()
    }
}
