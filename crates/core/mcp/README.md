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

## Integration status

**Sync** and **watch_task** are compiled but not enabled in default builds. They are intended for tighter coordination with ecosystem service discovery and lifecycle hooks (ports, health, failover). Re-enabling them is tracked as internal work against the existing MCP task and monitoring layers—no external component handoff is required.

**Likely integration points when enabled:**
- `sync/` — distributed state synchronization
- `task/` — service task orchestration
- `monitoring/` — health and metrics collection
- `context_manager.rs` — shared configuration management

## Current status
- **Compilation:** ✅ All errors resolved (0 compilation errors)
- **Tests:** ✅ Ready for testing
- **Disabled features:** sync, watch_task (off by default until wired to deployment policy)
