# MCP Module Restructuring Plan

## Progress Update (2024-05-13)

We've made significant progress in implementing the MCP module restructuring plan, resolving most of the identified issues:

1. **Type Improvements**:
   - ✅ Implemented `ProtocolState` enum with proper variants and default implementation
   - ✅ Added `ProtocolVersion` struct with `Display` trait implementation
   - ✅ Fixed serialization for `MessageId` and `ErrorSeverity`
   - ✅ Implemented `Display` trait for `MessageType` enum

2. **Error Handling**:
   - ✅ Resolved duplicate `ProtocolError` definition issue
   - ✅ Added string conversion support to `SquirrelError`
   - ✅ Renamed `ErrorContext` to `LocalErrorContext` to resolve naming conflicts
   - ✅ Improved error propagation between module layers

3. **Module Structure**:
   - ✅ Fixed imports across the MCP system
   - ✅ Added proper async handling for RwLock contexts
   - ✅ Updated protocol handlers to use type-safe registration
   - ✅ Resolved trait conflicts between `CommandHandler` and `MessageHandler`
   - ✅ Fixed method signature differences in trait implementations
   - ✅ Updated adapter to use consistent types and return values

## Current Issues

1. **Testing and Documentation**:
   - ⚠️ Integration tests need updating to use new interface patterns
   - ⚠️ Protocol adapter needs comprehensive testing
   - ⚠️ Need to document examples of the new DI patterns

## Next Steps

1. **Testing Improvements**:
   - Create comprehensive test suite for protocol adapter
   - Update existing tests to use new interfaces
   - Add tests for error conditions and edge cases

2. **Documentation Updates**:
   - Create migration guide for users of the previous APIs
   - Document examples of proper DI usage
   - Update interface documentation with examples

## Original Plan

The MCP (Message Control Protocol) module has significant structural issues that need to be addressed to properly implement Dependency Injection. This document outlines the plan for restructuring the module to address these issues.

## Current Issues

1. **Type Name Conflicts**: 
   - The name `MCPProtocol` is defined multiple times (as a trait and a struct)
   - `EncryptionFormat` is defined and reimported in multiple locations

2. **Initialization Patterns**:
   - Adapters use "initialize on-demand" fallbacks
   - Missing explicit initialization requirements

3. **Module Organization**:
   - Circular dependencies between modules
   - Inconsistent type exports and imports

## Goals

1. Eliminate type name conflicts
2. Remove "initialize on-demand" fallbacks
3. Implement proper error handling for uninitialized adapters
4. Restructure modules to avoid circular dependencies
5. Standardize type definitions and exports

## Implementation Plan

### Phase 1: Reorganize Type Definitions

1. **Create a centralized types module**:

```rust
// crates/core/src/mcp/types.rs
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// MCP Protocol version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolVersion {
    V1,
    V2,
    // ...
}

/// MCP Message types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Request,
    Response,
    Notification,
    // ...
}

/// MCP Encryption formats
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionFormat {
    None,
    AES256,
    // ...
}

// Other shared types...
```

2. **Rename conflicting types**:

```rust
// crates/core/src/mcp/protocol/mod.rs

// Rename the trait to avoid conflict
pub trait MCPProtocolTrait: Send + Sync {
    fn handle_message(&self, message: &MCPMessage) -> Result<MCPMessage>;
    // ...
}

// Implementation struct
pub struct MCPProtocolImpl {
    // ...
}

impl MCPProtocolTrait for MCPProtocolImpl {
    // ...
}

// The main exported type
pub struct MCPProtocol {
    inner: Arc<dyn MCPProtocolTrait>,
}
```

### Phase 2: Restructure Error Types

1. **Create specific error types for each module**:

```rust
// crates/core/src/mcp/errors.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MCPError {
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    
    #[error("Context error: {0}")]
    Context(#[from] ContextError),
    
    #[error("General error: {0}")]
    General(String),
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Protocol not initialized")]
    NotInitialized,
    
    #[error("Protocol already initialized")]
    AlreadyInitialized,
    
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),
    
    // Other protocol-specific errors
}

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("Context not initialized")]
    NotInitialized,
    
    #[error("Context already initialized")]
    AlreadyInitialized,
    
    // Other context-specific errors
}
```

### Phase 3: Update Module Exports

1. **Restructure module exports**:

```rust
// crates/core/src/mcp/mod.rs
pub mod types;
pub mod errors;
pub mod protocol;
pub mod context;

// Re-export key types
pub use types::{ProtocolVersion, MessageType, EncryptionFormat, MCPMessage};
pub use errors::{MCPError, ProtocolError, ContextError};
pub use protocol::{MCPProtocolTrait, MCPProtocol, MCPProtocolAdapter};
pub use context::{MCPContext, MCPContextAdapter};
```

### Phase 4: Fix Adapter Implementations

1. **Update Protocol Adapter**:

```rust
// crates/core/src/mcp/protocol/adapter.rs
use crate::mcp::errors::ProtocolError;
use crate::mcp::types::MCPMessage;
use std::sync::Arc;

pub struct MCPProtocolAdapter {
    inner: Option<Arc<MCPProtocol>>,
}

impl MCPProtocolAdapter {
    pub fn new() -> Self {
        Self { inner: None }
    }
    
    pub fn with_protocol(protocol: Arc<MCPProtocol>) -> Self {
        Self { inner: Some(protocol) }
    }
    
    pub fn is_initialized(&self) -> bool {
        self.inner.is_some()
    }
    
    pub fn initialize(&mut self) -> Result<(), ProtocolError> {
        if self.is_initialized() {
            return Err(ProtocolError::AlreadyInitialized);
        }
        
        let config = ProtocolConfig::default();
        let protocol = MCPProtocol::new(config);
        self.inner = Some(Arc::new(protocol));
        Ok(())
    }
    
    pub fn handle_message(&self, message: &MCPMessage) -> Result<MCPMessage, ProtocolError> {
        match &self.inner {
            Some(protocol) => protocol.handle_message(message),
            None => Err(ProtocolError::NotInitialized)
        }
    }
    
    // Other methods...
}

// Factory functions
pub fn create_initialized_protocol_adapter() -> Result<MCPProtocolAdapter, ProtocolError> {
    let mut adapter = MCPProtocolAdapter::new();
    adapter.initialize()?;
    Ok(adapter)
}

pub fn create_protocol_adapter_with_config(config: ProtocolConfig) -> Result<MCPProtocolAdapter, ProtocolError> {
    let mut adapter = MCPProtocolAdapter::new();
    adapter.initialize_with_config(config)?;
    Ok(adapter)
}
```

2. **Update Context Adapter**:

```rust
// Similar to Protocol Adapter, update with proper initialization and error handling
```

### Phase 5: Update Tests

1. **Add test module to lib.rs**:

```rust
// crates/core/src/lib.rs
#[cfg(test)]
mod mcp_tests;
```

2. **Create test module**:

```rust
// crates/core/src/mcp_tests.rs
#[cfg(test)]
mod protocol_tests {
    use crate::mcp::protocol::{MCPProtocolAdapter, ProtocolConfig};
    use crate::mcp::types::MCPMessage;
    
    #[tokio::test]
    async fn test_protocol_adapter() {
        // Create and initialize
        let mut adapter = MCPProtocolAdapter::new();
        adapter.initialize().unwrap();
        
        // Test operations
        let message = MCPMessage::default();
        let result = adapter.handle_message(&message).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_uninitialized_adapter() {
        // Create without initializing
        let adapter = MCPProtocolAdapter::new();
        
        // Should fail on operations
        let message = MCPMessage::default();
        let result = adapter.handle_message(&message).await;
        assert!(result.is_err());
    }
}
```

## Migration Steps

1. **Create New Types Module**:
   - Create `types.rs` with centralized type definitions
   - Update imports in other modules

2. **Rename Conflicting Types**:
   - Change `MCPProtocol` trait to `MCPProtocolTrait`
   - Update all implementations

3. **Restructure Error Types**:
   - Create dedicated error module
   - Define specific error types

4. **Update Module Exports**:
   - Reorganize exports in `mod.rs`
   - Ensure consistent imports

5. **Fix Adapter Implementations**:
   - Remove "initialize on-demand" fallbacks
   - Add proper error handling
   - Add factory functions

6. **Update Tests**:
   - Create new test module
   - Update tests for explicit initialization

## Timeline

- **Day 1**: Create types module and restructure error types
- **Day 2**: Rename conflicting types and update module exports
- **Day 3**: Fix adapter implementations
- **Day 4**: Update tests and documentation
- **Day 5**: Verify all changes and run tests

## Testing Strategy

1. **Unit Tests**:
   - Test each adapter in isolation
   - Verify initialization behavior
   - Test error handling

2. **Integration Tests**:
   - Test interactions between components
   - Verify proper dependency injection
   - Test with real message examples

## Success Criteria

1. No more type conflicts
2. No more "initialize on-demand" fallbacks
3. Clear error handling for uninitialized adapters
4. All tests pass
5. Documentation updated with examples 