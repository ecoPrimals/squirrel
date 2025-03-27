use std::collections::HashMap;
use sysinfo::{System, SystemExt, CpuExt, DiskExt, NetworkExt};
use dashboard_core::data::{SystemSnapshot, NetworkSnapshot, InterfaceStats, DashboardData, MetricsSnapshot};
use chrono::Utc;

/// Monitoring to Dashboard adapter for converting between monitoring and dashboard data formats
#[derive(Debug)]
pub struct MonitoringToDashboardAdapter {
    resource_adapter: ResourceMetricsCollectorAdapter,
    protocol_adapter: ProtocolMetricsAdapter,
}

impl MonitoringToDashboardAdapter {
    /// Create a new adapter
    pub fn new() -> Self {
        Self {
            resource_adapter: ResourceMetricsCollectorAdapter::new(),
            protocol_adapter: ProtocolMetricsAdapter::new(),
        }
    }
    
    /// Collect dashboard data from monitoring metrics
    pub fn collect_dashboard_data(&mut self) -> DashboardData {
        let (system, network) = self.resource_adapter.collect_dashboard_data();
        let metrics = self.protocol_adapter.collect_metrics();
        
        // Create dashboard data
        DashboardData {
            system,
            network,
            alerts: dashboard_core::data::AlertsSnapshot {
                active: Vec::new(),
                recent: Vec::new(),
                counts: HashMap::new(),
            },
            metrics,
            timestamp: Utc::now(),
        }
    }
}

/// Protocol metrics adapter for collecting MCP protocol metrics
#[derive(Debug)]
pub struct ProtocolMetricsAdapter {
    // State tracking for metrics
    message_counter: u64,
    transaction_counter: u64,
    error_counter: u64,
    last_update: chrono::DateTime<Utc>,
    
    // MCP-specific metrics
    mcp_requests: u64,
    mcp_responses: u64,
    mcp_transactions: u64,
    mcp_connection_errors: u64,
    mcp_protocol_errors: u64,
    
    // Cache calculated rates
    message_rate: f64,
    transaction_rate: f64,
    error_rate: f64,
    mcp_success_rate: f64,
    
    // Store recent latency values for histogram
    latency_values: Vec<f64>,
}

impl ProtocolMetricsAdapter {
    /// Create a new protocol metrics adapter
    pub fn new() -> Self {
        Self {
            message_counter: 0,
            transaction_counter: 0,
            error_counter: 0,
            last_update: Utc::now(),
            
            // Initialize MCP-specific metrics
            mcp_requests: 0,
            mcp_responses: 0,
            mcp_transactions: 0,
            mcp_connection_errors: 0,
            mcp_protocol_errors: 0,
            
            // Initialize rates
            message_rate: 0.0,
            transaction_rate: 0.0,
            error_rate: 0.0,
            mcp_success_rate: 100.0,
            
            // Initialize latency histogram
            latency_values: Vec::with_capacity(20),
        }
    }
    
    /// Try to get MCP metrics from the MCP crate
    fn try_collect_mcp_metrics(&mut self) -> bool {
        // This will be implemented once we have direct integration with the MCP crate
        // For now, we'll continue to use simulated data but with MCP-specific metrics
        
        // TODO: Replace with real MCP metrics collection
        // Example of how this might look:
        // if let Some(mcp_stats) = try_get_mcp_stats() {
        //     self.mcp_requests = mcp_stats.requests;
        //     self.mcp_responses = mcp_stats.responses;
        //     self.mcp_transactions = mcp_stats.transactions;
        //     self.mcp_connection_errors = mcp_stats.connection_errors;
        //     self.mcp_protocol_errors = mcp_stats.protocol_errors;
        //     return true;
        // }
        
        // For now, simulate MCP metrics
        self.mcp_requests += 8 + (rand::random::<u64>() % 15);
        self.mcp_responses += 7 + (rand::random::<u64>() % 14);
        self.mcp_transactions += 4 + (rand::random::<u64>() % 8);
        
        if rand::random::<u8>() % 100 < 2 {
            // 2% chance of a connection error
            self.mcp_connection_errors += 1;
        }
        
        if rand::random::<u8>() % 100 < 3 {
            // 3% chance of a protocol error
            self.mcp_protocol_errors += 1;
        }
        
        true
    }
    
    /// Collect protocol metrics
    pub fn collect_metrics(&mut self) -> MetricsSnapshot {
        // Try to collect real MCP metrics first
        let has_mcp_metrics = self.try_collect_mcp_metrics();
        
        // Update generic counters based on MCP metrics if available
        if has_mcp_metrics {
            // Use MCP metrics as the source for our generic metrics
            self.message_counter = self.mcp_requests + self.mcp_responses;
            self.transaction_counter = self.mcp_transactions;
            self.error_counter = self.mcp_connection_errors + self.mcp_protocol_errors;
        } else {
            // Fall back to simulated data if MCP metrics aren't available
            self.message_counter += 10 + (rand::random::<u64>() % 20);
            self.transaction_counter += 5 + (rand::random::<u64>() % 10);
            
            if rand::random::<u8>() % 100 < 5 {
                // 5% chance of an error
                self.error_counter += 1;
            }
        }
        
        // Calculate rates based on time difference
        let now = Utc::now();
        let time_diff = (now - self.last_update).num_seconds() as f64;
        
        // Prevent division by zero
        if time_diff > 0.0 {
            self.message_rate = (self.mcp_requests + self.mcp_responses) as f64 / time_diff;
            self.transaction_rate = self.mcp_transactions as f64 / time_diff;
        }
        
        // Calculate error rate
        if self.message_counter > 0 {
            self.error_rate = (self.error_counter as f64 / self.message_counter as f64) * 100.0;
        }
        
        // Calculate MCP success rate
        if self.mcp_transactions > 0 {
            let successful_transactions = self.mcp_transactions - 
                (self.mcp_connection_errors + self.mcp_protocol_errors).min(self.mcp_transactions);
            self.mcp_success_rate = (successful_transactions as f64 / self.mcp_transactions as f64) * 100.0;
        }
        
        self.last_update = now;
        
        // Create metrics snapshot
        let mut counters = HashMap::new();
        counters.insert("protocol.messages".to_string(), self.message_counter);
        counters.insert("protocol.transactions".to_string(), self.transaction_counter);
        counters.insert("protocol.errors".to_string(), self.error_counter);
        
        // Add MCP-specific counters
        counters.insert("mcp.requests".to_string(), self.mcp_requests);
        counters.insert("mcp.responses".to_string(), self.mcp_responses);
        counters.insert("mcp.transactions".to_string(), self.mcp_transactions);
        counters.insert("mcp.connection_errors".to_string(), self.mcp_connection_errors);
        counters.insert("mcp.protocol_errors".to_string(), self.mcp_protocol_errors);
        
        let mut gauges = HashMap::new();
        gauges.insert("protocol.message_rate".to_string(), self.message_rate);
        gauges.insert("protocol.transaction_rate".to_string(), self.transaction_rate);
        gauges.insert("protocol.error_rate".to_string(), self.error_rate);
        gauges.insert("mcp.success_rate".to_string(), self.mcp_success_rate);
        
        // Update latency histogram with new values
        // For now, simulate latency values
        // In the future, this should come from actual MCP measurements
        self.latency_values.push(rand::random::<f64>() * 100.0);
        
        // Keep the histogram at a reasonable size
        if self.latency_values.len() > 20 {
            self.latency_values.remove(0);
        }
        
        let mut histograms = HashMap::new();
        histograms.insert("protocol.latency".to_string(), self.latency_values.clone());
        
        MetricsSnapshot {
            values: HashMap::new(),
            counters,
            gauges,
            histograms,
        }
    }
}

/// Resource metrics collector adapter for connecting system metrics to dashboard-core
#[derive(Debug)]
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
            cpu_usage: cpu_usage as f64,  // Convert f32 to f64
            memory_used,
            memory_total,
            disk_used,
            disk_total,
            load_average: [0.0, 0.0, 0.0], // Default values as sysinfo doesn't provide this on all platforms
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
                is_up: true, // Default to true as sysinfo doesn't provide this information directly
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_can_be_converted_to_dashboard_format() {
        let mut collector = ResourceMetricsCollectorAdapter::new();
        let (system, _network) = collector.collect_dashboard_data();
        
        // Verify system metrics
        assert!(system.cpu_usage >= 0.0 && system.cpu_usage <= 100.0);
        assert!(system.memory_used <= system.memory_total);
        assert!(system.disk_used <= system.disk_total);
        // No need to check uptime >= 0 as it's an unsigned type
        
        // Verify network metrics - no need to check >= 0 as these are unsigned types
        
        // There might not be network interfaces on all systems,
        // so this assertion may fail in some environments
        // Commented out to avoid false test failures
        // assert!(!network.interfaces.is_empty());
    }
    
    #[test]
    fn test_system_metrics_collection() {
        let mut collector = ResourceMetricsCollectorAdapter::new();
        let system = collector.collect_system_metrics();
        
        // Verify CPU usage is in valid range
        assert!(system.cpu_usage >= 0.0 && system.cpu_usage <= 100.0);
        
        // Verify memory values make sense
        assert!(system.memory_used <= system.memory_total);
        assert!(system.memory_total > 0);
        
        // Verify disk values make sense
        assert!(system.disk_used <= system.disk_total);
    }
    
    #[test]
    fn test_network_metrics_collection() {
        let mut collector = ResourceMetricsCollectorAdapter::new();
        let network = collector.collect_network_metrics();
        
        // For unsigned types, no need to check >= 0
        // Just verify we have network data by checking interface count
        if !network.interfaces.is_empty() {
            let interface = network.interfaces.values().next().unwrap();
            // Make sure interface has sensible values
            assert_eq!(interface.rx_bytes + interface.tx_bytes > 0 || 
                       interface.rx_packets + interface.tx_packets > 0, 
                       network.rx_bytes + network.tx_bytes > 0 || 
                       network.rx_packets + network.tx_packets > 0);
        }
    }
} 