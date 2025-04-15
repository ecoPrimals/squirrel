use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use async_trait::async_trait;
use std::sync::Mutex;

use dashboard_core::service::DashboardService;
use dashboard_core::data::{DashboardData, Metrics};
use dashboard_core::health::HealthStatus;
use dashboard_core::error::DashboardError;
use dashboard_core::update::DashboardUpdate;
use dashboard_core::config::DashboardConfig;

// AI Chat related imports
use squirrel_integration::mcp_ai_tools::{McpAiToolsAdapter, McpAiToolsConfig, AiMessageType};
use squirrel_mcp::{MCPInterface, config::McpConfig};

// --- Mock Dashboard Service ---
#[derive(Debug, Clone)]
pub struct MockDashboardService {
    pub data: DashboardData,
}

impl MockDashboardService {
    pub fn new() -> Self {
        // Create default dashboard data
        Self {
            data: DashboardData {
                metrics: Metrics::default(),
                alerts: Vec::new(),
                health_status: HealthStatus::Ok,
                last_update: Utc::now(),
            },
        }
    }
}

#[async_trait]
impl DashboardService for MockDashboardService {
    async fn get_dashboard_data(&self) -> Result<DashboardData, DashboardError> {
        Ok(self.data.clone())
    }

    async fn get_metric_history(&self, _metric_name: &str, _time_period: Duration) -> Result<Vec<f64>, DashboardError> {
        // Return some mock data
        Ok(vec![10.0, 20.0, 30.0, 25.0, 15.0])
    }

    async fn add_alert(&self, _alert: dashboard_core::data::Alert) -> Result<(), DashboardError> {
        Ok(())
    }

    async fn acknowledge_alert(&self, _alert_id: &str, _acknowledged_by: &str) -> Result<(), DashboardError> {
        Ok(())
    }

    async fn subscribe(&self) -> tokio::sync::mpsc::Receiver<DashboardUpdate> {
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        rx
    }

    async fn update_config(&self, _config: DashboardConfig) -> Result<(), DashboardError> {
        Ok(())
    }

    async fn update_dashboard_data(&self, _data: DashboardData) -> Result<(), DashboardError> {
        Ok(())
    }

    async fn start(&self) -> Result<(), DashboardError> {
        Ok(())
    }

    async fn stop(&self) -> Result<(), DashboardError> {
        Ok(())
    }
}

// --- Create a mock McpAiToolsAdapter ---
pub fn create_mock_adapter() -> Arc<McpAiToolsAdapter> {
    let mock_mcp = Arc::new(MockMCP::new());
    
    let config = McpAiToolsConfig {
        openai_api_key: Some("mock-api-key".to_string()),
        default_model: Some("gpt-3.5-turbo".to_string()),
    };
    
    let adapter = McpAiToolsAdapter::new(mock_mcp.clone(), config);
    Arc::new(adapter)
}

// --- MockMCP ---
#[derive(Debug)]
pub struct MockMCP {
    pub messages: Arc<Mutex<Vec<String>>>,
}

impl MockMCP {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl MCPInterface for MockMCP {
    async fn send_text(&self, message: &str) -> anyhow::Result<String> {
        // Store the message
        self.messages.lock().unwrap().push(message.to_string());
        
        // Return a default response
        Ok("Mock MCP response".to_string())
    }
    
    async fn connect(&self, _config: McpConfig) -> anyhow::Result<()> {
        Ok(())
    }
    
    async fn disconnect(&self) -> anyhow::Result<()> {
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        true
    }
}

// --- Add helper methods for testing ---
pub trait ChatTestHelpers {
    fn add_user_message(&mut self, message: String);
    fn add_ai_message(&mut self, message: String);
}

impl ChatTestHelpers for ui_terminal::widgets::chat::ChatState {
    fn add_user_message(&mut self, message: String) {
        self.messages.push(ui_terminal::widgets::chat::ChatMessage {
            content: message,
            is_user: true,
            timestamp: chrono::Utc::now(),
        });
    }
    
    fn add_ai_message(&mut self, message: String) {
        self.messages.push(ui_terminal::widgets::chat::ChatMessage {
            content: message,
            is_user: false,
            timestamp: chrono::Utc::now(),
        });
    }
} 