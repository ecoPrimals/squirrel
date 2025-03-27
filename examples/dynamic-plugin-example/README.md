# Dynamic Plugin Example

This crate demonstrates how to create a dynamically loadable plugin for the Squirrel Plugin System. The plugin provides both commands and tools that can be loaded and executed at runtime.

## Overview

This example shows:

1. How to implement a dynamic plugin that can be loaded from a shared library
2. How to export the necessary FFI functions with `#[no_mangle]`
3. How to implement both the `CommandsPlugin` and `ToolPlugin` traits
4. How to manage plugin state and resources
5. How to provide proper metadata for the plugin system

## Building the Plugin

To build the plugin as a shared library:

```bash
# Navigate to the example directory
cd examples/dynamic-plugin-example

# Build the dynamic library
cargo build --release
```

This will create a shared library in the target/release directory:
- Windows: `dynamic_plugin_example.dll`
- Linux: `libdynamic_plugin_example.so`
- macOS: `libdynamic_plugin_example.dylib`

## Plugin Features

### Commands

This plugin provides two commands:

1. **greet** - Greets a user in different languages
   - Parameters:
     - `name` (optional): The name to greet
     - `language` (optional): The language to use (en, es, fr, de, ja)
   - Example Usage:
     ```json
     {
       "name": "John",
       "language": "es"
     }
     ```
   - Example Output:
     ```json
     {
       "greeting": "¡Hola, John!",
       "language": "es"
     }
     ```

2. **calculate** - Performs a calculation with two numbers
   - Parameters:
     - `a` (required): First number
     - `b` (required): Second number
     - `operation` (optional): Operation to perform (add, subtract, multiply, divide)
   - Example Usage:
     ```json
     {
       "a": 10,
       "b": 5,
       "operation": "multiply"
     }
     ```
   - Example Output:
     ```json
     {
       "result": 50,
       "operation": "multiply",
       "a": 10,
       "b": 5
     }
     ```

### Tools

This plugin provides two tools:

1. **format** - Formats text in different styles
   - Parameters:
     - `text` (required): The text to format
     - `style` (optional): The formatting style (normal, uppercase, lowercase, title, reverse)
   - Example Usage:
     ```json
     {
       "text": "hello world",
       "style": "title"
     }
     ```
   - Example Output:
     ```json
     {
       "original": "hello world",
       "formatted": "Hello World",
       "style": "title"
     }
     ```

2. **convert** - Converts between different units
   - Parameters:
     - `value` (required): The value to convert
     - `from_unit` (required): The source unit (m, km, cm, ft, kg, lb, c, f)
     - `to_unit` (required): The target unit (m, km, cm, ft, kg, lb, c, f)
   - Example Usage:
     ```json
     {
       "value": 100,
       "from_unit": "m",
       "to_unit": "ft"
     }
     ```
   - Example Output:
     ```json
     {
       "value": 100,
       "from_unit": "m",
       "to_unit": "ft",
       "result": 328.084,
       "conversion_factor": 3.28084
     }
     ```

## Loading the Plugin

To load the plugin in your application:

```rust
use std::path::Path;
use squirrel_plugins::management::PluginRegistry;

async fn load_example_plugin() -> Result<()> {
    // Create plugin registry
    let registry = PluginRegistry::new();
    
    // Path to the compiled library
    let plugin_path = Path::new("path/to/libdynamic_plugin_example.so"); // adjust for your platform
    
    // Register and initialize the plugin
    let plugin_id = registry.register_dynamic_plugin(plugin_path).await?;
    registry.initialize_plugin(plugin_id).await?;
    registry.start_plugin(plugin_id).await?;
    
    // Now you can use the plugin
    // Get all command plugins
    let command_plugins = registry.get_command_plugins().await;
    
    // Find our plugin and execute a command
    for plugin in command_plugins {
        let result = plugin.execute_command(
            "greet", 
            serde_json::json!({
                "name": "World",
                "language": "fr"
            })
        ).await?;
        
        println!("Command result: {}", result);
    }
    
    // Shutdown the plugin when done
    registry.stop_plugin(plugin_id).await?;
    registry.shutdown_plugin(plugin_id).await?;
    
    Ok(())
}
```

## Plugin Structure

The plugin consists of:

1. **ExampleDynamicPlugin struct** - The main plugin implementation
2. **Plugin trait implementation** - Core lifecycle methods (initialize, start, stop, shutdown)
3. **CommandsPlugin trait implementation** - Command-related functionality
4. **ToolPlugin trait implementation** - Tool-related functionality
5. **FFI export functions** - The C-compatible functions for loading the plugin:
   - `create_plugin()` - Creates the plugin instance
   - `get_plugin_metadata()` - Returns metadata about the plugin
   - `destroy_plugin()` - Cleans up when the plugin is unloaded

## Best Practices Demonstrated

This example demonstrates several best practices for plugin development:

1. **Resource Management** - Proper cleanup in the `destroy_plugin` function
2. **Thread Safety** - Using Arc and RwLock for shared state
3. **Error Handling** - Proper error propagation and checking
4. **Documentation** - Complete documentation of all functions and methods
5. **Stable IDs** - Using a consistent UUID for the plugin
6. **Metadata** - Providing complete plugin metadata
7. **Command/Tool Schema** - Providing schemas for all commands and tools

## Testing

You can test the plugin by loading it with the Squirrel Plugin System, or you can use a dedicated test harness for dynamic libraries.

## Further Reading

For more information about creating dynamic plugins, please see:
- [Dynamic Plugin Development Guide](../../specs/plugins/DYNAMIC_PLUGIN_GUIDE.md)
- [Plugin System Implementation Status](../../specs/plugins/IMPLEMENTATION_STATUS.md) 