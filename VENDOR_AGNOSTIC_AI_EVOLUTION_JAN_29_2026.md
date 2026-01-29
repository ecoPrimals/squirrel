# Vendor-Agnostic AI Evolution - January 29, 2026

## 🎯 TRUE PRIMAL Evolution

**Date**: January 29, 2026  
**Priority**: 🔴 **HIGH** - Architectural Evolution  
**Status**: 🔄 **PLANNED**  
**Goal**: Eliminate vendor-specific adapters, achieve full AI provider agnosticism

---

## Vision

**Current State** ❌: Hardcoded vendor-specific adapters (Anthropic, OpenAI)  
**Target State** ✅: **Fully agnostic AI interface** - Large models and local models handled identically

### TRUE PRIMAL Principles

1. **No Hardcoding**: No vendor-specific code in Squirrel
2. **Capability-Based**: Discover AI providers by capabilities, not names
3. **Runtime Discovery**: All providers discovered at runtime
4. **Uniform Interface**: One unified AI interface for all models
5. **Zero Coupling**: Squirrel knows nothing about specific vendors

---

## Current Architecture Problems

### Problem 1: Vendor-Specific Adapters ❌

**Files**:
- `crates/main/src/api/ai/adapters/anthropic.rs` (321 lines)
- `crates/main/src/api/ai/adapters/openai.rs` (322 lines)

**Issues**:
- Hardcoded API endpoints (`api.anthropic.com`, `api.openai.com`)
- Vendor-specific request/response formats
- Hardcoded provider names (`"anthropic"`, `"openai"`)
- Direct coupling to vendor schemas

**Coupling Example**:
```rust
// HARDCODED - BAD!
"https://api.anthropic.com/v1/messages"
"anthropic-version": "2023-06-01"
struct AnthropicResponse { ... }
```

### Problem 2: Initialization Logic ❌

**File**: `crates/main/src/api/ai/router.rs` (lines 91-132)

**Issues**:
```rust
// HARDCODED VENDOR CHECKS - BAD!
if let Ok(Ok(adapter)) = tokio::time::timeout(
    std::time::Duration::from_secs(2),
    async { AnthropicAdapter::new() }
).await {
    // Check if available...
}
```

- Manually instantiating each vendor adapter
- Hardcoded adapter types
- No extensibility (adding new vendor = code changes)

### Problem 3: Model-Specific Logic ❌

**Issues**:
- Different request formats for each vendor
- Different response parsing for each vendor
- Different error handling for each vendor
- Cannot handle local models uniformly

---

## Target Architecture: Fully Agnostic

### Layer 1: Universal AI Interface

**One unified interface for ALL AI providers**:

```rust
/// Universal AI capability interface
#[async_trait]
pub trait AiCapability {
    /// Generate text completion
    async fn complete(
        &self,
        request: UniversalAiRequest,
    ) -> Result<UniversalAiResponse, PrimalError>;
    
    /// Check if provider is available
    async fn is_available(&self) -> bool;
    
    /// Get provider capabilities
    fn capabilities(&self) -> Vec<String>;
    
    /// Get provider metadata
    fn metadata(&self) -> ProviderMetadata;
}

/// Universal request format (vendor-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalAiRequest {
    pub prompt: String,
    pub messages: Option<Vec<Message>>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub model: Option<String>,
    pub stream: bool,
    // No vendor-specific fields!
}

/// Universal response format (vendor-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalAiResponse {
    pub text: String,
    pub provider_id: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
    pub metadata: HashMap<String, serde_json::Value>,
    // No vendor-specific fields!
}
```

### Layer 2: Capability-Based Discovery

**Discover AI providers by capabilities, not names**:

```rust
/// Discover AI providers by capability
pub async fn discover_ai_providers() -> Vec<Arc<dyn AiCapability>> {
    let mut providers = Vec::new();
    
    // 1. Query capability registry for "ai.completion"
    if let Ok(provider) = discover_capability("ai.completion").await {
        providers.push(create_universal_adapter(provider));
    }
    
    // 2. Query for "ai.chat"
    if let Ok(provider) = discover_capability("ai.chat").await {
        providers.push(create_universal_adapter(provider));
    }
    
    // 3. Query for "ai.inference"  
    if let Ok(provider) = discover_capability("ai.inference").await {
        providers.push(create_universal_adapter(provider));
    }
    
    // NO HARDCODED VENDORS!
    providers
}
```

### Layer 3: Dynamic Protocol Adaptation

**Adapt to provider's protocol at runtime**:

```rust
/// Universal AI adapter (works with ANY provider)
pub struct UniversalAiAdapter {
    socket: PathBuf,
    capabilities: Vec<String>,
    metadata: ProviderMetadata,
}

impl UniversalAiAdapter {
    async fn complete(&self, request: UniversalAiRequest) -> Result<UniversalAiResponse, PrimalError> {
        // Connect to provider
        let stream = UnixStream::connect(&self.socket).await?;
        
        // Build capability-based RPC request
        let rpc_request = json!({
            "jsonrpc": "2.0",
            "method": "ai.complete",  // OR "ai.chat", "ai.inference"
            "params": {
                "prompt": request.prompt,
                "messages": request.messages,
                "max_tokens": request.max_tokens,
                "temperature": request.temperature,
                "model": request.model,
            },
            "id": Uuid::new_v4().to_string(),
        });
        
        // Send and receive
        // ... (standard JSON-RPC flow)
        
        // Parse universal response
        let response: UniversalAiResponse = parse_universal_response(rpc_response)?;
        Ok(response)
    }
}
```

---

## Evolution Plan

### Phase 1: Immediate Fix ✅

**Status**: ✅ **COMPLETE**

- Fix HTTP response body parsing (string → JSON)
- Files: `anthropic.rs`, `openai.rs`
- Enables current integration to work

### Phase 2: Create Universal Interface (Week 1)

**Files to Create**:
1. `crates/main/src/api/ai/universal.rs` - Universal AI interface
2. `crates/main/src/api/ai/discovery.rs` - Capability-based discovery
3. `crates/main/src/api/ai/adapter.rs` - Universal adapter implementation

**Steps**:
1. Define `AiCapability` trait
2. Define `UniversalAiRequest` and `UniversalAiResponse`
3. Implement capability-based discovery
4. Create `UniversalAiAdapter`

**Success Criteria**:
- Universal interface compiles
- Discovery logic works
- Universal adapter can call any provider via Unix socket

### Phase 3: Migrate Router (Week 1-2)

**File to Modify**:
- `crates/main/src/api/ai/router.rs`

**Changes**:
```rust
// BEFORE (hardcoded):
if let Ok(Ok(adapter)) = AnthropicAdapter::new() { ... }
if let Ok(Ok(adapter)) = OpenAiAdapter::new() { ... }

// AFTER (dynamic):
let providers = discover_ai_providers().await;
for provider in providers {
    if provider.is_available().await {
        self.providers.push(provider);
    }
}
```

**Success Criteria**:
- Router discovers providers dynamically
- No hardcoded vendor names
- Works with any provider implementing `ai.complete`

### Phase 4: Deprecate Vendor Adapters (Week 2)

**Files to Deprecate**:
- `crates/main/src/api/ai/adapters/anthropic.rs` → Archive
- `crates/main/src/api/ai/adapters/openai.rs` → Archive

**Process**:
1. Mark adapters as `#[deprecated]`
2. Add deprecation warnings
3. Document migration path
4. Move to `archive/vendor_adapters_jan_2026/`

**Success Criteria**:
- Zero references to vendor-specific adapters
- All tests passing with universal adapters
- Documentation updated

### Phase 5: Clean HTTP Delegation (Week 2-3)

**Current**: Vendor adapters delegate to Songbird for HTTP  
**Target**: Universal adapter delegates to ANY `http.request` provider

**Changes**:
- Remove vendor-specific HTTP formatting
- Use universal request → HTTP mapping
- Support multiple HTTP providers (not just Songbird)

**Success Criteria**:
- HTTP delegation works for any AI provider
- No Songbird-specific code
- No Anthropic/OpenAI-specific code

### Phase 6: Local Model Support (Week 3-4)

**Goal**: Support local models identically to cloud models

**Examples**:
- Ollama (local LLM server)
- llama.cpp (local inference)
- Hugging Face Transformers (local)
- Custom fine-tuned models

**Implementation**:
```rust
// Local model primal registers with capability
{
    "capability": "ai.completion",
    "socket": "/tmp/ollama-provider.sock",
    "metadata": {
        "provider_type": "local",
        "models": ["llama2", "mistral", "codellama"]
    }
}

// Squirrel discovers it the SAME WAY as cloud providers!
// No special handling needed!
```

**Success Criteria**:
- Local models discovered automatically
- Same interface as cloud models
- Zero special-casing in Squirrel

---

## Benefits

### 1. TRUE PRIMAL Architecture ✅
- Zero hardcoding
- Runtime discovery
- Capability-based
- Vendor-agnostic

### 2. Extensibility ✅
- Add new AI providers: Zero code changes in Squirrel
- Support new models: Zero code changes in Squirrel
- New capabilities: Discovered automatically

### 3. Local Models ✅
- Local and cloud models: Same interface
- No special handling needed
- Uniform user experience

### 4. Maintainability ✅
- Less code (delete 600+ lines of vendor adapters)
- No vendor API changes to track
- Single universal interface to maintain

### 5. Testing ✅
- Test universal interface once
- Works with all providers
- Easy to mock for testing

---

## Migration Example

### Before (Vendor-Specific) ❌

```rust
// Hardcoded Anthropic adapter
let adapter = AnthropicAdapter::new().await?;
let response = adapter.generate_text(prompt).await?;
```

**Problems**:
- Hardcoded vendor
- Vendor-specific adapter
- Vendor-specific API

### After (Universal) ✅

```rust
// Discover any AI provider
let providers = discover_ai_providers().await;
let provider = providers.first().ok_or("No AI provider")?;

// Universal interface
let response = provider.complete(UniversalAiRequest {
    prompt,
    ..Default::default()
}).await?;
```

**Benefits**:
- Works with ANY provider
- No vendor knowledge needed
- Same code for cloud and local

---

## Protocol Standard

### Capability: `ai.complete`

**Method**: `ai.complete`  
**Params**:
```json
{
  "prompt": "What is 2+2?",
  "messages": [...],  // Optional for chat
  "max_tokens": 100,
  "temperature": 0.7,
  "model": "model-name"  // Optional, provider chooses default
}
```

**Response**:
```json
{
  "text": "Four.",
  "model": "claude-3-haiku",
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 2,
    "total_tokens": 12
  },
  "metadata": {}
}
```

### Capability: `ai.chat`

**Method**: `ai.chat`  
**Params**:
```json
{
  "messages": [
    {"role": "user", "content": "Hello!"},
    {"role": "assistant", "content": "Hi there!"},
    {"role": "user", "content": "What is 2+2?"}
  ],
  "max_tokens": 100,
  "temperature": 0.7,
  "model": "model-name"
}
```

**Response**: Same as `ai.complete`

---

## Testing Strategy

### 1. Mock AI Provider
Create a mock AI primal for testing:
```rust
// Mock AI provider for tests
struct MockAiProvider {
    responses: Vec<String>,
}

impl AiCapability for MockAiProvider {
    async fn complete(&self, req: UniversalAiRequest) -> Result<UniversalAiResponse, PrimalError> {
        Ok(UniversalAiResponse {
            text: self.responses[0].clone(),
            provider_id: "mock".to_string(),
            model: "mock-model".to_string(),
            usage: None,
            metadata: HashMap::new(),
        })
    }
}
```

### 2. Integration Tests
Test with real providers:
- Cloud provider (via Songbird → Anthropic/OpenAI)
- Local provider (Ollama, llama.cpp)
- Custom provider (biomeOS AI primals)

### 3. Capability Discovery Tests
Test discovery mechanisms:
- Registry discovery
- Environment variable discovery
- Socket scan discovery

---

## Timeline

| Phase | Duration | Status |
|-------|----------|--------|
| **Phase 1: Immediate Fix** | Day 1 | ✅ Complete |
| **Phase 2: Universal Interface** | Week 1 | 🔄 Planned |
| **Phase 3: Migrate Router** | Week 1-2 | 📋 Pending |
| **Phase 4: Deprecate Vendors** | Week 2 | 📋 Pending |
| **Phase 5: Clean HTTP** | Week 2-3 | 📋 Pending |
| **Phase 6: Local Models** | Week 3-4 | 📋 Pending |

**Total**: 3-4 weeks for complete evolution

---

## Success Metrics

### Code Metrics
- [ ] Zero vendor-specific adapters
- [ ] Zero hardcoded endpoints
- [ ] Zero hardcoded provider names
- [ ] -600 lines of vendor code deleted
- [ ] +300 lines of universal code added
- [ ] Net: -300 lines (cleaner!)

### Functional Metrics
- [ ] Works with Anthropic (via Songbird)
- [ ] Works with OpenAI (via Songbird)
- [ ] Works with local Ollama
- [ ] Works with custom AI primals
- [ ] Same code for all providers
- [ ] Discovery automatic (zero config)

### Architecture Metrics
- [ ] TRUE PRIMAL compliant
- [ ] Capability-based discovery
- [ ] Runtime flexibility
- [ ] Zero compile-time coupling
- [ ] Uniform interface

---

## Implementation Notes

### Backward Compatibility

During migration, support both:
- **Legacy**: Vendor-specific adapters (deprecated)
- **Modern**: Universal interface (preferred)

Users can opt-in to universal interface:
```bash
# Legacy (still works)
ANTHROPIC_API_KEY=... squirrel server

# Modern (universal)
AI_CAPABILITY_PROVIDERS=auto squirrel server
```

### Environment Variables

**Current** (vendor-specific):
- `ANTHROPIC_API_KEY`
- `OPENAI_API_KEY`
- `HTTP_REQUEST_PROVIDER_SOCKET`

**Future** (universal):
- `AI_PROVIDER_SOCKETS` - Comma-separated list
- `AI_CAPABILITY_REGISTRY` - Registry socket
- Provider-specific config via provider primal, not Squirrel

---

## Related Work

### Ecosystem Integration

**Tower Atomic Stack**:
- BearDog: Security/auth (already capability-based)
- Songbird: HTTP delegation (already capability-based)
- Neural API: Capability routing (already capability-based)
- Squirrel: AI inference (NEEDS evolution to capability-based)

**Goal**: All primals use capability-based architecture!

---

## Next Steps (Immediate)

1. ✅ **Fix parsing bug** (DONE - this commit)
2. 📝 **Review with team** - Get feedback on architecture
3. 🔨 **Create universal interface** - Start Phase 2
4. 🧪 **Test with mock provider** - Validate interface
5. 🚀 **Migrate router** - Start using universal interface

---

## Questions for Team

1. **Capability names**: Use `ai.complete`, `ai.chat`, or `ai.inference`?
2. **Protocol standard**: Define in wateringHole/ for all primals?
3. **Migration timeline**: 3-4 weeks acceptable?
4. **Backward compat**: How long to support legacy adapters?
5. **Local models**: Priority for Phase 6?

---

## Conclusion

This evolution moves Squirrel from **vendor-specific** to **fully agnostic**, aligning with TRUE PRIMAL architecture principles:

**Current** ❌:
- Hardcoded Anthropic adapter
- Hardcoded OpenAI adapter
- Vendor-specific protocols
- Cannot support local models easily

**Future** ✅:
- Universal AI interface
- Capability-based discovery
- Works with ANY AI provider
- Cloud and local models identical
- Zero vendor coupling
- TRUE PRIMAL architecture

**Impact**: Squirrel becomes a **universal AI interface**, not an Anthropic/OpenAI client!

---

**Date**: January 29, 2026  
**Priority**: 🔴 **HIGH**  
**Status**: Phase 1 ✅ Complete, Phase 2 📋 Planned  
**Vision**: TRUE PRIMAL - Zero Vendor Coupling

🚀 **Let's build a truly agnostic AI interface!**

