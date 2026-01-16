# 🎯 Cursor MCP Quick Test Guide

**Status**: ✅ All backend tests passing  
**Date**: January 15, 2026

## ✅ Backend Tests Complete

All systems verified and operational:

```
✅ Squirrel process running (PID: 1164453)
✅ Server healthy (uptime: 258s)
✅ MCP wrapper responding
✅ AI generation working (tested: counted 1,2,3)
✅ Providers active: OpenAI, HuggingFace
```

## 🧪 How to Test in Cursor IDE

### Method 1: Check MCP Server Connection

1. **Open Cursor Settings**
   - Press `Ctrl+,` (or `Cmd+,` on Mac)
   - Search for "MCP" or "Model Context Protocol"

2. **Verify Squirrel is Listed**
   - Look for "squirrel" in the MCP servers list
   - Status should show: "Connected" or "Active"
   - If not visible, check the troubleshooting section below

3. **Check Server Logs**
   ```bash
   tail -f ~/Development/ecoPrimals/phase1/squirrel/squirrel-mcp.log
   ```
   - You should see connection attempts from Cursor

### Method 2: Use Command Palette

1. **Open Command Palette**
   - Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on Mac)

2. **Search for MCP Commands**
   - Type "MCP"
   - Look for commands like:
     - `MCP: List Servers`
     - `MCP: Show Server Status`
     - `MCP: Test Connection`

3. **Run MCP Test**
   - Select `MCP: Test Connection`
   - Choose "squirrel" from the list
   - Should show success message

### Method 3: Test AI Features

1. **Open a Code File**
   - Any `.js`, `.py`, `.rs`, etc.

2. **Try Code Completion**
   - Start typing a function
   - If Squirrel is active, you'll see AI-powered completions

3. **Try Chat/Questions**
   - Open Cursor's AI chat panel
   - Ask: "What AI provider are you using?"
   - If routed through Squirrel, check logs to see which provider was used

4. **Monitor the Routing**
   ```bash
   tail -f squirrel-mcp.log | grep -i "ai\|provider\|generate"
   ```
   - Watch requests come in and see which provider handles them

### Method 4: Direct MCP Protocol Test

If you want to test the raw MCP connection:

```bash
# Test list_tools method
echo '{"jsonrpc":"2.0","method":"list_tools","params":{},"id":1}' | \
  ~/Development/ecoPrimals/phase1/squirrel/mcp-wrapper.sh

# Test generate_text method
echo '{"jsonrpc":"2.0","method":"generate_text","params":{"prompt":"Hello Squirrel!","max_tokens":20},"id":2}' | \
  ~/Development/ecoPrimals/phase1/squirrel/mcp-wrapper.sh
```

## 🎭 What to Expect

### AI Request Flow

When you use AI in Cursor:

```
Your Request in Cursor
    ↓
Cursor detects MCP server "squirrel"
    ↓
Sends JSON-RPC request to mcp-wrapper.sh
    ↓
Wrapper converts to HTTP and calls http://localhost:9010
    ↓
Squirrel's AI Router decides:
    • Simple/Privacy → Ollama (local)
    • Complex/Quality → OpenAI GPT-4
    • Images → DALL-E or HuggingFace
    ↓
Response streams back through chain
    ↓
Displayed in Cursor
```

### Log Output Example

When working, you'll see logs like:

```
2026-01-15T16:53:22Z INFO AI request received
2026-01-15T16:53:22Z DEBUG Routing to provider: ollama
2026-01-15T16:53:22Z INFO Ollama generated response (45 tokens)
2026-01-15T16:53:22Z INFO Request completed in 123ms
```

## 🔍 Verification Checklist

- [ ] Cursor restarted after MCP config created
- [ ] Squirrel server running (`ps aux | grep squirrel`)
- [ ] Health check passes (`curl http://localhost:9010/health`)
- [ ] MCP wrapper responds (run test script)
- [ ] "squirrel" appears in Cursor MCP settings
- [ ] AI features work in Cursor
- [ ] Logs show activity when using AI

## 🐛 Troubleshooting

### Cursor Doesn't See Squirrel

**Problem**: Squirrel not in MCP server list

**Solutions**:
1. Verify config exists: `cat ~/.cursor/mcp.json`
2. Restart Cursor completely (close all windows)
3. Check Cursor logs for MCP errors
4. Ensure wrapper script is executable: `chmod +x ~/Development/ecoPrimals/phase1/squirrel/mcp-wrapper.sh`

### Connection Errors

**Problem**: MCP connection fails

**Solutions**:
1. Check Squirrel is running: `./test-mcp-connection.sh`
2. Test wrapper manually: `echo '{"jsonrpc":"2.0","method":"list_tools","params":{},"id":1}' | ./mcp-wrapper.sh`
3. Check logs: `tail -50 squirrel-mcp.log`
4. Restart Squirrel: `kill $(cat squirrel-mcp.pid) && ./start-mcp-server.sh`

### No AI Response

**Problem**: Squirrel connected but AI not responding

**Solutions**:
1. Test direct API: `curl -X POST http://localhost:9010/ai/generate-text -H "Content-Type: application/json" -d '{"prompt":"test","max_tokens":10}'`
2. Check API keys: `cat mcp-config.env | grep API_KEY`
3. Verify Ollama: `ollama list`
4. Check provider logs in `squirrel-mcp.log`

### Slow Responses

**Problem**: AI takes too long to respond

**Solutions**:
1. Check which provider is being used (logs)
2. Local Ollama should be fast (< 1s for simple)
3. OpenAI may take longer (2-5s)
4. Consider preferring local: Set `AI_ROUTER_PREFER_LOCAL=true`

## 📊 Performance Expectations

| Provider | Latency | Quality | Privacy | Cost |
|----------|---------|---------|---------|------|
| **Ollama (local)** | 0.1-1s | Good | 100% | Free |
| **OpenAI GPT-4** | 2-5s | Best | Cloud | $$$ |
| **HuggingFace** | 3-8s | Medium | Cloud | $$ |

## 🎮 Control Panel

### Start/Stop/Restart

```bash
# Check status
./test-mcp-connection.sh

# View logs
tail -f squirrel-mcp.log

# Stop
kill $(cat squirrel-mcp.pid)

# Start
./start-mcp-server.sh

# Restart
kill $(cat squirrel-mcp.pid) && sleep 2 && ./start-mcp-server.sh
```

### Prefer Local AI

Edit `mcp-config.env`:
```bash
AI_ROUTER_PREFER_LOCAL=true
AI_ROUTER_FALLBACK_CLOUD=true  # Still use cloud if local fails
```

Restart Squirrel for changes to take effect.

### Monitor Provider Usage

```bash
# Watch all AI requests
tail -f squirrel-mcp.log | grep "AI request\|provider\|generated"

# Count requests by provider
grep "provider:" squirrel-mcp.log | cut -d: -f4 | sort | uniq -c
```

## 🎯 Success Indicators

You'll know it's working when:

1. **Cursor shows Squirrel** in MCP server list
2. **Logs show activity** when using AI in Cursor
3. **AI responses appear** in Cursor's UI
4. **Performance is good** (especially with Ollama)
5. **No errors** in logs or Cursor console

## 📚 Additional Resources

- **Main docs**: `CURSOR_INTEGRATION_COMPLETE.md`
- **Test script**: `test-mcp-connection.sh`
- **Logs**: `squirrel-mcp.log`
- **Config**: `~/.cursor/mcp.json`
- **Wrapper**: `mcp-wrapper.sh`

---

**Last Updated**: January 15, 2026  
**All Tests**: ✅ PASSING  
**Status**: 🚀 READY FOR USE

**Next**: Try using AI features in Cursor and watch the logs!

