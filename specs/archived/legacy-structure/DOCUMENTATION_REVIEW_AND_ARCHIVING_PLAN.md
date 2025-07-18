---
title: Documentation Review and Archiving Plan - Post Code Reorganization
description: Comprehensive plan for updating and archiving specs and docs after code reorganization
version: 1.0.0
date: 2025-07-16
status: active
priority: high
---

# 📋 Documentation Review and Archiving Plan

## Executive Summary

**Context**: Recently completed comprehensive code reorganization  
**Achievement**: All code consolidated under `code/crates/` with clean workspace structure  
**Need**: Update documentation to reflect new structure and archive outdated content  
**Status**: Documentation audit and reorganization required

## 🎯 Current Assessment

### ✅ **Well-Organized Areas**
- **Current Status Reports**: Up-to-date technical debt and production readiness tracking
- **Archive System**: Good dated archiving pattern already established
- **Active Specs**: MCP protocol specs appear current and relevant

### ⚠️ **Areas Needing Review**
- **Root Level Files**: Many loose files need archiving or updating
- **Integration Specs**: May need updates after code reorganization
- **Project Structure References**: Need to reflect new `code/crates/` structure
- **Documentation Overlap**: Some redundant content across specs/ and docs/

## 📊 Detailed Review Plan

### **Phase 1: Root Level Cleanup (Immediate)**

#### Files to Archive (2025-07-16-post-code-reorganization/)
- `specs/COMPREHENSIVE_CODEBASE_REVIEW_REPORT.md` → Archive (completed milestone)
- `specs/DOCUMENTATION_HOUSEKEEPING_SUMMARY.md` → Archive (completed)
- `specs/ECOSYSTEM_API_STANDARDIZATION_GUIDE.md` → Archive (outdated)
- `specs/SPECS.md` → Archive (outdated)
- `specs/README.md` → Update with current structure

#### Files to Update
- `specs/current/COMPREHENSIVE_STATUS_REPORT.md` → Update with code reorganization completion
- `specs/current/CURRENT_TECHNICAL_DEBT_TRACKER.md` → Update post-reorganization status
- `specs/current/NEXT_STEPS_ROADMAP.md` → Update development priorities
- `specs/current/PRODUCTION_READINESS_TRACKER.md` → Update readiness metrics

### **Phase 2: Active Specs Review**

#### MCP Protocol Specs (`specs/active/mcp-protocol/`)
- ✅ **Keep Active**: Core protocol specifications
- ⚠️ **Review**: Integration examples for new code structure
- ⚠️ **Update**: Path references to use `code/crates/` structure

#### Context Specs (`specs/active/context/`)
- ✅ **Keep Active**: Context management specifications
- ⚠️ **Review**: Implementation details for new module structure

#### Plugin Specs (`specs/active/plugins/`)
- ✅ **Keep Active**: Plugin system specifications
- ⚠️ **Update**: Plugin structure references for new `code/crates/plugins/` location

### **Phase 3: Integration Specs Review**

#### Areas for Review
- **API Client Specs**: Update for new integration structure
- **Context Adapter Specs**: Verify alignment with current implementation
- **Web Integration**: Review relevance and update
- **Tools Integration**: Update for new `code/crates/tools/` structure

### **Phase 4: Docs Directory Review**

#### Documentation Structure
```
docs/
├── api/                          ← Review and update
├── API_DOCUMENTATION.md          ← Major update needed
├── CONFIGURATION.md              ← Update for new config location
├── COORDINATOR_MIGRATION_GUIDE.md ← Archive (completed)
├── ecosystem-integration/        ← Review relevance
├── elimination-reports/          ← Archive (completed)
└── historical/                   ← Keep as-is
```

#### Action Items
- **API Documentation**: Update for current API structure
- **Configuration**: Update paths to reflect `code/crates/config/`
- **Migration Guides**: Archive completed migrations
- **Ecosystem Integration**: Review and update or archive

## 🗂️ Archiving Strategy

### **Create Archive Directory**
```
specs/archived/2025-07-16-post-code-reorganization/
├── completed-milestones/
│   ├── COMPREHENSIVE_CODEBASE_REVIEW_REPORT.md
│   ├── DOCUMENTATION_HOUSEKEEPING_SUMMARY.md
│   └── ECOSYSTEM_API_STANDARDIZATION_GUIDE.md
├── outdated-specs/
│   ├── old-integration-specs/
│   └── deprecated-implementations/
└── superseded-documentation/
    ├── old-project-structure/
    └── outdated-readmes/
```

### **Archive Criteria**
- **Completed Milestones**: Work that's been finished
- **Outdated References**: Documentation referring to old structure
- **Superseded Content**: Replaced by newer documentation
- **Historical Value**: Keep for reference but not current

## 📝 Update Priorities

### **High Priority (This Week)**
1. **Update Current Status Reports** - Reflect code reorganization completion
2. **Update Project Structure References** - Use `code/crates/` paths
3. **Archive Completed Milestones** - Clean up root directory
4. **Update README files** - Reflect current state

### **Medium Priority (Next Week)**
1. **Review Integration Specs** - Verify alignment with current code
2. **Update API Documentation** - Reflect current API structure
3. **Review Plugin Documentation** - Update for new plugin structure
4. **Consolidate Overlapping Content** - Remove redundancy

### **Low Priority (Future)**
1. **Historical Documentation Review** - Organize historical content
2. **Ecosystem Integration Review** - Determine relevance
3. **Advanced Documentation Features** - Improve navigation and search

## 🎯 Success Metrics

- **✅ Clean Root Directory**: Only current and active files in specs/
- **✅ Updated References**: All documentation reflects `code/crates/` structure
- **✅ Clear Organization**: Logical separation of current vs archived content
- **✅ Accurate Status**: Documentation matches actual project state
- **✅ Reduced Redundancy**: Consolidated overlapping content

## 🚀 Implementation Timeline

### **Week 1**: Root cleanup and current status updates
### **Week 2**: Active specs review and integration documentation
### **Week 3**: Docs directory reorganization
### **Week 4**: Final review and consolidation

## 📋 Acceptance Criteria

- [ ] All outdated root-level files archived
- [ ] Current status reports updated with code reorganization
- [ ] All path references updated to use `code/crates/`
- [ ] Active specs reviewed and updated
- [ ] Documentation redundancy eliminated
- [ ] Clear separation of current vs historical content
- [ ] Updated README files reflect current structure
- [ ] API documentation matches current implementation

This plan ensures the documentation ecosystem accurately reflects the current codebase structure while preserving historical context and maintaining clarity for future development. 