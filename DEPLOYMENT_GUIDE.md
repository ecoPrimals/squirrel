# Squirrel MCP Deployment Guide

## Overview

Squirrel MCP is a production-ready AI Agent Platform with Machine Context Protocol (MCP) support. Each primal operates standalone by default and automatically discovers and connects to ecosystem services (Beardog, NestGate, ToadStool) when available via Songbird discovery.

## 🚀 Quick Start

### Standalone Deployment (Recommended)

```bash
# 1. Clone and build
git clone <repo-url>
cd squirrel-mcp
cargo build --release

# 2. Configure environment (optional - has sane defaults)
export SQUIRREL_MCP_HOST="0.0.0.0"
export SQUIRREL_MCP_PORT="8080"
export SONGBIRD_DISCOVERY_ENDPOINT="https://songbird.your-domain.com"

# 3. Run
./target/release/squirrel-mcp
```

The system will:
- ✅ Start immediately in standalone mode
- ✅ Provide full MCP functionality locally  
- ✅ Auto-discover ecosystem services via Songbird (if available)
- ✅ Auto-connect to Beardog for authentication (if available)
- ✅ Gracefully fallback to local services if ecosystem unavailable

## 🏗️ Architecture

### Standalone Operation
- **Core MCP Protocol**: WebSocket transport, message handling, plugin execution
- **Local Authentication**: Built-in JWT/session management with Beardog fallback
- **Local Storage**: In-memory/file-based with NestGate fallback
- **Local Compute**: Direct execution with ToadStool fallback

### Auto-Discovery Integration
- **Songbird Discovery**: Automatic service discovery and health monitoring
- **Beardog Authentication**: Enterprise security when available
- **NestGate Storage**: Distributed storage when available  
- **ToadStool Compute**: Distributed compute when available

## 📋 Configuration

### Environment Variables

#### Core Configuration
```bash
# Network
SQUIRREL_MCP_HOST="0.0.0.0"          # Bind address
SQUIRREL_MCP_PORT="8080"             # Main port
SQUIRREL_MCP_WEBSOCKET_PORT="8081"   # WebSocket port
SQUIRREL_MCP_DASHBOARD_PORT="8082"   # Dashboard port

# Ecosystem Discovery (Optional)
SONGBIRD_DISCOVERY_ENDPOINT="https://songbird.domain.com"  # Auto-discovery
SQUIRREL_ECOSYSTEM_ENABLED="true"                          # Enable ecosystem integration
SQUIRREL_ECOSYSTEM_MODE="sovereign"                        # sovereign|coordinated|standalone
```

#### Beardog Authentication (Auto-configured when available)
```bash
BEARDOG_AUTH_ENDPOINT="https://beardog.domain.com:8443"
BEARDOG_API_KEY="your-api-key"
BEARDOG_AUTO_AUTH="true"              # Auto-connect when available
BEARDOG_FALLBACK_TO_LOCAL="true"      # Fallback to local auth
```

#### AI Providers (Optional)
```bash
OPENAI_API_KEY="your-openai-key"
ANTHROPIC_API_KEY="your-anthropic-key"
OLLAMA_ENDPOINT="http://localhost:11434"
```

#### NestGate Storage (Auto-configured when available)
```bash
NESTGATE_STORAGE_ENDPOINT="https://nestgate.domain.com:8444"
SQUIRREL_MCP_DATABASE_URL="nestgate://nestgate.domain.com:8444"
```

#### ToadStool Compute (Auto-configured when available)
```bash
TOADSTOOL_COMPUTE_ENDPOINT="https://toadstool.domain.com:8445"
```

### Configuration Modes

#### 1. Pure Standalone (No Ecosystem)
```bash
export SQUIRREL_ECOSYSTEM_ENABLED="false"
./target/release/squirrel-mcp
```

#### 2. Sovereign (Standalone + Auto-Discovery) - **RECOMMENDED**
```bash
export SQUIRREL_ECOSYSTEM_MODE="sovereign"
export SONGBIRD_DISCOVERY_ENDPOINT="https://songbird.domain.com"
./target/release/squirrel-mcp
```

#### 3. Coordinated (Requires Ecosystem)
```bash
export SQUIRREL_ECOSYSTEM_MODE="coordinated"
export BEARDOG_AUTH_ENDPOINT="https://beardog.domain.com:8443"
export NESTGATE_STORAGE_ENDPOINT="https://nestgate.domain.com:8444"
./target/release/squirrel-mcp
```

## 🔍 Service Discovery Flow

### 1. Startup Sequence
```
1. Initialize local services (auth, storage, compute)
2. Start MCP protocol listener
3. Query Songbird for ecosystem services (if configured)
4. Auto-connect to discovered services
5. Register with ecosystem (if available)
6. Begin serving MCP requests
```

### 2. Runtime Discovery
- **Health Checks**: Continuous monitoring of ecosystem services
- **Automatic Reconnection**: Reconnects to services when they become available
- **Graceful Degradation**: Falls back to local services on failure
- **Load Balancing**: Distributes requests across available services

## 🔐 Security Integration

### Beardog Auto-Authentication
When Songbird discovers Beardog:
1. **Auto-Connect**: Establishes secure connection using API key
2. **JWT Verification**: Validates tokens via Beardog API
3. **Session Management**: Synchronized session state
4. **Compliance Monitoring**: Audit logging and compliance checks
5. **Encryption**: HSM-backed encryption for sensitive data

### Fallback Security
When Beardog unavailable:
1. **Local JWT**: Self-signed tokens with configurable expiration
2. **Session Storage**: Local session management
3. **Basic Auth**: Username/password authentication
4. **File-based Users**: Simple user database

## 🚀 Deployment Scenarios

### Development Environment
```bash
# Start with local services only
export SQUIRREL_ECOSYSTEM_ENABLED="false"
cargo run
```

### Staging Environment
```bash
# Use local services with ecosystem discovery
export SQUIRREL_ECOSYSTEM_MODE="sovereign"
export SONGBIRD_DISCOVERY_ENDPOINT="https://staging-songbird.domain.com"
./target/release/squirrel-mcp
```

### Production Environment
```bash
# Full ecosystem integration with fallbacks
export SQUIRREL_ECOSYSTEM_MODE="sovereign"
export SONGBIRD_DISCOVERY_ENDPOINT="https://songbird.domain.com"
export BEARDOG_API_KEY="production-api-key"
export BEARDOG_FALLBACK_TO_LOCAL="true"
./target/release/squirrel-mcp
```

### Docker Deployment
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/squirrel-mcp /usr/local/bin/
EXPOSE 8080 8081 8082
CMD ["squirrel-mcp"]
```

### Kubernetes Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: squirrel-mcp
spec:
  replicas: 3
  selector:
    matchLabels:
      app: squirrel-mcp
  template:
    metadata:
      labels:
        app: squirrel-mcp
    spec:
      containers:
      - name: squirrel-mcp
        image: squirrel-mcp:latest
        ports:
        - containerPort: 8080
        - containerPort: 8081
        - containerPort: 8082
        env:
        - name: SQUIRREL_ECOSYSTEM_MODE
          value: "sovereign"
        - name: SONGBIRD_DISCOVERY_ENDPOINT
          value: "https://songbird.cluster.local"
        - name: BEARDOG_FALLBACK_TO_LOCAL
          value: "true"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: squirrel-mcp-service
spec:
  selector:
    app: squirrel-mcp
  ports:
  - name: http
    port: 8080
    targetPort: 8080
  - name: websocket
    port: 8081
    targetPort: 8081
  - name: dashboard
    port: 8082
    targetPort: 8082
  type: LoadBalancer
```

## 📊 Monitoring & Health Checks

### Health Endpoints
- **Health Check**: `GET /health` - Service health status
- **Metrics**: `GET /metrics` - Prometheus metrics
- **Status**: `GET /status` - Ecosystem connection status

### Key Metrics
- `squirrel_mcp_connections_total` - Total MCP connections
- `squirrel_mcp_requests_total` - Total requests processed
- `squirrel_mcp_ecosystem_status` - Ecosystem service status
- `squirrel_mcp_auth_requests_total` - Authentication requests
- `squirrel_mcp_plugin_executions_total` - Plugin executions

## 🔧 Troubleshooting

### Common Issues

#### 1. Cannot Connect to Ecosystem Services
```bash
# Check Songbird connectivity
curl https://songbird.domain.com/health

# Check service endpoints
curl https://beardog.domain.com:8443/health
curl https://nestgate.domain.com:8444/health
curl https://toadstool.domain.com:8445/health

# Verify configuration
echo $SONGBIRD_DISCOVERY_ENDPOINT
echo $BEARDOG_FALLBACK_TO_LOCAL
```

#### 2. Authentication Failures
```bash
# Check Beardog status
curl -H "Authorization: Bearer $BEARDOG_API_KEY" \
     https://beardog.domain.com:8443/health

# Enable local auth fallback
export BEARDOG_FALLBACK_TO_LOCAL="true"
```

#### 3. Service Discovery Issues
```bash
# Enable debug logging
export RUST_LOG="squirrel_mcp=debug"

# Check Songbird endpoint
export SONGBIRD_DISCOVERY_ENDPOINT="https://songbird.domain.com"

# Disable ecosystem temporarily
export SQUIRREL_ECOSYSTEM_ENABLED="false"
```

### Logs Analysis
```bash
# Check connection logs
journalctl -u squirrel-mcp | grep "ecosystem"

# Check authentication logs  
journalctl -u squirrel-mcp | grep "beardog"

# Check MCP protocol logs
journalctl -u squirrel-mcp | grep "protocol"
```

## 🔗 Integration Points

### For Other Primal Teams

#### 1. MCP Client Integration
```rust
use squirrel_mcp::{McpClient, McpMessage};

let client = McpClient::connect("ws://squirrel-mcp:8081").await?;
let response = client.send_request(request).await?;
```

#### 2. Plugin Development
```rust
use squirrel_mcp::Plugin;

#[derive(Plugin)]
struct MyPlugin;

impl Plugin for MyPlugin {
    async fn execute(&self, args: &[&str]) -> Result<String> {
        // Your plugin logic here
        Ok("Plugin executed successfully".to_string())
    }
}
```

#### 3. Service Registration
```bash
# Register your service with Songbird for discovery
curl -X POST https://songbird.domain.com/register \
  -H "Content-Type: application/json" \
  -d '{
    "service": "my-primal",
    "endpoint": "https://my-primal.domain.com:8080",
    "capabilities": ["compute", "storage"],
    "health_check": "/health"
  }'
```

## 📦 Binary Distribution

### Pre-built Binaries
Available for:
- `x86_64-unknown-linux-gnu` (Linux x64)
- `aarch64-unknown-linux-gnu` (Linux ARM64)
- `x86_64-pc-windows-msvc` (Windows x64)
- `x86_64-apple-darwin` (macOS x64)
- `aarch64-apple-darwin` (macOS ARM64)

### Installation Script
```bash
curl -sSf https://releases.squirrel-mcp.com/install.sh | sh
```

## 🆘 Support

- **Documentation**: https://docs.squirrel-mcp.com
- **Issues**: https://github.com/your-org/squirrel-mcp/issues
- **Chat**: #squirrel-mcp on Slack
- **Email**: support@squirrel-mcp.com

## 📄 License

Licensed under either of
- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option. 