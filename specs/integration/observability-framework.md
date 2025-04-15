---
version: 1.0.0
last_updated: 2024-04-02
status: draft
priority: high
phase: 2
---

# Observability Framework Integration Specification

## Overview
This document specifies the Observability Framework integration requirements for the Squirrel MCP project, focusing on metrics collection, distributed tracing, structured logging, and alerting across all system components.

## Integration Status
- Current Progress: 85%
- Target Completion: Q2 2024
- Priority: High

## Observability Architecture

### 1. Metrics Collection
```rust
#[async_trait]
pub trait MetricsCollector: Send + Sync {
    fn record_counter(&self, name: &str, value: u64, labels: HashMap<String, String>);
    fn record_gauge(&self, name: &str, value: f64, labels: HashMap<String, String>);
    fn record_histogram(&self, name: &str, value: f64, labels: HashMap<String, String>);
    fn start_timer(&self, name: &str) -> Timer;
    
    async fn collect_metrics(&self) -> Result<MetricsSnapshot>;
    async fn reset_metrics(&self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub timestamp: DateTime<Utc>,
    pub counters: HashMap<String, CounterValue>,
    pub gauges: HashMap<String, GaugeValue>,
    pub histograms: HashMap<String, HistogramValue>,
}

#[derive(Debug, Clone)]
pub struct Timer {
    start_time: Instant,
    name: String,
    labels: HashMap<String, String>,
}

impl Timer {
    pub fn stop(self) -> Duration {
        let elapsed = self.start_time.elapsed();
        // Report elapsed time to metrics system
        elapsed
    }
}
```

### 2. Distributed Tracing
```rust
#[async_trait]
pub trait Tracer: Send + Sync {
    fn create_span(&self, name: &str, parent: Option<SpanContext>) -> Span;
    fn current_span(&self) -> Option<Span>;
    fn record_event(&self, event: TraceEvent);
    
    async fn export_traces(&self) -> Result<()>;
    async fn flush(&self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct Span {
    pub id: SpanId,
    pub trace_id: TraceId,
    pub parent_id: Option<SpanId>,
    pub name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub attributes: HashMap<String, Value>,
    pub events: Vec<SpanEvent>,
    pub status: SpanStatus,
}

impl Span {
    pub fn add_event(&mut self, name: &str, attributes: HashMap<String, Value>) {
        self.events.push(SpanEvent {
            name: name.to_string(),
            timestamp: Utc::now(),
            attributes,
        });
    }
    
    pub fn set_attribute(&mut self, key: &str, value: Value) {
        self.attributes.insert(key.to_string(), value);
    }
    
    pub fn end(&mut self) {
        self.end_time = Some(Utc::now());
    }
}

#[derive(Debug, Clone)]
pub enum SpanStatus {
    Unset,
    Ok,
    Error { message: String },
}
```

### 3. Structured Logging
```rust
#[async_trait]
pub trait Logger: Send + Sync {
    fn log(&self, record: LogRecord);
    fn log_with_span(&self, record: LogRecord, span: &Span);
    
    async fn flush(&self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct LogRecord {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub module_path: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub fields: HashMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[macro_export]
macro_rules! log_info {
    ($logger:expr, $message:expr $(, $key:ident = $value:expr)* $(,)?) => {{
        let mut fields = ::std::collections::HashMap::new();
        $(
            fields.insert(stringify!($key).to_string(), $value.into());
        )*
        $logger.log(LogRecord {
            timestamp: ::chrono::Utc::now(),
            level: LogLevel::Info,
            target: module_path!().to_string(),
            message: $message.to_string(),
            module_path: Some(module_path!().to_string()),
            file: Some(file!().to_string()),
            line: Some(line!()),
            fields,
        });
    }};
}
```

### 4. Alerting System
```rust
#[async_trait]
pub trait AlertManager: Send + Sync {
    async fn trigger_alert(&self, alert: Alert) -> Result<AlertId>;
    async fn resolve_alert(&self, id: AlertId) -> Result<()>;
    async fn get_active_alerts(&self) -> Result<Vec<Alert>>;
    async fn get_alert_history(&self, since: DateTime<Utc>) -> Result<Vec<AlertRecord>>;
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub id: Option<AlertId>,
    pub name: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub attributes: HashMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone)]
pub struct AlertRecord {
    pub alert: Alert,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub duration: Option<Duration>,
}
```

### 5. Health Checking
```rust
#[async_trait]
pub trait HealthChecker: Send + Sync {
    async fn check_health(&self) -> Result<HealthStatus>;
    async fn register_check(&self, check: Box<dyn HealthCheck>) -> Result<()>;
    async fn unregister_check(&self, name: &str) -> Result<()>;
}

#[async_trait]
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self) -> Result<HealthCheckResult>;
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub status: SystemStatus,
    pub checks: HashMap<String, HealthCheckResult>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemStatus {
    Up,
    Degraded,
    Down,
}

#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub status: ComponentStatus,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentStatus {
    Up,
    Degraded,
    Down,
}
```

## Integration Requirements

### 1. Component Observability Integration
- Each component must implement metrics collection
- Critical components must implement distributed tracing
- All components must use structured logging
- System-wide health checks must be implemented
- Alerts must be triggered for critical errors

### 2. Configuration Requirements
- Dynamic log level configuration
- Component-specific metrics configuration
- Environment-specific observability settings
- Sampling configuration for tracing
- Alert threshold configuration

### 3. Security Requirements
- Secure metric collection
- PII-aware logging filters
- Access control for metrics endpoints
- Encryption for sensitive observability data
- Audit logging for security-sensitive operations

## Implementation Examples

### 1. Metrics Collection Implementation
```rust
impl MetricsCollector for PrometheusMetricsCollector {
    fn record_counter(&self, name: &str, value: u64, labels: HashMap<String, String>) {
        let counter = self
            .registry
            .get_counter_vec(&MetricId::new(name))
            .unwrap_or_else(|| {
                self.registry.register_counter_vec(
                    &MetricId::new(name),
                    name,
                    &format!("{} counter", name),
                    labels.keys().map(|k| k.as_str()).collect(),
                )
            });
            
        let label_values: Vec<&str> = labels.values().map(|v| v.as_str()).collect();
        counter.with_label_values(&label_values).inc_by(value);
    }
    
    fn start_timer(&self, name: &str) -> Timer {
        Timer {
            start_time: Instant::now(),
            name: name.to_string(),
            labels: HashMap::new(),
        }
    }
    
    async fn collect_metrics(&self) -> Result<MetricsSnapshot> {
        let families = self.registry.gather();
        
        // Transform metrics data into snapshot format
        let mut snapshot = MetricsSnapshot {
            timestamp: Utc::now(),
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
        };
        
        for family in families {
            for metric in family.get_metric() {
                // Process metrics and add to snapshot
                // ...
            }
        }
        
        Ok(snapshot)
    }
}
```

### 2. Distributed Tracing Implementation
```rust
impl Tracer for OpenTelemetryTracer {
    fn create_span(&self, name: &str, parent: Option<SpanContext>) -> Span {
        let context = self.tracer
            .span_builder(name)
            .with_parent(parent)
            .start(&self.tracer);
            
        Span {
            id: SpanId::from(context.span_context().span_id().to_bytes()),
            trace_id: TraceId::from(context.span_context().trace_id().to_bytes()),
            parent_id: parent.map(|p| SpanId::from(p.span_id().to_bytes())),
            name: name.to_string(),
            start_time: Utc::now(),
            end_time: None,
            attributes: HashMap::new(),
            events: Vec::new(),
            status: SpanStatus::Unset,
        }
    }
    
    fn current_span(&self) -> Option<Span> {
        let current = self.tracer.current_span();
        if current.is_valid() {
            // Convert current span to our Span type
            // ...
            Some(/* converted span */)
        } else {
            None
        }
    }
    
    async fn export_traces(&self) -> Result<()> {
        // Trigger export of traces
        self.exporter.export().await?;
        Ok(())
    }
}
```

## Testing Strategy

### 1. Metrics Testing
```rust
#[tokio::test]
async fn test_metrics_collection() {
    let metrics = PrometheusMetricsCollector::new();
    
    // Test counter recording
    metrics.record_counter(
        "test_counter",
        42,
        HashMap::from([("label1".to_string(), "value1".to_string())]),
    );
    
    // Test gauge recording
    metrics.record_gauge(
        "test_gauge",
        3.14,
        HashMap::from([("label1".to_string(), "value1".to_string())]),
    );
    
    // Test timer
    let timer = metrics.start_timer("test_timer");
    tokio::time::sleep(Duration::from_millis(10)).await;
    let elapsed = timer.stop();
    assert!(elapsed >= Duration::from_millis(10));
    
    // Test metrics snapshot
    let snapshot = metrics.collect_metrics().await.unwrap();
    assert!(snapshot.counters.contains_key("test_counter"));
    assert!(snapshot.gauges.contains_key("test_gauge"));
    assert!(snapshot.histograms.contains_key("test_timer"));
}
```

### 2. Distributed Tracing Testing
```rust
#[tokio::test]
async fn test_distributed_tracing() {
    let tracer = OpenTelemetryTracer::new();
    
    // Create root span
    let mut root_span = tracer.create_span("root", None);
    root_span.set_attribute("service.name", "test-service".into());
    
    // Create child span
    let parent_context = SpanContext::new(root_span.trace_id, root_span.id, TraceFlags::SAMPLED);
    let mut child_span = tracer.create_span("child", Some(parent_context));
    child_span.set_attribute("operation", "test-operation".into());
    
    // Add events to spans
    child_span.add_event("processing", HashMap::from([("item_count".to_string(), 42.into())]));
    
    // End spans
    child_span.end();
    root_span.end();
    
    // Export traces
    tracer.export_traces().await.unwrap();
    
    // Verify export was called (implementation-dependent)
    assert!(tracer.exporter.was_called());
}
```

## Component Integration

### 1. MCP Protocol Observability Integration
- Collect metrics for message throughput, latency, and error rates
- Implement tracing for message flow across components
- Add structured logging for protocol operations
- Create health checks for protocol services
- Define alerts for protocol errors and performance degradation

### 2. Context Management Observability
- Track metrics for context operations and state size
- Add distributed tracing for context lifecycle
- Implement structured logging for state changes
- Add health checks for persistence stores
- Define alerts for context corruption or loss

### 3. Tool Management Observability
- Collect metrics for tool execution time and resource usage
- Add tracing for tool lifecycle and dependencies
- Implement logging for tool operations and errors
- Create health checks for tool registry and executors
- Define alerts for failing tools and resource exhaustion

## Visualization and Dashboards

### 1. Metrics Dashboard
- Real-time performance metrics
- Historical trends
- Component-specific views
- Resource utilization graphs
- Error rate visualization

### 2. Trace Visualization
- End-to-end request flow
- Component dependency graphs
- Latency distribution
- Error paths
- Bottleneck identification

### 3. Logs Dashboard
- Real-time log streaming
- Log filtering and search
- Context-aware log grouping
- Error aggregation
- Pattern detection

### 4. Health Dashboard
- System health status
- Component health indicators
- Historical uptime
- Incident timeline
- Alert visualization

## Alerting Configuration

### 1. Alert Rules
```yaml
alerts:
  - name: high_error_rate
    description: Error rate exceeds threshold
    query: 'rate(errors_total[5m]) / rate(requests_total[5m]) > 0.05'
    severity: warning
    for: 5m
    labels:
      team: core
    annotations:
      summary: High error rate detected
      dashboard: https://grafana.example.com/d/errors
      
  - name: service_down
    description: Service is not responding
    query: 'up{service="mcp"} == 0'
    severity: critical
    for: 1m
    labels:
      team: core
    annotations:
      summary: MCP service is down
      runbook: https://wiki.example.com/runbooks/service_down
```

### 2. Alert Notifications
```yaml
notifications:
  - name: slack_core_team
    type: slack
    channel: '#core-alerts'
    triggers:
      - warning
      - critical
    
  - name: email_ops
    type: email
    recipients:
      - ops@example.com
    triggers:
      - critical
    
  - name: pagerduty
    type: pagerduty
    service_key: '1234567890'
    triggers:
      - critical
```

## Migration Guide

### 1. Implementation Phases
1. Add structured logging to all components
2. Implement basic metrics collection
3. Add health checks for critical services
4. Implement distributed tracing
5. Set up alerting system
6. Create visualization dashboards
7. Implement correlation between logs, metrics, and traces

### 2. Breaking Changes
- Log format changes
- Metric naming standardization
- Trace context propagation requirements
- Component health check interfaces

### 3. Migration Steps
1. Update logging implementation to use structured format
2. Add metrics collection to critical components
3. Implement health checks for core services
4. Add trace context propagation to inter-component communication
5. Set up visualization and alerting infrastructure
6. Train team on observability tools and practices

## Version Control

This specification is version controlled alongside the codebase. Updates are tagged with corresponding software releases.

---

Last Updated: 2024-04-02
Version: 1.0.0 