# 🚀 Start Here - Squirrel Quick Start

**Welcome to Squirrel!** This is your entry point.

---

## ⚡ 30-Second Overview

**Squirrel is a Universal Tool Orchestration Platform** that:

1. **Routes AI intelligently** - OpenAI, Ollama (local), HuggingFace
2. **Orchestrates tools** - Any service can register capabilities
3. **Connects agents** - MCP server for Cursor IDE and other agents
4. **Enables discovery** - Capability-based, zero hardcoding

**Think of it as**: The universal adapter that lets agents DO things across the ecoPrimals ecosystem.

---

## 🎯 What Do You Want to Do?

### I'm a Cursor IDE User

**Goal**: Use Squirrel as your AI backend in Cursor

**Read**: [CURSOR_INTEGRATION_COMPLETE.md](CURSOR_INTEGRATION_COMPLETE.md)  
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
