---
version: 1.1.0
last_updated: 2024-03-28
status: in_progress
priority: high
---

# Dashboard Integration Specification

## Overview
This document describes the dashboard integration component of the Squirrel monitoring system, focusing on the visualization, interaction, and configuration of monitoring data through user-facing dashboards.

## Dashboard Components

### 1. Dashboard Data Model
```rust
pub struct DashboardData {
    /// System-wide metrics
    pub system_metrics: SystemMetricsSnapshot,
    /// Protocol metrics
    pub protocol_metrics: ProtocolMetricsSnapshot,
    /// Tool execution metrics
    pub tool_metrics: HashMap<String, ToolMetricsSnapshot>,
    /// Alert history
    pub alerts: Vec<AlertHistory>,
    /// Health status
    pub health: SystemHealth,
    /// Network statistics
    pub network: NetworkMetricsSnapshot,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

pub struct SystemMetricsSnapshot {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub uptime: Duration,
    pub thread_count: u32,
    pub error_count: u64,
}

pub struct ProtocolMetricsSnapshot {
    pub messages_processed: u64,
    pub avg_latency: Duration,
    pub error_rate: f64,
    pub active_connections: u32,
}

pub struct NetworkMetricsSnapshot {
    pub bytes_received: u64,
    pub bytes_transmitted: u64,
    pub current_rx_rate: u64,
    pub current_tx_rate: u64,
    pub active_connections: u32,
    pub connection_error_rate: f64,
}

pub struct ToolMetricsSnapshot {
    pub executions: u64,
    pub avg_execution_time: Duration,
    pub success_rate: f64,
    pub memory_usage: u64,
}

pub struct AlertHistory {
    pub id: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub acknowledged: bool,
}
```

### 2. Dashboard Service
```rust
#[async_trait::async_trait]
pub trait DashboardService: Send + Sync {
    /// Get the current dashboard data
    async fn get_dashboard_data(&self) -> Result<DashboardData>;
    
    /// Get historical data for a specific metric
    async fn get_metric_history(
        &self,
        metric_name: &str,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
        resolution: HistoryResolution,
    ) -> Result<Vec<MetricDataPoint>>;
    
    /// Acknowledge an alert
    async fn acknowledge_alert(&self, alert_id: &str) -> Result<()>;
    
    /// Configure dashboard settings
    async fn configure_dashboard(&self, config: DashboardConfig) -> Result<()>;
    
    /// Subscribe to real-time dashboard updates
    async fn subscribe(&self) -> Result<mpsc::Receiver<DashboardUpdate>>;
}

pub enum HistoryResolution {
    Minute,
    Hour,
    Day,
}

pub struct MetricDataPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: MetricValue,
}

pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<(f64, u64)>),
    Text(String),
}

pub struct DashboardUpdate {
    pub update_type: UpdateType,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub enum UpdateType {
    Metrics,
    Alerts,
    Health,
    Network,
}
```

### 3. Dashboard Configuration
```rust
pub struct DashboardConfig {
    /// Update interval in seconds
    pub update_interval: u64,
    /// Which metric categories to display
    pub displayed_categories: HashSet<MetricCategory>,
    /// Custom dashboard panels
    pub custom_panels: Vec<PanelConfig>,
    /// Alert display settings
    pub alert_settings: AlertDisplaySettings,
    /// Data retention period
    pub retention_period: Duration,
}

pub enum MetricCategory {
    System,
    Protocol,
    Tool,
    Network,
    Custom(String),
}

pub struct PanelConfig {
    pub id: String,
    pub title: String,
    pub panel_type: PanelType,
    pub metrics: Vec<String>,
    pub position: PanelPosition,
    pub size: PanelSize,
    pub refresh_rate: u64,
}

pub enum PanelType {
    LineChart,
    BarChart,
    Gauge,
    Table,
    StatusPanel,
    Custom(String),
}

pub struct AlertDisplaySettings {
    pub show_acknowledged: bool,
    pub max_alerts: usize,
    pub group_by_source: bool,
    pub auto_refresh: bool,
}
```

## Implementation

### 1. Dashboard Implementation
```rust
pub struct DashboardServiceImpl {
    monitoring_service: Arc<dyn MonitoringService>,
    database: Arc<dyn MetricsDatabase>,
    config: DashboardConfig,
    subscribers: Arc<RwLock<Vec<mpsc::Sender<DashboardUpdate>>>>,
    update_interval: Duration,
    is_running: AtomicBool,
}

impl DashboardServiceImpl {
    pub fn new(
        monitoring_service: Arc<dyn MonitoringService>,
        database: Arc<dyn MetricsDatabase>,
        config: DashboardConfig,
    ) -> Self;
    
    async fn start_update_loop(&self);
    async fn collect_dashboard_data(&self) -> Result<DashboardData>;
    async fn send_update(&self, update: DashboardUpdate);
    async fn store_metric_history(&self, data: &DashboardData);
}
```

### 2. Database Integration
```rust
#[async_trait::async_trait]
pub trait MetricsDatabase: Send + Sync {
    async fn store_metrics(&self, data: &DashboardData) -> Result<()>;
    
    async fn get_metric_history(
        &self,
        metric_name: &str,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
        resolution: HistoryResolution,
    ) -> Result<Vec<MetricDataPoint>>;
    
    async fn store_alert(&self, alert: &AlertHistory) -> Result<()>;
    
    async fn get_alerts(
        &self,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
        limit: usize,
    ) -> Result<Vec<AlertHistory>>;
    
    async fn acknowledge_alert(&self, alert_id: &str) -> Result<()>;
    
    async fn cleanup_old_data(&self, retention_period: Duration) -> Result<u64>;
}
```

## User Interface Components

### 1. Web Dashboard
The web dashboard presents monitoring data through a responsive web interface:

- **System Overview Panel**: Displays CPU, memory, and uptime
- **Protocol Metrics Panel**: Displays message processing stats
- **Tool Performance Panel**: Displays tool execution metrics
- **Network Activity Panel**: Displays network traffic metrics
- **Health Status Panel**: Displays component health status
- **Alerts Panel**: Displays active and recent alerts

### 2. Dashboard API
The dashboard provides a RESTful API for external access:

```
GET  /api/dashboard               - Get current dashboard data
GET  /api/metrics/{metric}        - Get specific metric data
GET  /api/metrics/history/{metric}- Get historical data for metric
POST /api/alerts/{id}/acknowledge - Acknowledge an alert
GET  /api/health                  - Get health status
GET  /api/config                  - Get dashboard configuration
POST /api/config                  - Update dashboard configuration
WS   /api/ws                      - WebSocket for real-time updates
```

### 3. Console Dashboard
A console-based terminal UI dashboard is also available:

```rust
pub struct ConsoleDashboard {
    dashboard_service: Arc<dyn DashboardService>,
    display_config: ConsoleDisplayConfig,
    is_running: AtomicBool,
}

impl ConsoleDashboard {
    pub fn new(dashboard_service: Arc<dyn DashboardService>) -> Self;
    pub async fn start(&self) -> Result<()>;
    pub async fn stop(&self) -> Result<()>;
    async fn refresh_display(&self) -> Result<()>;
    async fn handle_input(&self) -> Result<()>;
}
```

## Integration Points

### 1. Monitoring Service Integration
The dashboard integrates with the monitoring service to collect data:

```rust
// In DashboardServiceImpl
async fn collect_data_from_monitoring(&self) -> Result<DashboardData> {
    let status = self.monitoring_service.status().await?;
    
    // Transform monitoring data into dashboard format
    let dashboard_data = DashboardData {
        system_metrics: extract_system_metrics(&status),
        protocol_metrics: extract_protocol_metrics(&status),
        tool_metrics: extract_tool_metrics(&status),
        alerts: extract_alerts(&status),
        health: status.health,
        network: extract_network_metrics(&status),
        last_updated: chrono::Utc::now(),
    };
    
    Ok(dashboard_data)
}
```

### 2. Alert System Integration
The dashboard integrates with the alert system for notifications:

```rust
// In DashboardServiceImpl
async fn handle_alert(&self, alert: Alert) -> Result<()> {
    // Convert alert to AlertHistory
    let alert_history = AlertHistory {
        id: Uuid::new_v4().to_string(),
        severity: alert.severity,
        message: alert.message,
        source: alert.source,
        timestamp: chrono::Utc::now(),
        acknowledged: false,
    };
    
    // Store alert in database
    self.database.store_alert(&alert_history).await?;
    
    // Send update to subscribers
    self.send_update(DashboardUpdate {
        update_type: UpdateType::Alerts,
        data: serde_json::to_value(&alert_history)?,
        timestamp: chrono::Utc::now(),
    }).await;
    
    Ok(())
}
```

### 3. Metrics Database Integration
The dashboard uses a metrics database for historical data:

```rust
// Implementation using an SQL database
pub struct SqlMetricsDatabase {
    pool: sqlx::PgPool,
}

#[async_trait::async_trait]
impl MetricsDatabase for SqlMetricsDatabase {
    async fn store_metrics(&self, data: &DashboardData) -> Result<()> {
        // Store metrics in database
        let mut tx = self.pool.begin().await?;
        
        // Insert system metrics
        sqlx::query!(
            "INSERT INTO system_metrics (timestamp, cpu_usage, memory_usage, uptime, thread_count, error_count)
             VALUES ($1, $2, $3, $4, $5, $6)",
            data.last_updated,
            data.system_metrics.cpu_usage,
            data.system_metrics.memory_usage as i64,
            data.system_metrics.uptime.as_secs() as i64,
            data.system_metrics.thread_count as i32,
            data.system_metrics.error_count as i64,
        )
        .execute(&mut tx)
        .await?;
        
        // Insert other metrics...
        
        tx.commit().await?;
        Ok(())
    }
    
    // Other database operations...
}
```

## Data Visualization

### 1. Chart Components
The dashboard provides various chart components:

- **Time Series Charts**: For tracking metrics over time
- **Gauges**: For displaying current values against thresholds
- **Bar Charts**: For comparing metrics across categories
- **Heat Maps**: For visualizing complex data distributions
- **Status Indicators**: For showing component health

### 2. Visualization Configuration
Charts are configurable through the dashboard API:

```rust
pub struct ChartConfig {
    pub id: String,
    pub title: String,
    pub type_: ChartType,
    pub metrics: Vec<String>,
    pub time_range: TimeRange,
    pub refresh_rate: u64,
    pub thresholds: Option<Thresholds>,
    pub colors: Option<ColorScheme>,
}

pub enum ChartType {
    Line,
    Bar,
    Gauge,
    HeatMap,
    StatusIndicator,
}

pub struct TimeRange {
    pub duration: Duration,
    pub resolution: HistoryResolution,
}

pub struct Thresholds {
    pub warning: f64,
    pub critical: f64,
}

pub enum ColorScheme {
    Default,
    Traffic,
    Health,
    Custom(Vec<String>),
}
```

## Performance Characteristics

### 1. Dashboard Performance
- UI refresh rate: Configurable, default 5 seconds
- Data collection overhead: < 1% CPU
- Memory usage: < 50MB for dashboard service
- Database storage: Configurable retention period

### 2. Scaling Considerations
- Supports up to 100 concurrent dashboard users
- Time series compression for historical data
- Automatic data downsampling for long time ranges
- Read-heavy database optimizations

## Error Handling

### 1. UI Error Handling
- Graceful degradation of unavailable components
- Clear error messages for data collection failures
- Automatic reconnection for WebSocket failures
- Fallback to cached data when backend is unavailable

### 2. Backend Error Handling
- Circuit breaker for database connections
- Retry with backoff for transient errors
- Error logging and alerting for persistent issues
- Data validation before storage and display

## Testing Strategy

### 1. UI Testing
- Component-level tests for UI elements
- End-to-end tests for dashboard workflows
- Visual regression testing for UI changes
- Browser compatibility testing

### 2. Backend Testing
- Unit tests for data transformation logic
- Integration tests for database operations
- Mock services for monitoring integration
- Performance testing under load

## Success Criteria

- [x] Web dashboard implementation (UI components and structure)
- [x] Console dashboard implementation 
- [x] Historical data storage
- [ ] Real-time updates via WebSocket (In Progress)
- [x] Alert integration
- [x] Configurable visualization
- [x] API endpoints for external access
- [ ] Dashboard layout persistence (In Progress)
- [ ] Multiple clients support (Pending)
- [ ] Enhanced test coverage for dashboard components (Pending)

## Dependencies

- tokio = "1.0"
- serde = { version = "1.0", features = ["derive"] }
- serde_json = "1.0"
- chrono = { version = "0.4", features = ["serde"] }
- sqlx = { version = "0.6", features = ["postgres", "runtime-tokio-rustls", "chrono"] }
- warp = "0.3"
- tui = "0.19"
- crossterm = "0.25"
- uuid = { version = "1.0", features = ["v4", "serde"] }

## Migration Guide

For components integrating with the dashboard:

1. Add dashboard service to your application:

```rust
let monitoring_service = // ... obtain monitoring service
let metrics_database = SqlMetricsDatabase::new(&db_url).await?;

let dashboard_config = DashboardConfig {
    update_interval: 5,
    displayed_categories: HashSet::from_iter([
        MetricCategory::System,
        MetricCategory::Protocol,
        MetricCategory::Network,
    ]),
    custom_panels: vec![],
    alert_settings: AlertDisplaySettings::default(),
    retention_period: Duration::from_secs(60 * 60 * 24 * 30), // 30 days
};

let dashboard_service = DashboardServiceImpl::new(
    monitoring_service, 
    Arc::new(metrics_database),
    dashboard_config,
);

// Start the dashboard services
dashboard_service.start().await?;
```

2. Start the web server for the dashboard:

```rust
let dashboard_api = DashboardApi::new(Arc::new(dashboard_service));
let routes = dashboard_api.routes();

warp::serve(routes)
    .run(([127, 0, 0, 1], 3030))
    .await;
```

3. Access the dashboard at `http://localhost:3030`

## Future Enhancements

1. **Custom Dashboard Builder**: Allow users to create custom dashboards
2. **Dashboard Sharing**: Export and import dashboard configurations
3. **Mobile Interface**: Optimize for mobile devices
4. **Advanced Alerting**: Add alert correlation and prediction
5. **User Authentication**: Add user-specific dashboard views
6. **Plugin System**: Allow third-party dashboard extensions

<version>1.1.0</version> 