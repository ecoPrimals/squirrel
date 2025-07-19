use std::sync::Arc;

pub struct CallHandler {
    tool_manager: Arc<dyn ToolManager>,
}

impl CallHandler {
    pub fn new(tool_manager: Arc<dyn ToolManager>) -> Self {
        Self { tool_manager }
    }
} 