# Dynamic Plugin Loader Example

This is an example application that demonstrates how to load and interact with dynamically loaded plugins in the Squirrel Plugin System.

## Overview

This application:

1. Loads a dynamic plugin from a shared library (.dll, .so, .dylib)
2. Initializes and starts the plugin
3. Provides an interactive CLI to execute commands and tools exposed by the plugin
4. Properly stops and unloads the plugin when done

## Building the Loader

To build the plugin loader:

```bash
# Navigate to the example directory
cd examples/dynamic-plugin-loader

# Build the application
cargo build --release
```

This will create an executable in the target/release directory.

## Using the Loader

First, make sure you have built a dynamic plugin:

```bash
# Build the example dynamic plugin
cd examples/dynamic-plugin-example
cargo build --release
```

Then, run the loader:

```bash
# Run the loader with a specific plugin path
./target/release/dynamic-plugin-loader --plugin-path ../dynamic-plugin-example/target/release/libdynamic_plugin_example.so

# Or run the loader without arguments to be prompted for the path
./target/release/dynamic-plugin-loader
```

## Interactive Usage

The application provides an interactive menu system:

1. First, you select whether to interact with the plugin as a Command Plugin or a Tool Plugin
2. For Command Plugins:
   - Select from the available commands
   - View command help information
   - Enter command arguments
   - See the command result
3. For Tool Plugins:
   - Select from the available tools
   - View tool metadata
   - Enter tool arguments
   - See the tool result

## Plugin Lifecycle

The application demonstrates the proper plugin lifecycle:

1. **Registration**: Load the plugin from the shared library
2. **Initialization**: Initialize plugin resources
3. **Start**: Start plugin operations
4. **Interaction**: Use plugin commands and tools
5. **Stop**: Stop plugin operations
6. **Shutdown**: Clean up plugin resources

## Code Structure

The main components of the loader application are:

- **main**: Handles the initialization, loading, and lifecycle management of the plugin
- **interact_with_plugin**: Provides the main interaction menu
- **interact_with_command_plugin**: Handles interaction with command plugins
- **interact_with_tool_plugin**: Handles interaction with tool plugins

## Error Handling

The application demonstrates proper error handling for plugin operations:

- File not found errors for missing plugins
- Type casting errors for incompatible plugins
- Execution errors for failed commands/tools
- Proper propagation of plugin-specific errors

## Logging

The application uses the `tracing` crate for logging, which provides:

- Log levels (debug, info, warn, error)
- Log filtering
- Structured logging
- Context for errors

## Further Development

This example can be extended in several ways:

1. **Multiple Plugin Support**: Load multiple plugins at once
2. **Plugin Dependencies**: Handle plugin dependencies
3. **Hot Reloading**: Support reloading plugins without restarting
4. **GUI Interface**: Create a graphical interface for plugin interaction
5. **Remote Plugins**: Support loading plugins from remote sources 