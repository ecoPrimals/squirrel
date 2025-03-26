# PowerShell script for building test plugins on Windows
#
# This script builds the test dynamic plugin for Windows
# and copies it to the correct location with the appropriate name.

# Ensure errors cause the script to stop
$ErrorActionPreference = "Stop"

Write-Host "Building test plugin for Windows..."

# Create a temporary directory
$TempDir = New-Item -ItemType Directory -Path "$env:TEMP\test-plugin-build-$(Get-Random)" -Force
Write-Host "Using temporary directory: $TempDir"

# Create a new crate in the temporary directory
Write-Host "Creating test plugin crate..."
Push-Location $TempDir
cargo new --lib "test-dynamic-plugin"
Set-Location "test-dynamic-plugin"

# Update Cargo.toml
Write-Host "Configuring test plugin crate..."
@"
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
"@ | Out-File -FilePath "Cargo.toml" -Encoding utf8

# Copy the test plugin implementation
Write-Host "Copying test plugin implementation..."
$PluginTemplate = Resolve-Path -Path "..\..\..\src\plugins\examples\test_dynamic_plugin.rs"
if (Test-Path $PluginTemplate) {
    Copy-Item -Path $PluginTemplate -Destination "src\lib.rs"
} else {
    Write-Error "Error: Plugin template not found at $PluginTemplate"
    exit 1
}

# Build the plugin
Write-Host "Building test plugin..."
cargo build --release

# Create the output directory if it doesn't exist
$TestPluginsDir = "..\..\..\test_plugins"
if (-not (Test-Path $TestPluginsDir)) {
    New-Item -ItemType Directory -Path $TestPluginsDir -Force | Out-Null
}

# Copy the built plugin to the output directory
Write-Host "Copying plugin to output location..."
Copy-Item -Path "target\release\test_dynamic_plugin.dll" -Destination "$TestPluginsDir\test_plugin.dll"

Write-Host "Test plugin built successfully for Windows"
Write-Host "Plugin location: test_plugins\test_plugin.dll"

# Clean up
Write-Host "Cleaning up..."
Pop-Location
Remove-Item -Path $TempDir -Recurse -Force

Write-Host "Done!" 