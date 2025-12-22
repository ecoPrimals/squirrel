# 🚀 Quick Reference Guide
## Common Commands and Workflows

**Last Updated**: December 21, 2025

---

## ⚡ Essential Commands

### Build & Test
```bash
# Quick verification
cargo build && cargo test --lib

# Full quality check
./QUICK_VERIFICATION.sh

# Comprehensive verification
./VERIFY_QUALITY.sh
```

### Testing
```bash
# All library tests
cargo test --lib

# Specific package
cargo test -p universal-patterns --lib

# Specific test
cargo test --lib test_name

# With output
cargo test --lib -- --nocapture

# Integration tests
cargo test --test '*'
```

### Coverage
```bash
# Generate HTML coverage report
cargo llvm-cov --lib --html

# Open report
open target/llvm-cov/html/index.html  # macOS
xdg-open target/llvm-cov/html/index.html  # Linux

# Text summary
cargo llvm-cov --lib
```

### Code Quality
```bash
# Format code
cargo fmt --all

# Check format without changing
cargo fmt --all --check

# Lint with clippy
cargo clippy --all --lib

# Pedantic linting
cargo clippy --all --lib -- -D warnings
```

### Documentation
```bash
# Generate and open docs
cargo doc --open

# Build docs without opening
cargo doc --no-deps

# Check doc links
cargo doc --all --no-deps
```

---

## 📊 Quality Checks

### Pre-Commit Checklist
```bash
# 1. Format
cargo fmt --all

# 2. Build
cargo build

# 3. Test
cargo test --lib

# 4. Lint
cargo clippy --all --lib

# All green? Good to commit!
```

### Full Quality Gate
```bash
# Run complete verification
./VERIFY_QUALITY.sh

# Expected output:
# ✅ Compilation: PASS
# ✅ Tests: PASS (647/647)
# ✅ Formatting: PASS
# ✅ Linting: PASS (or warnings listed)
```

---

## 🔍 Finding Things

### Search Codebase
```bash
# Find by name (ripgrep)
rg "function_name" --type rust

# Find in specific directory
rg "MyStruct" crates/universal-patterns/

# Case insensitive
rg -i "keyword"

# Find TODOs
rg "TODO|FIXME" crates/
```

### Find Files
```bash
# By name
fd "config.rs"

# By pattern
fd "test.*\.rs"

# In specific directory
fd "mod.rs" crates/core/
```

### Project Stats
```bash
# Line counts
tokei

# Test counts
cargo test --lib -- --list | wc -l

# File counts
find crates -name "*.rs" | wc -l
```

---

## 🧪 Testing Workflows

### Writing Tests
```bash
# 1. Add test to module
# 2. Run to verify
cargo test --lib test_name

# 3. Check coverage
cargo llvm-cov --lib --html
```

### Debugging Tests
```bash
# Run with output
cargo test --lib test_name -- --nocapture

# Run single test
cargo test --lib --exact test_module::test_name

# Show all tests
cargo test --lib -- --list
```

### Test Coverage Campaign
```bash
# 1. Check current coverage
cargo llvm-cov --lib

# 2. Find untested modules
rg "^pub " crates/ -A5 | grep -v "#\[test\]"

# 3. Add tests

# 4. Verify improvement
cargo llvm-cov --lib
```

---

## 📚 Documentation

### Key Files
- **[README.md](README.md)** - Project overview
- **[START_HERE.md](START_HERE.md)** - Getting started
- **[STATUS.md](STATUS.md)** - Current status
- **[QUICK_REFERENCE.md](QUICK_REFERENCE.md)** - This file

### Reports (Dec 21, 2025)
- **[COMPREHENSIVE_REVIEW_DEC_21_2025.md](COMPREHENSIVE_REVIEW_DEC_21_2025.md)** - Full audit
- **[FINAL_SESSION_REPORT_DEC_21_2025.md](FINAL_SESSION_REPORT_DEC_21_2025.md)** - Session summary
- **[TEST_COVERAGE_PROGRESS_DEC_21_2025.md](TEST_COVERAGE_PROGRESS_DEC_21_2025.md)** - Coverage tracking

### Standards
- **[FILE_SIZE_POLICY.md](FILE_SIZE_POLICY.md)** - Code size guidelines
- **[SOVEREIGNTY_COMPLIANCE.md](SOVEREIGNTY_COMPLIANCE.md)** - Privacy compliance
- **[MAINTENANCE_GUIDE.md](MAINTENANCE_GUIDE.md)** - Maintenance procedures

---

## 🛠️ Development Workflows

### New Feature
```bash
# 1. Create branch
git checkout -b feature/my-feature

# 2. Implement feature
# ... edit code ...

# 3. Add tests
# ... write tests ...

# 4. Verify
cargo test --lib
cargo clippy --all --lib

# 5. Document
# ... add docs ...

# 6. Commit
git add .
git commit -m "feat: add my feature"
```

### Bug Fix
```bash
# 1. Reproduce bug with test
# ... write failing test ...

# 2. Fix bug
# ... edit code ...

# 3. Verify test passes
cargo test --lib test_name

# 4. Run all tests
cargo test --lib

# 5. Commit
git commit -m "fix: resolve bug"
```

### Refactoring
```bash
# 1. Ensure tests pass first
cargo test --lib

# 2. Refactor
# ... edit code ...

# 3. Verify tests still pass
cargo test --lib

# 4. Check no behavior change
cargo test --lib -- --nocapture

# 5. Commit
git commit -m "refactor: improve code organization"
```

---

## 🎯 Common Tasks

### Check Project Status
```bash
# Quick status
cat STATUS.md

# Detailed report
cat FINAL_SESSION_REPORT_DEC_21_2025.md

# Test coverage
cargo llvm-cov --lib
```

### Update Dependencies
```bash
# Check for updates
cargo outdated

# Update Cargo.lock
cargo update

# Test after update
cargo test --lib
```

### Clean Build
```bash
# Clean all build artifacts
cargo clean

# Rebuild from scratch
cargo build --release

# Verify tests
cargo test --lib
```

---

## 📈 Metrics & Analysis

### Coverage Analysis
```bash
# Full HTML report
cargo llvm-cov --lib --html
open target/llvm-cov/html/index.html

# By package
cargo llvm-cov --lib --package universal-patterns

# JSON output
cargo llvm-cov --lib --json
```

### Performance Benchmarks
```bash
# Run all benchmarks
cargo bench

# Specific benchmark
cargo bench --bench arc_str_performance_suite

# With output
cargo bench -- --nocapture
```

### Code Analysis
```bash
# Dead code
cargo build 2>&1 | rg "never used"

# Unused dependencies
cargo-udeps

# Security audit
cargo audit
```

---

## 🔧 Troubleshooting

### Build Issues
```bash
# Clean and rebuild
cargo clean && cargo build

# Update toolchain
rustup update stable

# Check Rust version
rustc --version
```

### Test Issues
```bash
# Run single test with output
cargo test --lib test_name -- --nocapture

# Check test isolation
cargo test --lib -- --test-threads=1

# Debug test
RUST_LOG=debug cargo test --lib test_name
```

### Coverage Issues
```bash
# Reinstall llvm-cov
cargo install cargo-llvm-cov --force

# Clean coverage data
rm -rf target/llvm-cov

# Regenerate
cargo llvm-cov --lib --html
```

---

## 🚀 Deployment

### Release Build
```bash
# Build release
cargo build --release

# Run release tests
cargo test --release

# Check binary size
ls -lh target/release/
```

### Docker
```bash
# Build image
docker build -t squirrel:latest .

# Run container
docker run -it squirrel:latest

# Test in container
docker-compose -f docker-compose.test.yml up
```

---

## 📊 Current Status (Dec 21, 2025)

### Quality Metrics ✅
- **Grade**: B++ (89/100)
- **Tests**: 647/647 passing (100%)
- **Stability**: 100% (no flaky tests)
- **Coverage**: ~37% (target: 90%)
- **Build**: Zero errors
- **Linting**: Clippy passing (pedantic on some crates)

### Active Work 🔄
- Test coverage campaign (37% → 90%)
- Adding ~50+ tests per session
- Systematic module improvement

---

## 🎓 Learning Resources

### Rust Docs
```bash
# Open Rust book
rustup doc --book

# Standard library docs
rustup doc --std

# Cargo book
rustup doc --cargo
```

### Project Docs
```bash
# Architecture
cat docs/architecture/README.md

# Guides
ls docs/guides/

# Specifications
ls specs/active/
```

---

## 💡 Pro Tips

### Speed Up Development
```bash
# Use sccache for faster builds
cargo install sccache
export RUSTC_WRAPPER=sccache

# Parallel tests (default)
cargo test --lib

# Watch mode for tests
cargo watch -x "test --lib"
```

### Better Testing
```bash
# Test with nextest (faster test runner)
cargo install cargo-nextest
cargo nextest run --lib

# Coverage with exclusions
cargo llvm-cov --lib --ignore-filename-regex tests/
```

### Documentation
```bash
# Generate docs with private items
cargo doc --document-private-items

# Check doc tests only
cargo test --doc
```

---

## 🆘 Quick Help

### Stuck?
1. Check [STATUS.md](STATUS.md) for current state
2. Review [COMPREHENSIVE_REVIEW_DEC_21_2025.md](COMPREHENSIVE_REVIEW_DEC_21_2025.md)
3. Search docs: `rg "your question" docs/`
4. Check examples: `ls examples/`

### Need More Info?
- Architecture: `docs/architecture/`
- Guides: `docs/guides/`
- API Reference: `cargo doc --open`
- Specifications: `specs/active/`

---

**Last Updated**: December 21, 2025  
**Status**: Production-Capable ✅  
**Grade**: B++ (89/100)

🐿️ **Quick Reference for Fast Development** 🦀
