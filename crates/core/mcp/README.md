# MCP Core Module

## Overview
Core functionality for the Machine Context Protocol (MCP) system, including context management, task orchestration, and distributed synchronization.

## Features
- ✅ Context management and validation
- ✅ Task creation, assignment, and lifecycle management
- ✅ Health monitoring and metrics collection
- ✅ Plugin security and sandboxing
- ✅ Distributed tracing and observability
- ⚠️ Sync functionality (temporarily disabled)
- ⚠️ Watch_task real-time monitoring (temporarily disabled)

## Integration Status

### 🔗 **Nestgate Port Manager Integration**
**Status:** Waiting for handoff from NAS team

The sync and watch_task features are designed to integrate with the Nestgate port manager for:
- **Service orchestration** - Port assignment and service deployment coordination
- **Health monitoring** - Real-time service status tracking
- **Recovery management** - Automated failover and service recovery
- **Context synchronization** - Shared state management across distributed services

**Next Steps:**
1. Re-enable sync functionality once Nestgate port manager is available
2. Implement watch_task for real-time service monitoring
3. Create service adapters for port manager integration
4. Design notification systems for service lifecycle events

**Integration Points:**
- `sync/` module - Distributed state synchronization
- `task/` module - Service task orchestration  
- `monitoring/` module - Health and metrics collection
- `context_manager.rs` - Shared configuration management

## Current Status
- **Compilation:** ✅ All errors resolved (0 compilation errors)
- **Tests:** ✅ Ready for testing
- **Disabled Features:** sync, watch_task (awaiting Nestgate integration) 