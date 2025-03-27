#!/bin/bash
set -e

echo "Building squirrel-web with UI..."
cargo build --bin web_server

echo "Starting server..."
cargo run --bin web_server

# If you want to include the API docs feature:
# cargo run --bin web_server --features api-docs 