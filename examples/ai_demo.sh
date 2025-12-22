#!/usr/bin/env bash
# AI Capability Demonstration Script
# 
# Demonstrates the new vendor-agnostic AI endpoints

set -e

HOST="${SQUIRREL_HOST:-localhost}"
PORT="${SQUIRREL_PORT:-9090}"
BASE_URL="http://$HOST:$PORT"

echo "🤖 Squirrel AI Capability Demo"
echo "================================"
echo

# Check if server is running
echo "📡 Checking if Squirrel is running at $BASE_URL..."
if ! curl -s -f "$BASE_URL/health" > /dev/null; then
    echo "❌ Squirrel is not running at $BASE_URL"
    echo "💡 Start Squirrel first with: cargo run"
    exit 1
fi
echo "✅ Squirrel is running"
echo

# Query capabilities
echo "🔍 Querying available AI capabilities..."
echo "GET $BASE_URL/api/v1/capabilities"
curl -s "$BASE_URL/api/v1/capabilities" | jq '.'
echo
echo

# Generate text
echo "💬 Test 1: Text Generation"
echo "POST $BASE_URL/ai/generate-text"
echo
curl -X POST "$BASE_URL/ai/generate-text" \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Write a haiku about AI and nature working together",
    "max_tokens": 100,
    "temperature": 0.7
  }' | jq '.'
echo
echo

# Generate image
echo "🎨 Test 2: Image Generation"
echo "POST $BASE_URL/ai/generate-image"
echo
curl -X POST "$BASE_URL/ai/generate-image" \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "A futuristic AI network connecting with nature",
    "size": "512x512",
    "n": 1
  }' | jq '.'
echo
echo

# Universal execute endpoint - text
echo "🌟 Test 3: Universal /ai/execute (text)"
echo "POST $BASE_URL/ai/execute"
echo
curl -X POST "$BASE_URL/ai/execute" \
  -H "Content-Type: application/json" \
  -d '{
    "action": "text.generation",
    "input": {
      "prompt": "Explain quantum computing in one sentence",
      "max_tokens": 50
    },
    "requirements": {
      "cost_preference": "optimize",
      "quality": "high"
    }
  }' | jq '.'
echo
echo

# Universal execute endpoint - image
echo "🌟 Test 4: Universal /ai/execute (image)"
echo "POST $BASE_URL/ai/execute"
echo
curl -X POST "$BASE_URL/ai/execute" \
  -H "Content-Type: application/json" \
  -d '{
    "action": "image.generation",
    "input": {
      "prompt": "A serene digital landscape",
      "size": "512x512",
      "n": 1
    },
    "requirements": {
      "quality": "high",
      "max_latency_ms": 30000
    }
  }' | jq '.'
echo
echo

echo "✅ AI Capability Demo Complete!"
echo
echo "💡 Tips:"
echo "  - Set OPENAI_API_KEY to enable OpenAI (GPT, DALL-E)"
echo "  - Set HUGGINGFACE_API_KEY to enable HuggingFace (Stable Diffusion)"
echo "  - Provider selection is automatic based on availability and requirements"
echo "  - Use /ai/execute for maximum flexibility and future compatibility"

