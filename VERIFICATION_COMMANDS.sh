#!/bin/bash
# Verification Commands - November 17, 2025
# Run these commands to verify the codebase status

echo "==================================="
echo "🔍 SQUIRREL CODEBASE VERIFICATION"
echo "==================================="
echo ""

echo "1️⃣  Building library..."
cargo build --lib 2>&1 | tail -5
echo ""

echo "2️⃣  Running library tests..."
cargo test --workspace --lib --quiet 2>&1 | tail -3
echo ""

echo "3️⃣  Running integration tests..."
cargo test --test api_integration_tests --quiet 2>&1 | tail -3
echo ""

echo "4️⃣  Checking formatting..."
if cargo fmt --check 2>&1 | grep -q "Diff"; then
    echo "❌ Formatting issues found"
else
    echo "✅ No formatting issues"
fi
echo ""

echo "5️⃣  Quick metrics..."
echo "   - Total files: $(find . -name '*.rs' -not -path './target/*' -not -path './archive/*' | wc -l)"
echo "   - TODO count: $(rg -i 'TODO|FIXME' --type rust -g '!target/*' -g '!archive/*' -g '!../archive/*' 2>/dev/null | wc -l)"
echo "   - Unwrap count: $(rg -w 'unwrap\(\)|expect\(' --type rust -g '!target/*' -g '!archive/*' 2>/dev/null | wc -l)"
echo ""

echo "==================================="
echo "✅ VERIFICATION COMPLETE"
echo "==================================="
echo ""
echo "For detailed reports, see:"
echo "  - 00_READ_THIS_FIRST_NOV_17_2025.md"
echo "  - COMPLETE_SUCCESS_NOV_17_2025.md"
echo "  - COMPREHENSIVE_AUDIT_REPORT_NOV_17_2025.md"
