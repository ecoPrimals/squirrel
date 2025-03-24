# Web Interface

The Web Interface provides HTTP and WebSocket APIs for the Squirrel system.

## Current Status

The Web Interface is currently operational with the following features:
- HTTP API for command execution
- WebSocket API for real-time events
- Authentication with JWT tokens
- Job management for long-running operations
- Health checking endpoints
- Plugin system (in migration process)

## Plugin System Migration

The Web Interface plugin system is currently being migrated to the unified `squirrel_plugins` crate architecture. This migration follows the direct conversion approach as outlined in the specs/plugins/migration-plan.md document.

### Migration Status

- Migration plan completed (see PLUGIN_MIGRATION_PLAN.md)
- Plugin adapter implemented to bridge between legacy and unified systems
- Test framework for plugin migration created
- Cargo.toml updated for future integration with unified plugin system

### Next Steps in Migration

1. Complete the integration with the unified plugin registry
2. Update all plugin API endpoints to use the unified system
3. Implement plugin discovery using the unified system
4. Complete comprehensive testing

For more details on the migration process, see PLUGIN_MIGRATION_PLAN.md.

## Running the Web Interface

### Prerequisites

- Rust 1.67 or higher
- SQLite (if using DB mode)

### Installation

```bash
git clone <repository>
cd squirrel/crates/web
```

### Build and Run

With mock database (no actual DB needed):

```bash
cargo build --features mock-db
cargo run --features mock-db
```

With real database:

```bash
# Create the database
./setup-db.ps1    # On Windows
./setup-db.sh     # On Unix/Linux/Mac

# Run with database
cargo build --features db
cargo run --features db
```

### Configuration

Configuration is loaded from environment variables:

- `SQUIRREL_WEB_HOST`: Hostname to bind to (default: 127.0.0.1)
- `SQUIRREL_WEB_PORT`: Port to listen on (default: 8080)
- `SQUIRREL_DB_URL`: Database URL (default: sqlite:test.db)
- `SQUIRREL_JWT_SECRET`: JWT secret key (default: generated randomly)

## API Documentation

API documentation is available at:

- `/api/docs` - OpenAPI documentation (coming soon)
- `/api/health` - Health check endpoint
- `/api/commands` - Command execution endpoints
- `/api/jobs` - Job management endpoints
- `/api/plugins` - Plugin management endpoints
- `/ws` - WebSocket endpoint

## Development Notes

### Feature Flags

- `mock-db` - Use in-memory database (for development)
- `db` - Use real database
- `server` - Build the HTTP server (always enabled by default)

### Testing

```bash
cargo test
```

### Adding a New Plugin

Currently, the Web Interface is migrating its plugin system. During this transition, you can still create plugins using the legacy system, which will be gradually migrated to the unified system.

Legacy plugin example:

```rust
use squirrel_web::plugins::{Plugin, WebPlugin, PluginMetadata, WebEndpoint, HttpMethod};
use async_trait::async_trait;
use serde_json::Value;
use anyhow::Result;
use uuid::Uuid;

#[derive(Debug)]
pub struct MyPlugin {
    metadata: PluginMetadata,
}

impl MyPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: Uuid::new_v4(),
                name: "My Plugin".to_string(),
                version: "0.1.0".to_string(),
                description: "My awesome plugin".to_string(),
                author: "Your Name".to_string(),
                capabilities: vec!["awesome".to_string()],
                dependencies: vec![],
            },
        }
    }
}

#[async_trait]
impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl WebPlugin for MyPlugin {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        vec![
            WebEndpoint {
                path: "/my-endpoint".to_string(),
                method: HttpMethod::Get,
                permissions: vec![],
            },
        ]
    }
    
    async fn handle_web_endpoint(&self, _endpoint: &WebEndpoint, _data: Option<Value>) -> Result<Value> {
        Ok(serde_json::json!({ "status": "ok" }))
    }
}
```

After the migration is complete, plugins will be created using the unified plugin system:

```rust
// Future API (coming soon)
use squirrel_plugins::core::Plugin;
use squirrel_plugins::web::WebPlugin;
// ... implementation ...
```

Stay tuned for updates on the plugin system migration! 