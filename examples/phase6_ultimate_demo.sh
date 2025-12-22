#!/usr/bin/env bash
# Phase 6 - Ultimate Vision Demo
#
# Demonstrates provider self-registration and dynamic action discovery

set -e

SQUIRREL_URL="${SQUIRREL_URL:-http://localhost:9090}"

echo "🌟 Phase 6: Ultimate Vision Demo"
echo "================================="
echo "ONE endpoint for ALL AI types - Infinite extensibility!"
echo

# Check if Squirrel is running
if ! curl -s -f "$SQUIRREL_URL/health" > /dev/null; then
    echo "❌ Squirrel is not running at $SQUIRREL_URL"
    echo "💡 Start Squirrel first with: cargo run"
    exit 1
fi
echo "✅ Squirrel is running"
echo

# 1. Register a new provider with custom action
echo "📝 Step 1: Register a NEW provider with CUSTOM action"
echo "POST $SQUIRREL_URL/api/v1/providers/register"
echo
curl -X POST "$SQUIRREL_URL/api/v1/providers/register" \
  -H "Content-Type: application/json" \
  -d '{
    "provider_id": "future-ai-001",
    "provider_name": "Future AI Provider",
    "advertised_capabilities": [
      {
        "action": "video.generation",
        "input_schema": {
          "prompt": "string",
          "duration_seconds": "integer",
          "style": "string"
        },
        "output_schema": {
          "video_url": "string",
          "duration": "integer",
          "format": "string"
        },
        "cost_per_unit": 0.50,
        "avg_latency_ms": 45000,
        "quality": "high"
      },
      {
        "action": "3d.model.generation",
        "input_schema": {
          "description": "string",
          "format": "string",
          "polygon_count": "integer"
        },
        "output_schema": {
          "model_url": "string",
          "format": "string",
          "polygon_count": "integer"
        },
        "cost_per_unit": 1.00,
        "avg_latency_ms": 60000,
        "quality": "premium"
      }
    ]
  }' | jq '.'
echo
echo

# 2. List all registered actions
echo "📋 Step 2: List ALL registered actions (including new ones!)"
echo "GET $SQUIRREL_URL/api/v1/actions"
echo
curl -s "$SQUIRREL_URL/api/v1/actions" | jq '.'
echo
echo

# 3. List all providers
echo "👥 Step 3: List all registered providers"
echo "GET $SQUIRREL_URL/api/v1/providers"
echo
curl -s "$SQUIRREL_URL/api/v1/providers" | jq '.'
echo
echo

# 4. Register another provider for same action
echo "📝 Step 4: Register ANOTHER provider for video.generation"
echo
curl -X POST "$SQUIRREL_URL/api/v1/providers/register" \
  -H "Content-Type: application/json" \
  -d '{
    "provider_id": "budget-ai-002",
    "provider_name": "Budget AI Provider",
    "advertised_capabilities": [
      {
        "action": "video.generation",
        "input_schema": {
          "prompt": "string",
          "duration_seconds": "integer"
        },
        "output_schema": {
          "video_url": "string"
        },
        "cost_per_unit": 0.05,
        "avg_latency_ms": 120000,
        "quality": "medium"
      }
    ]
  }' | jq '.'
echo
echo

# 5. Query actions again (should show 2 providers for video.generation)
echo "📊 Step 5: Query actions (now 2 providers for video.generation!)"
echo
curl -s "$SQUIRREL_URL/api/v1/actions" | jq '.'
echo
echo

# 6. Register a completely NEW action type
echo "🆕 Step 6: Register a COMPLETELY NEW action type (smell.generation)"
echo
curl -X POST "$SQUIRREL_URL/api/v1/providers/register" \
  -H "Content-Type: application/json" \
  -d '{
    "provider_id": "sensory-ai-003",
    "provider_name": "Sensory AI Provider",
    "advertised_capabilities": [
      {
        "action": "smell.generation",
        "input_schema": {
          "description": "string",
          "intensity": "float"
        },
        "output_schema": {
          "scent_code": "string",
          "intensity": "float",
          "duration_minutes": "integer"
        },
        "cost_per_unit": 2.50,
        "avg_latency_ms": 5000,
        "quality": "premium"
      }
    ]
  }' | jq '.'
echo
echo

# 7. Final action list
echo "🎊 Step 7: Final action list (Squirrel never knew about these!)"
echo
curl -s "$SQUIRREL_URL/api/v1/actions" | jq '.actions'
echo
echo

# 8. Deregister a provider
echo "🗑️  Step 8: Deregister budget provider"
echo "DELETE $SQUIRREL_URL/api/v1/providers/budget-ai-002"
echo
curl -X DELETE "$SQUIRREL_URL/api/v1/providers/budget-ai-002" | jq '.'
echo
echo

echo "✅ Phase 6 Demo Complete!"
echo
echo "🌟 KEY POINTS:"
echo "  • Providers self-register their capabilities"
echo "  • Squirrel NEVER knew about video/3d/smell generation"
echo "  • No code changes needed for new AI types"
echo "  • Infinite extensibility achieved!"
echo
echo "🔮 FUTURE:"
echo "  • Any provider can register ANY action"
echo "  • Squirrel routes based on schemas & requirements"
echo "  • Supports AI types that don't exist yet"
echo "  • True capability-based architecture"
echo
echo "🚀 This is the ULTIMATE vision!"

