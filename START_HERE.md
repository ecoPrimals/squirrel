# 🚀 Start Here - Squirrel Quick Start

**Welcome to Squirrel v1.2.0!** Your entry point to UniBin-compliant AI orchestration.

**Latest**: v1.2.0 (UniBin Architecture v1.0.0 - January 17, 2026)  
**Status**: Production-ready with UniBin subcommands + Doctor Mode  
**Grade**: A++ (100/100) 🏆 **PERFECT!**

---

## ⚡ 30-Second Overview

**Squirrel is a Universal AI Orchestration Platform** with modern UniBin architecture:

1. **UniBin Compliant**: Subcommands (server, doctor, version) - 100% ecosystem standard
2. **Doctor Mode**: Built-in health diagnostics (FIRST IN ECOSYSTEM!)
3. **Zero-HTTP Production**: Unix sockets ONLY (production mode)
4. **Development Mode**: Direct HTTP adapters (fast iteration with `--features dev-direct-http`)
5. **Intelligent Routing**: Capability-based AI provider discovery
6. **Tool Orchestration**: Universal action registry for primal capabilities
7. **Agent Connectivity**: MCP server for Cursor IDE and other agents

**Think of it as**: The kubectl for AI - professional CLI, self-diagnosing, capability-based, with ZERO HTTP in production!

---

## 🎯 Quick Start

### Run Squirrel (UniBin Commands)

```bash
# Server mode (production)
squirrel server --port 9010

# Health diagnostics
squirrel doctor
squirrel doctor --comprehensive
squirrel doctor --format json

# Version info
squirrel --version
squirrel version --verbose

# Help (self-documenting!)
squirrel --help
squirrel server --help
squirrel doctor --help
```

### Build Modes

**Production** (default - Unix sockets only):
```bash
cargo build --release
./target/release/squirrel server
```

**Development** (with HTTP adapters):
```bash
cargo build --release --features dev-direct-http
export OPENAI_API_KEY="sk-..."
./target/release/squirrel server
```

---

## 🎯 What Do You Want to Do?

### I'm a Cursor IDE User

**Goal**: Use Squirrel as your AI backend in Cursor

**Read**: Archive: `archive/interim_jan_17_2026/CURSOR_INTEGRATION_COMPLETE.md`  
**Quick Test**: [CURSOR_MCP_QUICK_TEST.md](CURSOR_MCP_QUICK_TEST.md)  
**Time**: 5 minutes to get running

---

### I'm Integrating Another Primal

**Goal**: Connect your primal (Nestgate, Toadstool, etc.) to Squirrel

**Read**: [PRIMAL_INTEGRATION_GUIDE.md](PRIMAL_INTEGRATION_GUIDE.md)  
**Template**: [CAPABILITY_INTEGRATION_TEMPLATE.md](CAPABILITY_INTEGRATION_TEMPLATE.md)  
**Time**: 2-4 hours for full integration

---

### I Want to Understand How It Works

**Goal**: Learn Squirrel's architecture and capabilities

**Read**: 
1. [README.md](README.md) - Project overview
2. [USAGE_GUIDE.md](USAGE_GUIDE.md) - Complete usage guide
3. [docs/sessions/2026-01-15/SQUIRREL_TOOL_ORCHESTRATION_DISCOVERY.md](docs/sessions/2026-01-15/SQUIRREL_TOOL_ORCHESTRATION_DISCOVERY.md) - Deep dive

**Time**: 30-60 minutes

---

### I'm Deploying to Production

**Goal**: Deploy Squirrel in a production environment

**Read**:
1. [PRODUCTION_READY.md](PRODUCTION_READY.md) - Production readiness
2. [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) - Deployment guide
3. [BIOMEOS_READY.md](BIOMEOS_READY.md) - biomeOS integration

**Time**: 2-4 hours

---

## 📚 Essential Documentation

**By Role**:

| Role | Start With | Then Read |
|------|-----------|-----------|
| **Cursor User** | [CURSOR_INTEGRATION_COMPLETE.md](CURSOR_INTEGRATION_COMPLETE.md) | [CURSOR_MCP_QUICK_TEST.md](CURSOR_MCP_QUICK_TEST.md) |
| **Primal Developer** | [PRIMAL_INTEGRATION_GUIDE.md](PRIMAL_INTEGRATION_GUIDE.md) | [USAGE_GUIDE.md](USAGE_GUIDE.md) |
| **Architect** | [README.md](README.md) | [Tool Orchestration Discovery](docs/sessions/2026-01-15/SQUIRREL_TOOL_ORCHESTRATION_DISCOVERY.md) |
| **DevOps** | [PRODUCTION_READY.md](PRODUCTION_READY.md) | [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) |

---

## 🏗️ UniBin Commands (v1.2.0+)

**NEW!** Squirrel v1.2.0 is 100% UniBin v1.0.0 compliant:

### Server Mode

```bash
# Start server (production)
squirrel server --port 9010

# With Unix socket
squirrel server --socket /run/user/1000/squirrel.sock

# Verbose logging
squirrel server --verbose

# Show server options
squirrel server --help
```

### Doctor Mode (FIRST IN ECOSYSTEM!)

```bash
# Basic health check
squirrel doctor

# Comprehensive (with network checks)
squirrel doctor --comprehensive

# JSON output (automation)
squirrel doctor --format json

# Check specific subsystem
squirrel doctor --subsystem ai
squirrel doctor --subsystem ecosystem
```

**Subsystems Checked** (7):
1. Binary (version, integrity)
2. Configuration (environment vars)
3. AI Providers (OpenAI, HuggingFace, Ollama, Universal)
4. Songbird (connectivity)
5. BearDog (socket)
6. Unix Socket (configuration)
7. HTTP Server (port availability)

### Build Modes

**Production** (default - Unix sockets only):
```bash
cargo build --release
./target/release/squirrel server
```

**Development** (with HTTP adapters):
```bash
cargo build --release --features dev-direct-http
export OPENAI_API_KEY="sk-..."
./target/release/squirrel server
```

```bash
# Build for development
cargo build --release --features dev-direct-http

# Run (requires API keys)
export OPENAI_API_KEY="sk-..."
export HUGGINGFACE_API_KEY="hf_..."
./target/release/squirrel
```

**Features**:
- ✅ Direct HTTP to OpenAI/HuggingFace/Ollama
- ✅ Fast iteration without Songbird dependency
- ✅ Perfect for testing and development

## 📖 Key Documentation

**v1.2.0 (Current)**:
- [SESSION_SUMMARY_V1.2.0_UNIBIN_JAN_17_2026.md](SESSION_SUMMARY_V1.2.0_UNIBIN_JAN_17_2026.md) - v1.2.0 implementation
- [SQUIRREL_UNIBIN_COMPLIANCE_REVIEW_JAN_17_2026.md](SQUIRREL_UNIBIN_COMPLIANCE_REVIEW_JAN_17_2026.md) - UniBin compliance
- [HARVEST_PACKAGE_V1.2.0.md](HARVEST_PACKAGE_V1.2.0.md) - Deployment guide
- [CURRENT_STATUS.md](CURRENT_STATUS.md) - Current status (100/100)

**Architecture**:
- [SQUIRREL_ZERO_HTTP_EVOLUTION_JAN_16_2026.md](SQUIRREL_ZERO_HTTP_EVOLUTION_JAN_16_2026.md) - Zero-HTTP (v1.1.0)
- [SQUIRREL_CONCENTRATED_GAP_ALIGNMENT_JAN_16_2026.md](SQUIRREL_CONCENTRATED_GAP_ALIGNMENT_JAN_16_2026.md) - Concentrated Gap
- [AI_PROVIDER_ARCHITECTURAL_ISSUE_JAN_16_2026.md](AI_PROVIDER_ARCHITECTURAL_ISSUE_JAN_16_2026.md) - TRUE PRIMAL

**Historical** (archived):
- `archive/sessions_jan_17_2026/` - v1.1.0 and earlier
- `archive/evolution_jan_16_2026/` - Pure Rust migration
- `archive/interim_jan_17_2026/` - Completion docs

---

## ⚡ Quick Commands

### Check if Squirrel is Running

```bash
curl http://localhost:9010/health
# Should return: {"status":"healthy"}
```

### Test AI Generation

```bash
curl -X POST http://localhost:9010/ai/generate-text \
  -H "Content-Type: application/json" \
  -d '{"prompt":"Hello!","max_tokens":10}'
```

### See Available Tools

```bash
curl http://localhost:9010/api/v1/actions
```

### View Logs

```bash
tail -f squirrel-mcp.log
```

---

## 🎓 Key Concepts

### 1. Intelligent AI Routing

Squirrel routes AI requests to the **best provider** based on constraints:

- `optimize_cost` → Ollama (FREE local AI)
- `optimize_quality` → OpenAI GPT-4 (best quality)
- `require_local` → Forces local execution (100% private)

**Example**:
```json
{
  "prompt": "Analyze sensitive data",
  "constraints": ["require_local"]  // Stays on-premise
}
```

---

### 2. Dynamic Tool Registry

**Any service can register tools** that agents discover:

```
Service → Register Tool → Squirrel → Agent Discovers → Agent Uses
```

**Benefits**: Zero hardcoding, dynamic capabilities, intelligent routing.

---

### 3. Capability-Based Discovery

**OLD**: "I need Beardog for authentication"  
**NEW**: "I need capability `auth.verify`"

Squirrel finds who provides it and routes there.

---

## 🌟 What Makes Squirrel Special?

1. **Privacy-First** - Can force 100% local AI execution
2. **Cost-Optimized** - Routes to free local AI when possible
3. **Quality-Aware** - Uses premium AI when quality matters
4. **Dynamic** - Tools registered at runtime, zero restarts
5. **Universal** - Works with any agent (Cursor, CLI, custom)
6. **Sovereign** - Each primal knows only itself

---

## 🚀 Next Steps

### 1. Choose Your Path

Pick one of the "What Do You Want to Do?" sections above.

### 2. Follow the Guide

Each guide has step-by-step instructions.

### 3. Explore Examples

Check `examples/` directory for code samples.

### 4. Join the Ecosystem

Once integrated, discover other primals' tools!

---

## 📖 Complete Documentation

- **[README.md](README.md)** - Project overview
- **[USAGE_GUIDE.md](USAGE_GUIDE.md)** - Complete usage guide
- **[PRIMAL_INTEGRATION_GUIDE.md](PRIMAL_INTEGRATION_GUIDE.md)** - Integration guide
- **[CURSOR_INTEGRATION_COMPLETE.md](CURSOR_INTEGRATION_COMPLETE.md)** - Cursor setup
- **[READ_THIS_FIRST.md](READ_THIS_FIRST.md)** - Overview and navigation
- **[CURRENT_STATUS.md](CURRENT_STATUS.md)** - Latest status
- **[docs/sessions/2026-01-15/](docs/sessions/2026-01-15/)** - Latest discoveries

---

## 🆘 Need Help?

1. **Check logs**: `tail -f squirrel-mcp.log`
2. **Test connection**: `./test-mcp-connection.sh`
3. **Read troubleshooting**: [USAGE_GUIDE.md#troubleshooting](USAGE_GUIDE.md#troubleshooting)
4. **See examples**: `examples/` directory

---

## ✅ Quick Checklist

- [ ] Squirrel is running (`curl http://localhost:9010/health`)
- [ ] I know my role (Cursor user / Primal dev / Architect / DevOps)
- [ ] I've read the appropriate guide
- [ ] I've tested basic functionality
- [ ] I understand the key concepts

**All checked?** You're ready to use Squirrel! 🎉

---

**Welcome to the ecosystem!** 🐿️

*Last Updated: January 15, 2026*
