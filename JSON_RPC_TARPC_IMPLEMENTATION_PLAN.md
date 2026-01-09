# JSON-RPC + tarpc Implementation Plan for Squirrel

**Date**: January 9, 2026  
**Priority**: HIGH (biomeOS integration blocker)  
**Estimated Time**: 12-16 hours  
**Status**: Planning Phase

---

## 🎯 Objective

Evolve Squirrel to support modern inter-primal communication protocols:
1. **JSON-RPC 2.0** over Unix sockets (for biomeOS integration)
2. **tarpc** for high-performance RPC (for peer-to-peer)
3. Maintain backward compatibility with existing REST API

---

## 📋 Requirements from biomeOS Team

From the biomeOS handoff message:

> **"We need to evolve to be json-rpc and tarpc first like ../songbird/ and ../beardog/"**

### Key Points:
1. Primal code only has self-knowledge
2. Discovers other primals at runtime
3. Capability-based discovery (not hardcoded names)
4. Unix socket IPC for local communication
5. tarpc for high-performance remote communication
6. JSON-RPC 2.0 for biomeOS coordination

---

## 🏗️ Architecture Reference: Songbird v3.19.3

### What Songbird Implemented:

**1. Unix Socket JSON-RPC Server**
- Socket path: `/tmp/songbird-{node_id}.sock`
- Protocol: JSON-RPC 2.0 (newline-delimited)
- Library: `jsonrpsee` v0.24
- 3 core APIs:
  - `discover_by_family` - Filter peers by genetic family
  - `create_genetic_tunnel` - Establish BTSP tunnel
  - `announce_capabilities` - Broadcast capabilities

**2. tarpc High-Performance RPC**
- Version: 0.34
- Use case: Peer-to-peer communication
- Features: Native Rust RPC, high throughput

**3. Hybrid Architecture**
- REST API (existing, for HTTP clients)
- Unix Socket + JSON-RPC (for biomeOS)
- tarpc (for peer-to-peer)
- Intelligent protocol selection based on context

---

## 🎨 Proposed Squirrel Architecture

### Phase 1: JSON-RPC Over Unix Sockets (4-6 hours)

**Goal**: Enable biomeOS to communicate with Squirrel via Unix socket

**Implementation**:
```
/tmp/squirrel-{node_id}.sock  ← Unix socket
    ↓
JSON-RPC 2.0 Server (jsonrpsee)
    ↓
Squirrel Core APIs
```

**APIs to Expose**:
1. **`query_ai`** - Send AI inference request
   ```json
   {
     "jsonrpc": "2.0",
     "method": "query_ai",
     "params": {
       "prompt": "Analyze system topology",
       "provider": "auto",
       "priority": 5
     },
     "id": 1
   }
   ```

2. **`list_providers`** - Get available AI providers
   ```json
   {
     "jsonrpc": "2.0",
     "method": "list_providers",
     "params": {},
     "id": 2
   }
   ```

3. **`announce_capabilities`** - Advertise Squirrel capabilities
   ```json
   {
     "jsonrpc": "2.0",
     "method": "announce_capabilities",
     "params": {
       "capabilities": [
         "ai.inference",
         "ai.multi-provider",
         "ai.local-ollama",
         "mcp.protocol"
       ]
     },
     "id": 3
   }
   ```

4. **`health_check`** - Get health status
   ```json
   {
     "jsonrpc": "2.0",
     "method": "health_check",
     "params": {},
     "id": 4
   }
   ```

**Dependencies to Add**:
```toml
[dependencies]
jsonrpsee = { version = "0.24", features = ["server", "client"] }
tokio-util = { version = "0.7", features = ["codec"] }
```

**Files to Create**:
- `crates/main/src/rpc/mod.rs` - RPC module entry
- `crates/main/src/rpc/types.rs` - Request/Response DTOs
- `crates/main/src/rpc/server.rs` - JSON-RPC server
- `crates/main/src/rpc/handlers.rs` - API handlers
- `crates/main/src/rpc/unix_socket.rs` - Unix socket setup

---

### Phase 2: tarpc Protocol Implementation (4-6 hours)

**Goal**: Enable high-performance peer-to-peer communication

**Implementation**:
```
Squirrel A (tarpc client)
    ↓
Network (TCP/QUIC)
    ↓
Squirrel B (tarpc server)
```

**Service Definition**:
```rust
#[tarpc::service]
pub trait SquirrelService {
    /// Execute AI inference task
    async fn execute_ai_task(task: AITask, context: RequestContext) -> Result<AIResponse>;
    
    /// Get provider capabilities
    async fn get_capabilities() -> Vec<Capability>;
    
    /// Delegate task to another Squirrel
    async fn delegate_task(task: AITask, target_provider: String) -> Result<DelegationResponse>;
    
    /// Streaming AI response (optional)
    async fn stream_ai_response(task: AITask) -> impl Stream<Item = AIChunk>;
}
```

**Dependencies to Add**:
```toml
[dependencies]
tarpc = { version = "0.34", features = ["full"] }
bincode = "1.3"  # For efficient serialization
```

**Files to Create**:
- `crates/main/src/tarpc/mod.rs` - tarpc module entry
- `crates/main/src/tarpc/service.rs` - Service trait definition
- `crates/main/src/tarpc/server.rs` - tarpc server implementation
- `crates/main/src/tarpc/client.rs` - tarpc client wrapper

---

### Phase 3: Protocol Selection & Integration (2-3 hours)

**Goal**: Intelligent routing between protocols

**Selection Logic**:
```rust
enum CommunicationProtocol {
    UnixSocketJsonRpc,  // Local biomeOS communication
    TarpcTcp,           // Remote Squirrel-to-Squirrel
    RestHttp,           // External HTTP clients
}

impl Squirrel {
    fn select_protocol(&self, target: &Target) -> CommunicationProtocol {
        match target {
            Target::BiomeOS => UnixSocketJsonRpc,
            Target::RemoteSquirrel(node) => TarpcTcp,
            Target::ExternalClient => RestHttp,
        }
    }
}
```

**Integration Points**:
1. Update `main.rs` to start all servers concurrently
2. Update capability discovery to advertise protocols
3. Update routing logic to select protocol dynamically
4. Add integration tests

---

### Phase 4: Testing & Documentation (1-2 hours)

**Testing Strategy**:

1. **Unit Tests**:
   - JSON-RPC request/response parsing
   - tarpc service method implementations
   - Protocol selection logic

2. **Integration Tests**:
   - Unix socket JSON-RPC E2E
   - tarpc client-server E2E
   - Multi-protocol concurrent operation

3. **Manual Testing**:
   ```bash
   # Test Unix socket
   echo '{"jsonrpc":"2.0","method":"health_check","params":{},"id":1}' | \
     nc -U /tmp/squirrel-test.sock
   
   # Test tarpc
   cargo run --example tarpc_client -- --target localhost:9011
   ```

**Documentation**:
- `RPC_ARCHITECTURE.md` - Overall architecture
- `JSON_RPC_API.md` - JSON-RPC API reference
- `TARPC_USAGE.md` - tarpc usage guide
- Update `ENVIRONMENT_VARIABLES.md` with new config

---

## 📊 Implementation Checklist

### Phase 1: JSON-RPC Unix Sockets ⏳
- [ ] Add jsonrpsee dependency
- [ ] Create RPC module structure
- [ ] Implement Unix socket server
- [ ] Define API methods
- [ ] Write handlers
- [ ] Add unit tests
- [ ] Integration test with mock biomeOS

### Phase 2: tarpc Implementation ⏳
- [ ] Add tarpc dependency
- [ ] Define service trait
- [ ] Implement server
- [ ] Implement client
- [ ] Add codec/transport layer
- [ ] Write unit tests
- [ ] Peer-to-peer integration test

### Phase 3: Integration ⏳
- [ ] Protocol selection logic
- [ ] Update main.rs startup
- [ ] Capability discovery updates
- [ ] Concurrent server management
- [ ] Error handling & graceful shutdown
- [ ] Integration tests

### Phase 4: Documentation ⏳
- [ ] Architecture documentation
- [ ] API reference docs
- [ ] Usage examples
- [ ] biomeOS integration guide
- [ ] Performance benchmarks

---

## 🎯 Success Criteria

1. **JSON-RPC Functional**: biomeOS can call Squirrel APIs via Unix socket
2. **tarpc Functional**: Two Squirrel instances can communicate
3. **Backward Compatible**: Existing REST API still works
4. **Well Tested**: >80% coverage of new code
5. **Documented**: Complete API reference and usage guides
6. **Performance**: <10ms latency for local JSON-RPC, <50ms for remote tarpc

---

## 🚀 Next Steps

1. ✅ **Complete**: Planning and architecture design
2. ⏳ **Next**: Implement Phase 1 (JSON-RPC + Unix sockets)
3. ⏳ **Then**: Implement Phase 2 (tarpc)
4. ⏳ **Then**: Integration and testing
5. ⏳ **Finally**: Documentation and handoff to biomeOS

**Estimated Total Time**: 12-16 hours over 2-3 days

---

## 📚 Reference Materials

### Songbird Implementation:
- `songbird/BIOMEOS_HANDOFF_V3_19_3.md` - Complete integration guide
- `songbird/Cargo.toml` - Dependencies (jsonrpsee, tarpc)
- `songbird/showcase/05-albatross-multiplex/` - tarpc examples

### Dependencies:
- jsonrpsee: https://docs.rs/jsonrpsee/
- tarpc: https://docs.rs/tarpc/

### Related Specs:
- `squirrel/specs/ecosystem_integration.md`
- `squirrel/ENVIRONMENT_VARIABLES.md`
- `squirrel/CAPABILITY_BASED_ARCHITECTURE.md`

---

## 💡 Design Principles

1. **Self-Knowledge Only**: Squirrel knows its own capabilities
2. **Runtime Discovery**: Discovers other primals dynamically
3. **Protocol Agnostic**: Uses best protocol for each context
4. **Backward Compatible**: Don't break existing integrations
5. **Performance First**: Optimize for low latency
6. **Secure by Default**: All communication authenticated/encrypted
7. **Graceful Degradation**: Falls back if preferred protocol unavailable

---

🐿️ **Ready to begin implementation!** 🦀

