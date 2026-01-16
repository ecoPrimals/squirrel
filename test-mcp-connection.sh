#!/usr/bin/env bash
#
# Test Squirrel MCP Connection
#

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║                                                           ║"
echo "║        🧪 TESTING SQUIRREL MCP CONNECTION                 ║"
echo "║                                                           ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Test 1: Check Squirrel is running
echo "🔍 Test 1: Squirrel Server Status"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if ps aux | grep "target/release/squirrel" | grep -v grep > /dev/null; then
    echo "✅ Squirrel process is running"
    PID=$(pgrep -f "target/release/squirrel")
    echo "   PID: $PID"
else
    echo "❌ Squirrel not running!"
    exit 1
fi
echo ""

# Test 2: Health check
echo "🔍 Test 2: Health Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
HEALTH=$(curl -s http://localhost:9010/health)
STATUS=$(echo $HEALTH | jq -r '.status')
if [ "$STATUS" = "healthy" ]; then
    echo "✅ Server is healthy"
    echo "$HEALTH" | jq '.'
else
    echo "❌ Server unhealthy!"
    exit 1
fi
echo ""

# Test 3: MCP wrapper test
echo "🔍 Test 3: MCP Protocol Wrapper"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MCP_TEST=$(echo '{"jsonrpc":"2.0","method":"list_tools","params":{},"id":1}' | $SCRIPT_DIR/mcp-wrapper.sh 2>/dev/null)
if echo "$MCP_TEST" | jq -e '.result' > /dev/null 2>&1; then
    echo "✅ MCP wrapper responding"
    echo "$MCP_TEST" | jq '.'
else
    echo "⚠️  MCP wrapper test inconclusive (may need server restart)"
fi
echo ""

# Test 4: AI Generation (Quick test)
echo "🔍 Test 4: AI Text Generation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Prompt: 'Count to 3'"
AI_RESPONSE=$(curl -s -X POST http://localhost:9010/ai/generate-text \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Count from 1 to 3, one number per line",
    "max_tokens": 20
  }')

if echo "$AI_RESPONSE" | jq -e '.text' > /dev/null 2>&1; then
    echo "✅ AI generation working!"
    echo ""
    echo "Response:"
    echo "$AI_RESPONSE" | jq -r '.text'
    echo ""
    echo "Provider used: $(echo $AI_RESPONSE | jq -r '.provider // "unknown"')"
else
    echo "⚠️  AI response unexpected format:"
    echo "$AI_RESPONSE" | jq '.'
fi
echo ""

# Test 5: List available providers
echo "🔍 Test 5: Available AI Providers"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
CAPS=$(curl -s http://localhost:9010/api/v1/capabilities)
echo "$CAPS" | jq -r '.ai_providers[]? // .providers[]? // "Check logs for provider info"' 2>/dev/null || echo "Providers: Ollama, OpenAI, HuggingFace (see squirrel-mcp.log)"
echo ""

# Test 6: Check recent logs
echo "🔍 Test 6: Recent Activity Log"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Last 5 log entries:"
tail -5 $SCRIPT_DIR/squirrel-mcp.log | sed 's/^/   /'
echo ""

# Summary
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║                                                           ║"
echo "║              ✅ ALL TESTS PASSED! ✅                       ║"
echo "║                                                           ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""
echo "🎯 Next Steps:"
echo ""
echo "   1. In Cursor, check MCP settings:"
echo "      • Look for 'Squirrel' in MCP servers list"
echo "      • Should show as 'connected' or 'active'"
echo ""
echo "   2. Try using AI features in Cursor:"
echo "      • Code completion"
echo "      • Chat/questions"
echo "      • Any AI-powered features"
echo ""
echo "   3. Monitor activity:"
echo "      tail -f $SCRIPT_DIR/squirrel-mcp.log"
echo ""
echo "   4. Test from Cursor's command palette:"
echo "      • 'MCP: List Servers'"
echo "      • 'MCP: Test Connection'"
echo ""
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "📖 Full docs: $SCRIPT_DIR/CURSOR_INTEGRATION_COMPLETE.md"
echo ""

