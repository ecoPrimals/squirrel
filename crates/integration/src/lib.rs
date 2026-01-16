//! Squirrel Integration
//!
//! This crate provides integration adapters for Squirrel components.

pub mod mcp_ai_tools;
pub mod web_integration;

pub use mcp_ai_tools::create_mcp_ai_tools_adapter;
pub use mcp_ai_tools::create_mcp_ai_tools_adapter_with_config;
pub use mcp_ai_tools::McpAiToolsAdapter;
pub use mcp_ai_tools::McpAiToolsConfig;

// Re-export web integration components that actually exist
pub use web_integration::{
    ApiGateway, ApiGatewayConfig, DashboardConfig, DashboardService, McpBridgeConfig, McpWebBridge,
    ServiceInfo, ServiceRegistry, WebIntegrationConfig, WebIntegrationError,
    WebIntegrationFramework, WebIntegrationResult, WebSocketConnection, WebSocketManager,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_ai_tools_config_default() {
        let _config = McpAiToolsConfig::default();
        // Test that default construction works
        assert!(true);
    }

    #[test]
    fn test_api_gateway_config_default() {
        let config = ApiGatewayConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert!(config.enable_cors);
        assert_eq!(config.rate_limit, 1000);
    }

    #[test]
    fn test_api_gateway_creation() {
        let config = ApiGatewayConfig::default();
        let gateway = ApiGateway::new(config.clone());
        assert_eq!(gateway.config.host, config.host);
        assert_eq!(gateway.config.port, config.port);
    }

    #[test]
    fn test_dashboard_config_default() {
        let config = DashboardConfig::default();
        assert!(config.enable_real_time);
        assert_eq!(config.update_interval, 5);
    }

    #[test]
    fn test_dashboard_service_creation() {
        let config = DashboardConfig::default();
        let service = DashboardService::new(config.clone());
        assert_eq!(service.config.enable_real_time, config.enable_real_time);
        assert_eq!(service.config.update_interval, config.update_interval);
    }

    #[tokio::test]
    async fn test_websocket_manager_creation() {
        let manager = WebSocketManager::new();
        let connections = manager.connections.read().await;
        assert_eq!(connections.len(), 0);
    }

    #[tokio::test]
    async fn test_service_registry_creation() {
        let registry = ServiceRegistry::new();
        let services = registry.services.read().await;
        assert_eq!(services.len(), 0);
    }
}
