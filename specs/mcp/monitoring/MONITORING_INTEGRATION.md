---
version: 1.0.0
last_updated: 2024-09-18
status: implementing
priority: high
---

# MCP Integration with Monitoring System

## Overview

This document outlines how the Machine Context Protocol (MCP) Resilience Framework integrates with the global Monitoring System. For a comprehensive specification of the integration, please refer to the main integration document: [MCP and Monitoring System Integration](../../integration/mcp-monitoring-integration.md).

## Implementation Status

The integration between MCP and the monitoring system has been implemented with the following components:

1. **HealthMonitoringBridge**: Implemented in `crates/mcp/src/integration/monitoring_bridge_impl.rs`
   - Mediates between the MCP resilience health monitor and the monitoring system
   - Forwards health data on configurable intervals
   - Registers alert handlers for bidirectional communication

2. **ResilienceHealthCheckAdapter**: Implemented in `crates/mcp/src/integration/health_check_adapter.rs`
   - Adapts resilience health checks to monitoring system format
   - Handles health status conversion between systems
   - Generates consistent metrics for component health

3. **AlertToRecoveryAdapter**: Implemented in `crates/mcp/src/integration/alert_recovery_adapter.rs`
   - Converts monitoring alerts to resilience recovery actions
   - Maps alert severity to failure severity
   - Triggers appropriate recovery strategies

4. **Example Implementation**: Created in `crates/mcp/examples/monitoring_integration.rs`
   - Demonstrates the complete integration setup
   - Shows how to configure and use the components together
   - Includes simulated health status changes and alert handling

5. **Documentation**: Created in `crates/mcp/MCP_MONITORING_INTEGRATION.md`
   - Details usage instructions
   - Provides code examples
   - Explains best practices

### Implementation Challenges

During implementation, several API compatibility issues were discovered:

1. **Alert and Metric API Discrepancies**: The actual monitoring system's Alert and Metric structs have different structures than what was initially specified:
   - Alert lacks direct fields for severity, message, etc.
   - Metric has a different construction pattern and lacks timestamp field

2. **Type System Issues**: The actual implementation requires different type handling:
   - No downcast_ref method for AlertManager and MetricsCollector
   - Different patterns for mocking components in tests

3. **Initialization Requirements**: Additional configuration parameters are required by some components:
   - RecoveryStrategy requires explicit configuration

### Current Status and Next Steps

The implementation requires significant refactoring to match the actual monitoring system API. The team is:

1. Examining the actual API structure in detail
2. Redesigning the adapters to match the actual API
3. Revising the testing approach to work with the actual types
4. Updating the documentation to reflect the correct API usage

Once these issues are resolved, the integration will be fully functional.

## Integration Goals

- Maintain local resilience decisions with global visibility
- Enable bidirectional recovery actions between systems
- Provide seamless health status sharing between MCP and monitoring
- Minimize performance overhead on critical MCP operations
- Support full observability without tight coupling

## Key Integration Points

1. **Health Monitoring Bridge**: The central component that forwards MCP resilience health data to the monitoring system while maintaining independence.

2. **Resilience Component Health Check**: MCP components implement health checks that are usable by both MCP resilience and the monitoring system.

3. **Recovery Integration**: Recovery actions can be initiated from either the MCP resilience framework or the monitoring system.

4. **Health Status Consistency**: Ensures that health status values are consistently mapped between systems.

## MCP Implementation Requirements

### Resilience Health Check Interfaces

MCP components should implement health checks that are compatible with the integration:

```rust
/// Health check for an MCP component
pub struct MCPComponentHealthCheck {
    component_id: String,
    state: Arc<RwLock<ComponentState>>,
    // Additional fields...
}

// Implementation for resilience framework
#[resilience::async_trait]
impl resilience::health::HealthCheck for MCPComponentHealthCheck {
    fn id(&self) -> &str {
        &self.component_id
    }
    
    async fn check(&self) -> resilience::health::HealthCheckResult {
        let state = self.state.read().await;
        
        // Determine health status based on component state
        let status = if state.is_connected {
            if state.errors_since_last_check > 0 {
                resilience::health::HealthStatus::Degraded
            } else {
                resilience::health::HealthStatus::Healthy
            }
        } else {
            resilience::health::HealthStatus::Unhealthy
        };
        
        // Include metrics in result
        let mut metrics = HashMap::new();
        metrics.insert("errors_since_last_check".to_string(), state.errors_since_last_check);
        metrics.insert("latency_ms".to_string(), state.average_latency_ms);
        metrics.insert("uptime_seconds".to_string(), state.uptime_seconds);
        
        resilience::health::HealthCheckResult::new(
            self.component_id.clone(),
            status,
            format!("MCP component status: {}", status),
        ).with_metrics(metrics)
    }
}
```

### Using the Health Monitoring Bridge

MCP components should utilize the Health Monitoring Bridge for integration:

```rust
/// Initialize MCP component with monitoring integration
pub async fn initialize_mcp_component_with_monitoring(
    component_id: &str,
    resilience_monitor: Arc<resilience::health::HealthMonitor>,
    monitoring_adapter: Arc<monitoring::health::HealthCheckerAdapter>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create component-specific health check
    let component_state = Arc::new(RwLock::new(ComponentState::default()));
    let health_check = MCPComponentHealthCheck::new(component_id.to_string(), component_state.clone());
    
    // 2. Register with resilience monitor
    resilience_monitor.register(health_check).await?;
    
    // 3. Configure bridge if not already done
    let bridge_config = HealthMonitoringBridgeConfig {
        forward_interval: 10,
        forward_all_components: true,
        bidirectional: true,
    };
    
    // 4. Create and start bridge (or use existing one)
    let bridge = get_or_create_bridge(resilience_monitor, monitoring_adapter, bridge_config).await?;
    
    // 5. Ensure bridge is running
    if !bridge.is_running() {
        bridge.start().await?;
    }
    
    Ok(())
}
```

### Recovery Action Integration

MCP resilience components should implement recovery actions that can be triggered by monitoring alerts:

```rust
/// Recovery strategy for MCP components
pub struct MCPComponentRecoveryStrategy {
    // Strategy-specific fields...
}

impl resilience::recovery::RecoveryStrategy for MCPComponentRecoveryStrategy {
    fn handle_failure(
        &mut self, 
        failure: resilience::recovery::FailureInfo,
        default_action: impl FnOnce() -> Result<(), Box<dyn std::error::Error>>
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Log the failure
        log::warn!("Handling failure for {}: {}", failure.context, failure.message);
        
        // Determine recovery action based on component type and severity
        if failure.context.starts_with("transport-") {
            match failure.severity {
                resilience::recovery::FailureSeverity::Minor => {
                    // For minor transport issues, just retry
                    log::info!("Retrying transport connection...");
                    self.perform_transport_retry(&failure.context)?;
                }
                resilience::recovery::FailureSeverity::Moderate => {
                    // For moderate issues, reconnect
                    log::info!("Reconnecting transport...");
                    self.reconnect_transport(&failure.context)?;
                }
                resilience::recovery::FailureSeverity::Critical => {
                    // For critical issues, failover to backup
                    log::warn!("Failing over to backup transport...");
                    self.failover_transport(&failure.context)?;
                }
            }
        } else if failure.context.starts_with("protocol-") {
            // Protocol-specific recovery logic...
            self.handle_protocol_failure(&failure)?;
        } else {
            // For unknown components, use the default action
            default_action()?;
        }
        
        Ok(())
    }
}
```

## Testing and Verification

The MCP resilience framework should implement the following tests:

1. **Health Status Propagation Tests**: Verify that health status changes in MCP are correctly propagated to the monitoring system.
2. **Recovery Action Tests**: Ensure that recovery actions can be triggered by monitoring alerts.
3. **Bridge Performance Tests**: Measure the overhead of the health monitoring bridge on MCP operations.
4. **Fault Injection Tests**: Validate the integration under various fault scenarios.

## Configuration Reference

### MCP Configuration for Monitoring Integration

```toml
[mcp.resilience.monitoring_integration]
enabled = true
forward_interval_seconds = 10
forward_all_components = true
bidirectional = true

[mcp.resilience.monitoring_integration.component_mappings]
"protocol-adapter" = "mcp-protocol"
"security-manager" = "mcp-security"
"transport-layer" = "mcp-transport"
```

### Recovery Strategy Configuration

```toml
[mcp.resilience.recovery]
max_retry_attempts = 3
backoff_initial_ms = 100
backoff_max_ms = 5000
backoff_factor = 2.0

[mcp.resilience.recovery.alert_handlers]
monitoring_alerts = true
monitoring_alert_severities = ["Error", "Critical"]
```

## Related Documents

- [MCP and Monitoring System Integration](../../integration/mcp-monitoring-integration.md) - Primary integration specification
- [MCP Resilience Architecture](../resilience-implementation/ARCHITECTURE.md) - MCP resilience framework architecture
- [Health Monitoring Implementation](../resilience-implementation/health-monitoring.md) - MCP health monitoring details
- [Recovery Strategies](../resilience-implementation/recovery-strategies.md) - MCP recovery strategies
- [Monitoring Integration](../../monitoring/MCP_INTEGRATION.md) - Monitoring system perspective on integration 