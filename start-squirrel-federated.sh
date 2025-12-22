#!/bin/bash
# 🌐 Start Squirrel in Federated Mode
# Registers with Songbird service mesh on startup

set -e

echo "🐿️ Starting Squirrel in FEDERATED mode..."
echo ""

# Load API keys
SECRETS_PATH="/home/eastgate/Development/ecoPrimals/testing-secrets/api-keys.toml"

if [ -f "$SECRETS_PATH" ]; then
    echo "✓ Loading API keys from testing-secrets..."
    export OPENAI_API_KEY=$(grep 'openai_api_key' "$SECRETS_PATH" | cut -d'"' -f2)
    export ANTHROPIC_API_KEY=$(grep 'anthropic_api_key' "$SECRETS_PATH" | cut -d'"' -f2)
    export HUGGINGFACE_API_KEY=$(grep -A1 'hugging face' "$SECRETS_PATH" | tail -1)
fi

# Configure Songbird endpoint
export SONGBIRD_ENDPOINT="http://localhost:8081"
export SONGBIRD_ENABLED="true"
export SERVICE_HOST="127.0.0.1"
export SERVICE_PORT="9010"

echo "✓ Configuration:"
echo "  - Songbird endpoint: $SONGBIRD_ENDPOINT"
echo "  - Squirrel internal port: $SERVICE_PORT"
echo "  - Federation mode: ENABLED"
echo ""

# Start Squirrel (it will auto-register with Songbird)
echo "🚀 Starting Squirrel..."
echo "   (Will auto-register with Songbird on startup)"
echo ""

cd /home/eastgate/Development/ecoPrimals/squirrel
cargo run --release --bin squirrel

