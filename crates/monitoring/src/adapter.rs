use std::sync::Arc;
use crate::error::Result;
use super::{
    MonitoringService,
    MonitoringConfig,
    MonitoringServiceFactory,
    HealthCheckerAdapter,
    DefaultMetricCollector,
};
use super::alerts::AlertManagerAdapter;
use super::network::NetworkMonitorAdapter;
use super::alerts::NotificationManagerTrait;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tracing::{debug, error, info};
use sysinfo::{System, SystemExt, ProcessExt, NetworksExt, DiskExt, CpuExt, NetworkExt};
use dashboard_core::data::{SystemSnapshot, NetworkSnapshot, InterfaceStats};
use chrono::Utc;

/// Adapter for monitoring service factory
/// 
/// This adapter wraps a monitoring service factory to provide a consistent interface
/// and additional functionality while delegating to the underlying implementation.
#[derive(Debug)]
pub struct MonitoringServiceFactoryAdapter<N: NotificationManagerTrait + 'static = ()> {
    /// Underlying factory instance that will be used to create monitoring services
    /// The adapter delegates creation requests to this inner factory.
    pub inner: Option<Arc<MonitoringServiceFactory<N>>>,
}

impl<N: NotificationManagerTrait + Send + Sync + std::fmt::Debug + 'static> MonitoringServiceFactoryAdapter<N> {
    /// Creates a new factory adapter
    #[must_use]
    pub fn new() -> Self {
        Self { inner: None }
    }

    /// Creates a new adapter with an existing factory
    #[must_use]
    pub fn with_factory(factory: Arc<MonitoringServiceFactory<N>>) -> Self {
        Self {
            inner: Some(factory),
        }
    }

    /// Creates a service using the default configuration
    #[must_use]
    pub fn create_service(&self) -> Arc<MonitoringService> {
        if let Some(factory) = &self.inner {
            factory.create_service()
        } else {
            // Initialize on-demand with default configuration
            let factory: MonitoringServiceFactory<()> = MonitoringServiceFactory::with_config(MonitoringConfig::default());
            Arc::new(factory).create_service()
        }
    }

    /// Creates a service with a custom configuration
    #[must_use]
    pub fn create_service_with_config(&self, config: MonitoringConfig) -> Arc<MonitoringService> {
        if let Some(factory) = &self.inner {
            factory.create_service_with_config(config)
        } else {
            // Initialize on-demand with default configuration
            let factory: MonitoringServiceFactory<()> = MonitoringServiceFactory::with_config(config.clone());
            Arc::new(factory).create_service_with_config(config)
        }
    }

    /// Creates a service with explicit dependencies
    #[must_use]
    pub fn create_service_with_dependencies(
        &self,
        config: MonitoringConfig,
        health_checker: Arc<HealthCheckerAdapter>,
        metric_collector: Arc<DefaultMetricCollector>,
        alert_manager: Arc<AlertManagerAdapter>,
        network_monitor: Arc<NetworkMonitorAdapter>,
    ) -> Arc<MonitoringService> {
        if let Some(factory) = &self.inner {
            factory.create_service_with_dependencies(
                config,
                health_checker,
                metric_collector,
                alert_manager,
                network_monitor,
            )
        } else {
            // Initialize on-demand with default configuration
            let factory: MonitoringServiceFactory<()> = MonitoringServiceFactory::with_config(config.clone());
            Arc::new(factory).create_service_with_dependencies(
                config,
                health_checker,
                metric_collector,
                alert_manager,
                network_monitor,
            )
        }
    }

    /// Creates a service using adapter pattern for ongoing transition
    #[must_use]
    pub fn create_service_with_adapters(&self) -> Arc<MonitoringService> {
        if let Some(factory) = &self.inner {
            factory.create_service_with_adapters()
        } else {
            // Initialize on-demand with default configuration
            let factory: MonitoringServiceFactory<()> = MonitoringServiceFactory::with_config(MonitoringConfig::default());
            Arc::new(factory).create_service_with_adapters()
        }
    }

    /// Starts a new service with the default configuration
    pub async fn start_service(&self) -> Result<Arc<MonitoringService>> {
        if let Some(factory) = &self.inner {
            factory.start_service().await
        } else {
            // Initialize on-demand with default configuration
            let factory: MonitoringServiceFactory<()> = MonitoringServiceFactory::with_config(MonitoringConfig::default());
            Arc::new(factory).start_service().await
        }
    }

    /// Starts a new service with a custom configuration
    pub async fn start_service_with_config(&self, config: MonitoringConfig) -> Result<Arc<MonitoringService>> {
        if let Some(factory) = &self.inner {
            factory.start_service_with_config(config).await
        } else {
            // Initialize on-demand with default configuration
            let factory: MonitoringServiceFactory<()> = MonitoringServiceFactory::with_config(config.clone());
            Arc::new(factory).start_service_with_config(config).await
        }
    }
}

impl<N: NotificationManagerTrait + 'static> Default for MonitoringServiceFactoryAdapter<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new monitoring service factory adapter
#[must_use]
pub fn create_factory_adapter<N: NotificationManagerTrait + Send + Sync + std::fmt::Debug + 'static>() -> Arc<MonitoringServiceFactoryAdapter<N>> {
    Arc::new(MonitoringServiceFactoryAdapter::new())
}

/// Creates a new monitoring service factory adapter with an existing factory
#[must_use]
pub fn create_factory_adapter_with_factory<N: NotificationManagerTrait + Send + Sync + std::fmt::Debug + 'static>(
    factory: Arc<MonitoringServiceFactory<N>>
) -> Arc<MonitoringServiceFactoryAdapter<N>> {
    Arc::new(MonitoringServiceFactoryAdapter::with_factory(factory))
}

/// Resource metrics collector adapter for connecting monitoring to dashboard-core
#[derive(Debug, Clone)]
pub struct ResourceMetricsCollectorAdapter {
    system: System,
}

impl ResourceMetricsCollectorAdapter {
    /// Create a new resource metrics collector adapter
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        ResourceMetricsCollectorAdapter {
            system,
        }
    }
    
    /// Refresh system data
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }
    
    /// Collect system metrics and convert to dashboard-core format
    pub fn collect_system_metrics(&mut self) -> SystemSnapshot {
        self.refresh();
        
        // Collect CPU metrics
        let cpu_usage = self.system.global_cpu_info().cpu_usage();
        
        // Collect memory metrics
        let memory_used = self.system.used_memory();
        let memory_total = self.system.total_memory();
        
        // Collect disk metrics
        let disks = self.system.disks();
        let mut disk_used = 0;
        let mut disk_total = 0;
        
        for disk in disks {
            disk_used += disk.total_space() - disk.available_space();
            disk_total += disk.total_space();
        }
        
        // Create system snapshot
        SystemSnapshot {
            cpu_usage,
            memory_used,
            memory_total,
            disk_used,
            disk_total,
            load_average: [0.0, 0.0, 0.0], // Replace with actual values if available
            uptime: self.system.uptime(),
        }
    }
    
    /// Collect network metrics and convert to dashboard-core format
    pub fn collect_network_metrics(&mut self) -> NetworkSnapshot {
        self.refresh();
        
        let mut rx_bytes = 0;
        let mut tx_bytes = 0;
        let mut rx_packets = 0;
        let mut tx_packets = 0;
        let mut interfaces = HashMap::new();
        
        self.system.refresh_networks();
        
        for (name, network) in self.system.networks() {
            let rx_bytes_interface = network.received();
            let tx_bytes_interface = network.transmitted();
            let rx_packets_interface = network.packets_received();
            let tx_packets_interface = network.packets_transmitted();
            
            // Update totals
            rx_bytes += rx_bytes_interface;
            tx_bytes += tx_bytes_interface;
            rx_packets += rx_packets_interface;
            tx_packets += tx_packets_interface;
            
            // Store interface metrics
            interfaces.insert(name.clone(), InterfaceStats {
                name: name.clone(),
                rx_bytes: rx_bytes_interface,
                tx_bytes: tx_bytes_interface,
                rx_packets: rx_packets_interface,
                tx_packets: tx_packets_interface,
                is_up: true, // Fill with actual status if available
            });
        }
        
        // Create network snapshot
        NetworkSnapshot {
            rx_bytes,
            tx_bytes,
            rx_packets,
            tx_packets,
            interfaces,
        }
    }
    
    /// Collect all metrics as dashboard data
    pub fn collect_dashboard_data(&mut self) -> (SystemSnapshot, NetworkSnapshot) {
        let system_snapshot = self.collect_system_metrics();
        let network_snapshot = self.collect_network_metrics();
        
        (system_snapshot, network_snapshot)
    }
} 