#!/bin/bash
set -e

# Define directories
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="$ROOT_DIR/target/debug"
PORT=50052

# Make sure we're built
echo "Building task server and client..."
cd "$ROOT_DIR" && cargo build

# Check if there's already a service running on our port
if nc -z localhost $PORT 2>/dev/null; then
    echo "A service is already running on port $PORT, using existing server."
    SERVER_STARTED_BY_US=false
else
    # Start the server in the background
    echo "Starting task server..."
    $TARGET_DIR/taskserver &
    SERVER_PID=$!
    SERVER_STARTED_BY_US=true
    
    # Give the server a moment to start
    echo "Waiting for server to start..."
    sleep 2
fi

# Run the client
echo "Running test client..."
$TARGET_DIR/taskclient

# Check if client was successful
if [ $? -eq 0 ]; then
    echo "Test completed successfully!"
    EXIT_CODE=0
else
    echo "Test failed!"
    EXIT_CODE=1
fi

# Cleanup
if [ "$SERVER_STARTED_BY_US" = true ] && [ -n "$SERVER_PID" ]; then
    echo "Stopping server (PID: $SERVER_PID)..."
    kill $SERVER_PID 2>/dev/null || true
fi

exit $EXIT_CODE 