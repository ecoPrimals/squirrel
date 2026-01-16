# Squirrel v1.1.0 - Local Evolution Implementation Plan

**Date**: January 16, 2026  
**Version**: v1.0.3 → v1.1.0  
**Goal**: Zero-HTTP in production (100% pure Rust including transitive!)  
**Timeline**: 2-3 hours (Squirrel-only, independent of Songbird)

---

## 🎯 **Objective**

Implement the **Squirrel-side** changes for Zero-HTTP evolution:
- Make HTTP adapters **optional** (dev feature flag)
- Update `AiRouter` for production mode (Unix sockets only)
- Prepare for Songbird AI proxy integration
- Maintain backward compatibility

**Key Principle**: This work is **independent** of Songbird. We're preparing Squirrel's architecture to support both modes (dev HTTP + production Unix sockets).

---

## 📋 **Implementation Checklist**

### Phase 1: Cargo Feature Flags (30 min)

- [ ] **1.1** Update `crates/Cargo.toml` to add `dev-direct-http` feature
- [ ] **1.2** Update `crates/main/Cargo.toml` to make HTTP deps optional
- [ ] **1.3** Update `crates/integration/web/Cargo.toml` for optional reqwest
- [ ] **1.4** Verify feature builds: `cargo build` vs `cargo build --features dev-direct-http`

### Phase 2: Conditional Compilation (45 min)

- [ ] **2.1** Wrap OpenAI adapter with `#[cfg(feature = "dev-direct-http")]`
- [ ] **2.2** Wrap HuggingFace adapter with `#[cfg(feature = "dev-direct-http")]`
- [ ] **2.3** Wrap Ollama adapter with `#[cfg(feature = "dev-direct-http")]`
- [ ] **2.4** Update `crates/main/src/api/ai/adapters/mod.rs` for conditional exports
- [ ] **2.5** Update `AiRouter` to use conditional compilation

### Phase 3: AiRouter Refactoring (30 min)

- [ ] **3.1** Split `new_with_discovery()` into production and dev modes
- [ ] **3.2** Implement production-only path (UniversalAiAdapter ONLY)
- [ ] **3.3** Implement dev-only path (includes HTTP adapters)
- [ ] **3.4** Update `load_legacy_adapters_parallel()` for conditional compilation
- [ ] **3.5** Add clear logging for which mode is active

### Phase 4: Configuration Updates (15 min)

- [ ] **4.1** Update `config/production.toml` (remove API keys)
- [ ] **4.2** Update `config/development.toml` (keep API keys for dev mode)
- [ ] **4.3** Add documentation comments about feature flags
- [ ] **4.4** Create example Songbird proxy configuration

### Phase 5: Testing (30 min)

- [ ] **5.1** Test production build: `cargo test` (no HTTP!)
- [ ] **5.2** Test dev build: `cargo test --features dev-direct-http`
- [ ] **5.3** Verify `cargo tree | grep ring` (should be empty in prod!)
- [ ] **5.4** Verify `cargo tree | grep openssl` (should be empty in prod!)
- [ ] **5.5** Integration tests for UniversalAiAdapter

### Phase 6: Documentation (15 min)

- [ ] **6.1** Update `README.md` with feature flag usage
- [ ] **6.2** Update `START_HERE_v1.0.3.md` to mention v1.1.0
- [ ] **6.3** Update `CURRENT_STATUS.md` to v1.1.0 status
- [ ] **6.4** Add migration notes for upgrading to v1.1.0

**Total Time**: ~2.5 hours (Squirrel-only)

---

## 🔧 **Detailed Implementation**

### **1. Cargo Feature Flags**

#### 1.1 Update `crates/Cargo.toml` (Workspace)

```toml
# Add to workspace Cargo.toml
[workspace.dependencies]
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"], optional = true }
```

#### 1.2 Update `crates/main/Cargo.toml`

```toml
[features]
default = []
dev-direct-http = [
    "reqwest",
    "openai-adapter",
    "huggingface-adapter",
    "ollama-adapter",
]

[dependencies]
# HTTP client - OPTIONAL (dev mode only!)
reqwest = { workspace = true, optional = true }

# Always available (production)
tokio = { workspace = true, features = ["full"] }
serde = { workspace = true }
serde_json = { workspace = true }
# ... other always-on deps

# Optional AI adapters (dev mode only)
# These will be conditional compilation in the code
```

#### 1.3 Update `crates/integration/web/Cargo.toml`

```toml
[features]
default = []
dev-direct-http = ["reqwest"]

[dependencies]
reqwest = { workspace = true, optional = true }
# ... other deps
```

---

### **2. Conditional Compilation**

#### 2.1 Wrap OpenAI Adapter

File: `crates/main/src/api/ai/adapters/openai.rs`

```rust
// Add at the top of file
#![cfg(feature = "dev-direct-http")]

// Rest of file remains unchanged
// This entire file is only compiled when dev-direct-http feature is enabled
```

#### 2.2 Wrap HuggingFace Adapter

File: `crates/main/src/api/ai/adapters/huggingface.rs`

```rust
// Add at the top of file
#![cfg(feature = "dev-direct-http")]

// Rest of file remains unchanged
```

#### 2.3 Wrap Ollama Adapter

File: `crates/main/src/api/ai/adapters/ollama.rs`

```rust
// Add at the top of file
#![cfg(feature = "dev-direct-http")]

// Rest of file remains unchanged
```

#### 2.4 Update `mod.rs` for Conditional Exports

File: `crates/main/src/api/ai/adapters/mod.rs`

```rust
use async_trait::async_trait;

// These are always available
pub mod universal;
pub use universal::UniversalAiAdapter;

// These are dev-only (HTTP adapters)
#[cfg(feature = "dev-direct-http")]
pub mod openai;
#[cfg(feature = "dev-direct-http")]
pub use openai::OpenAIAdapter;

#[cfg(feature = "dev-direct-http")]
pub mod ollama;
#[cfg(feature = "dev-direct-http")]
pub use ollama::OllamaAdapter;

#[cfg(feature = "dev-direct-http")]
pub mod huggingface;
#[cfg(feature = "dev-direct-http")]
pub use huggingface::HuggingFaceAdapter;

// Trait and types remain unchanged
// ...
```

---

### **3. AiRouter Refactoring**

#### 3.1 Production-Only Router

File: `crates/main/src/api/ai/router.rs`

```rust
impl AiRouter {
    /// Create a new AI router with capability-based discovery (production mode)
    #[cfg(not(feature = "dev-direct-http"))]
    pub async fn new_with_discovery(
        songbird_client: Option<Arc<SongbirdClient>>,
    ) -> Result<Self, PrimalError> {
        info!("🚀 Starting AiRouter in PRODUCTION mode (Unix sockets ONLY!)");
        
        let mut providers: Vec<Arc<dyn AiProviderAdapter>> = Vec::new();

        // Discover AI providers via Songbird or environment
        if let Ok(socket_paths_str) = std::env::var("AI_PROVIDER_SOCKETS") {
            info!("Discovering AI providers from AI_PROVIDER_SOCKETS: {}", socket_paths_str);
            let socket_paths: Vec<PathBuf> = socket_paths_str
                .split(',')
                .filter_map(|s| {
                    let path = PathBuf::from(s.trim());
                    if path.exists() {
                        Some(path)
                    } else {
                        warn!("Configured AI provider socket path does not exist: {}", s);
                        None
                    }
                })
                .collect();

            for socket_path in socket_paths {
                match Self::create_universal_adapter_from_path(socket_path.clone()).await {
                    Ok(adapter) => {
                        info!(
                            "✅ Universal AI adapter initialized for capability '{}' at {}",
                            adapter.capability(),
                            socket_path.display()
                        );
                        providers.push(Arc::new(adapter));
                    }
                    Err(e) => {
                        error!(
                            "⚠️  Failed to initialize Universal AI adapter from {}: {}",
                            socket_path.display(),
                            e
                        );
                    }
                }
            }
        }

        if providers.is_empty() {
            warn!("⚠️  No AI providers available. Set AI_PROVIDER_SOCKETS or run in dev mode (--features dev-direct-http)");
        } else {
            info!(
                "✅ AiRouter initialized with {} provider(s) (PRODUCTION MODE)",
                providers.len()
            );
        }

        Ok(Self {
            providers: Arc::new(RwLock::new(providers)),
            selector: Arc::new(ProviderSelector::new()),
            enable_retry: true,
            max_retries: 2,
        })
    }

    /// Create a new AI router with capability-based discovery + dev HTTP adapters
    #[cfg(feature = "dev-direct-http")]
    pub async fn new_with_discovery(
        songbird_client: Option<Arc<SongbirdClient>>,
    ) -> Result<Self, PrimalError> {
        warn!("🔧 Starting AiRouter in DEVELOPMENT mode (HTTP adapters enabled!)");
        warn!("⚠️  This mode includes transitive ring/openssl dependencies");
        
        let mut providers: Vec<Arc<dyn AiProviderAdapter>> = Vec::new();

        // Discover Unix socket providers (if any)
        if let Ok(socket_paths_str) = std::env::var("AI_PROVIDER_SOCKETS") {
            info!("Discovering AI providers from AI_PROVIDER_SOCKETS: {}", socket_paths_str);
            // ... (same as production)
        }

        // Load legacy HTTP adapters for development
        let legacy_providers = Self::load_legacy_adapters_parallel().await;
        providers.extend(legacy_providers);

        if providers.is_empty() {
            warn!("⚠️  No AI providers available. Set AI_PROVIDER_SOCKETS, OPENAI_API_KEY, HUGGINGFACE_API_KEY, or install Ollama");
        } else {
            info!(
                "✅ AiRouter initialized with {} provider(s) (DEVELOPMENT MODE)",
                providers.len()
            );
        }

        Ok(Self {
            providers: Arc::new(RwLock::new(providers)),
            selector: Arc::new(ProviderSelector::new()),
            enable_retry: true,
            max_retries: 2,
        })
    }

    /// Loads legacy adapters (OpenAI, Ollama, HuggingFace) in parallel.
    /// Only available in dev-direct-http mode.
    #[cfg(feature = "dev-direct-http")]
    async fn load_legacy_adapters_parallel() -> Vec<Arc<dyn AiProviderAdapter>> {
        let mut providers: Vec<Arc<dyn AiProviderAdapter>> = Vec::new();

        let (openai_result, ollama_result, huggingface_result) = tokio::join!(
            async {
                if let Ok(openai) = OpenAIAdapter::new() {
                    info!("✅ OpenAI adapter initialized (DEV MODE - HTTP)");
                    Some(Arc::new(openai) as Arc<dyn AiProviderAdapter>)
                } else {
                    info!("⚠️  OpenAI adapter not available (OPENAI_API_KEY not set)");
                    None
                }
            },
            async {
                let ollama = OllamaAdapter::new();
                if ollama.is_available().await {
                    info!("✅ Ollama adapter initialized (DEV MODE - HTTP)");
                    Some(Arc::new(ollama) as Arc<dyn AiProviderAdapter>)
                } else {
                    info!("⚠️  Ollama not available (install: https://ollama.ai)");
                    None
                }
            },
            async {
                let huggingface = HuggingFaceAdapter::new();
                if huggingface.is_available().await {
                    info!("✅ HuggingFace adapter initialized (DEV MODE - HTTP)");
                    Some(Arc::new(huggingface) as Arc<dyn AiProviderAdapter>)
                } else {
                    info!("⚠️  HuggingFace adapter not available (HUGGINGFACE_API_KEY not set)");
                    None
                }
            }
        );

        if let Some(openai) = openai_result {
            providers.push(openai);
        }
        if let Some(ollama) = ollama_result {
            providers.push(ollama);
        }
        if let Some(huggingface) = huggingface_result {
            providers.push(huggingface);
        }

        providers
    }

    /// Legacy new() method - delegates to new_with_discovery()
    #[deprecated(
        since = "1.1.0",
        note = "Use `new_with_discovery()` for TRUE PRIMAL compliant capability-based AI provider discovery."
    )]
    pub async fn new() -> Result<Self, PrimalError> {
        Self::new_with_discovery(None).await
    }

    // ... rest of impl unchanged
}
```

---

### **4. Configuration Updates**

#### 4.1 Production Config (`config/production.toml`)

```toml
# Squirrel Production Configuration
# v1.1.0+ - Zero-HTTP Mode

[ai_router]
# Capability-based discovery (Unix sockets only!)
discovery_method = "songbird"
enable_retry = true
max_retries = 2

# AI provider Unix sockets (from Songbird)
# Set via AI_PROVIDER_SOCKETS environment variable:
# AI_PROVIDER_SOCKETS="/run/user/1000/songbird-ai-openai.sock,/run/user/1000/songbird-ai-huggingface.sock"

# NO API KEYS IN PRODUCTION!
# All external AI credentials are managed by Songbird
```

#### 4.2 Development Config (`config/development.toml`)

```toml
# Squirrel Development Configuration
# v1.1.0+ - Dev Mode (HTTP adapters enabled)

[ai_router]
# Discovery + direct HTTP adapters for fast iteration
discovery_method = "hybrid"
enable_retry = true
max_retries = 2

# API keys for direct HTTP access (DEV ONLY!)
# These are used when --features dev-direct-http is enabled
[ai_router.api_keys]
# openai_api_key = "sk-..." # Load from OPENAI_API_KEY env var
# huggingface_api_key = "hf_..." # Load from HUGGINGFACE_API_KEY env var

# NOTE: In dev mode, you can use:
# 1. Direct HTTP to OpenAI/HuggingFace (fast, requires API keys)
# 2. Unix sockets to Songbird proxy (slower, no API keys needed)
# 3. Unix sockets to Toadstool/NestGate (local AI)
```

---

### **5. Testing Strategy**

#### 5.1 Production Build Test

```bash
# Clean build
cargo clean

# Build production mode (NO HTTP!)
cargo build --release

# Verify no HTTP dependencies
cargo tree --edges normal | grep -E "(reqwest|hyper|ring|openssl)" || echo "✅ NO HTTP DEPS!"

# Run tests
cargo test --release

# Expected: All tests pass, using mocked UniversalAiAdapter
```

#### 5.2 Development Build Test

```bash
# Build dev mode (WITH HTTP!)
cargo build --release --features dev-direct-http

# Verify HTTP dependencies present
cargo tree --edges normal --features dev-direct-http | grep reqwest && echo "✅ HTTP DEPS PRESENT (expected in dev mode)"

# Run tests
cargo test --release --features dev-direct-http

# Expected: All tests pass, using real HTTP adapters
```

#### 5.3 Integration Tests

Create `crates/main/tests/ai_router_modes_test.rs`:

```rust
#[tokio::test]
#[cfg(not(feature = "dev-direct-http"))]
async fn test_production_mode_no_http_adapters() {
    let router = AiRouter::new_with_discovery(None).await.unwrap();
    
    // In production mode, should only have UniversalAiAdapter
    // (unless AI_PROVIDER_SOCKETS is set)
    let providers = router.providers.read().await;
    
    // All providers should be UniversalAiAdapter
    for provider in providers.iter() {
        assert_eq!(provider.provider_id(), "universal");
    }
}

#[tokio::test]
#[cfg(feature = "dev-direct-http")]
async fn test_dev_mode_has_http_adapters() {
    std::env::set_var("OPENAI_API_KEY", "test-key");
    
    let router = AiRouter::new_with_discovery(None).await.unwrap();
    
    // In dev mode, should have HTTP adapters (if API keys set)
    let providers = router.providers.read().await;
    
    // At least one provider should be available
    assert!(!providers.is_empty());
}
```

#### 5.4 Pure Rust Verification

```bash
# Production mode - should have ZERO ring/openssl
cargo tree --edges normal | grep ring
echo "Exit code: $?" # Should be 1 (not found)

cargo tree --edges normal | grep openssl
echo "Exit code: $?" # Should be 1 (not found)

# Dev mode - should have ring/openssl (via reqwest)
cargo tree --edges normal --features dev-direct-http | grep ring
echo "Exit code: $?" # Should be 0 (found)
```

---

### **6. Documentation**

#### 6.1 Usage Examples

Add to `README.md`:

```markdown
## Building Squirrel

### Production Mode (Default) - Zero HTTP

```bash
# Build for production (Unix sockets only!)
cargo build --release

# NO HTTP dependencies, 100% pure Rust (including transitive!)
# Requires Songbird AI proxy for external AI providers
```

### Development Mode - Direct HTTP

```bash
# Build for development (includes HTTP adapters)
cargo build --release --features dev-direct-http

# Includes OpenAI, HuggingFace, Ollama adapters
# Fast iteration, no Songbird dependency
# Uses ~14 transitive ring/openssl deps
```

## Running Squirrel

### Production

```bash
# Set AI provider sockets (from Songbird)
export AI_PROVIDER_SOCKETS="/run/user/1000/songbird-ai-openai.sock,/run/user/1000/songbird-ai-huggingface.sock"

# Run Squirrel
./target/release/squirrel
```

### Development

```bash
# Set API keys for direct access
export OPENAI_API_KEY="sk-..."
export HUGGINGFACE_API_KEY="hf_..."

# Run Squirrel with dev features
cargo run --features dev-direct-http
```
```

---

## 📊 **Verification Checklist**

### Build Verification

- [ ] `cargo build --release` succeeds (production mode)
- [ ] `cargo build --release --features dev-direct-http` succeeds (dev mode)
- [ ] Binary size: Production ~14MB, Dev ~17MB
- [ ] Compile time: Production ~1.5min, Dev ~2min

### Dependency Verification

- [ ] Production: `cargo tree | grep ring` → Empty
- [ ] Production: `cargo tree | grep openssl` → Empty
- [ ] Dev: `cargo tree --features dev-direct-http | grep ring` → Found
- [ ] Dev: `cargo tree --features dev-direct-http | grep openssl` → Found

### Test Verification

- [ ] `cargo test --release` → All pass (production)
- [ ] `cargo test --release --features dev-direct-http` → All pass (dev)
- [ ] Integration tests pass (both modes)

### Functionality Verification

- [ ] Production: UniversalAiAdapter works (Unix sockets)
- [ ] Dev: HTTP adapters work (OpenAI, HuggingFace, Ollama)
- [ ] Logging clearly indicates mode (PRODUCTION vs DEVELOPMENT)
- [ ] Error messages guide users to correct mode

---

## 🎯 **Success Criteria**

### Technical

- ✅ Production build: **ZERO** `ring`/`openssl` transitive deps
- ✅ Dev build: HTTP adapters functional
- ✅ All 187 tests passing (both modes)
- ✅ Binary size: ~3MB smaller in production
- ✅ Compile time: ~30s faster in production

### Architectural

- ✅ Clear separation: Production vs Dev modes
- ✅ Feature flag system: Clean and intuitive
- ✅ Backward compatible: Existing deployments unaffected
- ✅ Prepared for Songbird integration (Unix sockets ready)

### Documentation

- ✅ README updated with build instructions
- ✅ Configuration examples for both modes
- ✅ Clear migration guide
- ✅ Developer onboarding improved

---

## 🚀 **Next Steps After Local Implementation**

### Immediate (This Session)

1. ✅ Implement Cargo feature flags
2. ✅ Conditional compilation for HTTP adapters
3. ✅ AiRouter refactoring
4. ✅ Test both modes
5. ✅ Update documentation

### Coordination with Songbird (Future Session)

1. Share v1.1.0 implementation with Songbird team
2. Coordinate AI proxy service development
3. Define Unix socket API contract
4. Plan joint E2E testing
5. Deploy v1.1.0 in coordination

### Deployment (After Songbird Ready)

1. Test Squirrel (production) + Songbird (AI proxy) locally
2. E2E validation with real AI providers
3. Performance benchmarking
4. Deploy to biomeOS plasmidBin
5. Update documentation and announce v1.1.0!

---

## 📚 **Related Documentation**

- **[SQUIRREL_ZERO_HTTP_EVOLUTION_JAN_16_2026.md](SQUIRREL_ZERO_HTTP_EVOLUTION_JAN_16_2026.md)** - Full architectural design
- **[SQUIRREL_CONCENTRATED_GAP_ALIGNMENT_JAN_16_2026.md](SQUIRREL_CONCENTRATED_GAP_ALIGNMENT_JAN_16_2026.md)** - Ecosystem alignment
- **[CURRENT_STATUS.md](CURRENT_STATUS.md)** - Latest status

---

## 🎊 **Conclusion**

This local evolution plan prepares Squirrel for **Zero-HTTP** production mode while maintaining full development flexibility.

**Timeline**: 2-3 hours (Squirrel-only)  
**Complexity**: Low-Medium  
**Value**: Foundation for 100% pure Rust! 🏆

After this implementation:
- ✅ Squirrel can run in pure Rust mode (production)
- ✅ Developers can iterate fast (dev mode)
- ✅ Ready for Songbird AI proxy integration
- ✅ Clear path to v1.1.0 (A++ grade!)

🦀 **Zero HTTP. Pure Rust. TRUE PRIMAL.** 🌱✨

---

**Created**: January 16, 2026  
**Status**: Ready to implement  
**Target**: v1.1.0 release

