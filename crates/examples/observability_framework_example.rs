//! Observability Framework Example
//!
//! This example demonstrates the capabilities of the Observability Framework,
//! including metrics collection, distributed tracing, structured logging,
//! health checking, and alerting.

use std::collections::HashMap;
use std::error::Error;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::info;
use std::sync::Arc;

use squirrel_mcp::observability::{self, ObservabilityFramework, ObservabilityResult};
use squirrel_mcp::observability::metrics::{Counter, Gauge, Histogram};
use squirrel_mcp::observability::health::{HealthCheckResult, HealthStatus};
use squirrel_mcp::observability::alerting::{AlertSeverity, AlertType};
use squirrel_mcp::observability::logging::LogContext;

/// Simulate a component that processes requests
struct RequestProcessor {
    name: String,
    framework: ObservabilityFramework,
    request_counter: Arc<Counter>,
    active_requests: Arc<Gauge>,
    request_duration: Arc<Histogram>,
}

impl RequestProcessor {
    /// Create a new request processor
    fn new(name: impl Into<String>, framework: ObservabilityFramework) -> ObservabilityResult<Self> {
        let name_str = name.into();
        let name_clone = name_str.clone();
        
        // Register metrics
        let request_counter = framework.metrics.create_counter(
            format!("{}_requests_total", name_str),
            format!("Total number of requests processed by {}", name_str),
            Some("requests".to_string()),
            HashMap::new(),
        )?;
        
        let active_requests = framework.metrics.create_gauge(
            format!("{}_active_requests", name_str),
            format!("Number of requests currently being processed by {}", name_str),
            Some("requests".to_string()),
            HashMap::new(),
        )?;
        
        // Create histogram with bucket boundaries appropriate for request duration in ms
        let request_duration = framework.metrics.create_histogram(
            format!("{}_request_duration", name_str),
            format!("Duration of requests processed by {}", name_str),
            Some("milliseconds".to_string()),
            HashMap::new(),
            vec![5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0],
        )?;
        
        // Register health check
        framework.health_checker.register_check_fn(
            &name_str,
            "health_check",
            "Health check for the request processor",
            move || HealthCheckResult::healthy_with_message(format!("{} is healthy", name_clone))
        )?;
        
        Ok(Self {
            name: name_str,
            framework,
            request_counter,
            active_requests,
            request_duration,
        })
    }
    
    /// Process a request
    async fn process_request(&self, request_id: &str, duration_ms: u64) -> ObservabilityResult<()> {
        // Create a trace span for this request
        let span = self.framework.tracer.start_span(
            format!("{}_process_request", self.name)
        )?;
        
        // Get the span for adding attributes
        {
            let mut guard = span.lock().map_err(|e| {
                observability::ObservabilityError::TracingError(
                    format!("Failed to lock span: {}", e)
                )
            })?;
            
            guard.add_attribute("request_id", request_id);
            guard.add_attribute("component", &self.name);
        }
        
        // Create a logging context
        let _context = LogContext::new()
            .with_field("request_id", request_id)
            .with_field("component", &self.name);
        
        // Log request start
        self.framework.tracer.initialize()?;
        self.framework.metrics.initialize()?;
        
        // Update metrics
        self.request_counter.inc_one()?;
        self.active_requests.inc(1.0)?;
        
        // Log start of processing
        let start_time = Instant::now();
        self.framework.tracer.initialize()?;
        
        // Simulate request processing
        sleep(Duration::from_millis(duration_ms)).await;
        
        // Log and record completion
        let elapsed = start_time.elapsed().as_millis() as f64;
        self.request_duration.observe(elapsed)?;
        self.active_requests.dec(1.0)?;
        
        // Log completion
        self.framework.tracer.initialize()?;
        
        // If the request took longer than expected, create an alert
        if elapsed > 100.0 {
            let alert = self.framework.alert_manager.alert(
                &self.name,
                format!("Slow request processing in {}", self.name),
                format!("Request {} took {}ms to process (threshold: 100ms)", request_id, elapsed),
                AlertSeverity::Warning,
                AlertType::Performance,
            )?;
            
            info!("Created performance alert: {}", alert.id());
        }
        
        Ok(())
    }
    
    /// Simulate a health status change
    fn simulate_health_status_change(&self, new_status: HealthStatus) -> ObservabilityResult<()> {
        // Create a health status alert
        self.framework.alert_manager.create_alert(
            "health_status_change",
            &format!("Health status change for {}", self.name),
            AlertSeverity::Warning,
            Some(&format!("Health status changed to {:?}", new_status)),
            Some(&self.name),
            None
        )?;
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Create and initialize the observability framework
    info!("Initializing Observability Framework");
    let framework = observability::initialize().await?;
    
    // Create a request processor
    let processor = RequestProcessor::new("example_processor", framework.clone())?;
    
    // Process some requests
    info!("Processing requests");
    for i in 1..=5 {
        let request_id = format!("req-{}", i);
        let duration = if i == 3 { 150 } else { 50 }; // Make one request slow
        
        info!("Processing request {}", request_id);
        processor.process_request(&request_id, duration).await?;
    }
    
    // Simulate a health status change
    info!("Simulating health status change");
    processor.simulate_health_status_change(HealthStatus::Degraded)?;
    
    // Get and display metrics
    info!("Metrics collected:");
    if let Some(counter) = framework.metrics.get_counter(&format!("{}_requests_total", processor.name))? {
        info!("  Requests processed: {}", counter.value()?);
    }
    
    if let Some(gauge) = framework.metrics.get_gauge(&format!("{}_active_requests", processor.name))? {
        info!("  Active requests: {}", gauge.value()?);
    }
    
    if let Some(histogram) = framework.metrics.get_histogram(&format!("{}_request_duration", processor.name))? {
        info!("  Request duration (avg): {}ms", histogram.sum()? / histogram.count()? as f64);
        info!("  Request count: {}", histogram.count()?);
    }
    
    // Get active alerts
    let alerts = framework.alert_manager.get_alerts(
        None,
        None,
        None,
        Some(observability::alerting::AlertState::Active),
    )?;
    
    info!("Active alerts:");
    for alert in alerts {
        info!("  [{}] {}: {}", 
            alert.severity(), 
            alert.source(), 
            alert.summary()
        );
    }
    
    info!("Observability Framework example completed");
    Ok(())
} 