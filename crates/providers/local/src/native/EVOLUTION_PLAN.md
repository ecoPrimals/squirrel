# Native AI Provider Evolution Plan

## Current State
The native AI provider has **5 placeholder functions** returning fake data:
- `generate_embedding()` → Returns `[0.5; 768]`
- `classify_text()` → Uses text length as confidence  
- `summarize_text()` → Returns first/last sentences
- `translate_text()` → Uppercases strings
- `answer_question()` → Concatenates text

## Deep Debt Solution: Capability-Based Evolution

### Philosophy
Following Squirrel's **sovereignty principle**: Each primal has self-knowledge only and discovers others at runtime.

### Evolution Strategy

#### Phase 1: Remove Fake Implementations ✅
Replace placeholder functions with capability discovery:
```rust
// OLD (Fake):
pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
    Ok(vec![0.5; 768]) // FAKE!
}

// NEW (Capability-Based):
pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
    // Discover AI service with embedding capability
    let service = self.discover_capability("text-embedding").await?;
    service.generate_embedding(text).await
}
```

#### Phase 2: Implement Discovery Protocol
```rust
pub struct NativeAIProvider {
    config: NativeAIConfig,
    state: ProviderState,
    discovery: Arc<CapabilityDiscovery>,  // NEW
}

impl NativeAIProvider {
    async fn discover_capability(&self, capability: &str) -> Result<AIService> {
        // Use mDNS, service registry, or configuration to find services
        // Example: "text-embedding" → finds OpenAI, Anthropic, local models
        self.discovery.find_service_with_capability(capability).await
    }
}
```

#### Phase 3: Graceful Degradation
```rust
pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
    match self.discover_capability("text-embedding").await {
        Ok(service) => service.generate_embedding(text).await,
        Err(_) => {
            // Graceful degradation: use simple bag-of-words or return error
            Err(ProviderError::NoCapability("text-embedding".to_string()))
        }
    }
}
```

### Benefits
1. **No hardcoding** - Discovers AI services at runtime
2. **Vendor agnostic** - Works with any AI provider (OpenAI, Anthropic, local models)
3. **Sovereignty compliant** - No assumed dependencies
4. **Production ready** - Real AI integration, not fake data
5. **Testable** - Can mock discovery in tests

### Implementation Steps
1. ✅ Document evolution plan (this file)
2. ⏳ Create CapabilityDiscovery struct
3. ⏳ Implement service discovery protocol
4. ⏳ Replace 5 placeholder functions
5. ⏳ Add graceful degradation
6. ⏳ Add comprehensive tests
7. ⏳ Add configuration for discovery

### Timeline
- Discovery protocol: 8 hours
- Function evolution: 4 hours  
- Testing: 4 hours
- **Total**: ~16 hours

### Status
🔄 **IN PROGRESS** - Evolution strategy defined, implementation ready to begin

