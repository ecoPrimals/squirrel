# ⚡ Quick Start - Maintenance

**For developers maintaining the Squirrel codebase**

---

## 🎯 Daily Checks (2 minutes)

```bash
# Before committing
cargo check
cargo test

# Verify quality
./scripts/check-file-sizes.sh
./scripts/check-tech-debt.sh
```

**Expected**: All checks ✅ GREEN

---

## 📏 Key Standards

### **File Size**
- **Max**: 2000 lines per file
- **Warning**: 1500 lines
- **Action**: Split before 1800 lines

### **Technical Debt**
- **Target**: <0.05% density
- **Current**: 0.021% ✅
- **NEVER**: Add HACK markers

### **Code Quality**
- **Build**: Must pass
- **Tests**: Must pass
- **TODOs**: Link to issues

---

## ✅ Before Merging PR

```bash
# Run all checks
./scripts/quality-check.sh

# Verify
[ ] Build passes
[ ] Tests pass  
[ ] No files >2000 lines
[ ] No HACK markers added
[ ] TODOs have context
[ ] Documentation updated
```

---

## 🚨 If Checks Fail

### **File size violation**
```bash
# Find large files
find crates -name "*.rs" -exec wc -l {} + | sort -n | tail -10

# Split file into modules
# See MAINTENANCE_GUIDE.md for details
```

### **Tech debt spike**
```bash
# Find HACK markers
grep -r "HACK\|XXX" crates --include="*.rs" | grep -v target/

# Review and fix immediately
```

### **Build failure**
```bash
# Check errors
cargo check 2>&1 | less

# Fix and test
cargo test
```

---

## 📚 Full Documentation

**See**: `MAINTENANCE_GUIDE.md` for complete details

**Key Docs**:
- `START_HERE.md` - Current status
- `docs/adr/` - Architecture decisions
- `COMPREHENSIVE_CONSOLIDATION_ASSESSMENT_NOV_10_2025.md` - Full assessment

---

## 🎯 Current Quality

```
Grade:           A++ (98/100) ✅
File Discipline: 100% ✅
Tech Debt:       0.021% ✅
HACK Markers:    0 ✅
```

**Goal**: Maintain this world-class standard! ⭐

---

**Questions?** See `MAINTENANCE_GUIDE.md` or ask the team.

🐿️ **Keep up the excellent work!** ✅

