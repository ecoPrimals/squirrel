# MCP Monitoring Integration

This directory contains specifications and documentation related to the integration between the Machine Context Protocol (MCP) resilience framework and the global monitoring system.

## Directory Contents

- [`MONITORING_INTEGRATION.md`](./MONITORING_INTEGRATION.md) - Core document outlining how MCP integrates with the monitoring system
- [`README.md`](./README.md) - This file, providing directory overview

## Purpose

The integration between MCP and the monitoring system serves several critical purposes:

1. **Local + Global Health Observation**: Maintain MCP's local, low-latency health monitoring capability for resilience decisions while enabling global system visibility through the monitoring system.

2. **Bidirectional Recovery**: Allow recovery actions to be triggered by either the MCP resilience framework or the global monitoring system.

3. **Comprehensive Observability**: Enable full visibility into MCP component health, performance, and error states without tight coupling between systems.

4. **Standard Integration Patterns**: Demonstrate standard patterns for integrating Rust components with the monitoring system through adapter patterns.

## Related Documentation

- Primary integration specification: [MCP and Monitoring System Integration](../../integration/mcp-monitoring-integration.md)
- Monitoring system perspective: [Monitoring Integration with MCP](../../monitoring/MCP_INTEGRATION.md) 
- Observability framework: [Observability Framework](../../integration/observability-framework.md)
- MCP Resilience architecture: [MCP Resilience Framework](../resilience-implementation/ARCHITECTURE.md)

## Implementation Status

The integration between MCP resilience health monitoring and the monitoring system has been implemented with the following components:

- ✅ **Health Monitoring Bridge**: Implemented in `crates/mcp/src/integration/monitoring_bridge_impl.rs` - Mediates between the MCP resilience health monitor and the monitoring system
- ✅ **Resilience Health Check Adapters**: Implemented in `crates/mcp/src/integration/health_check_adapter.rs` - Adapts resilience health checks to the monitoring system's interface
- ✅ **Alert to Recovery Adapter**: Implemented in `crates/mcp/src/integration/alert_recovery_adapter.rs` - Converts monitoring alerts to resilience recovery actions
- ✅ **Example Application**: Implemented in `crates/mcp/examples/monitoring_integration.rs` - Demonstrates the integration
- ✅ **Integration Documentation**: Created in `crates/mcp/MCP_MONITORING_INTEGRATION.md`

The current implementation is being refined to ensure API compatibility with the existing monitoring system. 

See the [MCP Resilience Framework Progress Report](../resilience-implementation/PROGRESS_REPORT.md) for current implementation status. 