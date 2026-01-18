# Root Documentation Cleanup Plan - January 17, 2026

## 🎯 Goal
Clean root directory by archiving session documents and keeping only essential, current docs.

---

## 📊 Current State: 44 Root Documents

### ✅ KEEP (Essential Current Docs) - 12 files

#### Core Documentation
1. **README.md** - Main project readme
2. **START_HERE.md** - Quick start guide  
3. **CURRENT_STATUS.md** - Current status (just updated)
4. **CHANGELOG.md** - Version history
5. **DOCUMENTATION_INDEX.md** - Doc navigation
6. **ARCHIVE_INDEX.md** - Archive navigation

#### Architecture & Guides
7. **USAGE_GUIDE.md** - How to use Squirrel
8. **PRIMAL_INTEGRATION_GUIDE.md** - Integration patterns
9. **CAPABILITY_INTEGRATION_TEMPLATE.md** - Template for capabilities
10. **SOCKET_REGISTRY_SPEC.md** - Unix socket specification

#### Project Management
11. **PRE_PUSH_CHECKLIST.md** - Git hook checklist
12. **ROOT_DOCS_INDEX.md** - Legacy index (may update or archive)

---

### 📦 ARCHIVE (Session/Interim Docs) - 30 files

#### Group 1: v1.3.0 TRUE PRIMAL Evolution Sessions
**Target**: `archive/v1.3_true_primal_evolution/`

1. CODE_CLEANUP_ANALYSIS_JAN_17_2026.md
2. CODE_CLEANUP_AUDIT_POST_PRIMAL_JAN_17_2026.md
3. CODE_CLEANUP_SESSION_JAN_17_2026.md
4. COMPLETE_SESSION_SUMMARY_JAN_17_2026.md
5. COMPREHENSIVE_TESTING_SESSION_JAN_17_2026.md
6. DEBT_MIGRATION_PLAN_JAN_17_2026.md
7. DEEP_AUDIT_TODOS_DEAD_CODE_JAN_17_2026.md
8. DEEP_EVOLUTION_PLAN_JAN_17_2026.md (now completed)
9. DEPLOYMENT_READY_JAN_17_2026.md
10. DOCUMENTATION_CLEANUP_COMPLETE_JAN_17_2026.md
11. DOCUMENTATION_CLEANUP_JAN_17_2026.md
12. EVOLUTION_EXECUTION_SESSION_JAN_17_2026.md
13. EVOLUTION_EXECUTIVE_SUMMARY_JAN_17_2026.md
14. EVOLUTION_READY_FOR_APPROVAL_JAN_17_2026.md
15. EVOLUTION_STATUS_JAN_17_2026.md
16. FLAKY_TEST_RESOLUTION_JAN_17_2026.md
17. GIT_PUSH_CHECKLIST_JAN_17_2026.md
18. HARDCODING_FINAL_ASSESSMENT.md
19. PHASE_1.5_ZERO_HARDCODING_PLAN.md
20. PHASE1_COMPLETION_REPORT_JAN_17_2026.md
21. PUSH_NOTES_JAN_17_2026.md
22. QUALITY_ISSUES_FIXED_JAN_17_2026.md
23. REMAINING_QUALITY_WORK_JAN_17_2026.md
24. SESSION_SUMMARY_V1.2.0_UNIBIN_JAN_17_2026.md
25. SESSION_SUMMARY_ZERO_HARDCODING_JAN_17_2026.md
26. SQUIRREL_UNIBIN_COMPLIANCE_REVIEW_JAN_17_2026.md
27. TRUE_PRIMAL_EVOLUTION.md

#### Group 2: Production Status (Superseded)
**Target**: `archive/production_status_interim/`

28. BIOMEOS_READY.md (superseded by CURRENT_STATUS.md)
29. PRODUCTION_READY.md (superseded by CURRENT_STATUS.md)

#### Group 3: Project Context (Keep or Update)
**Decision Needed**:

30. PRIMALPULSE_LIVE.md - Is this still relevant?
31. PRIMALPULSE_PROJECT.md - Is this still relevant?
32. CURSOR_MCP_QUICK_TEST.md - Testing doc, archive?

---

## 📋 Execution Plan

### Step 1: Create Archive Directory
```bash
mkdir -p archive/v1.3_true_primal_evolution
mkdir -p archive/production_status_interim
```

### Step 2: Move v1.3.0 Session Docs (27 files)
All the `*_JAN_17_2026.md` files plus:
- DEEP_EVOLUTION_PLAN_JAN_17_2026.md (completed)
- HARDCODING_FINAL_ASSESSMENT.md
- PHASE_1.5_ZERO_HARDCODING_PLAN.md
- TRUE_PRIMAL_EVOLUTION.md

### Step 3: Move Superseded Status Docs (2 files)
- BIOMEOS_READY.md
- PRODUCTION_READY.md

### Step 4: Review and Decide on Project Context (3 files)
- PRIMALPULSE_LIVE.md
- PRIMALPULSE_PROJECT.md  
- CURSOR_MCP_QUICK_TEST.md

### Step 5: Create Archive README
Document what was archived and why.

### Step 6: Update Root Index
Update DOCUMENTATION_INDEX.md to reflect new structure.

---

## ✅ AFTER CLEANUP: 12-15 Root Documents

**Essential only**:
- Core docs (README, START_HERE, CURRENT_STATUS, CHANGELOG)
- Architecture guides (USAGE, INTEGRATION, SOCKET_REGISTRY)
- Navigation (DOCUMENTATION_INDEX, ARCHIVE_INDEX)
- Project management (PRE_PUSH_CHECKLIST)
- (Optional: PrimalPulse docs if still relevant)

---

## 🎯 Success Criteria

1. Root has ≤15 documents (down from 44)
2. All session docs archived with context
3. Archive has README explaining contents
4. DOCUMENTATION_INDEX updated
5. No loss of information (fossil record preserved)

---

**Status**: Ready to execute  
**Impact**: Clean root directory, easier navigation  
**Risk**: ZERO (moving to archive, not deleting)

