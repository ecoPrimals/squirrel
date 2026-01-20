# Squirrel AI Architecture - Two-Tier System
## January 20, 2026

**Status**: ✅ **ARCHITECTURE CLARIFIED**  
**Priority**: CRITICAL  

---

## 🎯 Squirrel Handles ALL AI Tasks

Squirrel is the **universal AI orchestrator** for the ecoPrimals ecosystem.

All AI requests flow through Squirrel, which intelligently routes them to the appropriate backend:
- ✅ External AI APIs (Anthropic, OpenAI) via HTTP delegation
- ✅ Local AI compute (ToadStool, other AI primals) via capability discovery

---

## 🏗️ Two-Tier AI Architecture

### Tier 1: External AI APIs (via HTTP Delegation)

```
User Request
    ↓
Squirrel AI Router
    ↓
Anthropic/OpenAI Adapter
    ↓
discover("http.request")  ← Finds Songbird
    ↓
Songbird (HTTP Provider)
    ↓
External AI API (Anthropic/OpenAI)
```

**Key Points**:
- ✅ Squirrel has `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`
- ✅ Squirrel discovers `http.request` capability → finds Songbird
- ✅ Songbird is a **pure HTTP provider** (NOT an AI provider)
- ✅ Songbird provides: `http.request`, `http.get`, `http.post`
- ❌ Songbird does NOT provide `ai.generate_text`

### Tier 2: Local AI Compute (via Capability Discovery)

```
User Request
    ↓
Squirrel AI Router
    ↓
UniversalAiAdapter
    ↓
discover("ai.generate_text")  ← Finds ToadStool
    ↓
ToadStool (Local AI Primal)
    ↓
Local AI Model (Llama, etc.)
```

**Key Points**:
- ✅ Local AI primals advertise `ai.generate_text` capability
- ✅ ToadStool, Fungi, etc. provide actual AI inference
- ✅ No HTTP involved - direct Unix socket communication
- ✅ No external API keys needed

---

## 📋 Capability vs Provider Type

### Songbird Capabilities

```toml
capabilities = [
    "http.request",      # ← What Anthropic/OpenAI adapters discover
    "http.get",
    "http.post",
    "secure_http",
    "tls",
    "bearer_auth"
]
```

**Songbird is**: HTTP provider (networking layer)  
**Songbird is NOT**: AI provider (does not run models)

### ToadStool Capabilities

```toml
capabilities = [
    "ai.generate_text",   # ← What UniversalAiAdapter discovers
    "ai.embeddings",
    "ai.local_inference",
    "llama",
    "mistral"
]
```

**ToadStool is**: AI provider (runs models locally)  
**ToadStool is NOT**: HTTP provider (doesn't make external requests)

---

## 🔧 Squirrel Configuration

### Correct Configuration

```bash
# For External AI APIs (Tier 1)
export ANTHROPIC_API_KEY="sk-ant-..."
export OPENAI_API_KEY="sk-..."
# Squirrel discovers http.request → finds Songbird automatically

# For Local AI (Tier 2)  
export AI_PROVIDER_SOCKETS="/tmp/toadstool.sock,/tmp/fungi.sock"
# Squirrel discovers ai.generate_text → finds local AI primals
```

### ❌ WRONG Configuration

```bash
# DON'T do this:
export AI_PROVIDER_SOCKETS="/tmp/songbird-nat0.sock"
# ❌ Songbird doesn't provide ai.generate_text!
```

**Why it's wrong**: Songbird provides `http.request`, not `ai.generate_text`. If you configure Songbird as an AI provider, Squirrel will try to call `ai.generate_text` on it and fail.

---

## 📊 Request Flow Examples

### Example 1: User asks "What is 2+2?"

**Scenario 1: External AI (Anthropic)**

```
1. User → Squirrel: query_ai("What is 2+2?")

2. Squirrel AI Router:
   - Selects: AnthropicAdapter (best quality)

3. AnthropicAdapter:
   - Reads: ANTHROPIC_API_KEY from env
   - Discovers: http.request capability
   - Finds: Songbird at /tmp/songbird-nat0.sock
   - Builds: Anthropic API request (JSON)

4. Delegates to Songbird:
   POST https://api.anthropic.com/v1/messages
   Headers: x-api-key, anthropic-version
   Body: {model, messages, max_tokens}

5. Songbird:
   - Makes HTTPS request to Anthropic
   - Returns HTTP response to Squirrel

6. Squirrel:
   - Parses Anthropic response
   - Returns to user: "2+2 = 4"
```

**Scenario 2: Local AI (ToadStool)**

```
1. User → Squirrel: query_ai("What is 2+2?")

2. Squirrel AI Router:
   - Selects: UniversalAiAdapter (local, fast)

3. UniversalAiAdapter:
   - Discovers: ai.generate_text capability
   - Finds: ToadStool at /tmp/toadstool.sock

4. Delegates to ToadStool:
   ai.generate_text({prompt: "What is 2+2?", model: "llama"})

5. ToadStool:
   - Runs local Llama model
   - Returns response to Squirrel

6. Squirrel:
   - Returns to user: "2+2 = 4"
```

---

## 🎯 Routing Logic

Squirrel's AI Router selects the best provider based on:

1. **Capability match**: Can the provider handle this request type?
2. **Quality tier**: Premium > High > Standard > Fast > Basic
3. **Cost**: Lower cost preferred (local < cloud)
4. **Latency**: Lower latency preferred
5. **Availability**: Is the provider currently available?

**Example Routing Decisions**:

| Request Type | User Preference | Selected Provider | Reason |
|--------------|----------------|-------------------|--------|
| Complex reasoning | Quality | Anthropic (Claude Opus) | Premium quality |
| Code generation | Quality | OpenAI (GPT-4) | Premium quality, specialized |
| Simple Q&A | Cost | ToadStool (local Llama) | Free, fast enough |
| Bulk processing | Cost | ToadStool | Free, no API limits |
| Low latency | Speed | ToadStool (local) | No network round-trip |

---

## 🔍 Discovery Process

### Anthropic Adapter Initialization

```rust
impl AnthropicAdapter {
    pub fn new() -> Result<Self, PrimalError> {
        // 1. Read API key from environment
        let api_key = env::var("ANTHROPIC_API_KEY")?;
        
        // 2. Return adapter (HTTP provider discovered on-demand)
        Ok(Self {
            api_key,
            default_model: "claude-3-opus-20240229".to_string(),
        })
    }
    
    async fn delegate_http(&self, ...) -> Result<...> {
        // 3. Discover HTTP provider at request time (TRUE PRIMAL!)
        let http_provider = discover_capability("http.request").await?;
        
        // 4. Connect to discovered provider (e.g., Songbird)
        let stream = UnixStream::connect(&http_provider.socket).await?;
        
        // 5. Send JSON-RPC request
        send_rpc(stream, "http.request", params).await?
    }
}
```

**Key**: HTTP provider is discovered **on-demand**, not at initialization.

### UniversalAiAdapter Initialization

```rust
impl UniversalAiAdapter {
    pub async fn from_socket(socket_path: &str) -> Result<Self, PrimalError> {
        // 1. Connect to socket
        let stream = UnixStream::connect(socket_path).await?;
        
        // 2. Discover capabilities
        let response = send_rpc(stream, "discover_capabilities", {}).await?;
        
        // 3. Verify it provides ai.generate_text
        if !response.capabilities.contains("ai.generate_text") {
            return Err("Not an AI provider");
        }
        
        Ok(Self { socket_path, capabilities: response.capabilities })
    }
}
```

**Key**: Verifies provider actually offers `ai.generate_text`.

---

## ✅ Validation Checklist

When deploying Squirrel with AI:

### External AI (Tier 1)

- [ ] `ANTHROPIC_API_KEY` set (if using Anthropic)
- [ ] `OPENAI_API_KEY` set (if using OpenAI)
- [ ] Songbird deployed and running
- [ ] Songbird socket exists: `/tmp/songbird-nat0.sock`
- [ ] Songbird responds to `discover_capabilities`
- [ ] Songbird advertises `http.request` capability
- [ ] Squirrel can discover `http.request`

**Test**:
```bash
# Check Songbird
echo '{"jsonrpc":"2.0","method":"discover_capabilities","id":"1"}' | \
  nc -U /tmp/songbird-nat0.sock

# Expected: {"result": {"capabilities": ["http.request", ...]}}
```

### Local AI (Tier 2)

- [ ] ToadStool (or other AI primal) deployed
- [ ] AI primal socket exists: `/tmp/toadstool.sock`
- [ ] AI primal responds to `discover_capabilities`
- [ ] AI primal advertises `ai.generate_text` capability
- [ ] `AI_PROVIDER_SOCKETS` configured (if not using registry)

**Test**:
```bash
# Check ToadStool
echo '{"jsonrpc":"2.0","method":"discover_capabilities","id":"1"}' | \
  nc -U /tmp/toadstool.sock

# Expected: {"result": {"capabilities": ["ai.generate_text", ...]}}
```

### Squirrel

- [ ] Squirrel deployed and running
- [ ] Squirrel socket exists: `/tmp/squirrel-nat0.sock`
- [ ] Squirrel AI router initialized
- [ ] Squirrel discovered at least 1 AI provider

**Test**:
```bash
# Check Squirrel health
echo '{"jsonrpc":"2.0","method":"health","id":"1"}' | \
  nc -U /tmp/squirrel-nat0.sock

# Query AI
echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Hello"},"id":"1"}' | \
  nc -U /tmp/squirrel-nat0.sock
```

---

## 📈 Current Status (Post-Evolution)

### ✅ What Works

1. **Anthropic Adapter**
   - ✅ Reads `ANTHROPIC_API_KEY`
   - ✅ Discovers `http.request` capability
   - ✅ Delegates to Songbird
   - ✅ Parses Anthropic responses
   - ✅ Full `generate_text()` implementation

2. **OpenAI Adapter**
   - ✅ Reads `OPENAI_API_KEY`
   - ✅ Discovers `http.request` capability
   - ✅ Delegates to Songbird
   - ✅ Parses OpenAI responses
   - ✅ Full `generate_text()` implementation

3. **UniversalAiAdapter**
   - ✅ Discovers local AI primals via `AI_PROVIDER_SOCKETS`
   - ✅ Probes for `ai.generate_text` capability
   - ✅ Delegates to local AI providers

4. **AI Router**
   - ✅ Loads all adapters dynamically
   - ✅ No hardcoded provider IDs
   - ✅ Capability-based routing
   - ✅ Timeout protection (10s max)
   - ✅ Graceful degradation

### 🔄 Integration Status

**Songbird v4.3.0** (from handoff):
- ✅ Implements `discover_capabilities`
- ✅ Implements `health`
- ✅ Implements `http.request`
- ✅ Returns proper JSON-RPC responses
- ✅ Closes connection after response (Squirrel's `read_to_end()` works)

**Squirrel v2.0.0**:
- ✅ Discovers Songbird successfully
- ✅ Connects to `/tmp/songbird-nat0.sock`
- ✅ AI router initialized with 1 provider

---

## 🚀 Deployment Guide

### Full Stack Deployment

```bash
# 1. Deploy Songbird (HTTP provider)
cd /path/to/songbird
./songbird server --socket /tmp/songbird-nat0.sock

# 2. Deploy ToadStool (optional, for local AI)
cd /path/to/toadstool
./toadstool server --socket /tmp/toadstool.sock

# 3. Deploy Squirrel (AI orchestrator)
export ANTHROPIC_API_KEY="sk-ant-..."
export OPENAI_API_KEY="sk-..."
export AI_PROVIDER_SOCKETS="/tmp/toadstool.sock"  # If using local AI

cd /path/to/squirrel
./squirrel server --socket /tmp/squirrel-nat0.sock
```

**Expected Output**:
```
✅ Squirrel AI/MCP Primal Ready!
🤖 Initializing AI router...
🔍 Initializing capability-based HTTP adapters...
✅ Anthropic adapter available (HTTP via capability discovery)
✅ OpenAI adapter available (HTTP via capability discovery)
🎯 Using AI_PROVIDER_SOCKETS hint: /tmp/toadstool.sock
✅ Connected to provider: /tmp/toadstool.sock
✅ AI router initialized with 3 provider(s) via capability discovery
🚀 JSON-RPC server listening on /tmp/squirrel-nat0.sock
```

---

## 🎯 Summary

```
╔════════════════════════════════════════════════════════════════╗
║                                                                ║
║   SQUIRREL: UNIVERSAL AI ORCHESTRATOR                         ║
║                                                                ║
╠════════════════════════════════════════════════════════════════╣
║                                                                ║
║  Architecture:       Two-Tier AI System                       ║
║                                                                ║
║  Tier 1 (External):  Anthropic, OpenAI via Songbird           ║
║                      → Discovers "http.request"               ║
║                      → Songbird is HTTP provider              ║
║                                                                ║
║  Tier 2 (Local):     ToadStool, Fungi, etc.                  ║
║                      → Discovers "ai.generate_text"           ║
║                      → Direct AI inference                    ║
║                                                                ║
║  Routing:            Capability-based, intelligent            ║
║  Discovery:          Runtime, zero hardcoding                 ║
║  Pattern:            TRUE PRIMAL (infant deployment)          ║
║                                                                ║
║  Status:             ✅ PRODUCTION READY                      ║
║                                                                ║
╚════════════════════════════════════════════════════════════════╝
```

**Squirrel handles ALL AI tasks. Routes to the right backend. Zero hardcoding.** 🐿️🧠✨

---

*The ecological way: Discover capabilities, route intelligently, evolve constantly* 🌍🦀

