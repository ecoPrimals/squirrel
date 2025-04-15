# Squirrel Plugin System

A robust, secure, and flexible plugin system for extending application functionality.

## Status: 🟢 Complete (100%)

The Squirrel Plugin System implementation is now complete. All components have been implemented, documented, and thoroughly tested.

## Features

- **Plugin Interface Architecture** - Clean and intuitive API for plugin development
- **Dynamic Loading** - Load plugins from compiled libraries at runtime
- **Resource Monitoring** - Track and limit resource usage of plugins
- **State Persistence** - Save and restore plugin state between sessions
- **Plugin Marketplace** - Discover, download, and update plugins from repositories
- **Security Framework** - Sandboxed execution and permission model
- **Cross-Platform Support** - Windows, Linux, and macOS compatibility
- **Extensive Documentation** - Comprehensive guides and API references
- **Testing Infrastructure** - Unit tests, integration tests, and fuzzing

## Documentation

The following documentation is available:

- [Getting Started Guide](docs/plugins/getting_started.md)
- [Plugin Development Guide](docs/plugins/plugin_development.md)
- [API Reference](docs/plugins/api_reference.md)
- [Resource Monitoring](docs/plugins/resource_monitoring.md)
- [State Persistence](docs/plugins/state_persistence.md)
- [Security Guide](docs/plugins/security.md)
- [Cross-Platform Testing](docs/plugins/cross_platform_testing.md)
- [Troubleshooting Guide](docs/plugins/troubleshooting.md)
- [Fuzzing Guide](docs/plugins/fuzzing_guide.md)
- [Address Sanitizer Guide](docs/devtools/address_sanitizer_guide.md)
- [Documentation Index](docs/plugins/index.md)
- [Implementation Summary](docs/plugins/implementation_summary.md)

## Implementation Details

The plugin system is designed with modularity and extensibility in mind:

```
plugins/
├── core/              # Core plugin interfaces and traits
├── dynamic/           # Dynamic library loading and validation
├── lifecycle/         # Plugin lifecycle management
├── marketplace/       # Plugin discovery and distribution
├── monitoring/        # Resource monitoring and limits
├── security/          # Security validation and sandboxing
└── state/             # State persistence mechanisms
```

For a comprehensive overview of the implementation, see the [Implementation Summary](docs/plugins/implementation_summary.md).

## Examples

Example plugins demonstrating various capabilities:

- [Hello World Plugin](examples/hello_world) - Simplest possible plugin
- [Calculator Plugin](examples/calculator) - Command-based plugin for arithmetic
- [File Browser Plugin](examples/file_browser) - Tool-based plugin with UI integration
- [State Demo Plugin](examples/state_demo) - Demonstrates state persistence
- [Resource Intensive Plugin](examples/resource_demo) - Demonstrates resource monitoring

## Building and Testing

```bash
# Build the plugin system
cargo build --release

# Run tests
cargo test

# Run fuzzing tests
cd fuzz && ./run_fuzzers.sh  # Linux/macOS
cd fuzz && ./run_fuzzers.ps1  # Windows

# Run fuzzing tests without Address Sanitizer
cd fuzz && ./run_fuzzers.sh --no-asan  # Linux/macOS
cd fuzz && ./run_fuzzers.ps1 -NoAsan  # Windows

# Run standalone ASAN checks on binaries
./tools/run_asan_check.sh --binary ./target/debug/plugin_host  # Linux/macOS
./tools/run_asan_check.ps1 -BinaryPath ./target/debug/plugin_host  # Windows
```

## Adapter Pattern Implementation

The `adapter-tests` crate provides a clean, standalone implementation of the Adapter Pattern in Rust. It demonstrates how to adapt different interfaces for command execution, authentication, and plugin systems.

Key features:
- Thread-safe command registry with Arc/Mutex
- Asynchronous execution of commands
- Authentication and authorization for MCP (Machine Context Protocol)
- Comprehensive test suite and examples

To run the adapter showcase example:

```bash
cargo run --example adapter_showcase -p adapter-tests
```

For a basic example:

```bash
cargo run --example basic -p adapter-tests
```

To run the tests:

```bash
cargo test -p adapter-tests
```

See the [adapter-tests README](crates/adapter-tests/README.md) for more details.

## Terminal UI Dashboard

The project includes a Terminal User Interface (TUI) dashboard for monitoring system metrics and MCP protocol activities:

```bash
# Run the basic dashboard
cargo run --package ui-terminal --bin main

# Run the custom demonstration dashboard
cargo run --package ui-terminal --example custom_dashboard -- --alerts --mcp
```

### MCP Protocol Integration

The Terminal UI features comprehensive integration with the Machine Context Protocol (MCP):

- **ConnectionHealth Monitoring**: Track connection quality metrics like latency, stability, and packet loss
- **Metrics Caching**: Efficient time-based caching of metrics
- **Performance Tracking**: Monitor and optimize metrics collection performance
- **Error Handling**: Robust error handling with reconnection support

Example MCP monitoring applications:

```bash
# Run the MCP monitor example
cargo run --package ui-terminal --example mcp_monitor -- --simulate-issues

# Run the custom dashboard with MCP integration
cargo run --package ui-terminal --example custom_dashboard -- --mcp
```

For more details on the MCP integration, see the following documentation:
- [MCP Implementation Summary](specs/ui/MCP_IMPLEMENTATION_SUMMARY.md)
- [MCP Examples](specs/ui/MCP_EXAMPLES.md)
- [Implementation Progress](specs/ui/IMPLEMENTATION_PROGRESS.md)

### Dashboard Options

The custom dashboard demonstration provides various command-line options:

- `--interval <SECONDS>`: Set the update interval (default: 3 seconds)
- `--alerts`: Enable simulated alerts
- `--mcp`: Show MCP protocol simulation
- `--mcp-server <ADDRESS>`: Specify the MCP server address
- `--mcp-interval <MS>`: Set MCP update interval in milliseconds
- `--simulate-issues`: Enable simulation of connection issues
- `--pattern <TYPE>`: Set CPU simulation pattern (sine, spike, random)

For more details on the Terminal UI capabilities, see the [ui-terminal README](crates/ui-terminal/README.md).

# OpenTelemetry, Jaeger, and Zipkin Setup

This repository contains a Docker Compose configuration for setting up OpenTelemetry Collector with Jaeger and Zipkin for distributed tracing.

## Components

- **OpenTelemetry Collector**: Central collector that receives, processes, and exports telemetry data
- **Jaeger**: End-to-end distributed tracing system
- **Zipkin**: Distributed tracing system

## Getting Started

### Prerequisites

- Docker and Docker Compose installed on your system

### Running the Services

1. Start all services using Docker Compose:

```bash
docker-compose up -d
```

2. Verify that the services are running:

```bash
docker-compose ps
```

### Accessing the UIs

- **Jaeger UI**: http://localhost:16686
- **Zipkin UI**: http://localhost:9411

## Sending Traces

The OpenTelemetry Collector is configured to receive traces via:

- OTLP gRPC: `localhost:4317`
- OTLP HTTP: `localhost:4318`

For Rust applications, you can use the following configurations:

```rust
// Configure OpenTelemetry with OTLP exporter
let tracer = opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("http://localhost:4317"))
    .install_batch(opentelemetry::runtime::Tokio)
    .expect("Failed to create tracer");
```

## Stopping the Services

To stop all services:

```bash
docker-compose down
```

## Configuration

- The OpenTelemetry Collector configuration is defined in `otel-collector-config.yaml`
- The services configuration is defined in `docker-compose.yml`

You can modify these files to adjust the configuration according to your needs. 
