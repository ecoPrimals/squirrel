//! Tests for the analytics dashboard integration module

use super::*;
use crate::dashboard::manager::DashboardManager;
use crate::dashboard::plugin::{DashboardPluginRegistry, PluginId};
use crate::analytics::{
    AnalyticsDataPoint, 
    AnalyticsTimeSeries,
    AnalyticsResult,
    AnalyticsConfig,
    create_analytics_service
};
use chrono::{DateTime, Utc};
use serde_json::json;
use tokio::sync::Mutex;
use std::sync::Arc;
use anyhow::Result;
use pretty_assertions::assert_eq;
use mockall::predicate::*;
use mockall::mock;

// Mock the dashboard manager for testing
mock! {
    DashboardManager {
        fn register_plugin(&self, plugin: Arc<dyn DashboardPlugin>) -> Result<PluginId>;
        fn get_plugin_registry(&self) -> Arc<dyn DashboardPluginRegistry>;
        fn initialize(&self) -> Result<()>;
    }
}

// Mock the plugin registry for testing
mock! {
    DashboardPluginRegistry {
        fn register_plugin(&self, plugin: Arc<dyn DashboardPlugin>) -> Result<PluginId>;
        fn get_plugin(&self, id: &PluginId) -> Option<Arc<dyn DashboardPlugin>>;
        fn get_plugins(&self) -> Vec<Arc<dyn DashboardPlugin>>;
        fn get_plugins_by_type(&self, plugin_type: &str) -> Vec<Arc<dyn DashboardPlugin>>;
    }
}

#[tokio::test]
async fn test_analytics_visualization_plugin_initialization() {
    // Create a plugin
    let plugin = AnalyticsVisualizationPlugin::new();
    
    // Initialize the plugin
    let result = plugin.initialize().await;
    assert!(result.is_ok());
    
    // Check metadata
    let metadata = plugin.get_metadata();
    assert_eq!(metadata.id, "analytics_visualization_plugin");
    assert_eq!(metadata.name, "Analytics Visualization Plugin");
    assert_eq!(metadata.plugin_type, "visualization");
    assert_eq!(metadata.version, "1.0.0");
}

#[tokio::test]
async fn test_analytics_data_source_plugin_initialization() {
    // Create a plugin
    let plugin = AnalyticsDataSourcePlugin::new();
    
    // Initialize the plugin
    let result = plugin.initialize().await;
    assert!(result.is_ok());
    
    // Check metadata
    let metadata = plugin.get_metadata();
    assert_eq!(metadata.id, "analytics_data_source_plugin");
    assert_eq!(metadata.name, "Analytics Data Source Plugin");
    assert_eq!(metadata.plugin_type, "data_source");
    assert_eq!(metadata.version, "1.0.0");
}

#[tokio::test]
async fn test_visualization_plugin_data_handling() {
    // Create a plugin
    let plugin = AnalyticsVisualizationPlugin::new();
    
    // Initialize the plugin
    let _ = plugin.initialize().await;
    
    // Generate test data
    let test_data = json!({
        "time_series": [
            {
                "name": "CPU Usage",
                "data": [
                    {
                        "timestamp": "2023-01-01T00:00:00Z",
                        "value": 30.5,
                        "metadata": { "unit": "percent" }
                    },
                    {
                        "timestamp": "2023-01-01T00:01:00Z",
                        "value": 32.1,
                        "metadata": { "unit": "percent" }
                    }
                ]
            }
        ]
    });
    
    // Update the plugin with data
    let update_result = plugin.update(test_data.clone()).await;
    assert!(update_result.is_ok());
    
    // Get data from the plugin
    let data_result = plugin.get_data().await;
    assert!(data_result.is_ok());
    
    let data = data_result.unwrap();
    
    // Check that the data contains our time series
    assert!(data.as_object().unwrap().contains_key("time_series"));
    
    // Render a visualization
    let render_result = plugin.render_visualization("line_chart", &test_data).await;
    assert!(render_result.is_ok());
    
    let visualization = render_result.unwrap();
    assert!(visualization.as_object().unwrap().contains_key("svg"));
}

#[tokio::test]
async fn test_data_source_plugin_query_capabilities() {
    // Create a plugin
    let plugin = AnalyticsDataSourcePlugin::new();
    
    // Initialize the plugin
    let _ = plugin.initialize().await;
    
    // Test the plugin capabilities
    let capabilities = plugin.get_capabilities().await.unwrap();
    
    assert!(capabilities.contains_key("time_series_analysis"));
    assert!(capabilities.contains_key("trend_detection"));
    assert!(capabilities.contains_key("pattern_recognition"));
    assert!(capabilities.contains_key("predictive_analytics"));
    
    // Test a query
    let query = json!({
        "type": "time_series_analysis",
        "params": {
            "metric": "cpu_usage",
            "start_time": "2023-01-01T00:00:00Z",
            "end_time": "2023-01-01T01:00:00Z",
            "aggregation": "avg"
        }
    });
    
    let result = plugin.query(&query).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dashboard_factory_creation() {
    // Create a factory
    let factory = AnalyticsDashboardFactory::new();
    
    // Create plugins
    let vis_plugin = factory.create_visualization_plugin();
    let data_plugin = factory.create_data_source_plugin();
    
    // Check that the plugins were created
    assert_eq!(vis_plugin.get_metadata().id, "analytics_visualization_plugin");
    assert_eq!(data_plugin.get_metadata().id, "analytics_data_source_plugin");
}

#[tokio::test]
async fn test_dashboard_factory_registration() {
    // Create a factory
    let factory = Arc::new(AnalyticsDashboardFactory::new());
    
    // Create a mock dashboard manager
    let mut mock_dashboard_manager = MockDashboardManager::new();
    let mut mock_plugin_registry = MockDashboardPluginRegistry::new();
    
    // Set up expectations
    mock_plugin_registry
        .expect_register_plugin()
        .times(2)
        .returning(|_| Ok(PluginId::new()));
    
    mock_dashboard_manager
        .expect_get_plugin_registry()
        .returning(move || Arc::new(mock_plugin_registry.clone()));
    
    // Register the factory with the mock dashboard manager
    let result = factory.register_with_dashboard(&mock_dashboard_manager).await;
    
    // Check that registration was successful
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_analytics_service_integration() {
    // Create analytics service
    let config = AnalyticsConfig::default();
    let service = create_analytics_service(config).unwrap();
    
    // Create a mock dashboard manager
    let mut mock_dashboard_manager = MockDashboardManager::new();
    let mut mock_plugin_registry = MockDashboardPluginRegistry::new();
    
    // Set up expectations
    mock_plugin_registry
        .expect_register_plugin()
        .times(2)
        .returning(|_| Ok(PluginId::new()));
    
    mock_dashboard_manager
        .expect_get_plugin_registry()
        .returning(move || Arc::new(mock_plugin_registry.clone()));
    
    // Register with the dashboard
    let result = service.register_with_dashboard(&mock_dashboard_manager).await;
    assert!(result.is_ok());
    
    // Test creating plugins
    let vis_plugin = service.create_visualization_plugin();
    let data_plugin = service.create_data_source_plugin();
    
    assert!(vis_plugin.is_ok());
    assert!(data_plugin.is_ok());
}

#[tokio::test]
async fn test_end_to_end_integration() {
    // Create test time series data
    let now = Utc::now();
    let data_points = vec![
        AnalyticsDataPoint {
            timestamp: now,
            value: 42.0,
            metadata: Some(json!({"unit": "percent"})),
        },
        AnalyticsDataPoint {
            timestamp: now + chrono::Duration::minutes(1),
            value: 43.5,
            metadata: Some(json!({"unit": "percent"})),
        },
    ];
    
    let time_series = AnalyticsTimeSeries {
        name: "test_metric".to_string(),
        data: data_points,
        metadata: Some(json!({"component": "test_component"})),
    };
    
    // Create the analytics result
    let result = AnalyticsResult {
        id: "test_result".to_string(),
        analysis_type: "time_series".to_string(),
        start_time: now,
        end_time: now + chrono::Duration::minutes(1),
        data: json!({
            "time_series": [time_series]
        }),
        metadata: None,
    };
    
    // Create a factory
    let factory = AnalyticsDashboardFactory::new();
    
    // Create a visualization plugin
    let plugin = factory.create_visualization_plugin();
    
    // Initialize the plugin
    let _ = plugin.initialize().await;
    
    // Update the plugin with our result data
    let update_result = plugin.update(json!(result)).await;
    assert!(update_result.is_ok());
    
    // Get data from the plugin
    let data_result = plugin.get_data().await;
    assert!(data_result.is_ok());
    
    // Render a visualization
    let params = json!({
        "chart_type": "line_chart",
        "id": "test_result",
    });
    
    let render_result = plugin.render_visualization("line_chart", &params).await;
    assert!(render_result.is_ok());
    
    let visualization = render_result.unwrap();
    assert!(visualization.as_object().unwrap().contains_key("svg"));
} 