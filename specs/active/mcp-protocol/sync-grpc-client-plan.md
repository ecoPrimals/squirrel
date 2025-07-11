"""
---
version: 1.0.0
last_updated: 2025-04-11
status: proposed
priority: high
crossRefs:
  - ../mcp-adapters/pyo3-integration-plan.md
  - ../mcp-adapters/architecture-overview.md
---

# MCP Context Synchronization - gRPC Client Implementation Plan

## 1. Overview

This specification outlines the plan to implement the client-side network communication logic within the `MCPSync` component (`crates/mcp/src/sync/mod.rs`). This logic will enable instances of the Rust MCP core (potentially running within Python workers via PyO3 bindings) to synchronize their context state with a central context server using gRPC.

## 2. Goal

Replace the current local-only placeholder logic in `MCPSync::sync_internal` with a gRPC client implementation that:

1.  Connects to a configured central context server.
2.  Sends locally queued context changes (`StateChange` or equivalent) to the server.
3.  Receives context changes from the server that occurred since the client's last known state.
4.  Applies received changes locally.
5.  Updates local synchronization metadata (version, timestamp).

## 3. Technology

*   **RPC Framework:** gRPC
*   **Rust Crates:**
    *   `tonic`: For gRPC client/server implementation.
    *   `prost`: For Protobuf message definition and code generation.
    *   `tonic-build`, `prost-build`: For code generation during the build process.
    *   (These dependencies have already been added to `crates/mcp/Cargo.toml`).

## 4. Implementation Steps

1.  **Define Protobuf Schema (`proto/mcp_sync.proto`):**
    *   _Status: Done (Basic schema defined)._
    *   Create a `proto` directory...
    *   Define `SyncService`, `SyncRequest`, `SyncResponse`, `ContextChange`.
    *   Data Mapping Strategy: Using `bytes` for `data`/`metadata` (via JSON serialization).

2.  **Create Build Script (`crates/mcp/build.rs`):
    *   _Status: Done (Using absolute paths)._
    *   Use `tonic_build`...
    *   Include generated code...

3.  **Update Sync Configuration (`crates/mcp/src/sync/mod.rs`):
    *   _Status: Done._
    *   Added `central_server_url` to `SyncConfig`.

4.  **Refactor `MCPSync::sync_internal` (`crates/mcp/src/sync/mod.rs`):
    *   _Status: Partially Implemented._
    *   Added gRPC client connection logic (`SyncServiceClient`).
    *   Added helper functions (`state_change_to_proto`, `proto_to_state_change`) for basic conversion (using JSON serialization to bytes).
    *   Added logic to send local changes (currently takes *all* from `self.changes`).
    *   Added logic to receive remote changes and apply them via `self.state_manager.apply_change` (using conversion helper).
    *   **TODO:** Implement proper local change retrieval based on `last_known_version` (don't just clone/clear `self.changes`).
    *   **TODO:** Refine conversion helpers (error handling, efficiency).
    *   **TODO:** Implement robust gRPC/network error handling (retries?).
    *   **TODO:** Handle client identification properly (replace `"temp_client_id"`).
    *   **TODO:** Decide on client-side persistence strategy post-sync.

5.  **Update `MCPSync::record_context_change`:**
    *   _Status: Implemented (Calls `state_manager.record_change`)._
    *   **TODO:** Ensure changes recorded here are correctly fetched/managed by the refined `sync_internal` logic (step 4 TODO).

6.  **Error Handling:**
    *   _Status: Basic mapping implemented._
    *   **TODO:** Implement more specific and robust error handling/retries.

## 5. Testing

*   _Status: Pending._
*   Unit tests for conversion functions.
*   Integration tests with mock/stub gRPC server.
*   End-to-end tests with central server.

## 6. Proof-of-Concept Plan (Simulated Sync)

To verify local state changes and application before the full server is ready:
1.  Temporarily modify `MCPSync::sync_internal`: Instead of making a gRPC call, construct a mock `SyncResponse` containing predefined `ProtoContextChange` messages.
2.  In the Python test suite (`test_mcp_bindings.py`):
    *   Create context A.
    *   Call `sync_py()`. The modified Rust code will simulate receiving changes (e.g., creation of context B) and apply them via `state_manager`.
    *   Call `get_context_py()` for context B.
    *   Assert that context B exists and has the expected data.

""" 