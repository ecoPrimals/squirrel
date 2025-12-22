#!/bin/bash
# Test Squirrel AI API endpoints

echo "Testing Squirrel AI API..."
echo ""

# Test health
echo "1. Health check:"
curl -s http://localhost:9010/health | jq '.status'
echo ""

# Test ecosystem
echo "2. Ecosystem status:"
curl -s http://localhost:9010/api/v1/ecosystem/status | jq '.ecosystem.discovered_primals'
echo ""

# List available AI routes
echo "3. Checking AI routes..."
for path in "/api/v1/ai" "/api/ai" "/ai" "/api/v1/ai/chat" "/api/ai/chat"; do
    echo -n "Testing $path... "
    response=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:9010$path)
    echo "HTTP $response"
done

