# Squirrel Universal AI Primal - Deployment Guide

**Date**: April 2026  
**Status**: **PRE-ALPHA** (v0.1.0-alpha.45) — functional for development and testing; not yet validated for production deployment

> **Architecture**: Squirrel uses **Unix sockets + JSON-RPC 2.0** exclusively. There are no HTTP endpoints. All communication is via Unix domain sockets and JSON-RPC. Optional tarpc provides high-performance binary RPC. biomeOS lifecycle integration uses `lifecycle.register` on startup and `lifecycle.status` heartbeat every 30 seconds.

---

## ✅ Pre-Deployment Verification

### System Requirements
- **Rust**: 1.94+ (edition 2024)
- **Operating System**: Linux (primary), macOS, or Windows
- **Memory**: 4GB RAM minimum, 8GB recommended
- **CPU**: 2+ cores
- **Storage**: 10GB free space

### Compilation Check
```bash
cargo check --all-targets
# Should complete with exit code 0 (warnings acceptable)
```

---

## 🚀 Production Deployment

### Step 1: Build Release Binary
```bash
cargo build --release
```

The `squirrel` binary is produced at `target/release/squirrel`.

### Step 2: Environment Configuration
```bash
# Optional: Override socket path (otherwise uses XDG or /tmp)
export SQUIRREL_SOCKET="/run/user/$(id -u)/biomeos/squirrel.sock"

# Logging
export RUST_LOG="info"

# biomeOS integration (when running under orchestrator)
export BIOMEOS_SOCKET_PATH="/run/user/$(id -u)/biomeos/neural-api.sock"
```

### Step 3: Start the Service
```bash
# Run Squirrel server (Unix socket + JSON-RPC)
./target/release/squirrel server

# Optional: Specify socket path
./target/release/squirrel server --socket /path/to/squirrel.sock
```

### Step 4: Health Verification
```bash
# Use built-in doctor for health checks
./target/release/squirrel doctor

# Or send JSON-RPC via client subcommand
./target/release/squirrel client --method system.ping

# Or run an existing example
cargo run --example observability_demo
```

---

## 📡 Socket Path Priority (5-Tier Fallback)

1. **CLI `--socket`** (highest priority)
2. **`SQUIRREL_SOCKET`** environment variable
3. **`BIOMEOS_SOCKET_PATH`** (Neural API orchestration)
4. **XDG Runtime**: `/run/user/<uid>/biomeos/squirrel.sock`
5. **Temp fallback**: `/tmp/squirrel-<family>-<node>.sock` (dev/testing only)

---

## biomeOS Lifecycle Integration

When running under biomeOS:

- **`lifecycle.register`** — Sent on startup when biomeOS socket is found
- **`lifecycle.status`** — Heartbeat every 30 seconds
- **SIGTERM/SIGINT** — Socket cleanup and graceful shutdown

All lifecycle communication uses Unix socket JSON-RPC (no HTTP).

---

## 🐳 Docker Deployment

### Dockerfile
```dockerfile
FROM rust:1.70 AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/squirrel /usr/local/bin/

# Unix socket - no TCP port needed for core operation
# Use volume mount for socket directory
VOLUME ["/run/squirrel"]

CMD ["squirrel", "server"]
```

### Build and Run
```bash
# Build Docker image
docker build -t squirrel:latest .

# Run container (mount socket directory)
docker run -v /run/user/$(id -u)/biomeos:/run/squirrel \
    -e SQUIRREL_SOCKET=/run/squirrel/squirrel.sock \
    -e RUST_LOG=info \
    squirrel:latest
```

---

## ⚙️ Configuration Options

### Environment Variables
| Variable | Description | Default |
|----------|-------------|---------|
| `SQUIRREL_SOCKET` | Unix socket path override | XDG or /tmp |
| `BIOMEOS_SOCKET_PATH` | biomeOS orchestrator path | - |
| `BIOMEOS_SOCKET` | Alternative biomeOS path | - |
| `RUST_LOG` | Logging level | info |

### CLI Options (server)
| Option | Description | Default |
|--------|-------------|---------|
| `--socket` | Unix socket path | Auto-detected |
| `--bind` | (Unused in Unix socket mode) | 0.0.0.0 |
| `--port` | (Unused in Unix socket mode) | 9010 |
| `--daemon` | Run as background daemon | false |

---

## 🔍 Health Monitoring

### JSON-RPC Methods for Health
- **`system.ping`** — Basic liveness
- **`system.health`** — Detailed health
- **`system.status`** — Service status
- **`system.metrics`** — Metrics (when monitoring feature enabled)

### Example: system.ping via squirrel client
```bash
squirrel client --method system.ping
```

### Example: Raw JSON-RPC over Unix socket
```bash
echo '{"jsonrpc":"2.0","method":"system.ping","params":{},"id":1}' | \
  socat - UNIX-CONNECT:/run/user/$(id -u)/biomeos/squirrel.sock
```

---

## 🔧 Troubleshooting

### Compilation Errors
```bash
cargo clean
cargo build --release
```

### Socket Not Found
```bash
# Verify socket path
ls -la $SQUIRREL_SOCKET

# Or check XDG path
ls -la $XDG_RUNTIME_DIR/biomeos/squirrel.sock
```

### Permission Issues
```bash
# Ensure socket directory exists and is writable
mkdir -p $XDG_RUNTIME_DIR/biomeos
chmod 700 $XDG_RUNTIME_DIR/biomeos
```

### Log Analysis
```bash
RUST_LOG=debug ./target/release/squirrel server
```

---

## 📊 Performance Monitoring

### Key Metrics
- **Request latency**: JSON-RPC over Unix socket (sub-ms local)
- **Memory usage**: < 1GB under normal load
- **CPU usage**: < 50% under normal load

### Monitoring
- Use `squirrel doctor` for subsystem health
- Use `system.metrics` JSON-RPC method (when `monitoring` feature enabled)
- No HTTP `/metrics` endpoint — use JSON-RPC

---

## 🔐 Security Considerations

### Unix Socket Security
- Socket files are owned by the running user
- Use `chmod` to restrict access (e.g., 0600 for single-user)
- XDG runtime directory provides user isolation

### No HTTP = No Network Exposure
- Squirrel does not listen on TCP by default
- All communication is local Unix sockets
- Reduces attack surface vs HTTP servers

---

## ✅ Deployment Checklist

- [ ] System requirements met
- [ ] Compilation successful
- [ ] Socket directory created (if using custom path)
- [ ] Environment variables configured (if needed)
- [ ] `squirrel doctor` passes
- [ ] biomeOS integration verified (if applicable)
- [ ] Logging configured
- [ ] Signal handling tested (SIGTERM cleanup)

---

## 📞 Support

For deployment issues:
1. Run `squirrel doctor` for diagnostics
2. Check logs with `RUST_LOG=debug`
3. Verify socket path and permissions
4. Review [CURRENT_STATUS.md](../../CURRENT_STATUS.md) for known issues

---

**Squirrel uses Unix sockets + JSON-RPC + tarpc — no HTTP server. 🐿️**
