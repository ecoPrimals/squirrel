---
title: Compute Infrastructure Tearout & Toadstool Integration Plan
description: Updated tearout plan to move compute infrastructure to Toadstool-Compute project
version: 2.0.0
date: 2025-01-26
author: Ecosystem Integration Team
priority: CRITICAL
status: TEAROUT_AND_INTEGRATION_ACTIVE
---

# 🍄 Compute Tearout & Toadstool Integration Plan

## Executive Summary

With the introduction of **Toadstool-Compute** as the dedicated compute platform, Squirrel needs to:

1. **🗑️ Remove orchestrator code** (pre-pre-songbird prototype)
2. **🍄 Move compute infrastructure** to Toadstool-Compute
3. **🎯 Refocus on pure MCP platform** responsibilities
4. **🔌 Integrate with Songbird + Toadstool** ecosystem

---

## 🏗️ **New Ecosystem Architecture**

### **Four-Project Ecosystem**
```yaml
ecosystem_roles:
  squirrel_mcp:
    role: "Machine Context Protocol Platform"
    responsibilities: ["MCP protocol", "AI agents", "plugin registry", "context management"]
    
  toadstool_compute:
    role: "Compute & Environment Management"
    responsibilities: ["execution environments", "sandboxing", "resource management", "cross-platform runtime"]
    
  songbird_orchestrator:
    role: "Discovery & Orchestration Hub"
    responsibilities: ["service discovery", "capability broadcasting", "request routing", "load balancing"]
    
  nestgate_storage:
    role: "Storage Domain Services"
    responsibilities: ["ZFS management", "storage protocols", "tier management", "backup operations"]
```

### **Communication Flow**
```
🐿️ Squirrel MCP ←→ 🎼 Songbird ←→ 🍄 Toadstool Compute
                   ↕
               🏠 NestGate Storage
```

---

## 🔄 **What Moves Where**

### **🚚 FROM Squirrel TO Toadstool-Compute**

#### **Execution Infrastructure**
```yaml
move_to_toadstool:
  sandboxing_system:
    - code/crates/services/app/src/plugin/sandbox/
    - code/crates/sdk/src/sandbox.rs
    - Cross-platform sandbox implementations
    
  execution_environments:
    - WASM runtime infrastructure
    - Container execution support
    - Native plugin execution
    - Resource monitoring and limits
    
  platform_specific:
    - Windows Job Objects implementation
    - macOS App Sandbox implementation  
    - Linux namespaces/seccomp implementation
    - Cross-platform capability detection
```

#### **Plugin Execution (NOT Plugin Registry)**
```yaml
move_execution_only:
  what_moves:
    - Plugin execution environments
    - Security sandboxing
    - Resource allocation
    - Performance monitoring
    
  what_stays_in_squirrel:
    - Plugin registry and metadata
    - Plugin discovery and management
    - MCP-specific plugin interfaces
    - AI agent plugin coordination
```

### **🏠 Keep IN Squirrel**

#### **Pure MCP Platform**
```yaml
keep_in_squirrel:
  mcp_core:
    - Machine Context Protocol implementation
    - AI agent coordination and workflows
    - Context management and storage
    - Multi-agent communication
    
  plugin_platform:
    - Plugin registry and metadata management
    - Plugin discovery and lifecycle
    - MCP plugin interfaces
    - AI-enhanced plugin recommendations
    
  ai_integration:
    - AI model management and switching
    - AI processing pipeline coordination
    - Agent behavior and learning
    - Context-aware AI operations
```

---

## 🛠️ **Updated Tearout Process**

### **Phase 1: Backup & Branch (5 minutes)**
```bash
# Create backup branch
git checkout -b compute-tearout-backup
git commit -a -m "backup: pre-compute tearout state"

# Create working branch  
git checkout -b compute-tearout-toadstool-integration
```

### **Phase 2: Remove Orchestrator Code (15 minutes)**
```bash
# 1. Remove orchestrator service
rm -rf code/crates/services/nestgate-orchestrator/

# 2. Remove orchestrator proto
rm code/proto/orchestrator.proto

# 3. Remove web orchestrator routing
rm -rf code/crates/integration/web/src/orchestrator/
rm code/crates/integration/web/src/tests/orchestrator_routing_tests.rs

# 4. Update Cargo.toml workspace members
# Remove "code/crates/services/nestgate-orchestrator"
```

### **Phase 3: Extract Compute Infrastructure (30 minutes)**
```bash
# 1. Create toadstool integration in squirrel
mkdir -p code/crates/integration/toadstool/

# 2. Move sandbox code to toadstool project
# (Coordinate with toadstool team for this step)

# 3. Replace execution with toadstool client calls
# Update plugin execution to use toadstool via songbird
```

### **Phase 4: Add Ecosystem Integration (20 minutes)**
```bash
# 1. Add ecosystem dependencies
# Add to root Cargo.toml:
# songbird-client = { path = "../songbird/client" }
# toadstool-client = { path = "../toadstool/client" }

# 2. Create ecosystem integration module
mkdir -p code/crates/integration/ecosystem/

# 3. Implement songbird registration
# Register MCP services with songbird discovery
```

### **Phase 5: Update Plugin System (25 minutes)**
```bash
# 1. Update plugin execution to use toadstool
# Modify plugin manager to request execution via songbird→toadstool

# 2. Keep plugin registry in squirrel
# Plugin metadata, discovery, lifecycle stays

# 3. Add MCP-specific plugin interfaces
# Enhance plugin system for MCP protocol integration
```

### **Phase 6: Test & Verify (15 minutes)**
```bash
# 1. Ensure project compiles
cargo check

# 2. Test MCP functionality
cargo test --package squirrel-mcp

# 3. Verify ecosystem integration
# Test songbird registration and toadstool execution requests
```

---

## 🔌 **New Integration Patterns**

### **Squirrel → Songbird → Toadstool Execution**
```rust
// code/crates/integration/ecosystem/src/lib.rs
use songbird_client::SongbirdClient;
use toadstool_client::ExecutionRequest;

pub struct EcosystemIntegration {
    songbird_client: SongbirdClient,
}

impl EcosystemIntegration {
    pub async fn new() -> Result<Self> {
        let songbird_client = SongbirdClient::connect("http://localhost:8080").await?;
        
        // Register MCP services with Songbird
        songbird_client.register_service(ServiceRegistration {
            service_id: "squirrel-mcp".to_string(),
            service_type: "mcp-platform".to_string(),
            capabilities: vec![
                "mcp-protocol".to_string(),
                "ai-agents".to_string(),
                "plugin-registry".to_string(),
                "context-management".to_string(),
            ],
            endpoints: vec!["http://localhost:8081".to_string()],
            metadata: HashMap::new(),
        }).await?;
        
        Ok(Self { songbird_client })
    }
    
    pub async fn execute_plugin(&self, plugin_id: &str, context: McpContext) -> Result<PluginResult> {
        // Get plugin metadata (stays in Squirrel)
        let plugin_metadata = self.get_plugin_metadata(plugin_id)?;
        
        // Request execution via Songbird → Toadstool
        let execution_request = ExecutionRequest {
            plugin_metadata,
            execution_environment: ExecutionEnvironment::Wasm,
            resource_requirements: ResourceRequirements::default(),
            mcp_context: context,
        };
        
        // Route through Songbird to optimal Toadstool instance
        let result = self.songbird_client
            .route_compute_request("toadstool-compute", execution_request).await?;
        
        Ok(result)
    }
}
```

### **Plugin Registry (Stays in Squirrel)**
```rust
// code/crates/core/mcp/src/plugin/registry.rs
pub struct McpPluginRegistry {
    plugins: HashMap<String, PluginMetadata>,
    mcp_interfaces: HashMap<String, McpPluginInterface>,
    ai_recommendations: AiPluginRecommendations,
}

impl McpPluginRegistry {
    pub async fn register_mcp_plugin(&mut self, plugin: McpPluginMetadata) -> Result<()> {
        // Register plugin with MCP-specific metadata
        self.plugins.insert(plugin.id.clone(), plugin.into());
        
        // Register MCP interfaces
        if let Some(mcp_interface) = plugin.mcp_interface {
            self.mcp_interfaces.insert(plugin.id.clone(), mcp_interface);
        }
        
        // Update AI recommendations
        self.ai_recommendations.update_for_plugin(&plugin).await?;
        
        Ok(())
    }
    
    pub async fn execute_plugin(&self, plugin_id: &str, context: McpContext) -> Result<PluginResult> {
        // Plugin execution is delegated to Toadstool via Songbird
        let ecosystem = EcosystemIntegration::new().await?;
        ecosystem.execute_plugin(plugin_id, context).await
    }
}
```

---

## 📋 **Updated Squirrel Responsibilities**

### **✅ What Squirrel SHOULD Do**
```yaml
mcp_platform_excellence:
  protocol_implementation:
    - Machine Context Protocol server
    - Context management and storage
    - Multi-agent coordination protocols
    - Agent communication and workflows
    
  plugin_platform:
    - Plugin registry and metadata management
    - Plugin discovery and lifecycle management
    - MCP-specific plugin interfaces
    - AI-enhanced plugin recommendations
    
  ai_integration:
    - AI model management and switching
    - AI agent behavior and learning
    - Context-aware AI operations
    - Multi-agent workflow coordination
    
  ecosystem_integration:
    - Register with Songbird discovery
    - Request compute via Songbird routing
    - Coordinate with Toadstool for execution
    - Maintain MCP context across ecosystem
```

### **❌ What Squirrel Should NOT Do (Moved to Toadstool)**
```yaml
compute_infrastructure:
  execution_environments: "🍄 Toadstool handles this"
  security_sandboxing: "🍄 Toadstool handles this"
  resource_management: "🍄 Toadstool handles this"
  cross_platform_runtime: "🍄 Toadstool handles this"
  
orchestration:
  service_discovery: "🎼 Songbird handles this"
  request_routing: "🎼 Songbird handles this"
  load_balancing: "🎼 Songbird handles this"
  capability_broadcasting: "🎼 Songbird handles this"
```

---

## 🎯 **Success Criteria**

### **Technical Goals**
- [ ] All orchestrator code removed from Squirrel
- [ ] Compute infrastructure moved to Toadstool-Compute
- [ ] Plugin registry remains in Squirrel with MCP integration
- [ ] Ecosystem integration working (Squirrel ↔ Songbird ↔ Toadstool)
- [ ] MCP platform functionality enhanced and focused

### **Architectural Goals**
- [ ] Clear separation: MCP platform vs Compute platform
- [ ] Songbird-centric communication (no direct project-to-project)
- [ ] Plugin execution delegated to Toadstool via Songbird
- [ ] MCP context maintained across ecosystem
- [ ] Performance maintained or improved

### **Team Goals**
- [ ] Squirrel team focused on MCP excellence
- [ ] Toadstool team owns compute infrastructure
- [ ] Clean handoff of compute responsibilities
- [ ] Ecosystem integration patterns established

---

## 🚀 **Post-Tearout: Squirrel's New Focus**

### **Pure MCP Platform Excellence**
With compute concerns moved to Toadstool, Squirrel can focus entirely on:

- **🧠 Advanced MCP Features**: Protocol innovations and enhancements
- **🤖 AI Agent Sophistication**: Complex multi-agent workflows
- **🔌 Plugin Ecosystem**: Rich MCP plugin platform
- **📊 Context Intelligence**: Smart context management and optimization

### **Ecosystem Leadership**
Squirrel becomes the **MCP platform leader** in the ecosystem:
- **Central MCP Hub**: All MCP functionality flows through Squirrel
- **AI Agent Coordinator**: Manages complex AI workflows
- **Plugin Platform**: Rich ecosystem of MCP-aware plugins
- **Context Manager**: Intelligent context sharing across ecosystem

---

**This tearout transforms Squirrel into a focused, powerful MCP platform while enabling Toadstool to excel at compute infrastructure! 🚀**
