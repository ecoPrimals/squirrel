#!/bin/bash
# Quick Verification Script for Squirrel
# Run this to verify current status

set -e

echo "🔍 Squirrel Quick Verification"
echo "================================"
echo ""

# Build check
echo "📦 Checking build..."
if cargo build --workspace --quiet; then
    echo "✅ Build PASSED"
else
    echo "❌ Build FAILED"
    exit 1
fi

# Test check
echo ""
echo "🧪 Running tests..."
TEST_RESULT=$(cargo test --lib --workspace --quiet 2>&1 | tail -1)
if echo "$TEST_RESULT" | grep -q "test result: ok"; then
    echo "✅ Tests PASSED"
    echo "   $TEST_RESULT"
else
    echo "❌ Tests FAILED"
    echo "   $TEST_RESULT"
    exit 1
fi

# Format check
echo ""
echo "✨ Checking formatting..."
if cargo fmt --all -- --check 2>&1 | grep -q "Diff in"; then
    echo "❌ Formatting FAILED (run 'cargo fmt --all')"
else
    echo "✅ Formatting PASSED"
fi

# Coverage check
echo ""
echo "📊 Checking test coverage..."
COVERAGE=$(cargo llvm-cov --lib --workspace --summary-only 2>&1 | grep "TOTAL" | awk '{print $10}')
echo "   Current coverage: $COVERAGE"
if [[ "${COVERAGE%\%}" < "60" ]]; then
    echo "   ⚠️  Below 60% target"
else
    echo "   ✅ Above 60% threshold"
fi

# Unwrap count
echo ""
echo "🔧 Checking unwraps/expects..."
UNWRAP_COUNT=$(grep -r "\.unwrap()\|\.expect(" crates --include="*.rs" | grep -v "/test" | grep -v "/tests/" | grep -v "/benches/" | grep -v "/examples/" | grep -v "target/" | wc -l)
echo "   Production code: $UNWRAP_COUNT unwraps/expects"
if [ "$UNWRAP_COUNT" -gt 100 ]; then
    echo "   ⚠️  Target: <100"
else
    echo "   ✅ Under target"
fi

# File size check
echo ""
echo "📏 Checking file sizes..."
LARGE_FILES=$(find crates -name "*.rs" -type f -exec wc -l {} + | awk '$1 > 1000 {print}' | grep -v "target/" | wc -l)
echo "   Files >1000 lines: $LARGE_FILES"
if [ "$LARGE_FILES" -gt 0 ]; then
    echo "   ⚠️  Target: 0 files >1000 lines"
    echo "   Files:"
    find crates -name "*.rs" -type f -exec wc -l {} + | awk '$1 > 1000 {print "      " $1 " " $2}' | grep -v "target/"
else
    echo "   ✅ All files under limit"
fi

# TODO count
echo ""
echo "📝 Checking TODOs/FIXMEs..."
TODO_COUNT=$(grep -r "TODO\|FIXME\|XXX\|HACK" crates --include="*.rs" | grep -v "target/" | wc -l)
echo "   Total markers: $TODO_COUNT"
if [ "$TODO_COUNT" -gt 50 ]; then
    echo "   ⚠️  Consider resolving"
else
    echo "   ✅ Reasonable amount"
fi

# Hardcoding check
echo ""
echo "🔌 Checking hardcoded values..."
HARDCODED_HOSTS=$(grep -rn "127\.0\.0\.1\|localhost" crates --include="*.rs" | grep -v test | grep -v example | grep -v target | grep -v "^crates/universal-constants" | wc -l)
HARDCODED_PORTS=$(grep -rn "8080\|8443\|8444\|50000\|50001\|50002" crates --include="*.rs" | grep -v test | grep -v example | grep -v target | grep -v "^crates/universal-constants" | wc -l)
echo "   Hardcoded hosts/IPs: $HARDCODED_HOSTS"
echo "   Hardcoded ports: $HARDCODED_PORTS"
if [ "$HARDCODED_HOSTS" -gt 10 ] || [ "$HARDCODED_PORTS" -gt 10 ]; then
    echo "   ⚠️  Migration incomplete"
else
    echo "   ✅ Mostly migrated"
fi

# Summary
echo ""
echo "================================"
echo "📊 SUMMARY"
echo "================================"
echo "Build:         ✅"
echo "Tests:         ✅"
echo "Formatting:    Check above"
echo "Coverage:      $COVERAGE"
echo "Unwraps:       $UNWRAP_COUNT (target: <100)"
echo "Large Files:   $LARGE_FILES (target: 0)"
echo "TODOs:         $TODO_COUNT"
echo "Hardcoding:    $HARDCODED_HOSTS hosts, $HARDCODED_PORTS ports"
echo ""
echo "✅ Quick verification complete!"
echo ""
echo "For detailed audit: See COMPREHENSIVE_AUDIT_REPORT_NOV_23_2025.md"

