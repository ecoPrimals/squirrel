---
title: Orchestrator Code Tearout & MCP Refocus Plan
description: Guide for Squirrel team to remove pre-pre-songbird orchestrator code and refocus on MCP platform
version: 1.0.0
date: 2025-01-26
author: Ecosystem Integration Team
priority: CRITICAL
status: TEAROUT_ACTIVE
---

# 🧹 Orchestrator Tearout & MCP Refocus Plan

## Executive Summary

The **orchestrator code in Squirrel** was **pre-pre-songbird prototype code** that has now evolved into the standalone **Songbird orchestrator project**. This plan guides the Squirrel team through:

1. **🗑️ Tearing out redundant orchestrator code**
2. **🎯 Refocusing on core MCP platform responsibilities**
3. **🔌 Integrating with production Songbird orchestrator**

---

## 🎯 **Squirrel's True Mission: MCP Platform**

### **What Squirrel Should Focus On**
```yaml
core_mcp_responsibilities:
  protocol_implementation: "Machine Context Protocol core"
  plugin_system: "Dynamic plugin loading with sandboxing"
  ai_integration: "AI model management and processing"
  web_interface: "MCP dashboard and control panel"
  authentication: "Auth service for MCP ecosystem"
  monitoring: "MCP service monitoring and metrics"
  command_registry: "MCP command processing system"

NOT_squirrel_responsibilities:
  service_orchestration: "❌ This is Songbird's job"
  load_balancing: "❌ This is Songbird's job"
  service_registry: "❌ This is Songbird's job"
  connection_proxy: "❌ This is Songbird's job"
```

### **Clear Value Proposition**
- **🐿️ Squirrel**: MCP platform that provides context and coordination
- **🎼 Songbird**: Universal orchestrator that manages services
- **🏠 NestGate**: Domain-specific NAS services

---

## 🔍 **Code Audit: What to Remove**

### **🗑️ Files/Directories to DELETE**

```bash
# Primary orchestrator service (entire directory)
code/crates/services/nestgate-orchestrator/

# Proto definition for orchestrator
code/proto/orchestrator.proto

# Web integration for orchestrator routing
code/crates/integration/web/src/orchestrator/
code/crates/integration/web/src/tests/orchestrator_routing_tests.rs

# Build scripts that compile orchestrator protos
# (Search and remove orchestrator references from build.rs files)
```

### **📝 Files to MODIFY (Remove Orchestrator References)**

```bash
# Main specs and documentation
SPECS.md                          # Remove "Nestgate Orchestrator (95% complete)"
README.md                         # Remove orchestrator references
specs/IMMEDIATE_ACTIONS.md        # Remove orchestrator testing tasks

# Web integration
code/crates/integration/web/src/lib.rs    # Remove orchestrator routing
code/crates/integration/web/src/tests/mod.rs  # Remove orchestrator test module

# Cargo configuration
Cargo.toml                        # Remove orchestrator workspace member
```

---

## 🛠️ **Step-by-Step Tearout Process**

### **Phase 1: Backup & Branch (5 minutes)**

```bash
# Create backup branch
git checkout -b orchestrator-tearout-backup
git commit -a -m "backup: pre-orchestrator tearout state"

# Create working branch
git checkout -b orchestrator-tearout
```

### **Phase 2: Remove Core Orchestrator Service (15 minutes)**

```bash
# 1. Remove the entire nestgate-orchestrator service
rm -rf code/crates/services/nestgate-orchestrator/

# 2. Remove proto definition
rm code/proto/orchestrator.proto

# 3. Update workspace Cargo.toml
# Remove "code/crates/services/nestgate-orchestrator" from members list
```

### **Phase 3: Clean Web Integration (20 minutes)**

```bash
# 1. Remove orchestrator web routing
rm -rf code/crates/integration/web/src/orchestrator/
rm code/crates/integration/web/src/tests/orchestrator_routing_tests.rs

# 2. Update web/src/lib.rs
# Remove: pub mod orchestrator;
# Remove: .nest("/api/orchestrator", orchestrator::orchestrator_router())
# Remove orchestrator-related comments

# 3. Update web/src/tests/mod.rs
# Remove: pub mod orchestrator_routing_tests;
```

### **Phase 4: Update Documentation (10 minutes)**

```bash
# 1. Update SPECS.md
# Remove: "- [x] **Nestgate Orchestrator** (95% complete) - *Performance optimized*"

# 2. Update README.md
# Remove orchestrator references in description
# Update to focus on MCP platform role

# 3. Update specs/IMMEDIATE_ACTIONS.md
# Remove orchestrator testing tasks
# Add Songbird integration tasks
```

### **Phase 5: Add Songbird Integration (15 minutes)**

```bash
# 1. Add Songbird dependency to root Cargo.toml
[dependencies]
songbird-orchestrator = { path = "../songbird" }

# 2. Create new integration module
mkdir -p code/crates/integration/songbird/
```

### **Phase 6: Test & Verify (10 minutes)**

```bash
# 1. Ensure project compiles
cargo check

# 2. Run tests (should pass without orchestrator tests)
cargo test

# 3. Verify no orchestrator references remain
grep -r "orchestrator" --exclude-dir=target .
grep -r "nestgate-orchestrator" --exclude-dir=target .
```

---

## 🔌 **New Songbird Integration Pattern**

### **How Squirrel Should Use Songbird**

```rust
// code/crates/integration/songbird/src/lib.rs
use songbird_orchestrator::prelude::*;

pub struct SquirrelMcpService {
    mcp_core: McpCore,
    plugin_manager: PluginManager,
}

impl UniversalService for SquirrelMcpService {
    async fn handle_request(&self, request: ServiceRequest) -> ServiceResponse {
        match request.request_type {
            RequestType::McpCommand => self.handle_mcp_command(request).await,
            RequestType::PluginLoad => self.handle_plugin_load(request).await,
            RequestType::AiProcessing => self.handle_ai_processing(request).await,
            _ => ServiceResponse::not_supported(),
        }
    }
    
    async fn health_check(&self) -> HealthStatus {
        // Check MCP service health
        HealthStatus::healthy()
    }
}

// Register with Songbird orchestrator
pub async fn register_with_songbird() -> Result<(), Box<dyn std::error::Error>> {
    let orchestrator_client = OrchestratorClient::connect("http://127.0.0.1:8080").await?;
    
    let mcp_service = SquirrelMcpService::new().await?;
    
    orchestrator_client.register_service(
        "squirrel-mcp",
        Box::new(mcp_service),
        ServiceConfig::default()
    ).await?;
    
    Ok(())
}
```

---

## 📋 **Updated Squirrel Responsibilities**

### **✅ What Squirrel SHOULD Do**

```yaml
mcp_platform_core:
  protocol_implementation:
    - Machine Context Protocol server
    - Context management and storage
    - Multi-agent coordination protocols
    
  plugin_system:
    - Dynamic plugin loading with sandboxing
    - Plugin discovery and management
    - Cross-platform plugin support
    
  ai_integration:
    - AI model management
    - AI processing pipeline
    - Model switching and optimization
    
  web_interface:
    - MCP dashboard and control panel
    - Real-time monitoring interface
    - Configuration management UI
    
  authentication:
    - User authentication and authorization
    - API key management
    - Session management
```

### **❌ What Squirrel Should NOT Do (Songbird's Job)**

```yaml
service_orchestration:
  - Service lifecycle management
  - Load balancing between services
  - Service registry and discovery
  - Health monitoring of external services
  - Connection proxying and routing
  - Circuit breaker patterns
  - Service mesh functionality
```

---

## 🎯 **Success Criteria**

### **Technical Goals**
- [ ] All orchestrator code removed from Squirrel
- [ ] Project compiles without orchestrator dependencies
- [ ] Tests pass without orchestrator-specific tests
- [ ] Songbird dependency added and integrated
- [ ] MCP services register with Songbird orchestrator

### **Focus Goals**
- [ ] Clear separation: Squirrel = MCP Platform, Songbird = Orchestrator
- [ ] Documentation updated to reflect new role
- [ ] Team refocused on MCP platform development
- [ ] No duplicate orchestration functionality

### **Integration Goals**
- [ ] Squirrel services work with Songbird orchestrator
- [ ] Clean integration pattern established
- [ ] Cross-project communication working
- [ ] Unified ecosystem with clear roles

---

## 📞 **Support During Tearout**

### **If You Need Help**
1. **Songbird Team**: Available for integration questions
2. **NestGate Team**: Available for ecosystem coordination
3. **Backup Branch**: `orchestrator-tearout-backup` if you need to revert

### **Key Contacts**
- **Ecosystem Integration**: Available for architectural questions
- **Build Issues**: Check with DevOps if compilation fails
- **Testing**: Skip orchestrator tests, focus on MCP functionality

---

## 🚀 **Post-Tearout: Squirrel's Bright Future**

With orchestrator code removed, Squirrel can focus on what it does best:

### **Immediate Benefits**
- **🎯 Clear Focus**: Pure MCP platform without orchestration confusion
- **🧹 Cleaner Codebase**: No duplicate/redundant orchestration code
- **⚡ Faster Development**: Team energy focused on MCP innovation
- **🔌 Better Integration**: Clean interface with production Songbird orchestrator

### **Long-term Vision**
- **🐿️ The MCP Platform**: Industry-leading Machine Context Protocol implementation
- **🔌 Universal Compatibility**: Works with any orchestrator (starting with Songbird)
- **🚀 Plugin Ecosystem**: Rich ecosystem of MCP plugins and extensions
- **🎯 Domain Expertise**: Deep specialization in AI agent coordination

---

This tearout represents a **major architectural improvement** that will benefit the entire ecosystem. The Squirrel team can now build the best MCP platform without being distracted by orchestration concerns that Songbird handles better.

**Ready to tear out and refocus! 🐿️🎯** 