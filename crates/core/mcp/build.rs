// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Proto files have been moved to ecosystem projects:
    // - mcp_task.proto → ToadStool (task execution)
    // - mcp_sync.proto → NestGate (data synchronization)
    //
    // Squirrel MCP core now focuses purely on JSON-RPC protocol
    // and no longer compiles gRPC proto files

    println!("cargo:rerun-if-changed=src/");

    Ok(())
}
