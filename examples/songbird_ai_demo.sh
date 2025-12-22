#!/usr/bin/env bash
# Songbird AI Integration Demo
# 
# Demonstrates distributed AI coordination via Songbird service mesh

set -e

SQUIRREL_URL="${SQUIRREL_URL:-http://localhost:9090}"
SONGBIRD_URL="${SONGBIRD_URL:-http://localhost:8080}"

echo "🤝 Songbird AI Integration Demo"
echo "================================"
echo "Squirrel:  $SQUIRREL_URL"
echo "Songbird:  $SONGBIRD_URL"
echo

# Check if Songbird is running
echo "📡 Checking Songbird availability..."
if curl -s -f "$SONGBIRD_URL/health" > /dev/null 2>&1; then
    echo "✅ Songbird is running"
else
    echo "⚠️  Songbird is not running at $SONGBIRD_URL"
    echo "💡 Squirrel will work without Songbird (local AI only)"
fi
echo

# Check if Squirrel is running
echo "📡 Checking Squirrel availability..."
if ! curl -s -f "$SQUIRREL_URL/health" > /dev/null; then
    echo "❌ Squirrel is not running at $SQUIRREL_URL"
    echo "💡 Start Squirrel first with: cargo run"
    exit 1
fi
echo "✅ Squirrel is running"
echo

# Query AI capabilities (local)
echo "🔍 Query Local AI Capabilities"
echo "GET $SQUIRREL_URL/api/v1/capabilities"
echo
curl -s "$SQUIRREL_URL/api/v1/capabilities" | jq '.'
echo
echo

# If Songbird is available, query distributed capabilities
if curl -s -f "$SONGBIRD_URL/health" > /dev/null 2>&1; then
    echo "🌐 Query Distributed AI Capabilities (via Songbird)"
    echo "GET $SONGBIRD_URL/api/v1/ai/capabilities/query?capability=image.generation"
    echo
    curl -s "$SONGBIRD_URL/api/v1/ai/capabilities/query?capability=image.generation" | jq '.' || {
        echo "⚠️  Songbird AI capability endpoint not yet implemented"
        echo "💡 This is expected - Songbird needs AI capability support added"
    }
    echo
    echo
    
    echo "📊 Check Songbird AI Registrations"
    echo "GET $SONGBIRD_URL/api/v1/ai/capabilities"
    echo
    curl -s "$SONGBIRD_URL/api/v1/ai/capabilities" | jq '.' || {
        echo "⚠️  Songbird AI capability endpoint not yet implemented"
        echo "💡 Phase 5 is complete on Squirrel side - Songbird needs updates"
    }
    echo
    echo
fi

# Generate image (will use local providers)
echo "🎨 Generate Image (Local Providers)"
echo "POST $SQUIRREL_URL/ai/generate-image"
echo
curl -X POST "$SQUIRREL_URL/ai/generate-image" \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "A distributed AI network across the ecosystem",
    "size": "512x512",
    "n": 1
  }' | jq '.'
echo
echo

echo "✅ Songbird Integration Demo Complete!"
echo
echo "💡 Key Points:"
echo "  - Squirrel registers AI capabilities with Songbird on startup"
echo "  - Heartbeat updates sent every 30 seconds"
echo "  - AI capabilities work locally even without Songbird"
echo "  - Songbird needs AI capability endpoints for full mesh coordination"
echo
echo "🔮 Future: When multiple Squirrels register:"
echo "  - Query Songbird to find all AI-capable services"
echo "  - Route requests to best available Squirrel instance"
echo "  - Load balance across distributed AI providers"
echo "  - Automatic failover if one Squirrel goes down"

