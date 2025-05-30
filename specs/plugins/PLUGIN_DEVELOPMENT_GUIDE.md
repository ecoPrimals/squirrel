# Squirrel Plugin Development Guide

This guide covers everything you need to know to develop, test, and publish plugins for the Squirrel MCP ecosystem.

## Table of Contents

1. [Overview](#overview)
2. [Getting Started](#getting-started)
3. [Plugin Architecture](#plugin-architecture)
4. [Development Environment Setup](#development-environment-setup)
5. [Creating Your First Plugin](#creating-your-first-plugin)
6. [Plugin API Reference](#plugin-api-reference)
7. [Testing Plugins](#testing-plugins)
8. [Security Guidelines](#security-guidelines)
9. [Publishing Plugins](#publishing-plugins)
10. [Best Practices](#best-practices)

## Overview

Squirrel plugins are WebAssembly (WASM) modules that extend the functionality of the Squirrel MCP platform. They run in a secure sandbox environment and can interact with the MCP protocol, external APIs, and user interfaces.

### Key Features

- **Secure Sandbox**: Plugins run in isolated WASM environments
- **MCP Integration**: Direct access to the Model Context Protocol
- **Cross-Platform**: Write once, run everywhere
- **Hot Reloading**: Development plugins can be reloaded without restart
- **Dependency Management**: Automatic dependency resolution and updates

## Getting Started

### Prerequisites

- Rust 1.70+ with `wasm32-unknown-unknown` target
- Node.js 18+ (for web components)
- Squirrel CLI tools
- Git for version control

### Installation

```bash
# Install Rust WASM target
rustup target add wasm32-unknown-unknown

# Install Squirrel CLI
cargo install squirrel-cli

# Install WASM tools
cargo install wasm-pack

# Create new plugin project
squirrel plugin new my-awesome-plugin
cd my-awesome-plugin
```

## Plugin Architecture

### Plugin Structure

```
my-plugin/
├── Cargo.toml           # Rust dependencies and metadata
├── plugin.toml          # Plugin configuration and metadata
├── src/
│   ├── lib.rs          # Main plugin entry point
│   ├── handlers/       # Command handlers
│   ├── utils/          # Utility functions
│   └── types.rs        # Type definitions
├── web/                # Web UI components (optional)
│   ├── package.json
│   ├── src/
│   └── dist/
├── tests/              # Unit and integration tests
├── docs/               # Plugin documentation
└── examples/           # Usage examples
```

### Plugin Configuration (`plugin.toml`)

```toml
[plugin]
name = "my-awesome-plugin"
version = "1.0.0"
description = "An awesome plugin that does amazing things"
author = "Your Name <your.email@example.com>"
license = "MIT"
homepage = "https://github.com/yourname/my-awesome-plugin"
repository = "https://github.com/yourname/my-awesome-plugin"
documentation = "https://docs.yoursite.com/my-awesome-plugin"

[plugin.metadata]
categories = ["development", "productivity"]
tags = ["git", "automation", "workflow"]
keywords = ["git", "commit", "automation"]

[plugin.sandbox]
network_access = true
file_system_access = ["./workspace", "./cache"]
memory_limit_mb = 256
cpu_limit_percent = 10
execution_timeout_seconds = 30

[plugin.dependencies]
"squirrel-sdk" = "^1.0.0"
"serde" = "^1.0"
"tokio" = { version = "^1.0", features = ["macros"] }

[plugin.mcp]
protocol_version = "1.0"
supports_streaming = true
supports_tools = true
supports_resources = true

[plugin.ui]
has_web_ui = true
ui_entry_point = "web/dist/index.html"
```

## Development Environment Setup

### 1. Initialize Plugin Project

```bash
squirrel plugin new my-plugin --template=full
cd my-plugin
```

### 2. Development Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
squirrel-sdk = "1.0"
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.4"
tokio = { version = "1.0", features = ["macros"] }

[dependencies.web-sys]
version = "0.3"
features = [
  "console",
  "Document",
  "Element",
  "HtmlElement",
  "Window",
]

[lib]
crate-type = ["cdylib"]
```

### 3. Development Server

```bash
# Start development server with hot reload
squirrel plugin dev

# Or manually
wasm-pack build --target web --out-dir pkg
python -m http.server 8000
```

## Creating Your First Plugin

### Basic Plugin Structure

```rust
// src/lib.rs
use squirrel_sdk::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct MyPlugin {
    config: PluginConfig,
}

#[wasm_bindgen]
impl MyPlugin {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<MyPlugin, JsValue> {
        utils::set_panic_hook();
        
        Ok(MyPlugin {
            config: PluginConfig::default(),
        })
    }

    /// Initialize the plugin
    #[wasm_bindgen]
    pub async fn initialize(&mut self, config: JsValue) -> Result<(), JsValue> {
        let config: PluginConfig = serde_wasm_bindgen::from_value(config)?;
        self.config = config;
        
        // Register command handlers
        self.register_handlers().await?;
        
        Ok(())
    }

    /// Handle MCP commands
    #[wasm_bindgen]
    pub async fn handle_command(&self, command: &str, params: JsValue) -> Result<JsValue, JsValue> {
        let params: serde_json::Value = serde_wasm_bindgen::from_value(params)?;
        
        let result = match command {
            "hello" => self.handle_hello(params).await,
            "process" => self.handle_process(params).await,
            _ => Err(PluginError::UnknownCommand(command.to_string())),
        };

        match result {
            Ok(response) => Ok(serde_wasm_bindgen::to_value(&response)?),
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }
}

impl MyPlugin {
    async fn register_handlers(&self) -> Result<(), PluginError> {
        let registry = CommandRegistry::global();
        
        registry.register_command(CommandDefinition {
            name: "hello".to_string(),
            description: "Say hello to the world".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name to greet"
                    }
                }
            }),
        }).await?;

        Ok(())
    }

    async fn handle_hello(&self, params: serde_json::Value) -> Result<serde_json::Value, PluginError> {
        let name = params.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("World");

        Ok(serde_json::json!({
            "message": format!("Hello, {}!", name),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    async fn handle_process(&self, params: serde_json::Value) -> Result<serde_json::Value, PluginError> {
        // Process data with the plugin's core functionality
        let input = params.get("input")
            .ok_or(PluginError::MissingParameter("input".to_string()))?;

        // Simulate processing
        let result = format!("Processed: {}", input);

        Ok(serde_json::json!({
            "result": result,
            "status": "success"
        }))
    }
}
```

### Plugin Configuration Types

```rust
// src/types.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub api_key: Option<String>,
    pub endpoint: String,
    pub timeout: u64,
    pub custom_settings: HashMap<String, serde_json::Value>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            endpoint: "https://api.example.com".to_string(),
            timeout: 30,
            custom_settings: HashMap::new(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
    
    #[error("Missing parameter: {0}")]
    MissingParameter(String),
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Processing error: {0}")]
    ProcessingError(String),
}
```

## Plugin API Reference

### Core SDK Components

#### Command Registry

```rust
use squirrel_sdk::commands::CommandRegistry;

// Register a command
let registry = CommandRegistry::global();
registry.register_command(CommandDefinition {
    name: "my_command".to_string(),
    description: "My awesome command".to_string(),
    parameters: command_schema,
}).await?;

// Handle command execution
registry.handle_command("my_command", params).await?;
```

#### MCP Client

```rust
use squirrel_sdk::mcp::McpClient;

// Send MCP message
let client = McpClient::global();
let response = client.send_message("list_tools", params).await?;

// Subscribe to events
client.subscribe("tool_updated", |event| {
    // Handle event
}).await?;
```

#### File System Access

```rust
use squirrel_sdk::fs::FileSystem;

// Read file (must be in allowed paths)
let fs = FileSystem::new();
let content = fs.read_to_string("./workspace/file.txt").await?;

// Write file
fs.write("./cache/output.txt", content).await?;
```

#### HTTP Client

```rust
use squirrel_sdk::http::HttpClient;

// Make HTTP request (if network access allowed)
let client = HttpClient::new();
let response = client.get("https://api.example.com/data")
    .send()
    .await?;
```

### UI Integration

#### Web Components

```typescript
// web/src/plugin-ui.ts
import { PluginSDK } from '@squirrel/web-sdk';

class MyPluginUI extends HTMLElement {
    private sdk: PluginSDK;

    constructor() {
        super();
        this.sdk = new PluginSDK();
    }

    connectedCallback() {
        this.innerHTML = `
            <div class="plugin-container">
                <h3>My Awesome Plugin</h3>
                <button id="execute">Execute Command</button>
                <div id="output"></div>
            </div>
        `;

        this.querySelector('#execute')?.addEventListener('click', () => {
            this.executeCommand();
        });
    }

    async executeCommand() {
        try {
            const result = await this.sdk.executeCommand('hello', {
                name: 'Plugin User'
            });
            
            this.querySelector('#output')!.textContent = 
                JSON.stringify(result, null, 2);
        } catch (error) {
            console.error('Command execution failed:', error);
        }
    }
}

customElements.define('my-plugin-ui', MyPluginUI);
```

## Testing Plugins

### Unit Tests

```rust
// tests/unit_tests.rs
use wasm_bindgen_test::*;
use my_plugin::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_hello_command() {
    let mut plugin = MyPlugin::new().unwrap();
    plugin.initialize(JsValue::NULL).await.unwrap();

    let params = serde_wasm_bindgen::to_value(&serde_json::json!({
        "name": "Test"
    })).unwrap();

    let result = plugin.handle_command("hello", params).await.unwrap();
    let response: serde_json::Value = serde_wasm_bindgen::from_value(result).unwrap();

    assert_eq!(response["message"], "Hello, Test!");
}
```

### Integration Tests

```bash
# Run tests in headless browser
wasm-pack test --headless --firefox

# Run tests in Node.js
wasm-pack test --node

# Run tests with Squirrel test runner
squirrel plugin test --integration
```

### Manual Testing

```bash
# Load plugin in development environment
squirrel plugin load ./pkg

# Test commands
squirrel command hello --name="World"

# Monitor plugin logs
squirrel plugin logs my-plugin --follow
```

## Security Guidelines

### Sandbox Restrictions

- **Network Access**: Only if explicitly requested and approved
- **File System**: Limited to specified directories
- **Memory**: Capped at configured limit
- **CPU**: Limited execution time and resource usage
- **System Calls**: Restricted to safe operations

### Best Practices

1. **Minimize Permissions**: Only request necessary sandbox permissions
2. **Validate Input**: Always validate and sanitize user input
3. **Error Handling**: Never expose sensitive information in errors
4. **Secure Dependencies**: Use only trusted dependencies
5. **Code Review**: Have plugins reviewed before publishing

### Security Checklist

- [ ] Input validation implemented
- [ ] Error messages don't leak sensitive data
- [ ] Dependencies are from trusted sources
- [ ] Minimal sandbox permissions requested
- [ ] No hardcoded secrets or API keys
- [ ] Proper authentication handling
- [ ] XSS prevention in web UI
- [ ] CSRF protection implemented

## Publishing Plugins

### Preparation

1. **Update Version**: Bump version in `plugin.toml` and `Cargo.toml`
2. **Documentation**: Complete README, API docs, and examples
3. **Testing**: Ensure all tests pass
4. **Security Review**: Complete security checklist
5. **Build**: Create production build

```bash
# Build for production
squirrel plugin build --release

# Run security scan
squirrel plugin security-scan

# Validate plugin
squirrel plugin validate ./pkg
```

### Publishing to Registry

```bash
# Login to plugin registry
squirrel auth login

# Publish plugin
squirrel plugin publish ./pkg

# Tag release
git tag v1.0.0
git push origin v1.0.0
```

### Plugin Metadata

Ensure your `plugin.toml` includes:

- Clear description and purpose
- Appropriate categories and tags
- Links to documentation and repository
- License information
- Contact information

## Best Practices

### Code Organization

- **Modular Design**: Break functionality into small, focused modules
- **Error Handling**: Use Result types and provide meaningful errors
- **Documentation**: Document all public APIs and complex logic
- **Testing**: Maintain high test coverage

### Performance

- **Lazy Loading**: Load resources only when needed
- **Caching**: Cache expensive computations
- **Memory Management**: Be mindful of memory usage in WASM
- **Async Operations**: Use async/await for I/O operations

### User Experience

- **Clear Commands**: Use descriptive command names and help text
- **Progress Feedback**: Show progress for long-running operations
- **Graceful Degradation**: Handle missing features gracefully
- **Responsive UI**: Ensure web components are responsive

### Maintenance

- **Version Management**: Use semantic versioning
- **Backward Compatibility**: Maintain API compatibility when possible
- **Deprecation**: Provide clear migration paths for breaking changes
- **Updates**: Regular updates for security and compatibility

## Examples

### Simple Text Processor Plugin

```rust
// A plugin that processes text with various transformations
use squirrel_sdk::prelude::*;

#[wasm_bindgen]
pub struct TextProcessor;

#[wasm_bindgen]
impl TextProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }

    #[wasm_bindgen]
    pub async fn process_text(&self, text: &str, operation: &str) -> Result<String, JsValue> {
        match operation {
            "uppercase" => Ok(text.to_uppercase()),
            "lowercase" => Ok(text.to_lowercase()),
            "reverse" => Ok(text.chars().rev().collect()),
            "word_count" => Ok(text.split_whitespace().count().to_string()),
            _ => Err(JsValue::from_str("Unknown operation")),
        }
    }
}
```

### API Integration Plugin

```rust
// A plugin that integrates with external APIs
use squirrel_sdk::prelude::*;

#[wasm_bindgen]
pub struct ApiIntegration {
    api_key: String,
}

#[wasm_bindgen]
impl ApiIntegration {
    #[wasm_bindgen(constructor)]
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    #[wasm_bindgen]
    pub async fn fetch_data(&self, endpoint: &str) -> Result<JsValue, JsValue> {
        let client = HttpClient::new();
        let response = client
            .get(&format!("https://api.example.com/{}", endpoint))
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let data = response.json().await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(serde_wasm_bindgen::to_value(&data)?)
    }
}
```

## Resources

- [Squirrel SDK Documentation](https://docs.squirrel.dev/sdk)
- [Plugin Registry](https://plugins.squirrel.dev)
- [Community Forum](https://community.squirrel.dev)
- [Example Plugins](https://github.com/squirrel-org/plugin-examples)
- [WASM Bindgen Book](https://rustwasm.github.io/wasm-bindgen/)

## Support

- **Documentation**: [docs.squirrel.dev](https://docs.squirrel.dev)
- **Community**: [Discord](https://discord.gg/squirrel)
- **Issues**: [GitHub Issues](https://github.com/squirrel-org/squirrel/issues)
- **Email**: plugin-support@squirrel.dev 