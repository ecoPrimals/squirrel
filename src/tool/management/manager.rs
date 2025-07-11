/// Core tool manager implementation
#[derive(Debug)]
pub struct CoreToolManager {
    /// Tools registry
    tools: Arc<RwLock<HashMap<String, Arc<Tool>>>>,
    /// Tool information cache
    tool_info: Arc<RwLock<HashMap<String, ToolInfo>>>, // Use ToolInfo from management module
    /// Resource manager
    resource_manager: Arc<dyn ResourceManager>,
    /// Recovery hooks
    recovery_hooks: Arc<RwLock<HashMap<String, Arc<RecoveryHook>>>>,
}

impl CoreToolManager {
    /// Create a new core tool manager
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            tool_info: Arc::new(RwLock::new(HashMap::new())),
            resource_manager: Arc::new(BasicResourceManager::new()),
            recovery_hooks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl ToolManager for CoreToolManager {
    async fn get_tool(&self, id: &str) -> crate::error::types::Result<Option<ToolInfo>> {
        let tool_info = self.tool_info.read().await;
        Ok(tool_info.get(id).cloned())
    }
    
    async fn list_tools(&self) -> crate::error::types::Result<Vec<ToolInfo>> {
        let tool_info = self.tool_info.read().await;
        Ok(tool_info.values().cloned().collect())
    }
} 