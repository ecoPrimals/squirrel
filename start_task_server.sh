#!/bin/bash
set -e

# Start the MCP task server
echo "Starting MCP Task Server..."
cd "$(dirname "$0")"

# Build the task server binary if needed
echo "Building task server binary..."
cargo build -p squirrel-mcp --bin task_server

# Run the task server
echo "Running task server..."
./target/debug/task_server --address "[::1]:50052" --verbose

echo "Task server stopped." 