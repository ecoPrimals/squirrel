# ⚠️ AI Provider Architecture - Hardcoding Issue Identified

**Date**: January 16, 2026  
**Severity**: Medium-High (Architectural violation)  
**Impact**: Violates TRUE PRIMAL philosophy

---

## 🎯 The Problem

### Current Implementation (WRONG)

Our AI provider system **hardcodes specific vendors**, violating the TRUE PRIMAL "infant pattern":

```rust
// crates/main/src/api/ai/router.rs:33-61
pub async fn new() -> Result<Self, PrimalError> {
    let mut providers: Vec<Arc<dyn AiProviderAdapter>> = Vec::new();

    // ❌ HARDCODED: OpenAI
    if let Ok(openai) = OpenAIAdapter::new() {
        providers.push(Arc::new(openai));
    }

    // ❌ HARDCODED: Ollama
    let ollama = OllamaAdapter::new();
    if ollama.is_available().await {
        providers.push(Arc::new(ollama));
    }

    // ❌ HARDCODED: HuggingFace
    let huggingface = HuggingFaceAdapter::new();
    if huggingface.is_available().await {
        providers.push(Arc::new(huggingface));
    }
    
    // ...
}
```

### What's Wrong?

1. **Hardcoded Vendor Names**: Explicitly initializes OpenAI, Ollama, HuggingFace
2. **No Discovery**: Doesn't query Songbird for AI capabilities
3. **Can't Use Primal AI**: Can't discover if Toadstool provides GPU inference
4. **Can't Use Stored Models**: Can't use models from NestGate storage
5. **Not Extensible**: Adding new providers requires code changes

---

## ✅ What It Should Be (TRUE PRIMAL)

### Capability-Based Discovery

```rust
// CORRECT APPROACH (not yet implemented)
pub async fn new(songbird_client: Arc<SongbirdClient>) -> Result<Self, PrimalError> {
    let mut providers: Vec<Arc<dyn AiProviderAdapter>> = Vec::new();

    // ✅ Discover text generation providers
    let text_gen_providers = songbird_client
        .discover_by_capability("ai:text-generation")
        .await?;
    
    for provider_socket in text_gen_providers {
        // Connect via Unix socket to ANY primal providing AI
        let adapter = UniversalAiAdapter::new(provider_socket);
        providers.push(Arc::new(adapter));
    }
    
    // ✅ Discover image generation providers
    let image_gen_providers = songbird_client
        .discover_by_capability("ai:image-generation")
        .await?;
    
    for provider_socket in image_gen_providers {
        let adapter = UniversalAiAdapter::new(provider_socket);
        providers.push(Arc::new(adapter));
    }
    
    Ok(Self { providers, .. })
}
```

### Example: Toadstool Provides GPU Inference

```
Squirrel: "I need text generation"
    ↓
Songbird: "These primals have ai:text-generation:"
    - toadstool (capability: ai:text-generation:gpu, cost: 0, quality: high)
    - nestgate (capability: ai:text-generation:stored, cost: 0, quality: medium)
    - external:openai (capability: ai:text-generation:cloud, cost: 0.002, quality: premium)
    ↓
Squirrel: Routes based on constraints
    - require_local=true → Toadstool or NestGate
    - optimize_quality=true → OpenAI
    - optimize_cost=true → Toadstool (GPU, free)
```

---

## 📊 Impact Analysis

### Current Violations

| Issue | Severity | TRUE PRIMAL Violation |
|-------|----------|----------------------|
| Hardcoded vendor names | High | ❌ Yes - infant pattern |
| No Songbird discovery | High | ❌ Yes - capability-based |
| Can't use primal AI | Medium | ❌ Yes - ecosystem isolation |
| External vendor only | Medium | ❌ Yes - not universal |

### What We CAN'T Do Currently

1. ❌ Use Toadstool's barraCUDA for GPU-accelerated inference
2. ❌ Use models stored in NestGate
3. ❌ Discover AI providers from other primals
4. ❌ Register new vendors without code changes
5. ❌ Let other primals provide AI services

---

## 🔧 Proposed Solution

### Phase 1: Universal AI Adapter

Create a **capability-based** adapter that works with ANY AI provider:

```rust
/// Universal AI adapter - works with any provider via capability
pub struct UniversalAiAdapter {
    socket_path: PathBuf,
    capability: String,  // e.g., "ai:text-generation"
    metadata: ProviderMetadata,
}

impl UniversalAiAdapter {
    /// Create from Songbird discovery
    pub async fn from_discovery(
        capability: &str,
        discovery_result: DiscoveredService,
    ) -> Result<Self, PrimalError> {
        Ok(Self {
            socket_path: discovery_result.socket_path,
            capability: capability.to_string(),
            metadata: discovery_result.metadata,
        })
    }
}

#[async_trait]
impl AiProviderAdapter for UniversalAiAdapter {
    async fn generate_text(&self, request: TextGenerationRequest) 
        -> Result<TextGenerationResponse, PrimalError> 
    {
        // Send JSON-RPC request via Unix socket
        let response = self.send_rpc_request("ai.generate_text", request).await?;
        Ok(response)
    }
}
```

### Phase 2: Update AiRouter

```rust
pub async fn new(songbird: Arc<SongbirdClient>) -> Result<Self, PrimalError> {
    let mut providers: Vec<Arc<dyn AiProviderAdapter>> = Vec::new();

    // Discover AI providers via Songbird
    let ai_capabilities = vec![
        "ai:text-generation",
        "ai:image-generation",
        "ai:embedding",
    ];
    
    for capability in ai_capabilities {
        let discovered = songbird.discover_by_capability(capability).await?;
        
        for service in discovered {
            let adapter = UniversalAiAdapter::from_discovery(capability, service).await?;
            providers.push(Arc::new(adapter));
        }
    }
    
    // Also register external vendors (if configured)
    if let Some(external) = self.load_external_vendors() {
        providers.extend(external);
    }
    
    Ok(Self { providers, .. })
}
```

### Phase 3: Capability Registration

Any primal can register AI capabilities:

```rust
// In Toadstool (GPU compute primal)
songbird.register_capability(CapabilityRegistration {
    primal_id: "toadstool",
    capability: "ai:text-generation:gpu",
    socket_path: "/run/user/1000/toadstool.sock",
    metadata: {
        "backend": "barraCUDA",
        "quality": "high",
        "cost": 0.0,
        "local": true,
        "max_tokens": 4096,
    },
}).await?;

// In NestGate (storage primal)
songbird.register_capability(CapabilityRegistration {
    primal_id: "nestgate",
    capability: "ai:text-generation:stored",
    socket_path: "/run/user/1000/nestgate.sock",
    metadata: {
        "backend": "stored-models",
        "quality": "medium",
        "cost": 0.0,
        "local": true,
    },
}).await?;
```

---

## 🎯 Benefits of TRUE PRIMAL Approach

### 1. Ecosystem Integration
- ✅ Toadstool can provide GPU-accelerated AI
- ✅ NestGate can serve stored models
- ✅ Any primal can offer AI capabilities

### 2. Zero Hardcoding
- ✅ No vendor names in code
- ✅ Discovery at runtime
- ✅ Infant pattern compliance

### 3. Flexibility
- ✅ Add providers without code changes
- ✅ Capability-based routing
- ✅ True universal adapter

### 4. Privacy & Cost
- ✅ Prefer local primals (Toadstool GPU)
- ✅ Fallback to cloud when needed
- ✅ Optimize based on user constraints

---

## 📋 Implementation Plan

### Week 1: Foundation
1. Create `UniversalAiAdapter` struct
2. Implement Unix socket JSON-RPC communication
3. Add Songbird capability discovery

### Week 2: Integration
4. Update `AiRouter` to use discovery
5. Keep legacy adapters as fallback (transitional)
6. Add comprehensive tests

### Week 3: Ecosystem
7. Document AI capability protocol
8. Share with Toadstool team (GPU integration)
9. Share with NestGate team (model storage)

### Week 4: Cleanup
10. Deprecate hardcoded adapters
11. Remove vendor-specific code
12. Fully capability-based

---

## ⚠️ Current Workaround (Interim)

Until we implement TRUE PRIMAL approach, the current adapters work but:

1. **Accept the technical debt**: We know it's not ideal
2. **Document the limitation**: This README serves that purpose
3. **Plan the evolution**: Roadmap above
4. **Don't make it worse**: Don't add more hardcoded vendors

---

## 🤝 Related Work

### Toadstool Integration
- Toadstool should expose `ai:text-generation:gpu` capability
- Uses barraCUDA for acceleration
- Squirrel discovers via Songbird, not hardcoded

### NestGate Integration
- NestGate should expose `ai:text-generation:stored` capability
- Serves pre-downloaded models
- Squirrel discovers dynamically

### External Vendors
- Can still support OpenAI, HuggingFace, etc.
- But discovered via capability registry, not hardcoded
- Configured externally, loaded dynamically

---

## 📚 TRUE PRIMAL Philosophy

> **Infant Pattern**: Each primal only knows itself at startup and discovers others at runtime.

Current AI provider system violates this by:
- Knowing about OpenAI, Ollama, HuggingFace at compile time
- Not discovering AI capabilities from ecosystem
- Not allowing primals to provide AI services

**This needs to evolve to capability-based discovery.** ✅

---

## ✅ Acceptance Criteria

For TRUE PRIMAL compliance, we need:

1. ✅ Zero hardcoded vendor names in router initialization
2. ✅ Songbird capability discovery for AI providers
3. ✅ Unix socket communication with discovered providers
4. ✅ Any primal can register AI capabilities
5. ✅ External vendors configured, not hardcoded
6. ✅ Constraint-based routing works with discovered providers

---

**Status**: ⚠️ **IDENTIFIED - NOT YET FIXED**  
**Priority**: Medium-High (architectural debt)  
**Timeline**: Week 2-3 (after enhanced routing)

---

*This issue was identified during TRUE PRIMAL philosophy review on Jan 16, 2026.*  
*Thank you for catching this architectural violation!* 🙏

