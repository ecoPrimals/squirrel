#!/bin/bash
# Build script for test plugins
#
# This script builds the test dynamic plugin for the current platform
# and copies it to the correct location with the appropriate name.

set -e  # Exit on error

# Detect platform
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    PLATFORM="linux"
    EXTENSION="so"
    OUTPUT_NAME="libtest_dynamic_plugin.so"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    PLATFORM="macos"
    EXTENSION="dylib"
    OUTPUT_NAME="libtest_dynamic_plugin.dylib"
elif [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    PLATFORM="windows"
    EXTENSION="dll"
    OUTPUT_NAME="test_dynamic_plugin.dll"
else
    echo "Unsupported platform: $OSTYPE"
    exit 1
fi

echo "Detected platform: $PLATFORM"
echo "Building test plugin..."

# Create a temporary directory
TEMP_DIR=$(mktemp -d)
echo "Using temporary directory: $TEMP_DIR"

# Create a new crate in the temporary directory
echo "Creating test plugin crate..."
cargo new --lib "$TEMP_DIR/test-dynamic-plugin"
cd "$TEMP_DIR/test-dynamic-plugin"

# Update Cargo.toml
echo "Configuring test plugin crate..."
cat > Cargo.toml << EOL
[package]
name = "test-dynamic-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
async-trait = "0.1.68"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.28", features = ["full"] }
uuid = { version = "1.3", features = ["v4", "serde"] }
squirrel-mcp = { path = "../../../squirrel-mcp" }
squirrel-plugins = { path = "../../../" }
EOL

# Copy the test plugin implementation
echo "Copying test plugin implementation..."
PLUGIN_TEMPLATE="$(realpath ../../../src/plugins/examples/test_dynamic_plugin.rs)"
if [ -f "$PLUGIN_TEMPLATE" ]; then
    cp "$PLUGIN_TEMPLATE" src/lib.rs
else
    echo "Error: Plugin template not found at $PLUGIN_TEMPLATE"
    exit 1
fi

# Build the plugin
echo "Building test plugin..."
cargo build --release

# Create the output directory if it doesn't exist
if [ ! -d "../test_plugins" ]; then
    mkdir -p "../test_plugins"
fi

# Copy the built plugin to the output directory
echo "Copying plugin to output location..."
if [ "$PLATFORM" = "windows" ]; then
    cp "target/release/$OUTPUT_NAME" "../test_plugins/test_plugin.$EXTENSION"
else
    cp "target/release/$OUTPUT_NAME" "../test_plugins/test_plugin.$EXTENSION"
fi

echo "Test plugin built successfully for $PLATFORM"
echo "Plugin location: test_plugins/test_plugin.$EXTENSION"

# Clean up
echo "Cleaning up..."
rm -rf "$TEMP_DIR"

echo "Done!" 