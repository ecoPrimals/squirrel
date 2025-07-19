//! Fault Injection System
//!
//! Provides systematic fault injection capabilities for chaos engineering experiments.
//! Supports network failures, resource exhaustion, service unavailability, and more.

use super::{FaultType, NetworkErrorType, ResourceType, ChaosError};
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use tokio::time::{sleep, interval};
use uuid::Uuid;

/// Fault injection orchestrator
#[derive(Debug)]
pub struct FaultInjector {
    /// Active fault injections by handle
    active_faults: Arc<RwLock<HashMap<String, ActiveFault>>>,
    /// Network fault controller
    network_faults: Arc<NetworkFaultController>,
    /// Resource exhaustion controller  
    resource_faults: Arc<ResourceFaultController>,
    /// Service unavailability controller
    service_faults: Arc<ServiceFaultController>,
    /// Memory pressure controller
    memory_faults: Arc<MemoryPressureController>,
    /// CPU starvation controller
    cpu_faults: Arc<CpuStarvationController>,
    /// Disk I/O fault controller
    disk_faults: Arc<DiskIoFaultController>,
}

/// Active fault injection tracking
#[derive(Debug)]
pub struct ActiveFault {
    /// Fault handle for identification
    pub handle: String,
    /// Type of fault being injected
    pub fault_type: FaultType,
    /// Start time of injection
    pub start_time: Instant,
    /// Cancellation token
    pub cancel_token: Arc<AtomicBool>,
    /// Fault-specific controller handle
    pub controller_handle: String,
}

/// Network fault injection controller
#[derive(Debug)]
pub struct NetworkFaultController {
    /// Active network faults
    active_faults: Arc<RwLock<HashMap<String, NetworkFaultInstance>>>,
    /// Request interceptor
    interceptor: Arc<NetworkRequestInterceptor>,
}

/// Network fault instance
#[derive(Debug)]
pub struct NetworkFaultInstance {
    pub handle: String,
    pub error_type: NetworkErrorType,
    pub failure_rate: f64,
    pub latency_ms: Option<u64>,
    pub cancel_token: Arc<AtomicBool>,
    pub requests_processed: Arc<AtomicU64>,
    pub requests_failed: Arc<AtomicU64>,
}

/// Network request interceptor for fault injection
#[derive(Debug)]
pub struct NetworkRequestInterceptor {
    /// Active intercepts
    intercepts: Arc<RwLock<HashMap<String, NetworkFaultInstance>>>,
}

/// Resource exhaustion controller
#[derive(Debug)]
pub struct ResourceFaultController {
    /// Active resource faults
    active_faults: Arc<RwLock<HashMap<String, ResourceFaultInstance>>>,
}

/// Resource fault instance
#[derive(Debug)]
pub struct ResourceFaultInstance {
    pub handle: String,
    pub resource_type: ResourceType,
    pub exhaustion_level: f64,
    pub duration: Duration,
    pub cancel_token: Arc<AtomicBool>,
    pub start_time: Instant,
    /// Resource-specific data
    pub resource_data: ResourceData,
}

/// Resource-specific data for fault injection
#[derive(Debug)]
pub enum ResourceData {
    Memory {
        allocated_bytes: Arc<AtomicU64>,
        allocation_handles: Arc<Mutex<Vec<Vec<u8>>>>,
    },
    Cpu {
        worker_threads: Arc<AtomicU64>,
        cpu_percentage: f64,
    },
    NetworkBandwidth {
        bandwidth_limit_mbps: u64,
        current_usage: Arc<AtomicU64>,
    },
    FileDescriptors {
        handles_to_hold: Arc<Mutex<Vec<std::fs::File>>>,
        target_count: u32,
    },
    DatabaseConnections {
        held_connections: Arc<AtomicU64>,
        max_connections: u32,
    },
    ThreadPool {
        held_threads: Arc<AtomicU64>,
        pool_size: u32,
    },
}

/// Service unavailability controller
#[derive(Debug)]
pub struct ServiceFaultController {
    /// Active service faults
    active_faults: Arc<RwLock<HashMap<String, ServiceFaultInstance>>>,
}

/// Service fault instance
#[derive(Debug)]
pub struct ServiceFaultInstance {
    pub handle: String,
    pub service_name: String,
    pub duration: Duration,
    pub error_response: Option<String>,
    pub cancel_token: Arc<AtomicBool>,
    pub start_time: Instant,
    pub requests_blocked: Arc<AtomicU64>,
}

/// Memory pressure controller
#[derive(Debug)]  
pub struct MemoryPressureController {
    /// Active memory pressure instances
    active_faults: Arc<RwLock<HashMap<String, MemoryPressureInstance>>>,
}

/// Memory pressure fault instance
#[derive(Debug)]
pub struct MemoryPressureInstance {
    pub handle: String,
    pub allocation_mb: u64,
    pub duration: Duration,
    pub gradual: bool,
    pub cancel_token: Arc<AtomicBool>,
    pub allocated_memory: Arc<Mutex<Vec<Vec<u8>>>>,
    pub current_allocation_mb: Arc<AtomicU64>,
}

/// CPU starvation controller
#[derive(Debug)]
pub struct CpuStarvationController {
    /// Active CPU starvation instances
    active_faults: Arc<RwLock<HashMap<String, CpuStarvationInstance>>>,
}

/// CPU starvation fault instance
#[derive(Debug)]
pub struct CpuStarvationInstance {
    pub handle: String,
    pub cpu_percentage: f64,
    pub duration: Duration,
    pub thread_count: usize,
    pub cancel_token: Arc<AtomicBool>,
    pub worker_handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

/// Disk I/O fault controller
#[derive(Debug)]
pub struct DiskIoFaultController {
    /// Active disk I/O faults
    active_faults: Arc<RwLock<HashMap<String, DiskIoFaultInstance>>>,
}

/// Disk I/O fault instance
#[derive(Debug)]
pub struct DiskIoFaultInstance {
    pub handle: String,
    pub failure_rate: f64,
    pub latency_ms: Option<u64>,
    pub target_paths: Vec<String>,
    pub cancel_token: Arc<AtomicBool>,
    pub operations_processed: Arc<AtomicU64>,
    pub operations_failed: Arc<AtomicU64>,
}

impl FaultInjector {
    /// Create a new fault injector
    pub fn new() -> Self {
        Self {
            active_faults: Arc::new(RwLock::new(HashMap::new())),
            network_faults: Arc::new(NetworkFaultController::new()),
            resource_faults: Arc::new(ResourceFaultController::new()),
            service_faults: Arc::new(ServiceFaultController::new()),
            memory_faults: Arc::new(MemoryPressureController::new()),
            cpu_faults: Arc::new(CpuStarvationController::new()),
            disk_faults: Arc::new(DiskIoFaultController::new()),
        }
    }

    /// Inject a fault into the system
    pub async fn inject_fault(&self, fault_type: FaultType) -> Result<String, ChaosError> {
        let handle = Uuid::new_v4().to_string();
        let cancel_token = Arc::new(AtomicBool::new(false));

        let controller_handle = match &fault_type {
            FaultType::NetworkFailure { rate, latency_ms, error_type } => {
                self.network_faults.inject_network_fault(
                    &handle,
                    error_type.clone(),
                    *rate,
                    *latency_ms,
                    cancel_token.clone(),
                ).await?
            }
            FaultType::ResourceExhaustion { resource, level, duration } => {
                self.resource_faults.inject_resource_fault(
                    &handle,
                    resource.clone(),
                    *level,
                    *duration,
                    cancel_token.clone(),
                ).await?
            }
            FaultType::ServiceUnavailable { service_name, duration, error_response } => {
                self.service_faults.inject_service_fault(
                    &handle,
                    service_name.clone(),
                    *duration,
                    error_response.clone(),
                    cancel_token.clone(),
                ).await?
            }
            FaultType::MemoryPressure { allocation_mb, duration, gradual } => {
                self.memory_faults.inject_memory_pressure(
                    &handle,
                    *allocation_mb,
                    *duration,
                    *gradual,
                    cancel_token.clone(),
                ).await?
            }
            FaultType::CpuStarvation { cpu_percentage, duration, threads } => {
                self.cpu_faults.inject_cpu_starvation(
                    &handle,
                    *cpu_percentage,
                    *duration,
                    *threads,
                    cancel_token.clone(),
                ).await?
            }
            FaultType::DiskIoFailure { failure_rate, latency_ms, target_paths } => {
                self.disk_faults.inject_disk_io_fault(
                    &handle,
                    *failure_rate,
                    *latency_ms,
                    target_paths.clone(),
                    cancel_token.clone(),
                ).await?
            }
        };

        let active_fault = ActiveFault {
            handle: handle.clone(),
            fault_type,
            start_time: Instant::now(),
            cancel_token,
            controller_handle,
        };

        {
            let mut faults = self.active_faults.write().await;
            faults.insert(handle.clone(), active_fault);
        }

        Ok(handle)
    }

    /// Stop a specific fault injection
    pub async fn stop_fault(&self, handle: &str) -> Result<(), ChaosError> {
        let fault = {
            let mut faults = self.active_faults.write().await;
            faults.remove(handle)
        };

        if let Some(fault) = fault {
            // Signal cancellation
            fault.cancel_token.store(true, Ordering::SeqCst);

            // Stop specific controller
            match fault.fault_type {
                FaultType::NetworkFailure { .. } => {
                    self.network_faults.stop_network_fault(&fault.controller_handle).await?;
                }
                FaultType::ResourceExhaustion { .. } => {
                    self.resource_faults.stop_resource_fault(&fault.controller_handle).await?;
                }
                FaultType::ServiceUnavailable { .. } => {
                    self.service_faults.stop_service_fault(&fault.controller_handle).await?;
                }
                FaultType::MemoryPressure { .. } => {
                    self.memory_faults.stop_memory_pressure(&fault.controller_handle).await?;
                }
                FaultType::CpuStarvation { .. } => {
                    self.cpu_faults.stop_cpu_starvation(&fault.controller_handle).await?;
                }
                FaultType::DiskIoFailure { .. } => {
                    self.disk_faults.stop_disk_io_fault(&fault.controller_handle).await?;
                }
            }

            Ok(())
        } else {
            Err(ChaosError::FaultInjectionError(
                format!("Fault handle not found: {}", handle)
            ))
        }
    }

    /// Get statistics for active faults
    pub async fn get_fault_statistics(&self) -> HashMap<String, FaultStatistics> {
        let faults = self.active_faults.read().await;
        let mut stats = HashMap::new();

        for (handle, fault) in faults.iter() {
            let duration = fault.start_time.elapsed();
            
            let fault_stats = match &fault.fault_type {
                FaultType::NetworkFailure { rate, .. } => {
                    if let Ok(network_stats) = self.network_faults.get_statistics(&fault.controller_handle).await {
                        FaultStatistics {
                            fault_type: "network_failure".to_string(),
                            duration,
                            operations_processed: network_stats.requests_processed,
                            operations_affected: network_stats.requests_failed,
                            effectiveness_rate: *rate,
                            additional_metrics: network_stats.additional_metrics,
                        }
                    } else {
                        FaultStatistics::default()
                    }
                }
                FaultType::MemoryPressure { allocation_mb, .. } => {
                    if let Ok(memory_stats) = self.memory_faults.get_statistics(&fault.controller_handle).await {
                        FaultStatistics {
                            fault_type: "memory_pressure".to_string(),
                            duration,
                            operations_processed: 1,
                            operations_affected: 1,
                            effectiveness_rate: memory_stats.current_allocation as f64 / *allocation_mb as f64,
                            additional_metrics: memory_stats.additional_metrics,
                        }
                    } else {
                        FaultStatistics::default()
                    }
                }
                // ... other fault types
                _ => FaultStatistics::default(),
            };

            stats.insert(handle.clone(), fault_stats);
        }

        stats
    }

    /// Stop all active fault injections
    pub async fn stop_all_faults(&self) -> Result<(), ChaosError> {
        let handles: Vec<String> = {
            let faults = self.active_faults.read().await;
            faults.keys().cloned().collect()
        };

        for handle in handles {
            self.stop_fault(&handle).await?;
        }

        Ok(())
    }
}

/// Fault injection statistics
#[derive(Debug, Clone)]
pub struct FaultStatistics {
    pub fault_type: String,
    pub duration: Duration,
    pub operations_processed: u64,
    pub operations_affected: u64,
    pub effectiveness_rate: f64,
    pub additional_metrics: HashMap<String, f64>,
}

impl Default for FaultStatistics {
    fn default() -> Self {
        Self {
            fault_type: "unknown".to_string(),
            duration: Duration::from_secs(0),
            operations_processed: 0,
            operations_affected: 0,
            effectiveness_rate: 0.0,
            additional_metrics: HashMap::new(),
        }
    }
}

// Network fault controller implementation
impl NetworkFaultController {
    pub fn new() -> Self {
        Self {
            active_faults: Arc::new(RwLock::new(HashMap::new())),
            interceptor: Arc::new(NetworkRequestInterceptor::new()),
        }
    }

    pub async fn inject_network_fault(
        &self,
        handle: &str,
        error_type: NetworkErrorType,
        failure_rate: f64,
        latency_ms: Option<u64>,
        cancel_token: Arc<AtomicBool>,
    ) -> Result<String, ChaosError> {
        let instance = NetworkFaultInstance {
            handle: handle.to_string(),
            error_type,
            failure_rate,
            latency_ms,
            cancel_token: cancel_token.clone(),
            requests_processed: Arc::new(AtomicU64::new(0)),
            requests_failed: Arc::new(AtomicU64::new(0)),
        };

        {
            let mut faults = self.active_faults.write().await;
            faults.insert(handle.to_string(), instance);
        }

        // Start fault injection task
        let interceptor = self.interceptor.clone();
        let handle_clone = handle.to_string();
        tokio::spawn(async move {
            interceptor.start_interception(handle_clone, cancel_token).await;
        });

        Ok(handle.to_string())
    }

    pub async fn stop_network_fault(&self, handle: &str) -> Result<(), ChaosError> {
        let mut faults = self.active_faults.write().await;
        if let Some(fault) = faults.remove(handle) {
            fault.cancel_token.store(true, Ordering::SeqCst);
            Ok(())
        } else {
            Err(ChaosError::FaultInjectionError(
                format!("Network fault not found: {}", handle)
            ))
        }
    }

    pub async fn get_statistics(&self, handle: &str) -> Result<NetworkFaultStatistics, ChaosError> {
        let faults = self.active_faults.read().await;
        if let Some(fault) = faults.get(handle) {
            Ok(NetworkFaultStatistics {
                requests_processed: fault.requests_processed.load(Ordering::SeqCst),
                requests_failed: fault.requests_failed.load(Ordering::SeqCst),
                additional_metrics: HashMap::new(),
            })
        } else {
            Err(ChaosError::FaultInjectionError(
                format!("Network fault not found: {}", handle)
            ))
        }
    }
}

#[derive(Debug)]
pub struct NetworkFaultStatistics {
    pub requests_processed: u64,
    pub requests_failed: u64,
    pub additional_metrics: HashMap<String, f64>,
}

impl NetworkRequestInterceptor {
    pub fn new() -> Self {
        Self {
            intercepts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_interception(&self, handle: String, cancel_token: Arc<AtomicBool>) {
        while !cancel_token.load(Ordering::SeqCst) {
            // Simulate network request interception
            // In a real implementation, this would integrate with the HTTP client
            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Intercept and potentially fail a network request
    pub async fn intercept_request(&self, _url: &str) -> Result<(), NetworkFaultError> {
        let intercepts = self.intercepts.read().await;
        
        for fault in intercepts.values() {
            fault.requests_processed.fetch_add(1, Ordering::SeqCst);
            
            // Check if we should inject a failure
            if rand::random::<f64>() < fault.failure_rate {
                fault.requests_failed.fetch_add(1, Ordering::SeqCst);
                
                // Inject latency if configured
                if let Some(latency) = fault.latency_ms {
                    sleep(Duration::from_millis(latency)).await;
                }
                
                // Return appropriate error type
                return Err(match fault.error_type {
                    NetworkErrorType::Timeout => NetworkFaultError::Timeout,
                    NetworkErrorType::ConnectionRefused => NetworkFaultError::ConnectionRefused,
                    NetworkErrorType::DnsFailure => NetworkFaultError::DnsFailure,
                    NetworkErrorType::ServerError => NetworkFaultError::ServerError,
                    NetworkErrorType::ServiceUnavailable => NetworkFaultError::ServiceUnavailable,
                    NetworkErrorType::PartialResponse => NetworkFaultError::PartialResponse,
                    NetworkErrorType::TlsFailure => NetworkFaultError::TlsFailure,
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NetworkFaultError {
    #[error("Network timeout")]
    Timeout,
    #[error("Connection refused")]
    ConnectionRefused,
    #[error("DNS resolution failure")]
    DnsFailure,
    #[error("Server error")]
    ServerError,
    #[error("Service unavailable")]
    ServiceUnavailable,
    #[error("Partial response received")]
    PartialResponse,
    #[error("TLS handshake failure")]
    TlsFailure,
}

// Memory pressure controller implementation
impl MemoryPressureController {
    pub fn new() -> Self {
        Self {
            active_faults: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn inject_memory_pressure(
        &self,
        handle: &str,
        allocation_mb: u64,
        duration: Duration,
        gradual: bool,
        cancel_token: Arc<AtomicBool>,
    ) -> Result<String, ChaosError> {
        let instance = MemoryPressureInstance {
            handle: handle.to_string(),
            allocation_mb,
            duration,
            gradual,
            cancel_token: cancel_token.clone(),
            allocated_memory: Arc::new(Mutex::new(Vec::new())),
            current_allocation_mb: Arc::new(AtomicU64::new(0)),
        };

        {
            let mut faults = self.active_faults.write().await;
            faults.insert(handle.to_string(), instance);
        }

        // Start memory allocation task
        let faults_ref = self.active_faults.clone();
        let handle_clone = handle.to_string();
        tokio::spawn(async move {
            Self::allocate_memory_task(faults_ref, handle_clone, cancel_token).await;
        });

        Ok(handle.to_string())
    }

    async fn allocate_memory_task(
        faults: Arc<RwLock<HashMap<String, MemoryPressureInstance>>>,
        handle: String,
        cancel_token: Arc<AtomicBool>,
    ) {
        let (allocation_mb, gradual, duration, allocated_memory, current_allocation) = {
            let faults_read = faults.read().await;
            if let Some(fault) = faults_read.get(&handle) {
                (
                    fault.allocation_mb,
                    fault.gradual,
                    fault.duration,
                    fault.allocated_memory.clone(),
                    fault.current_allocation_mb.clone(),
                )
            } else {
                return;
            }
        };

        let chunk_size_mb = if gradual { 1 } else { allocation_mb };
        let allocation_interval = if gradual {
            duration / allocation_mb as u32
        } else {
            Duration::from_secs(0)
        };

        let mut total_allocated = 0u64;
        while total_allocated < allocation_mb && !cancel_token.load(Ordering::SeqCst) {
            let to_allocate = std::cmp::min(chunk_size_mb, allocation_mb - total_allocated);
            
            // Allocate memory chunk (1MB = 1,048,576 bytes)
            let chunk = vec![0u8; (to_allocate * 1024 * 1024) as usize];
            
            {
                let mut memory = allocated_memory.lock().await;
                memory.push(chunk);
            }
            
            total_allocated += to_allocate;
            current_allocation.store(total_allocated, Ordering::SeqCst);
            
            if gradual && allocation_interval > Duration::from_secs(0) {
                sleep(allocation_interval).await;
            }
        }

        // Hold memory for remaining duration
        let remaining_duration = if gradual {
            Duration::from_secs(0) // Already spent time allocating gradually
        } else {
            duration
        };

        if remaining_duration > Duration::from_secs(0) {
            sleep(remaining_duration).await;
        }

        // Clean up allocated memory
        {
            let mut memory = allocated_memory.lock().await;
            memory.clear();
        }
        current_allocation.store(0, Ordering::SeqCst);
    }

    pub async fn stop_memory_pressure(&self, handle: &str) -> Result<(), ChaosError> {
        let mut faults = self.active_faults.write().await;
        if let Some(fault) = faults.remove(handle) {
            fault.cancel_token.store(true, Ordering::SeqCst);
            
            // Clean up allocated memory immediately
            {
                let mut memory = fault.allocated_memory.lock().await;
                memory.clear();
            }
            fault.current_allocation_mb.store(0, Ordering::SeqCst);
            
            Ok(())
        } else {
            Err(ChaosError::FaultInjectionError(
                format!("Memory pressure fault not found: {}", handle)
            ))
        }
    }

    pub async fn get_statistics(&self, handle: &str) -> Result<MemoryPressureStatistics, ChaosError> {
        let faults = self.active_faults.read().await;
        if let Some(fault) = faults.get(handle) {
            Ok(MemoryPressureStatistics {
                current_allocation: fault.current_allocation_mb.load(Ordering::SeqCst),
                target_allocation: fault.allocation_mb,
                additional_metrics: HashMap::new(),
            })
        } else {
            Err(ChaosError::FaultInjectionError(
                format!("Memory pressure fault not found: {}", handle)
            ))
        }
    }
}

#[derive(Debug)]
pub struct MemoryPressureStatistics {
    pub current_allocation: u64,
    pub target_allocation: u64,
    pub additional_metrics: HashMap<String, f64>,
}

// Stub implementations for other controllers
impl ResourceFaultController {
    pub fn new() -> Self {
        Self {
            active_faults: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn inject_resource_fault(
        &self,
        handle: &str,
        _resource: ResourceType,
        _level: f64,
        _duration: Duration,
        _cancel_token: Arc<AtomicBool>,
    ) -> Result<String, ChaosError> {
        // TODO: Implement resource exhaustion logic
        Ok(handle.to_string())
    }

    pub async fn stop_resource_fault(&self, _handle: &str) -> Result<(), ChaosError> {
        Ok(())
    }
}

impl ServiceFaultController {
    pub fn new() -> Self {
        Self {
            active_faults: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn inject_service_fault(
        &self,
        handle: &str,
        _service_name: String,
        _duration: Duration,
        _error_response: Option<String>,
        _cancel_token: Arc<AtomicBool>,
    ) -> Result<String, ChaosError> {
        // TODO: Implement service unavailability logic
        Ok(handle.to_string())
    }

    pub async fn stop_service_fault(&self, _handle: &str) -> Result<(), ChaosError> {
        Ok(())
    }
}

impl CpuStarvationController {
    pub fn new() -> Self {
        Self {
            active_faults: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn inject_cpu_starvation(
        &self,
        handle: &str,
        _cpu_percentage: f64,
        _duration: Duration,
        _threads: usize,
        _cancel_token: Arc<AtomicBool>,
    ) -> Result<String, ChaosError> {
        // TODO: Implement CPU starvation logic
        Ok(handle.to_string())
    }

    pub async fn stop_cpu_starvation(&self, _handle: &str) -> Result<(), ChaosError> {
        Ok(())
    }
}

impl DiskIoFaultController {
    pub fn new() -> Self {
        Self {
            active_faults: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn inject_disk_io_fault(
        &self,
        handle: &str,
        _failure_rate: f64,
        _latency_ms: Option<u64>,
        _target_paths: Vec<String>,
        _cancel_token: Arc<AtomicBool>,
    ) -> Result<String, ChaosError> {
        // TODO: Implement disk I/O fault logic
        Ok(handle.to_string())
    }

    pub async fn stop_disk_io_fault(&self, _handle: &str) -> Result<(), ChaosError> {
        Ok(())
    }
} 