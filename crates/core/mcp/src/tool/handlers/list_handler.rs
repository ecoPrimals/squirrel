use std::sync::Arc;

pub struct ListHandler {
    tool_manager: Arc<dyn ToolManager>,
}

impl ListHandler {
    pub fn new(tool_manager: Arc<dyn ToolManager>) -> Self {
        Self { tool_manager }
    }
} 