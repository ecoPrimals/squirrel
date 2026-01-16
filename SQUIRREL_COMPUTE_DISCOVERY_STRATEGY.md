# 🐿️ Squirrel × Compute Discovery: TRUE PRIMAL Pattern

**Date**: January 15, 2026  
**Pattern**: Capability-Based Discovery (NO Tight Coupling)  
**Reference**: NUCLEUS secure discovery + Neural API client patterns

---

## 🎯 THE CORRECTED VISION

### What We Were Doing WRONG ❌

```rust
// BAD: Tight coupling - Squirrel directly using barraCUDA
use toadstool::barracuda::BarraCudaAdapter;

impl Squirrel {
    fn new() -> Self {
        Self {
            gpu: BarraCudaAdapter::new(), // ❌ HARDCODED DEPENDENCY
        }
    }
}
```

**Problems**:
- Squirrel depends on toadstool implementation details
- Cannot work with other compute providers
- Violates TRUE PRIMAL sovereignty
- Creates tight coupling

---

### What We Should Do RIGHT ✅

```rust
// GOOD: Capability-based discovery
use biomeos_core::clients::transport::TransportClient;

impl Squirrel {
    async fn discover_compute(&self) -> Result<ComputeClient> {
        // Discover ANY primal providing "compute:gpu" capability
        let transport = TransportClient::discover_with_preference(
            "compute:gpu",  // Capability, NOT primal name!
            &self.family_id,
            TransportPreference::UnixSocket,
        ).await?;
        
        Ok(ComputeClient { transport })
    }
}
```

**Benefits**:
- ✅ Squirrel knows NOTHING about toadstool/barraCUDA
- ✅ Works with ANY compute provider
- ✅ TRUE PRIMAL sovereignty
- ✅ Zero hardcoding

---

## 🏗️ PROPER ARCHITECTURE

### Layer 1: toadstool Announces GPU Capability

**File**: `ecoPrimals/phase1/toadstool` (existing primal)

```rust
// toadstool announces it provides GPU compute
pub async fn announce_capabilities() -> Result<()> {
    let songbird = SongbirdClient::discover(&family_id).await?;
    
    songbird.announce_capabilities(json!({
        "primal": "toadstool",
        "node_id": "node-alpha",
        "family_id": "nat0",
        "capabilities": [
            {
                "type": "compute:gpu",
                "version": "1.0",
                "operations": 105,  // barraCUDA operations
                "methods": [
                    "inference.text_generation",
                    "inference.image_generation",
                    "training.fine_tune",
                    "compute.matmul",
                    "compute.convolution"
                ],
                "metadata": {
                    "vendor_agnostic": true,
                    "gpu_types": ["nvidia", "amd", "intel", "apple"],
                    "operations_validated": 105,
                    "fp32_validated": true
                }
            },
            {
                "type": "compute:cpu",
                "version": "1.0",
                "methods": ["workload.execute", "container.run"]
            }
        ],
        "endpoints": [
            {"type": "unix_socket", "path": "/run/user/1000/toadstool-nat0.sock"}
        ]
    })).await?;
    
    Ok(())
}
```

**Key Points**:
- toadstool announces "compute:gpu" capability
- Songbird indexes it
- NO mention of Squirrel
- toadstool only knows ITSELF

---

### Layer 2: Squirrel Discovers GPU Compute

**File**: `crates/main/src/discovery/compute_discovery.rs` (NEW)

```rust
//! Capability-based compute discovery for Squirrel
//! 
//! Discovers ANY primal providing compute capabilities (GPU, CPU, TPU, etc.)
//! without knowing their names or implementation details.

use anyhow::{Context, Result};
use biomeos_core::clients::songbird::SongbirdClient;
use biomeos_core::clients::transport::TransportClient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Compute discovery client
pub struct ComputeDiscovery {
    songbird: SongbirdClient,
    family_id: String,
}

impl ComputeDiscovery {
    /// Create new compute discovery
    pub async fn new(family_id: impl Into<String>) -> Result<Self> {
        let family_id = family_id.into();
        let songbird = SongbirdClient::discover(&family_id)
            .await
            .context("Songbird required for discovery")?;
        
        Ok(Self { songbird, family_id })
    }
    
    /// Discover primals with GPU compute capability
    pub async fn discover_gpu_compute(&self) -> Result<Vec<ComputeProvider>> {
        // Ask Songbird for any primal with "compute:gpu" capability
        let announcements = self.songbird
            .discover_by_capability("compute:gpu")
            .await
            .context("Failed to discover GPU compute providers")?;
        
        let mut providers = Vec::new();
        
        for announcement in announcements {
            // Extract GPU capability details
            let gpu_cap = announcement.capabilities.iter()
                .find(|c| c.capability_type == "compute:gpu")
                .context("Missing GPU capability in announcement")?;
            
            let provider = ComputeProvider {
                primal_name: announcement.primal.clone(),
                node_id: announcement.node_id.clone(),
                socket_path: announcement.get_unix_socket_path()
                    .context("No Unix socket endpoint")?,
                capability: gpu_cap.clone(),
                discovered_at: announcement.timestamp,
            };
            
            providers.push(provider);
        }
        
        info!("Discovered {} GPU compute providers", providers.len());
        
        Ok(providers)
    }
    
    /// Get a GPU compute client
    pub async fn get_gpu_client(&self) -> Result<Arc<GpuComputeClient>> {
        let providers = self.discover_gpu_compute().await?;
        
        let provider = providers.into_iter()
            .next()
            .context("No GPU compute providers available")?;
        
        info!("Connecting to GPU compute: {} (node: {})", 
            provider.primal_name, provider.node_id);
        
        let transport = TransportClient::from_unix_socket(&provider.socket_path)?;
        
        Ok(Arc::new(GpuComputeClient {
            transport,
            provider,
        }))
    }
}

/// A discovered compute provider
#[derive(Debug, Clone)]
pub struct ComputeProvider {
    pub primal_name: String,
    pub node_id: String,
    pub socket_path: PathBuf,
    pub capability: Capability,
    pub discovered_at: DateTime<Utc>,
}

/// Capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub capability_type: String,
    pub version: String,
    pub methods: Vec<String>,
    pub metadata: serde_json::Value,
}
```

---

### Layer 3: Squirrel Uses Generic Compute Interface

**File**: `crates/main/src/api/ai/compute_client.rs` (NEW)

```rust
//! Generic GPU compute client for Squirrel
//! 
//! Provides a generic interface for GPU inference without knowing
//! the underlying implementation (could be toadstool/barraCUDA,
//! could be something else entirely).

use anyhow::{Context, Result};
use biomeos_core::clients::transport::TransportClient;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// GPU compute client (generic interface)
pub struct GpuComputeClient {
    transport: TransportClient,
    provider: ComputeProvider,
}

impl GpuComputeClient {
    /// Check if text generation is supported
    pub fn supports_text_generation(&self) -> bool {
        self.provider.capability.methods.iter()
            .any(|m| m == "inference.text_generation")
    }
    
    /// Generate text using GPU
    pub async fn generate_text(&self, request: TextGenRequest) -> Result<TextGenResponse> {
        // Call generic JSON-RPC method
        let response = self.transport
            .call("inference.text_generation", Some(json!({
                "prompt": request.prompt,
                "max_tokens": request.max_tokens,
                "temperature": request.temperature,
                "model": request.model.unwrap_or("default".to_string())
            })))
            .await
            .context("GPU inference failed")?;
        
        serde_json::from_value(response)
            .context("Failed to parse GPU response")
    }
    
    /// Check if image generation is supported
    pub fn supports_image_generation(&self) -> bool {
        self.provider.capability.methods.iter()
            .any(|m| m == "inference.image_generation")
    }
    
    /// Generate image using GPU
    pub async fn generate_image(&self, request: ImageGenRequest) -> Result<ImageGenResponse> {
        let response = self.transport
            .call("inference.image_generation", Some(json!({
                "prompt": request.prompt,
                "width": request.width,
                "height": request.height,
                "model": request.model.unwrap_or("default".to_string())
            })))
            .await
            .context("GPU image generation failed")?;
        
        serde_json::from_value(response)
            .context("Failed to parse GPU image response")
    }
    
    /// Get provider information
    pub fn provider_info(&self) -> &ComputeProvider {
        &self.provider
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextGenRequest {
    pub prompt: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextGenResponse {
    pub text: String,
    pub tokens_generated: u32,
    pub latency_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageGenRequest {
    pub prompt: String,
    pub width: u32,
    pub height: u32,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageGenResponse {
    pub image_data: Vec<u8>, // PNG/JPEG bytes
    pub width: u32,
    pub height: u32,
    pub latency_ms: u64,
}
```

---

### Layer 4: Squirrel Routes to GPU or Cloud

**File**: `crates/main/src/api/ai/router.rs` (UPDATE)

```rust
// Update existing AiRouter to include GPU compute

use crate::discovery::compute_discovery::ComputeDiscovery;

pub struct AiRouter {
    // Existing providers
    ollama: Option<Arc<OllamaAdapter>>,
    openai: Option<Arc<OpenAiAdapter>>,
    huggingface: Option<Arc<HuggingFaceAdapter>>,
    
    // NEW: GPU compute (discovered at runtime)
    gpu_compute: Option<Arc<GpuComputeClient>>,
    compute_discovery: Arc<ComputeDiscovery>,
}

impl AiRouter {
    pub async fn new(family_id: impl Into<String>) -> Result<Self> {
        let family_id = family_id.into();
        
        // Initialize compute discovery
        let compute_discovery = Arc::new(ComputeDiscovery::new(&family_id).await?);
        
        // Try to discover GPU compute
        let gpu_compute = match compute_discovery.get_gpu_client().await {
            Ok(client) => {
                info!("✅ GPU compute available: {}", 
                    client.provider_info().primal_name);
                Some(client)
            }
            Err(e) => {
                warn!("⚠️  No GPU compute available: {}", e);
                None
            }
        };
        
        // Initialize other providers (Ollama, OpenAI, etc.)
        let ollama = OllamaAdapter::new().await.ok();
        let openai = OpenAiAdapter::new().await.ok();
        let huggingface = HuggingFaceAdapter::new().await.ok();
        
        Ok(Self {
            ollama,
            openai,
            huggingface,
            gpu_compute,
            compute_discovery,
        })
    }
    
    pub async fn generate_text(&self, request: UniversalAiRequest) -> Result<AiResponse> {
        // Priority 1: GPU compute (if available and local required)
        if self.gpu_compute.is_some() && 
           (request.constraints.contains(&Constraint::RequireLocal) ||
            request.constraints.contains(&Constraint::OptimizeCost)) {
            
            if let Some(gpu) = &self.gpu_compute {
                if gpu.supports_text_generation() {
                    info!("🎯 Routing to GPU compute (local, $0.00)");
                    
                    return self.generate_with_gpu(gpu, request).await;
                }
            }
        }
        
        // Priority 2: Local CPU (Ollama)
        if let Some(ollama) = &self.ollama {
            if request.constraints.contains(&Constraint::RequireLocal) {
                info!("🎯 Routing to Ollama (local CPU, $0.00)");
                return ollama.generate_text(request).await;
            }
        }
        
        // Priority 3: Cloud providers
        if request.constraints.contains(&Constraint::OptimizeQuality) {
            if let Some(openai) = &self.openai {
                info!("🎯 Routing to OpenAI (cloud, high quality)");
                return openai.generate_text(request).await;
            }
        }
        
        // Default: Best available
        self.select_best_provider(request).await
    }
    
    async fn generate_with_gpu(
        &self,
        gpu: &GpuComputeClient,
        request: UniversalAiRequest
    ) -> Result<AiResponse> {
        let gpu_request = TextGenRequest {
            prompt: request.input,
            max_tokens: request.max_tokens.unwrap_or(2048),
            temperature: request.temperature.unwrap_or(0.7),
            model: request.model,
        };
        
        let gpu_response = gpu.generate_text(gpu_request).await?;
        
        Ok(AiResponse {
            text: gpu_response.text,
            provider: format!("gpu:{}", gpu.provider_info().primal_name),
            cost: 0.0, // $0.00 for local GPU!
            latency_ms: gpu_response.latency_ms,
            tokens_generated: gpu_response.tokens_generated,
            privacy_level: PrivacyLevel::FullyLocal,
        })
    }
}
```

---

## 🔄 THE PROPER FLOW

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. STARTUP: toadstool announces capabilities                   │
├─────────────────────────────────────────────────────────────────┤
│ toadstool → Songbird: "I provide compute:gpu"                  │
│   - 105 operations (barraCUDA)                                  │
│   - Vendor-agnostic (NVIDIA, AMD, Intel, Apple)                │
│   - Methods: inference.*, training.*, compute.*                │
│   - Socket: /run/user/1000/toadstool-nat0.sock                 │
└─────────────────────────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────────────────────────┐
│ 2. DISCOVERY: Squirrel discovers compute                       │
├─────────────────────────────────────────────────────────────────┤
│ Squirrel → Songbird: "Who provides compute:gpu?"               │
│ Songbird → Squirrel: "toadstool at /run/user/1000/..."         │
│   - NO HARDCODING of "toadstool" in Squirrel!                  │
│   - Squirrel discovered it at runtime!                         │
└─────────────────────────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────────────────────────┐
│ 3. USAGE: Squirrel requests GPU inference                      │
├─────────────────────────────────────────────────────────────────┤
│ Squirrel → toadstool (via Unix socket):                        │
│   {"method": "inference.text_generation", "params": {...}}     │
│                                                                 │
│ toadstool → barraCUDA (internal):                              │
│   executor.generate_text(...) // Uses 105 GPU operations       │
│                                                                 │
│ toadstool → Squirrel:                                           │
│   {"result": {"text": "...", "latency_ms": 234}}               │
└─────────────────────────────────────────────────────────────────┘
```

**Key Points**:
- ✅ Squirrel NEVER imports toadstool code
- ✅ toadstool NEVER knows Squirrel exists
- ✅ Communication via JSON-RPC over Unix sockets
- ✅ Discovery via Songbird capability registry
- ✅ Both primals are sovereign

---

## 🚀 IMPLEMENTATION PLAN

### Week 1: toadstool Capability Announcement

**In toadstool repo**:
1. Add `announce_capabilities()` to toadstool
2. Register `inference.text_generation` JSON-RPC method
3. Expose barraCUDA operations via standard interface
4. Test with Songbird

**Deliverable**: toadstool announces "compute:gpu" capability

---

### Week 2: Squirrel Discovery Integration

**In Squirrel repo**:
1. Add `ComputeDiscovery` module
2. Implement `discover_gpu_compute()`
3. Create `GpuComputeClient` generic interface
4. Update `AiRouter` to use GPU when available
5. Add fallback logic (GPU → CPU → Cloud)

**Deliverable**: Squirrel discovers and uses GPU compute

---

### Week 3: Testing & Validation

**Integration tests**:
1. Start toadstool (with barraCUDA)
2. Start Squirrel
3. Verify Squirrel discovers GPU
4. Test GPU inference
5. Test fallback when GPU unavailable
6. Measure performance

**Deliverable**: Comprehensive tests passing

---

### Week 4: Production Deployment

**biomeOS integration**:
1. Update biomeOS to coordinate GPU discovery
2. Deploy both primals to `plasmidBin/`
3. Validate in NUCLEUS atomic
4. Document capability interface
5. Launch to production

**Deliverable**: Production deployment complete

---

## 💎 SUCCESS CRITERIA

### Functional Requirements

✅ **Zero Hardcoding**
- Squirrel NEVER mentions "toadstool" or "barraCUDA"
- Discovery happens at runtime via Songbird
- Works with ANY compute provider

✅ **TRUE PRIMAL Sovereignty**
- Squirrel only knows itself
- toadstool only knows itself
- Communication via standard interfaces

✅ **Graceful Degradation**
- If GPU unavailable → CPU (Ollama)
- If CPU unavailable → Cloud (OpenAI)
- Always has a fallback

✅ **Performance**
- GPU inference: 2-5x faster than CPU
- Cost: $0.00 for local GPU
- Latency: <2s for typical requests

### Quality Requirements

✅ **Testing**
- 20+ unit tests
- 10+ integration tests
- 5+ e2e tests
- Chaos tests (GPU failure, discovery failure)

✅ **Documentation**
- Capability specification
- Discovery protocol
- Integration guide for other primals

---

## 📊 COMPARISON: WRONG vs RIGHT

| Aspect | WRONG (Tight Coupling) | RIGHT (Capability Discovery) |
|--------|----------------------|------------------------------|
| **Imports** | `use toadstool::barracuda` | `use biomeos_core::clients::transport` |
| **Discovery** | Hardcoded dependency | Runtime via Songbird |
| **Communication** | Direct function calls | JSON-RPC over Unix sockets |
| **Flexibility** | toadstool ONLY | ANY compute provider |
| **Sovereignty** | Violated | Maintained |
| **Testing** | Requires toadstool | Mocks JSON-RPC interface |
| **Deployment** | Coupled | Independent |

---

## 🎉 BOTTOM LINE

### What We're Building

**The RIGHT Way**: Capability-based GPU discovery

**Key Principles**:
- ✅ Squirrel discovers "compute:gpu" capability (not "toadstool")
- ✅ toadstool announces it provides "compute:gpu" (not aware of Squirrel)
- ✅ Communication via JSON-RPC (not direct function calls)
- ✅ Both primals are sovereign (TRUE PRIMAL pattern)

### Benefits

**vs Tight Coupling**:
- ✅ Can replace toadstool with another compute provider
- ✅ Can add multiple GPU providers
- ✅ Squirrel works even if toadstool unavailable (fallback)
- ✅ Independent deployment and testing
- ✅ Future-proof architecture

### Timeline

**4 weeks to production**:
- Week 1: toadstool capability announcement
- Week 2: Squirrel discovery integration
- Week 3: Testing & validation
- Week 4: Production deployment

---

**STATUS**: ✅ **PROPER ARCHITECTURE DEFINED**  
**PATTERN**: 🎯 **CAPABILITY-BASED DISCOVERY** (like NUCLEUS)  
**SOVEREIGNTY**: ✅ **MAINTAINED** (TRUE PRIMAL)  
**NEXT**: 🚀 **Begin Week 1 implementation**

---

*"From tight coupling to capability discovery. From hardcoding to runtime discovery. From vendor lock-in to sovereign primals. This is the TRUE PRIMAL way."* ✨

**Let's build it right.** 🐿️🦈🌊

