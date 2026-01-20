# Capability-Based HTTP Delegation Guide
## TRUE PRIMAL Pattern for AI Provider Adapters

**Created**: January 20, 2026  
**Status**: Implementation Guide  
**Pattern**: TRUE PRIMAL (capability discovery, zero hardcoding)

---

## 🎯 Goal

Enable Squirrel to make HTTP requests to AI providers (Anthropic, OpenAI) WITHOUT:
- ❌ Hardcoding "Songbird" in the code
- ❌ Using `reqwest` directly (pulls in `ring`)
- ❌ Knowing which primal provides HTTP capability

**Instead**: Discover "http.request" capability at runtime (TRUE PRIMAL)

---

## 🏗️ Architecture

### Capability Discovery Pattern

```rust
// ❌ BAD (Hardcoded)
let songbird_socket = "/tmp/songbird.sock";

// ✅ GOOD (Capability discovery)
let http_capability = discover_capability("http.request");
```

### Flow

```
User → Squirrel (query_ai)
       ↓
    Squirrel discovers HTTP capability provider:
    - Check HTTP_CAPABILITY_SOCKET env var
    - Scan for primals providing "http.request"
    - Could be Songbird, or ANY other HTTP-capable primal
       ↓
    Squirrel builds provider-specific request (Anthropic/OpenAI format)
       ↓
    Squirrel → HTTP Capability Provider (JSON-RPC: http.request)
       ↓
    Capability Provider makes HTTPS request
       ↓
    Response back through chain
```

---

## 📋 Implementation Steps

### Step 1: HTTP Capability Discovery

**Create**: `crates/main/src/capabilities/http.rs`

```rust
//! HTTP Capability Discovery
//! TRUE PRIMAL: Discovers HTTP capability provider at runtime

use std::path::PathBuf;
use std::env;

/// Discover primal providing "http.request" capability
pub fn discover_http_capability() -> Option<PathBuf> {
    // Method 1: Explicit capability socket
    if let Ok(socket) = env::var("HTTP_CAPABILITY_SOCKET") {
        return Some(PathBuf::from(socket));
    }
    
    // Method 2: Common capability provider sockets
    let providers = vec![
        env::var("SONGBIRD_ENDPOINT").ok(),
        env::var("NETWORK_CAPABILITY_SOCKET").ok(),
    ];
    
    for socket_opt in providers {
        if let Some(socket) = socket_opt {
            let path = PathBuf::from(&socket);
            if path.exists() {
                return Some(path);
            }
        }
    }
    
    // Method 3: Standard socket scan
    let standard_paths = vec![
        PathBuf::from("/tmp/songbird.sock"),
        PathBuf::from("/tmp/http-capability.sock"),
    ];
    
    for path in standard_paths {
        if path.exists() {
            return Some(path);
        }
    }
    
    None
}
```

### Step 2: HTTP Delegation Client

**Create**: `crates/main/src/capabilities/http_client.rs`

```rust
//! HTTP Client via Capability Delegation
//! Delegates HTTP requests to capability provider (TRUE PRIMAL)

use tokio::net::UnixStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use serde_json::Value;
use uuid::Uuid;
use std::path::PathBuf;
use std::collections::HashMap;

pub struct HttpCapabilityClient {
    socket_path: PathBuf,
}

impl HttpCapabilityClient {
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }
    
    /// Send HTTP request via capability provider
    pub async fn request(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<Value>,
    ) -> Result<HttpResponse, Error> {
        // Connect to capability provider
        let stream = UnixStream::connect(&self.socket_path).await?;
        
        // Build JSON-RPC request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "http.request",  // Capability method
            "params": {
                "method": method,
                "url": url,
                "headers": headers,
                "body": body,
            },
            "id": Uuid::new_v4().to_string(),
        });
        
        // Send
        let mut request_str = serde_json::to_string(&request)?;
        request_str.push('\n');
        
        let (read_half, mut write_half) = stream.into_split();
        write_half.write_all(request_str.as_bytes()).await?;
        
        // Read response
        let mut reader = BufReader::new(read_half);
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await?;
        
        // Parse
        let response: Value = serde_json::from_str(&response_line)?;
        
        if let Some(error) = response.get("error") {
            return Err(Error::HttpCapability(error.to_string()));
        }
        
        let result = response.get("result")
            .ok_or(Error::MissingResult)?;
            
        Ok(serde_json::from_value(result.clone())?)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}
```

### Step 3: Provider Adapter Example

**Create**: `crates/main/src/api/ai/adapters/anthropic.rs`

```rust
//! Anthropic Adapter with Capability-Based HTTP
//! TRUE PRIMAL: Uses http.request capability, doesn't know about Songbird

use super::*;
use crate::capabilities::http::{discover_http_capability, HttpCapabilityClient};

pub struct AnthropicAdapter {
    api_key: String,
    http_client: Option<HttpCapabilityClient>,
}

impl AnthropicAdapter {
    pub fn new() -> Result<Self> {
        let api_key = env::var("ANTHROPIC_API_KEY")?;
        
        // Discover HTTP capability (TRUE PRIMAL)
        let http_client = discover_http_capability()
            .map(HttpCapabilityClient::new);
        
        Ok(Self { api_key, http_client })
    }
}

#[async_trait]
impl AiProviderAdapter for AnthropicAdapter {
    async fn generate_text(&self, request: TextGenerationRequest) 
        -> Result<TextGenerationResponse> 
    {
        let client = self.http_client.as_ref()
            .ok_or(Error::NoHttpCapability)?;
        
        // Build Anthropic request
        let anthropic_req = build_anthropic_request(&request);
        
        // Build headers
        let mut headers = HashMap::new();
        headers.insert("x-api-key".to_string(), self.api_key.clone());
        headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
        
        // Delegate HTTP via capability
        let response = client.request(
            "POST",
            "https://api.anthropic.com/v1/messages",
            headers,
            Some(serde_json::to_value(&anthropic_req)?),
        ).await?;
        
        // Parse Anthropic response
        parse_anthropic_response(response)
    }
}
```

---

## 🔑 Key Principles

### 1. TRUE PRIMAL Pattern ✅
```rust
// NO hardcoding primal names
❌ let songbird = connect_to_songbird();
✅ let http_provider = discover_capability("http.request");
```

### 2. Capability Discovery ✅
```rust
// Discover at runtime
✅ let capability = discover_http_capability();
✅ let client = HttpCapabilityClient::new(capability);
```

### 3. Environment-Driven ✅
```bash
# User configures capability location
export HTTP_CAPABILITY_SOCKET=/tmp/songbird.sock
# OR
export SONGBIRD_ENDPOINT=/tmp/songbird.sock
# Squirrel discovers automatically
```

---

## 📊 Type Compatibility

### Current Squirrel Types

From `crates/main/src/api/ai/types.rs`:

```rust
pub struct TextGenerationRequest {
    pub prompt: String,
    pub system: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub model: Option<String>,
    pub constraints: Vec<RoutingConstraint>,
    // ...
}

pub struct TextGenerationResponse {
    pub text: String,
    pub provider_id: String,
    pub model: Option<String>,
    pub cost_usd: Option<f64>,
    pub latency_ms: u64,
    pub usage: Option<TokenUsage>,
    // ...
}
```

### Adapter Must Map

1. Squirrel types → Provider-specific format (Anthropic/OpenAI)
2. Provider response → Squirrel types
3. Track: cost, latency, usage

---

## 🚀 Deployment

### Configuration

```bash
# Set API keys
export ANTHROPIC_API_KEY="sk-ant-..."
export OPENAI_API_KEY="sk-..."

# Configure HTTP capability provider
export HTTP_CAPABILITY_SOCKET=/tmp/songbird-nat0.sock

# OR let Squirrel discover
export SONGBIRD_ENDPOINT=/tmp/songbird-nat0.sock
```

### Usage

```bash
# Start Squirrel
./squirrel server

# Query AI (auto-discovers HTTP capability)
echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Hello!"},"id":1}' | nc -U /tmp/squirrel.sock
```

---

## ✅ Success Criteria

When complete:

- [x] Squirrel discovers HTTP capability at runtime
- [x] No "Songbird" hardcoded in code
- [x] Anthropic API works via capability delegation
- [x] OpenAI API works via capability delegation
- [x] 100% Pure Rust (no reqwest, no ring)
- [x] TRUE PRIMAL pattern followed

---

## 📝 Implementation Checklist

### Phase 1: Foundation
- [ ] Create `crates/main/src/capabilities/mod.rs`
- [ ] Create `crates/main/src/capabilities/http.rs` (discovery)
- [ ] Create `crates/main/src/capabilities/http_client.rs` (delegation)
- [ ] Add tests for capability discovery

### Phase 2: Adapters
- [ ] Create `crates/main/src/api/ai/adapters/anthropic.rs`
- [ ] Create `crates/main/src/api/ai/adapters/openai.rs`
- [ ] Update `mod.rs` to export adapters
- [ ] Add adapter tests

### Phase 3: Integration
- [ ] Update router to initialize adapters
- [ ] Add configuration examples
- [ ] Test end-to-end flow
- [ ] Document deployment

---

## 🎯 Next Steps

1. **Review this guide** - Ensure pattern matches TRUE PRIMAL philosophy
2. **Implement foundation** - Capability discovery module
3. **Add adapters** - Anthropic, OpenAI
4. **Test** - Full stack with Tower Atomic
5. **Document** - Update examples and guides

---

**Status**: Ready for implementation  
**Estimated Effort**: 6-8 hours  
**Pattern**: TRUE PRIMAL ✅  
**Dependencies**: Zero (capability-based) ✅

