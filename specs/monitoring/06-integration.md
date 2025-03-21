---
version: 1.0.0
last_updated: 2024-03-22
status: implemented
priority: high
---

# Monitoring Integration Patterns

## Overview
This document describes the integration patterns and approaches for connecting the monitoring system with other components of the Squirrel platform. It focuses on standardized integration methods, cross-component communication, and best practices for monitoring instrumentation.

## Integration Architecture

### 1. Layered Integration Model

The monitoring system uses a layered integration architecture:

```
┌─────────────────────────────────────────────────────────┐
│                   Consumer Applications                  │
│  (CLI, Web Interface, Dashboard, External Systems)       │
└───────────────────────────────┬─────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────┐
│                Monitoring Service API                    │
│  (Service Interface, Real-time Updates, History Access)  │
└───────────────────────────────┬─────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────┐
│               Component Adapters                         │
│  (MCP Adapter, Command Adapter, App Adapter, etc.)       │
└───────────────────────────────┬─────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────┐
│               Core Monitoring Services                   │
│  (Metrics, Alerts, Health, Network, Dashboard)           │
└───────────────────────────────┬─────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────┐
│               Storage and Export                         │
│  (Database, Prometheus Export, Log Files)                │
└─────────────────────────────────────────────────────────┘
```

### 2. Integration Patterns

#### A. Direct Service Injection
Components receive a direct reference to monitoring services:

```rust
pub struct ComponentWithMonitoring {
    // Component-specific fields
    config: ComponentConfig,
    state: ComponentState,
    
    // Monitoring integration
    monitoring: Arc<dyn MonitoringService>,
}

impl ComponentWithMonitoring {
    pub fn new(
        config: ComponentConfig,
        monitoring: Arc<dyn MonitoringService>
    ) -> Self {
        Self {
            config,
            state: ComponentState::default(),
            monitoring,
        }
    }
    
    async fn operation(&self) -> Result<()> {
        // Record operation start
        let start = Instant::now();
        
        // Perform operation
        let result = self.perform_operation().await;
        
        // Record metrics
        self.monitoring.record_operation(
            "component.operation",
            start.elapsed(),
            result.is_ok(),
        ).await?;
        
        result
    }
}
```

#### B. Event-Based Integration
Components interact through an event system:

```rust
// In component implementation
pub async fn perform_action(&self) -> Result<()> {
    // Perform the action
    let result = self.do_action().await;
    
    // Emit monitoring event
    self.event_bus.publish(MonitoringEvent::ActionCompleted {
        action: "component.action",
        duration: self.start_time.elapsed(),
        success: result.is_ok(),
        metadata: self.collect_metadata(),
    }).await?;
    
    result
}

// In monitoring implementation
async fn handle_monitoring_event(&self, event: MonitoringEvent) {
    match event {
        MonitoringEvent::ActionCompleted {
            action, duration, success, metadata
        } => {
            self.metrics.record_action(action, duration, success).await?;
            if let Some(meta) = metadata {
                self.store_metadata(action, meta).await?;
            }
        },
        // Handle other event types
    }
}
```

#### C. Adapter Pattern
Specialized adapters for different components:

```rust
pub struct McpMonitoringAdapter {
    monitoring_service: Arc<dyn MonitoringService>,
    mcp_config: McpConfig,
}

impl McpMonitoringAdapter {
    pub fn new(
        monitoring_service: Arc<dyn MonitoringService>,
        mcp_config: McpConfig,
    ) -> Self {
        Self {
            monitoring_service,
            mcp_config,
        }
    }
    
    pub async fn record_message(&self, message: &McpMessage) -> Result<()> {
        let message_type = message.message_type();
        let size = message.serialized_size();
        
        // Record message metrics
        self.monitoring_service.record_protocol_message(
            "mcp",
            message_type,
            size,
            MessageDirection::Received,
        ).await
    }
    
    // Other MCP-specific monitoring methods
}
```

#### D. Aspect-Oriented Approach
Using macros for cross-cutting monitoring:

```rust
/// Macro to monitor function execution
#[monitor_fn(name = "component.function", tags = ["component=example", "type=operation"])]
pub async fn component_function(&self, arg1: String) -> Result<()> {
    // Function implementation
    // Monitoring is automatically added by the macro
}

// Implementation of the monitoring macro
#[proc_macro_attribute]
pub fn monitor_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse attributes
    let args = parse_macro_arguments(attr);
    
    // Parse function
    let input_fn = parse_function(item);
    
    // Generate instrumented function
    quote! {
        #[#input_fn.attrs]
        pub async fn #input_fn.ident(#input_fn.inputs) -> #input_fn.output {
            let _monitor = MonitoringSpan::new(#args.name, #args.tags);
            #input_fn.body
        }
    }.into()
}
```

## Component-Specific Integration

### 1. MCP Protocol Integration

```rust
// MCP protocol monitoring integration
pub struct McpMonitoring {
    monitoring: Arc<dyn MonitoringService>,
    protocol_version: String,
}

impl McpMonitoring {
    pub fn new(
        monitoring: Arc<dyn MonitoringService>,
        protocol_version: String,
    ) -> Self {
        Self {
            monitoring,
            protocol_version,
        }
    }
    
    // Record message processing
    pub async fn record_message_processing(
        &self,
        message_type: &str,
        direction: MessageDirection,
        size: usize,
        duration: Duration,
        status: ProcessingStatus,
    ) -> Result<()> {
        self.monitoring.record_protocol_message(
            "mcp",
            message_type,
            size,
            direction,
        ).await?;
        
        self.monitoring.record_protocol_latency(
            "mcp",
            message_type,
            duration,
        ).await?;
        
        if let ProcessingStatus::Error(code) = status {
            self.monitoring.record_protocol_error(
                "mcp",
                message_type,
                code,
            ).await?;
        }
        
        Ok(())
    }
    
    // Record connection events
    pub async fn record_connection_event(
        &self,
        event_type: ConnectionEventType,
        peer_id: &str,
        duration: Option<Duration>,
    ) -> Result<()> {
        match event_type {
            ConnectionEventType::Connected => {
                self.monitoring.record_connection_attempt(true, duration.unwrap_or_default()).await?;
                self.monitoring.increment_active_connections().await?;
            },
            ConnectionEventType::Disconnected => {
                self.monitoring.decrement_active_connections().await?;
            },
            ConnectionEventType::Failed => {
                self.monitoring.record_connection_attempt(false, duration.unwrap_or_default()).await?;
            },
        }
        
        Ok(())
    }
}
```

### 2. Command System Integration

```rust
// Command monitoring integration
pub struct CommandMonitoring {
    monitoring: Arc<dyn MonitoringService>,
}

impl CommandMonitoring {
    pub fn new(monitoring: Arc<dyn MonitoringService>) -> Self {
        Self { monitoring }
    }
    
    // Monitor command execution
    pub async fn monitor_command<T, E>(
        &self,
        command_name: &str,
        command_fn: impl Future<Output = Result<T, E>>,
    ) -> Result<T, E>
    where
        E: std::error::Error,
    {
        let start = Instant::now();
        let command_result = command_fn.await;
        let duration = start.elapsed();
        
        // Record command execution metrics
        let _ = self.monitoring.record_tool_execution(
            command_name,
            duration,
            command_result.is_ok(),
            self.estimate_memory_usage(),
        ).await;
        
        command_result
    }
    
    // Record command start/stop events
    pub async fn command_started(&self, command: &str) -> Result<CommandTracker> {
        let tracker = CommandTracker {
            command: command.to_string(),
            start_time: Instant::now(),
            monitoring: self.monitoring.clone(),
        };
        
        self.monitoring.increment_active_commands(command).await?;
        
        Ok(tracker)
    }
}

// Command tracking helper
pub struct CommandTracker {
    command: String,
    start_time: Instant,
    monitoring: Arc<dyn MonitoringService>,
}

impl Drop for CommandTracker {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        let command = self.command.clone();
        let monitoring = self.monitoring.clone();
        
        // Spawn a task to record the command completion
        // This ensures we record even if the command panics
        tokio::spawn(async move {
            let _ = monitoring.record_command_completed(&command, duration).await;
            let _ = monitoring.decrement_active_commands(&command).await;
        });
    }
}
```

### 3. Application Integration

```rust
// Application monitoring integration
pub struct AppMonitoring {
    monitoring: Arc<dyn MonitoringService>,
    app_version: String,
    start_time: Instant,
}

impl AppMonitoring {
    pub fn new(
        monitoring: Arc<dyn MonitoringService>,
        app_version: String,
    ) -> Self {
        Self {
            monitoring,
            app_version,
            start_time: Instant::now(),
        }
    }
    
    // Record application events
    pub async fn record_app_event(&self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::Started { args } => {
                self.monitoring.record_app_start(
                    &self.app_version,
                    args,
                ).await?;
            },
            AppEvent::Shutdown { reason } => {
                self.monitoring.record_app_shutdown(
                    &self.app_version,
                    reason,
                    self.start_time.elapsed(),
                ).await?;
            },
            AppEvent::ConfigChanged { settings } => {
                self.monitoring.record_config_change(settings).await?;
            },
            // Other app events
        }
        
        Ok(())
    }
    
    // Monitor resource usage
    pub async fn start_resource_monitoring(&self, interval: Duration) -> Result<()> {
        self.monitoring.start_resource_monitoring(interval).await
    }
    
    // Record component health
    pub async fn update_component_health(
        &self,
        component: &str,
        status: ComponentHealthStatus,
        message: Option<String>,
    ) -> Result<()> {
        self.monitoring.update_component_health(
            component,
            status,
            message,
        ).await
    }
}
```

## Integration Best Practices

### 1. Instrumentation Guidelines

Follow these guidelines when instrumenting components:

1. **Use Consistent Naming**:
   - Metric names: `component.operation.metric_type`
   - Example: `mcp.message.processing_time`

2. **Include Appropriate Tags**:
   - Component: `component=mcp`
   - Operation type: `operation=message_processing`
   - Status: `status=success` or `status=error`

3. **Measure at Appropriate Points**:
   - Public API boundaries
   - Long-running operations
   - Resource-intensive operations
   - Error paths

4. **Avoid Over-Instrumentation**:
   - Don't monitor every function call
   - Focus on important operations
   - Batch frequent operations

### 2. Error Handling

Handle monitoring errors properly:

```rust
// Good: Non-blocking error handling
pub async fn operation_with_monitoring(&self) -> Result<()> {
    let start = Instant::now();
    
    // Perform main operation
    let result = self.perform_operation().await;
    
    // Record metrics, but don't propagate monitoring errors
    if let Err(err) = self.monitoring.record_operation(
        "component.operation",
        start.elapsed(),
        result.is_ok(),
    ).await {
        log::warn!("Failed to record monitoring data: {}", err);
    }
    
    // Return the result of the main operation
    result
}
```

### 3. Context Propagation

Propagate monitoring context through the call stack:

```rust
pub async fn process_request(&self, request: Request, monitoring_ctx: Option<MonitoringContext>) -> Result<Response> {
    // Create or inherit monitoring context
    let ctx = monitoring_ctx.unwrap_or_else(|| MonitoringContext::new("request"));
    
    // Add operation-specific data
    let ctx = ctx.with_attribute("request_id", request.id());
    
    // First step with monitoring
    let parsed_request = ctx.instrument("parse", || self.parse_request(request)).await?;
    
    // Pass context to sub-operations
    let result = self.process_parsed_request(parsed_request, Some(ctx.clone())).await?;
    
    // Complete the monitoring context
    ctx.complete().await;
    
    Ok(result)
}
```

### 4. Configuration

Provide flexible monitoring configuration:

```rust
pub struct ComponentMonitoringConfig {
    /// Whether monitoring is enabled
    pub enabled: bool,
    /// Collection interval in seconds
    pub interval: u64,
    /// Detail level
    pub detail_level: DetailLevel,
    /// Whether to include sensitive data
    pub include_sensitive_data: bool,
    /// Custom attributes to include
    pub attributes: HashMap<String, String>,
}

pub enum DetailLevel {
    Minimal,
    Standard,
    Verbose,
}
```

## Export Formats and Integration

### 1. Prometheus Integration

```rust
pub struct PrometheusExporter {
    registry: Registry,
    metrics: HashMap<String, PrometheusMetric>,
    config: PrometheusConfig,
}

impl PrometheusExporter {
    pub fn new(config: PrometheusConfig) -> Self {
        let registry = Registry::new();
        Self {
            registry,
            metrics: HashMap::new(),
            config,
        }
    }
    
    // Register metrics
    pub fn register_metrics(&mut self) -> Result<()> {
        // Register counters
        self.register_counter("mcp_messages_total", "Total MCP messages processed")?;
        self.register_counter("mcp_errors_total", "Total MCP errors encountered")?;
        
        // Register gauges
        self.register_gauge("mcp_active_connections", "Active MCP connections")?;
        self.register_gauge("system_memory_usage_bytes", "System memory usage in bytes")?;
        
        // Register histograms
        self.register_histogram(
            "mcp_message_duration_seconds",
            "MCP message processing duration in seconds",
            vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0],
        )?;
        
        Ok(())
    }
    
    // Export metrics
    pub fn export(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
    
    // Start the exporter HTTP server
    pub async fn start_server(&self) -> Result<()> {
        let exporter = self.clone();
        let addr = format!("{}:{}", self.config.host, self.config.port).parse()?;
        
        tokio::spawn(async move {
            let metrics_handler = make_service_fn(move |_| {
                let exporter = exporter.clone();
                async move {
                    Ok::<_, hyper::Error>(service_fn(move |_| {
                        let exporter = exporter.clone();
                        async move {
                            let metrics = exporter.export().unwrap_or_else(|e| {
                                format!("Error exporting metrics: {}", e)
                            });
                            
                            Ok::<_, hyper::Error>(Response::builder()
                                .status(StatusCode::OK)
                                .header("Content-Type", "text/plain")
                                .body(Body::from(metrics))
                                .unwrap())
                        }
                    }))
                }
            });
            
            let server = Server::bind(&addr).serve(metrics_handler);
            if let Err(e) = server.await {
                log::error!("Prometheus server error: {}", e);
            }
        });
        
        Ok(())
    }
}
```

### 2. Logging Integration

```rust
pub struct LoggingExporter {
    config: LoggingConfig,
}

impl LoggingExporter {
    pub fn new(config: LoggingConfig) -> Self {
        Self { config }
    }
    
    // Export metrics to logs
    pub async fn export_to_logs(&self, metrics: &MetricsSnapshot) -> Result<()> {
        if self.config.enabled {
            let log_level = match self.config.level {
                LogLevel::Debug => log::Level::Debug,
                LogLevel::Info => log::Level::Info,
                LogLevel::Trace => log::Level::Trace,
            };
            
            let format = match self.config.format {
                LogFormat::Json => self.format_as_json(metrics)?,
                LogFormat::Text => self.format_as_text(metrics)?,
            };
            
            log::log!(log_level, "Metrics snapshot: {}", format);
        }
        
        Ok(())
    }
    
    // Format metrics as JSON
    fn format_as_json(&self, metrics: &MetricsSnapshot) -> Result<String> {
        serde_json::to_string(metrics).map_err(|e| e.into())
    }
    
    // Format metrics as human-readable text
    fn format_as_text(&self, metrics: &MetricsSnapshot) -> Result<String> {
        // Format metrics as readable text
        let mut result = String::new();
        
        result.push_str(&format!("System: CPU={}%, Mem={}MB\n", 
            metrics.system.cpu_usage,
            metrics.system.memory_usage / (1024 * 1024)));
            
        result.push_str(&format!("Protocol: Msgs={}, Errors={}, Latency={}ms\n",
            metrics.protocol.messages_processed,
            metrics.protocol.error_count,
            metrics.protocol.avg_latency.as_millis()));
            
        result.push_str(&format!("Network: RX={}, TX={}, Conns={}\n",
            format_bytes(metrics.network.bytes_received),
            format_bytes(metrics.network.bytes_transmitted),
            metrics.network.active_connections));
            
        Ok(result)
    }
}
```

### 3. External System Integration

Example integration with Grafana:

```rust
pub struct GrafanaIntegration {
    client: GrafanaClient,
    config: GrafanaConfig,
}

impl GrafanaIntegration {
    pub fn new(config: GrafanaConfig) -> Result<Self> {
        let client = GrafanaClient::new(&config.url, &config.api_key)?;
        Ok(Self { client, config })
    }
    
    // Create Grafana dashboard
    pub async fn create_dashboard(&self) -> Result<String> {
        let dashboard = self.build_dashboard()?;
        let response = self.client.create_dashboard(dashboard).await?;
        Ok(response.uid)
    }
    
    // Build dashboard configuration
    fn build_dashboard(&self) -> Result<serde_json::Value> {
        let dashboard = json!({
            "dashboard": {
                "id": null,
                "title": "Squirrel Monitoring Dashboard",
                "tags": ["squirrel", "automated"],
                "timezone": "browser",
                "schemaVersion": 16,
                "version": 1,
                "refresh": "5s",
                "panels": [
                    // System metrics panel
                    {
                        "title": "System Metrics",
                        "type": "graph",
                        "gridPos": { "x": 0, "y": 0, "w": 12, "h": 8 },
                        "targets": [
                            {
                                "expr": "system_cpu_usage",
                                "legendFormat": "CPU Usage",
                                "refId": "A"
                            },
                            {
                                "expr": "system_memory_usage_bytes",
                                "legendFormat": "Memory Usage",
                                "refId": "B"
                            }
                        ]
                    },
                    // MCP metrics panel
                    {
                        "title": "MCP Protocol",
                        "type": "graph",
                        "gridPos": { "x": 12, "y": 0, "w": 12, "h": 8 },
                        "targets": [
                            {
                                "expr": "rate(mcp_messages_total[1m])",
                                "legendFormat": "Messages/sec",
                                "refId": "A"
                            },
                            {
                                "expr": "mcp_active_connections",
                                "legendFormat": "Connections",
                                "refId": "B"
                            }
                        ]
                    },
                    // Additional panels...
                ]
            }
        });
        
        Ok(dashboard)
    }
}
```

## Migration Guide

For components integrating with the monitoring system:

1. **Basic Integration**:
   ```rust
   // Get monitoring service
   let monitoring = app.get_monitoring_service()?;
   
   // Record metrics directly
   monitoring.record_metric("my_component.metric", value).await?;
   ```

2. **Advanced Integration**:
   ```rust
   // Create a component-specific adapter
   let component_monitoring = ComponentMonitoring::new(
       app.get_monitoring_service()?,
       ComponentMonitoringConfig::default(),
   );
   
   // Use higher-level monitoring APIs
   component_monitoring.record_operation_start("operation").await?;
   // ... perform operation ...
   component_monitoring.record_operation_complete("operation", result).await?;
   ```

3. **Full Integration**:
   ```rust
   // Create a fully instrumented component
   let my_component = MyComponent::new(
       ComponentConfig::default(),
       app.get_monitoring_service()?,
       app.get_metrics_database()?,
   );
   
   // Monitoring is automatically handled within the component
   my_component.perform_operation().await?;
   ```

## Future Enhancements

1. **OpenTelemetry Integration**: Standardize on OpenTelemetry for tracing and metrics
2. **Automated Instrumentation**: Add more procedural macros for low-effort instrumentation
3. **Correlation IDs**: Add request tracing with correlation IDs across components
4. **Complex Event Processing**: Add pattern detection in monitoring data
5. **Contextual Monitoring**: Add better context propagation across async boundaries
6. **Enhanced Health Checks**: Add dependency health checking and propagation

<version>1.0.0</version> 