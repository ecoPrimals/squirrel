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

## License

MIT License - See [LICENSE](LICENSE) for details.

## Contributors

DataScienceBioLab Team

---

© 2024 Squirrel Inc. All rights reserved. 