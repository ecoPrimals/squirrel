# Specs Directory Reorganization Plan
*Post-Ecosystem Elimination Cleanup*

## Overview
Following the successful ecosystem elimination that transformed Squirrel from a broad multi-agent platform into a focused Machine Context Protocol (MCP) system, we need to reorganize the specs directory to reflect this new architecture.

## Ecosystem Elimination Results
- **417+ files eliminated**, **102,543+ lines of code removed**
- **Clear separation of concerns** across ecosystem projects:
  - **Songbird**: Universal Service Orchestration (web, API, monitoring)
  - **ToadStool**: Universal Compute Platform (runtime, execution)
  - **NestGate**: Sovereign Storage System (persistence, data)
  - **BearDog**: Security & Compliance Framework (auth, security)
  - **Squirrel**: Machine Context Protocol Platform (MCP focus)

## New Specs Organization

### 🎯 **KEEP - Squirrel's Core MCP Focus**
```
specs/
├── mcp/                    # MCP Protocol specifications
├── plugins/                # Plugin interfaces and registry
├── ai-agents/             # AI agent coordination
├── commands/              # AI tools command system
├── context/               # Context management (state only)
├── python-bindings/       # PyO3 MCP bindings
├── integration/           # Cross-ecosystem integration
└── architecture/          # Core architecture docs
```

### 📦 **ARCHIVE - Moved to Other Ecosystem Projects**
```
specs/archived/ecosystem-elimination-2025/
├── web-api/               → Moved to Songbird
├── ui/                    → Moved to other projects
├── monitoring/            → Moved to Songbird  
├── storage/               → Moved to NestGate
├── security/              → Moved to BearDog
├── runtime/               → Moved to ToadStool
└── services/              → Moved to Songbird
```

## Implementation Steps

### Phase 1: Create New Structure
1. Create focused MCP directories
2. Update architecture documentation
3. Create ecosystem integration specs

### Phase 2: Archive Eliminated Components
1. Move web/API specs to archived/ecosystem-elimination-2025/
2. Move UI specs to archived/
3. Move monitoring specs to archived/
4. Move storage/persistence specs to archived/
5. Move security specs to archived/
6. Move runtime execution specs to archived/

### Phase 3: Update Cross-References
1. Update integration specs to reference ecosystem projects
2. Update architecture docs to reflect new boundaries
3. Create ecosystem collaboration guides

### Phase 4: Create New MCP-Focused Specs
1. Comprehensive MCP protocol specification
2. Plugin system architecture for MCP
3. AI agent coordination patterns
4. Cross-ecosystem communication protocols

## Key Architectural Changes Reflected

### Squirrel's New Identity
- **Before**: Broad multi-agent development platform
- **After**: Laser-focused Machine Context Protocol system

### Responsibilities
- ✅ **MCP Protocol Core**: Complete protocol implementation
- ✅ **AI Agent Coordination**: Agent-to-agent communication via MCP
- ✅ **Plugin Interfaces**: Plugin registry and lifecycle management
- ✅ **Python Bindings**: PyO3 bindings for MCP in Python
- ✅ **Commands System**: AI tool orchestration and execution
- ✅ **Context Management**: State management (not persistence)

### External Dependencies
- **Web Services**: Provided by Songbird
- **Runtime Execution**: Provided by ToadStool
- **Storage/Persistence**: Provided by NestGate
- **Security/Authentication**: Provided by BearDog
- **Service Orchestration**: Provided by Songbird

## Success Metrics
- [ ] All obsolete specs archived with clear migration notes
- [ ] New MCP-focused structure implemented
- [ ] Cross-ecosystem integration documented
- [ ] Zero redundancy with other ecosystem projects
- [ ] Clear separation of concerns maintained

## Timeline
- **Phase 1**: Immediate (create structure)
- **Phase 2**: Day 1 (archive elimination)
- **Phase 3**: Day 2 (update references)
- **Phase 4**: Week 1 (new MCP specs)

---
*This reorganization ensures the specs directory reflects Squirrel's successful transformation into a focused, specialized MCP platform within the broader ecosystem.* 