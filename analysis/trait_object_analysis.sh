#!/bin/bash
# Analyze trait object usage to determine what MUST keep async_trait

echo "=== TRAIT OBJECT USAGE ANALYSIS ==="
echo ""
echo "Traits that MUST keep async_trait (used as trait objects):"
echo ""

# Search for trait object patterns
echo "## Transport trait:"
grep -r "dyn.*Transport\|Box.*Transport\|Arc.*Transport" crates/core/mcp --include="*.rs" 2>/dev/null | wc -l

echo "## Plugin trait:"
grep -r "dyn.*Plugin\|Box.*Plugin\|Arc.*Plugin" crates/core/plugins --include="*.rs" 2>/dev/null | wc -l

echo "## Provider traits:"
grep -r "dyn.*Provider\|Box.*Provider\|Arc.*Provider" crates/tools/ai-tools crates/main --include="*.rs" 2>/dev/null | wc -l

echo "## Database traits:"
grep -r "dyn.*Database\|Box.*Database\|Arc.*Database" crates/integration/web --include="*.rs" 2>/dev/null | wc -l

echo ""
echo "Total trait object usage found"
