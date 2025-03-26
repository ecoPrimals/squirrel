# Squirrel Plugin Marketplace

The Plugin Marketplace is a system for discovering, downloading, and managing plugins from remote repositories. This document explains how to use the marketplace functionality.

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Running the Example Repository Server](#running-the-example-repository-server)
4. [Using the Marketplace CLI](#using-the-marketplace-cli)
5. [Integrating with Your Application](#integrating-with-your-application)
6. [Creating a Custom Repository](#creating-a-custom-repository)
7. [Security Considerations](#security-considerations)

## Overview

The Squirrel Plugin Marketplace system consists of several components:

- **Repository Manager**: Central component for managing multiple plugin repositories
- **Repository Providers**: Implementations for accessing specific repository types (e.g., HTTP)
- **Plugin Package**: Represents a plugin that can be downloaded from a repository
- **CLI Utility**: Command-line interface for interacting with repositories
- **Example Repository Server**: A simple HTTP server for testing and demonstration

## Prerequisites

- Rust 1.70 or newer
- Cargo
- Network access for downloading plugins

## Running the Example Repository Server

The example repository server provides a simple HTTP API for testing the marketplace functionality:

```bash
# Build and run the repository server
cargo run --bin plugin_repository_server --features="repository-server" -- --port 3000
```

This will start a server on http://127.0.0.1:3000 with several example plugins.

## Using the Marketplace CLI

The marketplace CLI utility provides commands for interacting with plugin repositories:

```bash
# Build the marketplace CLI
cargo run --bin plugin_marketplace -- --help
```

### Adding a Repository

```bash
cargo run --bin plugin_marketplace -- add-repo --id example --url http://127.0.0.1:3000
```

### Listing Repositories

```bash
cargo run --bin plugin_marketplace -- list-repos
```

### Listing Plugins

```bash
cargo run --bin plugin_marketplace -- list-plugins
```

### Searching for Plugins

```bash
cargo run --bin plugin_marketplace -- search "example"
```

### Getting Plugin Info

```bash
cargo run --bin plugin_marketplace -- info --repo example --id <plugin-uuid>
```

### Downloading a Plugin

```bash
cargo run --bin plugin_marketplace -- download --repo example --id <plugin-uuid>
```

## Integrating with Your Application

To integrate the plugin marketplace into your application, follow these steps:

### 1. Add Dependencies

Make sure your `Cargo.toml` includes the marketplace features:

```toml
[dependencies]
squirrel-plugins = { version = "0.1.0", features = ["marketplace"] }
```

### 2. Create a Repository Manager

```rust
use std::path::PathBuf;
use std::sync::Arc;
use squirrel_plugins::plugins::{
    create_repository_manager,
    HttpRepositoryProvider,
};

// Create a repository manager
let app_version = "1.0.0";
let download_dir = PathBuf::from("./plugins");
let repo_manager = create_repository_manager(app_version, download_dir)
    .expect("Failed to create repository manager");

// Add a repository
let repo_url = "http://example.com/plugins";
let provider = Arc::new(HttpRepositoryProvider::new(repo_url)
    .expect("Failed to create repository provider"));
repo_manager.add_repository("example", provider)
    .await
    .expect("Failed to add repository");
```

### 3. Discover and Download Plugins

```rust
// List plugins from all repositories
let plugins = repo_manager.list_plugins().await;
for (repo_id, repo_plugins) in plugins {
    println!("Repository: {}", repo_id);
    for plugin in repo_plugins {
        println!("  - {} ({})", plugin.metadata.name, plugin.metadata.id);
    }
}

// Search for plugins
let results = repo_manager.search_plugins("example").await;
for (repo_id, repo_plugins) in results {
    println!("Repository: {}", repo_id);
    for plugin in repo_plugins {
        println!("  - {} ({})", plugin.metadata.name, plugin.metadata.id);
    }
}

// Download a plugin
let plugin_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012")
    .expect("Invalid UUID");
let plugin_path = repo_manager.download_plugin("example", plugin_id)
    .await
    .expect("Failed to download plugin");
println!("Plugin downloaded to: {}", plugin_path.display());
```

### 4. Load the Plugin

After downloading a plugin, you can load it using the dynamic plugin loading system:

```rust
use squirrel_plugins::plugins::{
    create_library_loader,
    DynamicLibraryLoader,
};

// Create a dynamic library loader
let loader = create_library_loader()
    .expect("Failed to create library loader");

// Load the plugin
let plugin = loader.load_plugin(&plugin_path)
    .expect("Failed to load plugin");

// Use the plugin
// ...
```

## Creating a Custom Repository

To create a custom repository, you need to implement the `RepositoryProvider` trait:

```rust
use std::path::{Path, PathBuf};
use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;
use squirrel_plugins::plugins::marketplace::{
    RepositoryProvider,
    RepositoryInfo,
    PluginPackageInfo,
};
use squirrel_plugins::plugins::errors::{PluginError, Result};

struct CustomRepositoryProvider {
    // Your implementation details
}

#[async_trait]
impl RepositoryProvider for CustomRepositoryProvider {
    async fn get_repository_info(&self) -> Result<RepositoryInfo> {
        // Implement repository info retrieval
    }
    
    async fn list_plugins(&self) -> Result<Vec<PluginPackageInfo>> {
        // Implement plugin listing
    }
    
    async fn get_plugin_info(&self, plugin_id: Uuid) -> Result<PluginPackageInfo> {
        // Implement plugin info retrieval
    }
    
    async fn download_plugin(&self, plugin_id: Uuid, target_dir: &Path) -> Result<PathBuf> {
        // Implement plugin download
    }
    
    async fn search_plugins(&self, query: &str) -> Result<Vec<PluginPackageInfo>> {
        // Implement plugin search
    }
}
```

## Security Considerations

When using the plugin marketplace, consider the following security aspects:

1. **Plugin Source Verification**: Only download plugins from trusted repositories.
2. **Checksum Verification**: The marketplace automatically verifies plugin checksums.
3. **Signature Verification**: Consider implementing signature verification for plugins.
4. **Platform Compatibility**: The marketplace checks platform compatibility for plugins.
5. **API Version Compatibility**: The marketplace checks API version compatibility.
6. **Plugin Sandboxing**: Consider running plugins in a sandbox environment.
7. **Plugin Permissions**: Implement a permission system for plugins.

For more information, refer to the [Security Model](docs/security_model.md) documentation. 