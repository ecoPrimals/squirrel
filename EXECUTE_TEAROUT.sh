#!/bin/bash

set -e  # Exit on any error

echo "🚀 Starting Squirrel Tearout & Refocus Process"
echo "=============================================="

# Phase 1: Backup and Prepare
echo ""
echo "📦 Phase 1: Creating Backup..."
BACKUP_BRANCH="tearout-backup-$(date +%Y%m%d-%H%M%S)"
git checkout -b "$BACKUP_BRANCH" 2>/dev/null || echo "Branch $BACKUP_BRANCH already exists"
git add -A
git commit -m "backup: pre-tearout state" || echo "No changes to commit"

# Create working branch
WORKING_BRANCH="compute-tearout-toadstool-integration"
git checkout -b "$WORKING_BRANCH" 2>/dev/null || git checkout "$WORKING_BRANCH"

echo "✅ Backup created: $BACKUP_BRANCH"
echo "✅ Working branch: $WORKING_BRANCH"

# Phase 2: Verify toToadStool Migration (Already done)
echo ""
echo "🍄 Phase 2: Verifying Toadstool Migration..."
if [ -d "toToadStool/sandbox" ] && [ -d "toToadStool/resource-monitoring" ] && [ -d "toToadStool/sdk" ]; then
    echo "✅ Compute infrastructure successfully moved to toToadStool/"
    echo "   - Sandbox system: $(find toToadStool/sandbox -name "*.rs" | wc -l) files"
    echo "   - Resource monitoring: $(find toToadStool/resource-monitoring -name "*.rs" | wc -l) files"
    echo "   - SDK components: $(find toToadStool/sdk -name "*.rs" | wc -l) files"
else
    echo "❌ toToadStool migration incomplete. Please run the copy commands first."
    exit 1
fi

# Phase 3: Remove Orchestrator Code
echo ""
echo "🗑️ Phase 3: Removing Orchestrator Code..."

# Remove orchestrator service
if [ -d "code/crates/services/nestgate-orchestrator" ]; then
    echo "   Removing nestgate-orchestrator service..."
    rm -rf code/crates/services/nestgate-orchestrator/
    echo "   ✅ Removed code/crates/services/nestgate-orchestrator/"
else
    echo "   ℹ️ nestgate-orchestrator already removed"
fi

# Remove orchestrator proto
if [ -f "code/proto/orchestrator.proto" ]; then
    echo "   Removing orchestrator.proto..."
    rm code/proto/orchestrator.proto
    echo "   ✅ Removed code/proto/orchestrator.proto"
else
    echo "   ℹ️ orchestrator.proto already removed"
fi

# Remove web orchestrator integration
if [ -d "code/crates/integration/web/src/orchestrator" ]; then
    echo "   Removing web orchestrator integration..."
    rm -rf code/crates/integration/web/src/orchestrator/
    echo "   ✅ Removed code/crates/integration/web/src/orchestrator/"
else
    echo "   ℹ️ web orchestrator integration already removed"
fi

if [ -f "code/crates/integration/web/src/tests/orchestrator_routing_tests.rs" ]; then
    echo "   Removing orchestrator routing tests..."
    rm code/crates/integration/web/src/tests/orchestrator_routing_tests.rs
    echo "   ✅ Removed orchestrator routing tests"
else
    echo "   ℹ️ orchestrator routing tests already removed"
fi

# Remove orchestrator adapters
ORCHESTRATOR_ADAPTERS=(
    "code/crates/tools/ai-tools/src/orchestrator_adapter.rs"
    "code/crates/services/commands/src/orchestrator_adapter.rs"
    "code/crates/core/mcp/src/task/tests/orchestrator_integration_tests.rs"
)

for adapter in "${ORCHESTRATOR_ADAPTERS[@]}"; do
    if [ -f "$adapter" ]; then
        echo "   Removing $adapter..."
        rm "$adapter"
        echo "   ✅ Removed $adapter"
    else
        echo "   ℹ️ $adapter already removed"
    fi
done

# Phase 4: Create Integration Stubs
echo ""
echo "🔌 Phase 4: Creating Integration Stubs..."

# Create toadstool integration directory
mkdir -p code/crates/integration/toadstool/src
echo "✅ Created code/crates/integration/toadstool/"

# Create ecosystem integration directory
mkdir -p code/crates/integration/ecosystem/src
echo "✅ Created code/crates/integration/ecosystem/"

# Create basic Cargo.toml for toadstool integration
cat > code/crates/integration/toadstool/Cargo.toml << 'EOF'
[package]
name = "toadstool-integration"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
thiserror = "1.0"

# Will be added when Toadstool client is available
# toadstool-client = { path = "../../../toadstool/client" }
EOF

# Create basic lib.rs for toadstool integration
cat > code/crates/integration/toadstool/src/lib.rs << 'EOF'
//! Toadstool-Compute integration for Squirrel MCP
//!
//! This module provides integration with Toadstool-Compute for plugin execution.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Execution request to Toadstool-Compute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub plugin_id: String,
    pub execution_environment: ExecutionEnvironment,
    pub sandbox_config: SandboxConfig,
    pub mcp_context: serde_json::Value,
}

/// Execution environment types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEnvironment {
    Wasm,
    Native,
    Container,
}

/// Sandbox configuration (simplified for now)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub security_level: String,
    pub resource_limits: serde_json::Value,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            security_level: "standard".to_string(),
            resource_limits: serde_json::json!({}),
        }
    }
}

/// Toadstool client (stub for now)
pub struct ToadstoolClient {
    // Will be implemented when Toadstool client is available
}

impl ToadstoolClient {
    pub async fn new() -> Result<Self> {
        // TODO: Connect to Toadstool-Compute via Songbird
        Ok(Self {})
    }
    
    pub async fn execute_plugin(&self, _request: ExecutionRequest) -> Result<serde_json::Value> {
        // TODO: Send execution request to Toadstool-Compute
        anyhow::bail!("Toadstool integration not yet implemented")
    }
}
EOF

echo "✅ Created Toadstool integration stub"

# Phase 5: Test Compilation
echo ""
echo "🧪 Phase 5: Testing Compilation..."
echo "   Running cargo check..."
if cargo check --workspace 2>/dev/null; then
    echo "✅ Project compiles successfully after tearout!"
else
    echo "⚠️ Compilation issues detected. Manual fixes may be needed."
    echo "   Common issues to check:"
    echo "   - Update Cargo.toml workspace members"
    echo "   - Remove orchestrator imports"
    echo "   - Update module references"
fi

# Phase 6: Summary
echo ""
echo "📋 Phase 6: Tearout Summary"
echo "=========================="
echo ""
echo "✅ COMPLETED:"
echo "   - Compute infrastructure moved to toToadStool/"
echo "   - Orchestrator code removed from Squirrel"
echo "   - Integration stubs created"
echo "   - Project structure cleaned up"
echo ""
echo "🎯 NEXT STEPS:"
echo "   1. Update Cargo.toml workspace members"
echo "   2. Remove any remaining orchestrator imports"
echo "   3. Test MCP functionality"
echo "   4. Coordinate with Toadstool team for integration"
echo "   5. Establish Songbird routing"
echo ""
echo "📁 FILES MOVED TO TOADSTOOL:"
echo "   - Sandbox system (toToadStool/sandbox/)"
echo "   - Resource monitoring (toToadStool/resource-monitoring/)"
echo "   - SDK components (toToadStool/sdk/)"
echo ""
echo "🐿️ SQUIRREL NOW FOCUSES ON:"
echo "   - Pure MCP platform excellence"
echo "   - AI agent coordination"
echo "   - Plugin registry and metadata"
echo "   - Context management"
echo ""
echo "🍄 Ready for Toadstool-Compute integration!"
echo "🎼 Ready for Songbird orchestration!"
echo ""
echo "Backup branch: $BACKUP_BRANCH"
echo "Working branch: $WORKING_BRANCH"
echo ""
echo "🚀 Tearout completed successfully!" 