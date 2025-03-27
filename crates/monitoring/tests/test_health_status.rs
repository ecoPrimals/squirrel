use squirrel_monitoring::health::status::Status;
use squirrel_monitoring::health::HealthStatus;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::result::Result;
use rand::Rng;
use chrono::Utc;
use tokio::sync::mpsc;
use tokio::time;

/// Health status generator - simplified version for testing
pub struct HealthStatusGenerator {
    /// Component health states
    component_states: HashMap<String, Status>,
    /// Random number generator
    rng: Arc<Mutex<rand::rngs::ThreadRng>>,
}

// Implement Send and Sync for HealthStatusGenerator
unsafe impl Send for HealthStatusGenerator {}
unsafe impl Sync for HealthStatusGenerator {}

impl HealthStatusGenerator {
    /// Create a new health status generator
    pub fn new() -> Self {
        // Initial component health states
        let mut component_states = HashMap::new();
        component_states.insert("api_server".to_string(), Status::Healthy);
        component_states.insert("database".to_string(), Status::Healthy);
        component_states.insert("cache_service".to_string(), Status::Healthy);
        component_states.insert("metrics_collector".to_string(), Status::Healthy);
        component_states.insert("notification_service".to_string(), Status::Healthy);
        
        Self {
            component_states,
            rng: Arc::new(Mutex::new(rand::thread_rng())),
        }
    }
    
    /// Generate the next health status for each component
    pub fn next_health_status(&mut self) -> HashMap<String, HealthStatus> {
        let mut result = HashMap::new();
        
        // Create health status for each component
        for (component, status) in &self.component_states {
            // Create a HealthStatus object
            let health = HealthStatus {
                service: component.clone(),
                status: *status,
                message: format!("Component {} is {:?}", component, status),
                timestamp: Utc::now(),
            };
            
            result.insert(component.clone(), health);
        }
        
        result
    }
    
    /// Randomly change some components' status for testing
    pub fn randomize_statuses(&mut self) {
        let mut rng = self.rng.lock().unwrap();
        
        for (_, status) in self.component_states.iter_mut() {
            if rng.gen_bool(0.2) { // 20% chance to change status
                *status = match rng.gen_range(0..5) {
                    0 => Status::Healthy,
                    1 => Status::Warning,
                    2 => Status::Degraded,
                    3 => Status::Critical,
                    _ => Status::Unhealthy,
                };
            }
        }
    }
    
    /// Generate a series of health statuses for testing
    pub fn generate_series(&mut self, count: usize) -> Vec<HashMap<String, HealthStatus>> {
        let mut result = Vec::with_capacity(count);
        
        for _ in 0..count {
            self.randomize_statuses();
            result.push(self.next_health_status());
        }
        
        result
    }
}

#[tokio::test]
async fn test_health_status_generator_count() {
    let mut generator = HealthStatusGenerator::new();
    
    // Generate health status
    let health_status = generator.next_health_status();
    
    // Print debugging information
    println!("Health status size: {}", health_status.len());
    println!("Health status keys: {:?}", health_status.keys().collect::<Vec<_>>());
    for (key, value) in &health_status {
        println!("Key: {}, Status: {:?}", key, value.status);
    }
    
    // Verify we have exactly 5 components
    assert_eq!(health_status.len(), 5, "Expected exactly 5 components");
    
    // Verify we have the expected components
    assert!(health_status.contains_key("api_server"), "Missing api_server component");
    assert!(health_status.contains_key("database"), "Missing database component");
    assert!(health_status.contains_key("cache_service"), "Missing cache_service component");
    assert!(health_status.contains_key("metrics_collector"), "Missing metrics_collector component");
    assert!(health_status.contains_key("notification_service"), "Missing notification_service component");
}

#[tokio::test]
async fn test_health_status_generator_randomization() {
    let mut generator = HealthStatusGenerator::new();
    
    // Generate a series of 10 health status updates
    let status_series = generator.generate_series(10);
    
    // Verify we have the expected number of status updates
    assert_eq!(status_series.len(), 10, "Expected 10 status updates");
    
    // Verify that at least some statuses have changed (it's random, but very likely with 10 iterations)
    let mut has_changes = false;
    let first_status = &status_series[0];
    
    for status_update in &status_series[1..] {
        for (component, health) in status_update {
            if let Some(first_health) = first_status.get(component) {
                if health.status != first_health.status {
                    has_changes = true;
                    break;
                }
            }
        }
        if has_changes {
            break;
        }
    }
    
    assert!(has_changes, "Expected at least some status changes in randomization");
} 