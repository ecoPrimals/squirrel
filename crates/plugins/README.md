# Squirrel Plugins

This crate provides a unified plugin system for Squirrel applications, enabling extensibility and modular functionality across the ecosystem.

## For Plugin Silo Team

### Bidirectional Compatibility

The plugin system now features **bidirectional compatibility** for web plugins, which is crucial for your team during the migration period:

- **Legacy → New**: Legacy plugins can work with the new system via `LegacyWebPluginAdapter`
- **New → Legacy**: New plugins can work with legacy systems via `NewWebPluginAdapter`

This enables your team to:

1. **Develop new plugins with the modern API** immediately, without waiting for all systems to migrate
2. **Deploy plugins to mixed environments** where both old and new infrastructure exist
3. **Gradually migrate existing plugins** without disrupting current functionality

### Migration Resources

- **Migration Guide**: See the detailed [Web Plugin Migration Guide](docs/WEB_PLUGIN_MIGRATION.md)
- **Example Code**: Check the [bidirectional compatibility example](examples/bidirectional_compatibility.rs)
- **API Documentation**: The web plugin API is documented in the codebase

### Migration Timeline

The migration is proceeding in phases:

1. **Phase 1 (Current)**: Both old and new APIs supported via adapters
2. **Phase 2 (3 months)**: New API recommended for all new plugins
3. **Phase 3 (6 months)**: Old API deprecated
4. **Phase 4 (12 months)**: Old API removed

### Key Benefits of the New API

1. **Enhanced Routing**: Route pattern matching with automatic parameter extraction
2. **Structured Request/Response**: Type-safe, comprehensive request and response objects
3. **Builder Pattern**: Fluent, intuitive API for creating endpoints and components
4. **Improved Type Safety**: Enums for HTTP methods and component types
5. **UUID-based Identifiers**: Modern, collision-resistant identifiers

### Getting Started

To use the bidirectional compatibility features:

```rust
// Using a legacy plugin with the new system
let legacy_plugin = Arc::new(MyLegacyPlugin::new());
let adapted_for_new = LegacyWebPluginAdapter::new(legacy_plugin);
modern_registry.register_plugin(Arc::new(adapted_for_new)).await?;

// Using a modern plugin with legacy systems
let modern_plugin = Arc::new(MyModernPlugin::new());
let adapted_for_legacy = NewWebPluginAdapter::new(modern_plugin);
legacy_registry.register_plugin(Arc::new(adapted_for_legacy))?;
```

### Need Help?

If you have questions or issues implementing the compatibility features or migrating plugins, please contact the DataScienceBioLab team for assistance.

## Features

- Core plugin system with metadata, lifecycle, and dependency management
- Web plugin support for HTTP endpoints and UI components
- Machine Context Protocol (MCP) integration
- Command-line interface plugins
- Comprehensive testing utilities

## Usage

See the `examples/` directory for sample implementations of various plugin types.

## License

This crate is licensed under MIT OR Apache-2.0. 