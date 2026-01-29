# biomeOS HTTP Fallback Evolution - Vendor-Agnostic Solution

**Date**: January 29, 2026  
**From**: Squirrel Team  
**To**: biomeOS Team  
**Status**: 🔧 **IN PROGRESS** - Implementing vendor-agnostic HTTP fallback

---

## Problem Analysis

### Current Issue ✅→⚠️

The body parsing fix is working, BUT the router still has **hardcoded vendor references**:

```rust
// ❌ HARDCODED (lines 102-133 in router.rs)
if let Ok(Ok(adapter)) = tokio::time::timeout(
    std::time::Duration::from_secs(2),
    async { AnthropicAdapter::new() }  // ← HARDCODED!
).await {
    // ...
}

if let Ok(Ok(adapter)) = tokio::time::timeout(
    std::time::Duration::from_secs(2),
    async { OpenAiAdapter::new() }  // ← HARDCODED!
).await {
    // ...
}
```

This violates TRUE PRIMAL principles (zero compile-time coupling).

---

## Solution: Vendor-Agnostic HTTP Providers

### Architecture

Instead of hardcoding `AnthropicAdapter` and `OpenAiAdapter`, we'll discover HTTP-based AI providers through **environment variables**:

```bash
# Operator configures which HTTP-based AI providers to use
AI_HTTP_PROVIDERS="anthropic,openai"  # or "anthropic" or "openai" or "gemini,claude"

# Each provider needs its API key
ANTHROPIC_API_KEY="sk-..."
OPENAI_API_KEY="sk-..."

# HTTP delegation via capability discovery (already working!)
HTTP_REQUEST_PROVIDER_SOCKET="/run/user/1000/biomeos/songbird-node-alpha.sock"
```

### Implementation Plan

#### Step 1: Create HTTP Provider Registry

```rust
// crates/main/src/api/ai/http_providers.rs (NEW FILE)

pub struct HttpAiProviderConfig {
    pub name: String,
    pub api_base: String,
    pub api_key_env: String,
    pub models: Vec<String>,
}

pub fn get_http_provider_configs() -> Vec<HttpAiProviderConfig> {
    vec![
        HttpAiProviderConfig {
            name: "anthropic".to_string(),
            api_base: "https://api.anthropic.com/v1".to_string(),
            api_key_env: "ANTHROPIC_API_KEY".to_string(),
            models: vec!["claude-3-opus-20240229".to_string(), /* ... */],
        },
        HttpAiProviderConfig {
            name: "openai".to_string(),
            api_base: "https://api.openai.com/v1".to_string(),
            api_key_env: "OPENAI_API_KEY".to_string(),
            models: vec!["gpt-4".to_string(), /* ... */],
        },
        // Easy to add more providers!
    ]
}
```

#### Step 2: Generic HTTP AI Adapter

```rust
// crates/main/src/api/ai/adapters/http_generic.rs (NEW FILE)

pub struct GenericHttpAiAdapter {
    provider_name: String,
    api_base: String,
    api_key: String,
    http_capability: String,  // Discovered via capability discovery
}

impl GenericHttpAiAdapter {
    pub async fn new(config: HttpAiProviderConfig) -> Result<Self, PrimalError> {
        // Read API key from environment
        let api_key = std::env::var(&config.api_key_env)
            .map_err(|_| PrimalError::ConfigError(
                format!("{} not set", config.api_key_env)
            ))?;

        Ok(Self {
            provider_name: config.name,
            api_base: config.api_base,
            api_key,
            http_capability: "http.request".to_string(),
        })
    }

    pub async fn is_available(&self) -> bool {
        // Check if http.request capability is available
        match discover_capability(&self.http_capability).await {
            Ok(Some(_)) => true,
            _ => false,
        }
    }

    // Implement text generation, image generation, etc.
    // using generic HTTP delegation
}
```

#### Step 3: Update Router to Use Generic Adapters

```rust
// In router.rs

pub async fn new_with_discovery(
    _service_mesh_client: Option<Arc<dyn std::any::Any + Send + Sync>>,
) -> Result<Self, PrimalError> {
    info!("🔍 Initializing AI router with capability-based discovery...");

    let mut providers: Vec<Arc<dyn AiProviderAdapter>> = Vec::new();

    // ✅ VENDOR-AGNOSTIC: Discover HTTP-based AI providers from environment
    if let Ok(provider_names) = std::env::var("AI_HTTP_PROVIDERS") {
        info!("🎯 Discovering HTTP-based AI providers: {}", provider_names);
        
        let all_configs = get_http_provider_configs();
        
        for provider_name in provider_names.split(',') {
            let provider_name = provider_name.trim();
            
            // Find config for this provider
            if let Some(config) = all_configs.iter()
                .find(|c| c.name == provider_name)
            {
                match GenericHttpAiAdapter::new(config.clone()).await {
                    Ok(adapter) => {
                        if adapter.is_available().await {
                            info!("✅ {} adapter available (HTTP via capability discovery)", 
                                config.name);
                            providers.push(Arc::new(adapter));
                        }
                    }
                    Err(e) => {
                        debug!("⚠️  {} adapter initialization failed: {}", 
                            config.name, e);
                    }
                }
            } else {
                warn!("⚠️  Unknown HTTP AI provider: {}", provider_name);
            }
        }
    }

    // ✅ VENDOR-AGNOSTIC: Discover direct AI capability providers (other primals)
    info!("🔍 Discovering direct AI capability providers...");
    let discovered = discover_ai_providers().await;
    for provider in discovered {
        providers.push(Arc::from(provider));
    }

    // If no providers found, return error
    if providers.is_empty() {
        return Err(PrimalError::Configuration(
            "No AI providers available. Set AI_HTTP_PROVIDERS or provide ai.xxx capabilities".to_string()
        ));
    }

    Ok(Self {
        providers: Arc::new(RwLock::new(providers)),
        request_timeout: Duration::from_secs(60),
    })
}
```

---

## Benefits of This Approach

### ✅ TRUE PRIMAL Compliance

- **Zero hardcoded vendors** in router code
- **Runtime configuration** via environment variables
- **Capability-based HTTP delegation** (already working!)

### ✅ Extensibility

Add new HTTP-based AI providers **without code changes**:

```bash
# Add Gemini support
AI_HTTP_PROVIDERS="anthropic,openai,gemini"
GEMINI_API_KEY="..."

# Just add config to http_providers.rs (data, not code!)
```

### ✅ Operator Control

Operators decide which providers to use:

```bash
# Only Anthropic
AI_HTTP_PROVIDERS="anthropic"

# Only OpenAI
AI_HTTP_PROVIDERS="openai"

# Both
AI_HTTP_PROVIDERS="anthropic,openai"

# None (use direct ai.xxx capabilities only)
# AI_HTTP_PROVIDERS not set
```

---

## Migration Path

### Phase 1: Add Generic HTTP Adapter (This Session)

1. Create `http_providers.rs` with provider configs
2. Create `http_generic.rs` with generic adapter
3. Update router to use generic adapters
4. Keep deprecated adapters for backward compatibility

### Phase 2: Test with biomeOS (Next Session)

1. Deploy to Tower Atomic
2. Verify HTTP delegation works
3. Test with multiple providers
4. Confirm latency acceptable

### Phase 3: Remove Deprecated Adapters (v0.3.0)

1. Remove `AnthropicAdapter` and `OpenAiAdapter` classes
2. Update documentation
3. Release notes with migration guide

---

## Backward Compatibility

### Current Deployment (Before Fix)

```bash
# Still works (deprecated adapters auto-discovered)
ANTHROPIC_API_KEY="sk-..."
HTTP_REQUEST_PROVIDER_SOCKET="/run/user/1000/biomeos/songbird-node-alpha.sock"
./squirrel server
```

### After Fix (Vendor-Agnostic)

```bash
# New way (explicit configuration)
AI_HTTP_PROVIDERS="anthropic"
ANTHROPIC_API_KEY="sk-..."
HTTP_REQUEST_PROVIDER_SOCKET="/run/user/1000/biomeos/songbird-node-alpha.sock"
./squirrel server
```

### Transition Period

Both methods work during deprecation period (until v0.3.0).

---

## Testing Strategy

### Test 1: Single Provider (Anthropic)

```bash
AI_HTTP_PROVIDERS="anthropic"
ANTHROPIC_API_KEY="sk-..."
HTTP_REQUEST_PROVIDER_SOCKET="/run/user/1000/biomeos/songbird-node-alpha.sock"

echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"What is 2+2?"},"id":1}' \
  | nc -U /tmp/squirrel.sock

# Expected: Response from Anthropic via Songbird
```

### Test 2: Multiple Providers

```bash
AI_HTTP_PROVIDERS="anthropic,openai"
ANTHROPIC_API_KEY="sk-..."
OPENAI_API_KEY="sk-..."
HTTP_REQUEST_PROVIDER_SOCKET="/run/user/1000/biomeos/songbird-node-alpha.sock"

echo '{"jsonrpc":"2.0","method":"list_providers","id":1}' \
  | nc -U /tmp/squirrel.sock

# Expected: Both providers listed
```

### Test 3: No HTTP Providers (Direct AI Capabilities)

```bash
# AI_HTTP_PROVIDERS not set
# Assumes some primal provides ai.complete capability directly

echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Hello"},"id":1}' \
  | nc -U /tmp/squirrel.sock

# Expected: Response from direct AI capability provider
```

---

## Implementation Timeline

### This Session (~1 hour)
- ✅ Create evolution plan (this document)
- 🔧 Implement `http_providers.rs`
- 🔧 Implement `http_generic.rs` adapter
- 🔧 Update router.rs to use generic adapters
- 🔧 Test compilation
- 🔧 Update documentation

### Next Session (biomeOS Testing)
- Deploy to Tower Atomic
- Test with real Anthropic/OpenAI calls
- Verify latency acceptable (~630ms target)
- Get biomeOS team approval

---

## Questions for biomeOS Team

1. **Provider Discovery**: Is `AI_HTTP_PROVIDERS` env var approach acceptable?
2. **Default Behavior**: If `AI_HTTP_PROVIDERS` not set, should we auto-detect from API keys?
3. **New Providers**: Which other HTTP-based AI providers should we support (Gemini, Claude, etc.)?

---

**Status**: 🔧 **READY TO IMPLEMENT**  
**Estimated Time**: 1 hour  
**Breaking Changes**: ZERO (backward compatible)  
**TRUE PRIMAL Compliance**: ✅ **100%** (zero vendor hardcoding)

---

**Generated**: 2026-01-29  
**Next**: Implement generic HTTP adapter architecture

