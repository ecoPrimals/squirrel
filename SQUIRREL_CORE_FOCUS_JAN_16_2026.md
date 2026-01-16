# 🎯 Squirrel Core Focus - Refocused Jan 16, 2026

**Mission**: AI Orchestration, MCP Protocol, Model Interactions, Deployment

**NOT Our Domain**: GPU compute (that's Toadstool's job with barraCUDA!)

---

## ✅ What Squirrel SHOULD Focus On

### 1. AI Orchestration & Routing
**Current State**: Good foundation, needs enhancement

**Opportunities**:
- Improve intelligent model selection (cost/quality/latency optimization)
- Enhanced provider fallback and retry logic
- Load balancing across providers
- Caching and request deduplication
- A/B testing framework for model selection

### 2. MCP Protocol Implementation
**Current State**: Extensive features, well-implemented

**Opportunities**:
- Complete compliance with latest MCP spec
- Enhanced tool execution capabilities
- Better resource management
- Improved streaming performance
- Multi-agent coordination refinement

### 3. AI Provider Adapters
**Current State**: Mixed - some complete, some placeholders

**Gaps Identified**:
- ✅ OpenAI: Fully implemented (text + image)
- ✅ Ollama: Fully implemented (text, local)
- ❌ HuggingFace: **PLACEHOLDER - NOT IMPLEMENTED!**

**Opportunities**:
- Complete HuggingFace adapter
- Add Anthropic (Claude) adapter
- Add Google (Gemini) adapter
- Add local embedding models
- Streaming response support

### 4. Deployment & Integration
**Current State**: Excellent (TRUE PRIMAL, Unix sockets, biomeOS ready)

**Opportunities**:
- Enhanced monitoring and observability
- Better error reporting and diagnostics
- Performance optimization
- Configuration management improvements
- Health check enhancements

---

## 🚫 What Is NOT Squirrel's Domain

### Toadstool's Responsibilities:
- GPU compute orchestration
- barraCUDA (pure Rust CUDA parity)
- Direct hardware acceleration
- GPU resource management
- CUDA kernel execution

### How Squirrel Interacts with Toadstool:
```
Squirrel (AI Orchestration)
    ↓ Unix Socket Request
    ↓ Capability: "compute:gpu:*"
Songbird (Discovery)
    ↓ Socket Path
Toadstool (GPU Compute)
    ↓ barraCUDA Execution
    ↓ Unix Socket Response
Squirrel (Return Result to User)
```

**Key Principle**: Capability-based discovery, NOT tight coupling!

---

## 📊 Priority Matrix (What to Build Next)

### High Priority (Week 1-2)
1. **Complete HuggingFace Adapter** ⭐
   - Impact: High (adds major provider)
   - Effort: Medium
   - Blockers: None
   
2. **Enhanced AI Routing Logic** ⭐
   - Impact: High (better model selection)
   - Effort: Medium
   - Current: Basic implementation exists
   
3. **Streaming Response Support** ⭐
   - Impact: High (better UX)
   - Effort: Medium-High
   - Current: Infrastructure exists in MCP

### Medium Priority (Week 3-4)
4. **Anthropic Claude Adapter**
   - Impact: Medium-High (premium quality)
   - Effort: Medium
   - Blockers: API key needed
   
5. **Enhanced Monitoring**
   - Impact: Medium (production ops)
   - Effort: Low-Medium
   - Current: Basic metrics exist
   
6. **Request Caching Layer**
   - Impact: Medium (performance + cost)
   - Effort: Medium
   - Current: None

### Lower Priority (Later)
7. Google Gemini adapter
8. Local embedding models
9. Advanced A/B testing framework
10. Multi-model ensemble routing

---

## 🎯 Immediate Action Plan

### Task 1: Complete HuggingFace Adapter

**Current State**:
```rust
async fn generate_text(&self, _request: TextGenerationRequest) 
    -> Result<TextGenerationResponse, PrimalError> {
    // Placeholder - would implement HuggingFace Inference API calls
    Err(PrimalError::OperationFailed(
        "HuggingFace adapter not yet fully implemented".to_string(),
    ))
}
```

**What Needs to be Done**:
1. Implement HuggingFace Inference API integration
2. Add support for popular models (Mistral, Llama, etc.)
3. Implement proper error handling
4. Add retry logic
5. Add tests

**API Endpoint**: `https://api-inference.huggingface.co/models/{model_id}`

**Expected Outcome**: Squirrel can route to HuggingFace models!

---

### Task 2: Enhanced AI Routing

**Current State**: Basic provider selection exists

**Improvements Needed**:
1. **Cost Optimization**:
   - Track token usage per provider
   - Calculate cost per request
   - Route to cheapest suitable provider
   
2. **Quality Optimization**:
   - Track quality metrics per provider
   - User feedback integration
   - Quality-based model selection
   
3. **Latency Optimization**:
   - Track response times
   - Route to fastest suitable provider
   - Timeout handling
   
4. **Fallback Logic**:
   - If primary provider fails, try secondary
   - Exponential backoff
   - Circuit breaker pattern

**Expected Outcome**: Intelligent, self-optimizing AI routing!

---

### Task 3: Streaming Response Support

**Current State**: MCP has streaming infrastructure, not fully integrated

**Improvements Needed**:
1. OpenAI streaming support (`stream: true`)
2. Ollama streaming support
3. WebSocket streaming to clients
4. Server-Sent Events (SSE) support
5. Backpressure handling

**Expected Outcome**: Real-time AI responses for better UX!

---

## 📚 Technical Design: HuggingFace Adapter

### API Integration

```rust
// HuggingFace Inference API Request
#[derive(Debug, Serialize)]
struct HuggingFaceRequest {
    inputs: String,
    parameters: HuggingFaceParameters,
}

#[derive(Debug, Serialize)]
struct HuggingFaceParameters {
    max_new_tokens: Option<u32>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    do_sample: Option<bool>,
}

// HuggingFace Inference API Response
#[derive(Debug, Deserialize)]
struct HuggingFaceResponse {
    generated_text: Option<String>,
    error: Option<String>,
}

impl HuggingFaceAdapter {
    pub async fn generate_text(
        &self,
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        let start = Instant::now();
        
        // Get model from environment or use default
        let model = std::env::var("HUGGINGFACE_MODEL")
            .unwrap_or_else(|_| "mistralai/Mistral-7B-Instruct-v0.2".to_string());
        
        // Get API key from environment
        let api_key = std::env::var("HUGGINGFACE_API_KEY")
            .map_err(|_| PrimalError::ConfigurationError(
                "HUGGINGFACE_API_KEY not set".to_string()
            ))?;
        
        // Build request
        let hf_req = HuggingFaceRequest {
            inputs: request.prompt.clone(),
            parameters: HuggingFaceParameters {
                max_new_tokens: Some(request.max_tokens),
                temperature: Some(request.temperature),
                top_p: None,
                do_sample: Some(true),
            },
        };
        
        // Send request
        let url = format!(
            "https://api-inference.huggingface.co/models/{}",
            model
        );
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&hf_req)
            .send()
            .await
            .map_err(|e| PrimalError::NetworkError(
                format!("HuggingFace request failed: {}", e)
            ))?;
        
        // Parse response
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(PrimalError::ProviderError(format!(
                "HuggingFace API error {}: {}",
                status, error_text
            )));
        }
        
        let hf_response: Vec<HuggingFaceResponse> = response
            .json()
            .await
            .map_err(|e| PrimalError::ParseError(
                format!("Failed to parse HuggingFace response: {}", e)
            ))?;
        
        // Extract text
        let text = hf_response
            .first()
            .and_then(|r| r.generated_text.clone())
            .ok_or_else(|| PrimalError::ProviderError(
                "No generated text in HuggingFace response".to_string()
            ))?;
        
        let latency_ms = start.elapsed().as_millis() as u64;
        
        // Build response
        Ok(TextGenerationResponse {
            text: text.clone(),
            model: model.clone(),
            provider: "huggingface".to_string(),
            latency_ms,
            token_usage: None, // HF Inference API doesn't return token counts
            finish_reason: Some("complete".to_string()),
        })
    }
}
```

### Environment Variables

```bash
# Required
HUGGINGFACE_API_KEY=hf_xxxxxxxxxxxxxxxxxxxxx

# Optional
HUGGINGFACE_MODEL=mistralai/Mistral-7B-Instruct-v0.2
HUGGINGFACE_BASE_URL=https://api-inference.huggingface.co
```

### Supported Models

**Text Generation**:
- `mistralai/Mistral-7B-Instruct-v0.2`
- `meta-llama/Llama-2-7b-chat-hf`
- `meta-llama/Llama-2-13b-chat-hf`
- `tiiuae/falcon-7b-instruct`
- `HuggingFaceH4/zephyr-7b-beta`

---

## 📚 Technical Design: Enhanced AI Routing

### Routing Strategy Pattern

```rust
pub enum RoutingStrategy {
    /// Minimize cost
    OptimizeCost,
    /// Maximize quality
    OptimizeQuality,
    /// Minimize latency
    OptimizeLatency,
    /// Balance all factors
    Balanced,
    /// Custom weights
    Custom {
        cost_weight: f64,
        quality_weight: f64,
        latency_weight: f64,
    },
}

pub struct EnhancedAiRouter {
    providers: Vec<Box<dyn AiProviderAdapter>>,
    strategy: RoutingStrategy,
    metrics: Arc<RwLock<ProviderMetrics>>,
}

impl EnhancedAiRouter {
    pub async fn select_provider(
        &self,
        request: &TextGenerationRequest,
        constraints: &[Constraint],
    ) -> Result<&dyn AiProviderAdapter, PrimalError> {
        // Filter by constraints
        let mut candidates: Vec<_> = self.providers
            .iter()
            .filter(|p| self.satisfies_constraints(p, constraints))
            .collect();
        
        if candidates.is_empty() {
            return Err(PrimalError::NoProviderAvailable);
        }
        
        // Score candidates based on strategy
        let mut scored: Vec<_> = candidates
            .iter()
            .map(|p| {
                let score = self.calculate_score(p, &self.strategy);
                (p, score)
            })
            .collect();
        
        // Sort by score (higher is better)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Return best provider
        Ok(scored[0].0.as_ref())
    }
    
    fn calculate_score(
        &self,
        provider: &Box<dyn AiProviderAdapter>,
        strategy: &RoutingStrategy,
    ) -> f64 {
        match strategy {
            RoutingStrategy::OptimizeCost => {
                // Lower cost = higher score
                let cost = provider.cost_per_unit().unwrap_or(1.0);
                1.0 / (cost + 0.001)
            }
            RoutingStrategy::OptimizeQuality => {
                // Higher quality tier = higher score
                match provider.quality_tier() {
                    QualityTier::Basic => 1.0,
                    QualityTier::Standard => 2.0,
                    QualityTier::High => 3.0,
                    QualityTier::Premium => 4.0,
                }
            }
            RoutingStrategy::OptimizeLatency => {
                // Lower latency = higher score
                let latency = provider.avg_latency_ms() as f64;
                1.0 / (latency + 1.0)
            }
            RoutingStrategy::Balanced => {
                // Equal weights
                let cost_score = 1.0 / (provider.cost_per_unit().unwrap_or(1.0) + 0.001);
                let quality_score = match provider.quality_tier() {
                    QualityTier::Basic => 1.0,
                    QualityTier::Standard => 2.0,
                    QualityTier::High => 3.0,
                    QualityTier::Premium => 4.0,
                };
                let latency_score = 1.0 / (provider.avg_latency_ms() as f64 + 1.0);
                
                (cost_score + quality_score + latency_score) / 3.0
            }
            RoutingStrategy::Custom { cost_weight, quality_weight, latency_weight } => {
                let cost_score = 1.0 / (provider.cost_per_unit().unwrap_or(1.0) + 0.001);
                let quality_score = match provider.quality_tier() {
                    QualityTier::Basic => 1.0,
                    QualityTier::Standard => 2.0,
                    QualityTier::High => 3.0,
                    QualityTier::Premium => 4.0,
                };
                let latency_score = 1.0 / (provider.avg_latency_ms() as f64 + 1.0);
                
                (cost_score * cost_weight) + 
                (quality_score * quality_weight) + 
                (latency_score * latency_weight)
            }
        }
    }
}
```

---

## 🎯 Success Metrics

### For HuggingFace Adapter:
- ✅ Can successfully call HuggingFace Inference API
- ✅ Supports at least 3 popular models
- ✅ Handles errors gracefully
- ✅ Returns properly formatted responses
- ✅ Has comprehensive tests

### For Enhanced Routing:
- ✅ Reduces average cost by 30% (with cost optimization)
- ✅ Reduces average latency by 20% (with latency optimization)
- ✅ Handles provider failures gracefully (fallback works)
- ✅ Tracks metrics per provider
- ✅ Has comprehensive tests

### For Streaming:
- ✅ Can stream responses from OpenAI
- ✅ Can stream responses from Ollama
- ✅ Properly handles backpressure
- ✅ Works with WebSocket clients
- ✅ Has comprehensive tests

---

## 🚀 Next Steps

### This Session (Immediate):
1. ✅ Refocus documentation on Squirrel's domain
2. 🔄 Identify gaps in AI adapters
3. 🔄 Design HuggingFace adapter implementation
4. 📋 Create implementation roadmap

### Next Session (Week 1):
1. Implement HuggingFace adapter
2. Add comprehensive tests
3. Enhance AI routing logic
4. Add provider metrics tracking

### Week 2:
1. Implement streaming response support
2. Add Anthropic Claude adapter
3. Enhance monitoring and observability
4. Performance optimization

---

## 📚 Reference Documents

### For AI Provider Integration:
- HuggingFace Inference API: https://huggingface.co/docs/api-inference/
- OpenAI API: https://platform.openai.com/docs/api-reference
- Anthropic API: https://docs.anthropic.com/claude/reference

### For MCP Protocol:
- MCP Specification: Model Context Protocol docs
- Current implementation: `crates/core/mcp/src/`

### For Deployment:
- biomeOS integration: `ecoPrimals/phase2/biomeOS/`
- NUCLEUS protocol: `ecoPrimals/phase2/biomeOS/docs/NUCLEUS_SECURE_DISCOVERY_PROTOCOL.md`

---

**🎯 Squirrel's Mission: Best-in-class AI orchestration, not GPU compute!**

*Let Toadstool handle the barraCUDA GPU work.*  
*Squirrel focuses on intelligent AI routing and model interactions.*

---

**Updated**: January 16, 2026  
**Status**: Refocused on core competencies  
**Next**: Implement HuggingFace adapter + enhanced routing

