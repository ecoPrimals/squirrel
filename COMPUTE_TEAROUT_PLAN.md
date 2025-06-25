# 🍄 Compute Tearout & Code Cleanup Execution Plan

## Current Status: ACTIVE TEAROUT

### Phase 1: Systematic Cleanup & Tearout [CURRENT]

#### A. Fix Compilation Issues First
- [✅] Temporarily disabled problematic packages in workspace
- [⏳] Fix remaining web integration compilation issues  
- [⏳] Address type annotation problems in event bridge

#### B. Execute Tearout Plan
1. **Remove Orchestrator Infrastructure (15 min)**
   - Remove orchestrator service code
   - Remove orchestrator proto files
   - Clean up web orchestrator routing
   - Update Cargo workspace

2. **Extract Compute Infrastructure (30 min)**
   - Move sandbox code to toToadStool/ directory
   - Create toadstool integration stubs
   - Update plugin execution to use toadstool clients
   - Preserve plugin registry in squirrel

3. **Create Ecosystem Integration (20 min)**
   - Add songbird client integration
   - Implement service discovery registration
   - Create cross-project communication layer

4. **Clean Technical Debt (25 min)**
   - Fix all compiler warnings with `cargo fix`
   - Remove unused imports and variables
   - Update deprecated patterns
   - Improve error handling

#### C. Testing & Validation
- Test MCP functionality works
- Verify ecosystem integration points
- Run integration tests
- Document migration changes

### Expected Timeline: ~90 minutes

### Benefits After Completion:
- ✨ Clean, focused MCP platform 
- 🔧 Significantly reduced technical debt
- 🍄 Ready for toadstool integration
- 🎯 Clear architectural boundaries
- 📈 Improved maintainability

---

## Detailed Steps

### Step 1: Backup & Branch Management
```bash
git add . && git commit -m "state(tearout): pre-tearout backup with compilation fixes"
git push origin compute-tearout-toadstool-integration
```

### Step 2: Remove Orchestrator Code
- [ ] Remove `code/crates/services/nestgate-orchestrator/` (if exists)
- [ ] Remove orchestrator proto files
- [ ] Clean web orchestrator routes
- [ ] Update workspace Cargo.toml

### Step 3: Move Compute Infrastructure
- [ ] Examine current sandbox implementations
- [ ] Move to `toToadStool/` directory structure
- [ ] Create integration stubs in squirrel
- [ ] Update plugin execution calls

### Step 4: Fix Compilation & Warnings
- [ ] Use `cargo fix --all` to auto-fix issues
- [ ] Address remaining compilation errors
- [ ] Clean up unused imports
- [ ] Fix type annotations

### Step 5: Test & Document
- [ ] Run `cargo test` on core packages
- [ ] Verify MCP protocol functionality
- [ ] Document architectural changes
- [ ] Update integration specs

## Success Criteria
- [x] Project compiles clean (warnings OK for now)
- [ ] All MCP tests pass
- [ ] Core functionality preserved
- [ ] Clear separation of concerns
- [ ] Ready for ecosystem integration 